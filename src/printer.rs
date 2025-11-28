use crate::highlighter::{supports_true_color, Highlighter, LineHighlighter};
use std::io::{self, BufRead, Write};
use std::path::Path;

/// 列印檔案內容（streaming 模式）
pub fn print_content_streaming<R: BufRead>(
    mut reader: R,
    show_line_numbers: bool,
    file_path: Option<&Path>,
    enable_highlighting: bool,
    theme: Option<&str>,
    language: Option<&str>,
) -> io::Result<()> {
    // 準備高亮器（需要在外層創建以延長生命週期）
    let highlighter = if enable_highlighting {
        Highlighter::new(theme, supports_true_color()).ok()
    } else {
        None
    };

    let mut line_highlighter = if let Some(ref hl) = highlighter {
        // 讀取第一行用於語法檢測
        let mut first_line = String::new();
        reader.read_line(&mut first_line)?;

        let first_line_opt = if first_line.is_empty() {
            None
        } else {
            Some(first_line.trim_end())
        };

        let mut lh = hl.prepare_for_file(file_path, first_line_opt, language);

        // 處理第一行
        if !first_line.is_empty() {
            print_single_line(&mut lh, &first_line, 1, show_line_numbers)?;
        }

        Some(lh)
    } else {
        None
    };

    // 處理剩餘的行
    let mut line_buffer = String::new();
    let mut line_number = 2; // 第一行已經處理過了

    while reader.read_line(&mut line_buffer)? > 0 {
        if let Some(ref mut lh) = line_highlighter {
            print_single_line(lh, &line_buffer, line_number, show_line_numbers)?;
        } else {
            // 無語法高亮
            print_plain_line(&line_buffer, line_number, show_line_numbers)?;
        }

        line_buffer.clear();
        line_number += 1;
    }

    Ok(())
}

/// 列印單行（帶語法高亮）
fn print_single_line(
    highlighter: &mut LineHighlighter,
    line: &str,
    line_number: usize,
    show_line_numbers: bool,
) -> io::Result<()> {
    let mut stdout = io::stdout().lock();

    let highlighted = highlighter
        .highlight_line(line)
        .unwrap_or_else(|_| line.to_string());

    if show_line_numbers {
        match write!(stdout, "{} {}", line_number, highlighted) {
            Ok(_) => Ok(()),
            Err(e) if e.kind() == io::ErrorKind::BrokenPipe => Ok(()),
            Err(e) => Err(e),
        }
    } else {
        match write!(stdout, "{}", highlighted) {
            Ok(_) => Ok(()),
            Err(e) if e.kind() == io::ErrorKind::BrokenPipe => Ok(()),
            Err(e) => Err(e),
        }
    }
}

/// 列印單行（無語法高亮）
fn print_plain_line(line: &str, line_number: usize, show_line_numbers: bool) -> io::Result<()> {
    let mut stdout = io::stdout().lock();

    if show_line_numbers {
        match writeln!(stdout, "{} {}", line_number, line.trim_end()) {
            Ok(_) => Ok(()),
            Err(e) if e.kind() == io::ErrorKind::BrokenPipe => Ok(()),
            Err(e) => Err(e),
        }
    } else {
        match write!(stdout, "{}", line) {
            Ok(_) => Ok(()),
            Err(e) if e.kind() == io::ErrorKind::BrokenPipe => Ok(()),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_print_streaming() {
        let content = "line 1\nline 2\nline 3\n";
        let reader = Cursor::new(content);
        // 這個測試主要確保函數不會 panic
        let _ = print_content_streaming(reader, false, None, false, None, None);
    }

    #[test]
    fn test_print_streaming_with_line_numbers() {
        let content = "line 1\nline 2\nline 3\n";
        let reader = Cursor::new(content);
        let _ = print_content_streaming(reader, true, None, false, None, None);
    }
}
