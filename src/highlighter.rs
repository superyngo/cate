use anyhow::{Context, Result};
use once_cell::sync::Lazy;
use std::path::Path;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, Theme, ThemeSet};
use syntect::parsing::{SyntaxReference, SyntaxSet};
use syntect::util::as_24_bit_terminal_escaped;

/// 嵌入的語法集（來自 bat 專案）
///
/// 此檔案來自 bat (https://github.com/sharkdp/bat)
/// 授權：MIT License / Apache License 2.0
/// 包含 219 種語法定義，原始來源為 Sublime Text packages (MIT License)
/// 詳見 THIRD-PARTY-LICENSES.md
const SERIALIZED_SYNTAX_SET: &[u8] = include_bytes!("../assets/syntaxes.bin");

/// 語法集是否壓縮（與 bat 保持一致）
const COMPRESS_SYNTAXES: bool = false;

/// 全域語法集（延遲載入，使用 bat 的載入方式）
static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(|| {
    load_from_binary(SERIALIZED_SYNTAX_SET, COMPRESS_SYNTAXES)
        .expect("Failed to load embedded syntax set")
});

/// 全域主題集（使用 syntect 內建主題）
static THEME_SET: Lazy<ThemeSet> = Lazy::new(ThemeSet::load_defaults);

/// 從二進位資料載入（與 bat 的 from_binary 相同邏輯）
fn load_from_binary<T>(data: &[u8], compressed: bool) -> Result<T>
where
    T: for<'de> serde::Deserialize<'de>,
{
    if compressed {
        bincode::deserialize_from(flate2::read::ZlibDecoder::new(data))
            .context("Failed to decompress and deserialize")
    } else {
        bincode::deserialize(data).context("Failed to deserialize")
    }
}

/// 語法高亮器
pub struct Highlighter {
    theme: Theme,
    true_color: bool,
}

impl Highlighter {
    /// 建立新的高亮器
    pub fn new(theme_name: Option<&str>, true_color: bool) -> Result<Self> {
        let theme_name = theme_name.unwrap_or("InspiredGitHub");
        let theme = THEME_SET
            .themes
            .get(theme_name)
            .context(format!("Theme '{}' not found", theme_name))?
            .clone();

        Ok(Self { theme, true_color })
    }

    /// 對整個檔案內容進行高亮
    pub fn highlight(&self, content: &str, file_path: Option<&Path>) -> Result<String> {
        // 檢測語法
        let syntax = self.detect_syntax(content, file_path);

        // 如果是純文字，直接返回
        if syntax.name == "Plain Text" {
            return Ok(content.to_string());
        }

        // 逐行高亮（保留換行符，與 bat 一致）
        let mut highlighter = HighlightLines::new(syntax, &self.theme);
        let mut output = String::with_capacity(content.len() + content.len() / 4);

        // 使用 split_inclusive 保留換行符，這對 syntect 的狀態機很重要
        for line in content.split_inclusive('\n') {
            let ranges: Vec<(Style, &str)> = highlighter
                .highlight_line(line, &SYNTAX_SET)
                .context("Failed to highlight line")?;

            let escaped = if self.true_color {
                as_24_bit_terminal_escaped(&ranges[..], false)
            } else {
                self.as_8bit_terminal_escaped(&ranges[..])
            };

            output.push_str(&escaped);
            // 不需要額外添加換行符，因為 line 已經包含了（除了最後一行可能沒有）
        }

        Ok(output)
    }

    /// 檢測檔案的語法類型
    fn detect_syntax(&self, content: &str, file_path: Option<&Path>) -> &SyntaxReference {
        // 1. 嘗試從檔案路徑檢測
        if let Some(path) = file_path {
            // 從副檔名檢測
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if let Some(syntax) = SYNTAX_SET.find_syntax_by_extension(ext) {
                    return syntax;
                }
            }

            // 從檔名檢測（例如 Makefile, Dockerfile）
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if let Some(syntax) = SYNTAX_SET.find_syntax_by_name(name) {
                    return syntax;
                }

                // 特殊檔名處理
                match name.to_lowercase().as_str() {
                    "makefile" | "gnumakefile" => {
                        if let Some(syntax) = SYNTAX_SET.find_syntax_by_name("Makefile") {
                            return syntax;
                        }
                    }
                    "dockerfile" => {
                        if let Some(syntax) = SYNTAX_SET.find_syntax_by_name("Dockerfile") {
                            return syntax;
                        }
                    }
                    _ => {}
                }
            }
        }

        // 2. 從第一行檢測（shebang）
        if let Some(first_line) = content.lines().next() {
            if first_line.starts_with("#!") {
                if let Some(syntax) = SYNTAX_SET.find_syntax_by_first_line(first_line) {
                    return syntax;
                }
            }
        }

        // 3. 回退到純文字
        SYNTAX_SET.find_syntax_plain_text()
    }

    /// 將 syntect 顏色轉為 8-bit ANSI 色碼（相容模式）
    fn as_8bit_terminal_escaped(&self, ranges: &[(Style, &str)]) -> String {
        let mut output = String::new();

        for (style, text) in ranges {
            // 使用 ansi_colours 庫進行精確的 RGB -> 256 色映射（與 bat 相同）
            let fg = style.foreground;
            let color_code = ansi_colours::ansi256_from_rgb((fg.r, fg.g, fg.b));
            output.push_str(&format!("\x1b[38;5;{}m{}\x1b[0m", color_code, text));
        }

        output
    }

    /// 列出可用主題
    pub fn available_themes() -> Vec<String> {
        THEME_SET.themes.keys().cloned().collect()
    }

    /// 列出可用語法
    pub fn available_syntaxes() -> Vec<String> {
        SYNTAX_SET
            .syntaxes()
            .iter()
            .map(|s| s.name.clone())
            .collect()
    }
}

/// 檢測終端是否支援 24-bit 真彩色
pub fn supports_true_color() -> bool {
    std::env::var("COLORTERM")
        .map(|v| v == "truecolor" || v == "24bit")
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_syntax_detection() {
        let highlighter = Highlighter::new(None, true).unwrap();
        let code = "fn main() { println!(\"Hello\"); }";
        let path = Path::new("test.rs");

        let syntax = highlighter.detect_syntax(code, Some(path));
        assert_eq!(syntax.name, "Rust");
    }

    #[test]
    fn test_shebang_detection() {
        let highlighter = Highlighter::new(None, true).unwrap();
        let code = "#!/bin/bash\necho 'Hello'";

        let syntax = highlighter.detect_syntax(code, None);
        assert!(syntax.name.contains("Bash") || syntax.name.contains("Shell"));
    }
}
