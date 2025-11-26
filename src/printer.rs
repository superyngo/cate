use crate::highlighter::{supports_true_color, Highlighter};
use std::io::{self, Write};
use std::path::Path;

/// 列印檔案內容
pub fn print_content(
    content: &str,
    show_line_numbers: bool,
    file_path: Option<&Path>,
    enable_highlighting: bool,
    theme: Option<&str>,
) -> io::Result<()> {
    // 如果啟用語法高亮，先進行高亮處理
    let processed_content = if enable_highlighting {
        match Highlighter::new(theme, supports_true_color()) {
            Ok(highlighter) => {
                match highlighter.highlight(content, file_path) {
                    Ok(highlighted) => highlighted,
                    Err(_) => content.to_string(), // 高亮失敗，回退到原文
                }
            }
            Err(_) => content.to_string(),
        }
    } else {
        content.to_string()
    };

    if show_line_numbers {
        print_with_line_numbers(&processed_content)
    } else {
        let mut stdout = io::stdout().lock();
        match write!(stdout, "{}", processed_content) {
            Ok(_) => Ok(()),
            Err(e) if e.kind() == io::ErrorKind::BrokenPipe => Ok(()), // 忽略 broken pipe
            Err(e) => Err(e),
        }
    }
}

fn print_with_line_numbers(content: &str) -> io::Result<()> {
    let lines: Vec<&str> = content.lines().collect();
    let total_lines = lines.len();
    let width = total_lines.to_string().len();

    let mut stdout = io::stdout().lock();

    for (i, line) in lines.iter().enumerate() {
        match writeln!(stdout, "{:>width$} {}", i + 1, line, width = width) {
            Ok(_) => {}
            Err(e) if e.kind() == io::ErrorKind::BrokenPipe => return Ok(()), // 忽略 broken pipe
            Err(e) => return Err(e),
        }
    }

    // 如果最後一行沒有換行符，也要確保輸出完整
    if !content.ends_with('\n') && !content.is_empty() {
        match writeln!(stdout) {
            Ok(_) => Ok(()),
            Err(e) if e.kind() == io::ErrorKind::BrokenPipe => Ok(()), // 忽略 broken pipe
            Err(e) => Err(e),
        }
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_with_line_numbers() {
        let content = "line 1\nline 2\nline 3";
        // 這個測試主要確保函數不會 panic
        print_with_line_numbers(content);
    }

    #[test]
    fn test_line_number_width() {
        let content = (1..=100)
            .map(|i| format!("line {}", i))
            .collect::<Vec<_>>()
            .join("\n");
        print_with_line_numbers(&content);
    }
}
