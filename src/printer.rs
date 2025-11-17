/// 打印文件內容，可選顯示行號
pub fn print_content(content: &str, show_line_numbers: bool) {
    if show_line_numbers {
        print_with_line_numbers(content);
    } else {
        print!("{}", content);
    }
}

/// 帶行號打印內容
fn print_with_line_numbers(content: &str) {
    let lines: Vec<&str> = content.lines().collect();
    let total_lines = lines.len();

    // 計算行號的寬度（用於對齊）
    let line_number_width = total_lines.to_string().len();

    for (index, line) in lines.iter().enumerate() {
        let line_number = index + 1;
        println!("{:>width$}  {}", line_number, line, width = line_number_width);
    }

    // 如果最後一行沒有換行符，也要確保輸出完整
    if !content.ends_with('\n') && !content.is_empty() {
        println!();
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
        let content = (1..=100).map(|i| format!("line {}", i)).collect::<Vec<_>>().join("\n");
        print_with_line_numbers(&content);
    }
}
