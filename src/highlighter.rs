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

/// 語法高亮器（用於創建 LineHighlighter）
pub struct Highlighter {
    theme: Theme,
    true_color: bool,
}

impl Highlighter {
    /// 建立新的高亮器
    pub fn new(theme_name: Option<&str>, true_color: bool) -> Result<Self> {
        let theme_name = theme_name.unwrap_or("base16-eighties.dark");
        let theme = THEME_SET
            .themes
            .get(theme_name)
            .context(format!("Theme '{}' not found", theme_name))?
            .clone();

        Ok(Self { theme, true_color })
    }

    /// 準備一個逐行高亮器
    pub fn prepare_for_file<'a>(
        &'a self,
        file_path: Option<&Path>,
        first_line: Option<&str>,
        language: Option<&str>,
    ) -> LineHighlighter<'a> {
        // 檢測語法（優先使用手動指定的語言）
        let syntax = if let Some(lang) = language {
            self.find_syntax_by_name(lang)
                .unwrap_or_else(|| self.detect_syntax(first_line, file_path))
        } else {
            self.detect_syntax(first_line, file_path)
        };
        let is_plain_text = syntax.name == "Plain Text";

        LineHighlighter {
            highlighter: HighlightLines::new(syntax, &self.theme),
            true_color: self.true_color,
            is_plain_text,
        }
    }

    /// 根據語言名稱查找語法
    fn find_syntax_by_name(&self, name: &str) -> Option<&SyntaxReference> {
        // 嘗試精確匹配
        if let Some(syntax) = SYNTAX_SET.find_syntax_by_name(name) {
            return Some(syntax);
        }

        // 嘗試模糊匹配（不區分大小寫）
        let name_lower = name.to_lowercase();
        SYNTAX_SET.syntaxes().iter().find(|s| {
            s.name.to_lowercase() == name_lower
                || s.file_extensions
                    .iter()
                    .any(|ext| ext.to_lowercase() == name_lower)
        })
    }

    /// 檢測檔案的語法類型
    fn detect_syntax(
        &self,
        first_line: Option<&str>,
        file_path: Option<&Path>,
    ) -> &SyntaxReference {
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
        if let Some(first_line) = first_line {
            if first_line.starts_with("#!") {
                if let Some(syntax) = SYNTAX_SET.find_syntax_by_first_line(first_line) {
                    return syntax;
                }
            }
        }

        // 3. 回退到純文字
        SYNTAX_SET.find_syntax_plain_text()
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

/// 逐行高亮器（有狀態）
pub struct LineHighlighter<'a> {
    highlighter: HighlightLines<'a>,
    true_color: bool,
    is_plain_text: bool,
}

impl<'a> LineHighlighter<'a> {
    /// 高亮單行（保持語法狀態）
    pub fn highlight_line(&mut self, line: &str) -> Result<String> {
        // 如果是純文字，直接返回
        if self.is_plain_text {
            return Ok(line.to_string());
        }

        // 長行保護：超過 16KB 的行跳過語法高亮（與 bat 一致）
        const MAX_LINE_LENGTH: usize = 16 * 1024;
        if line.len() > MAX_LINE_LENGTH {
            // 仍然需要高亮一個換行符來更新狀態
            let _ = self
                .highlighter
                .highlight_line("\n", &SYNTAX_SET)
                .context("Failed to highlight line")?;
            return Ok(line.to_string());
        }

        // 逐行高亮
        let ranges: Vec<(Style, &str)> = self
            .highlighter
            .highlight_line(line, &SYNTAX_SET)
            .context("Failed to highlight line")?;

        let escaped = if self.true_color {
            as_24_bit_terminal_escaped(&ranges[..], false)
        } else {
            self.as_8bit_terminal_escaped(&ranges[..])
        };

        Ok(escaped)
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
        let path = Path::new("test.rs");

        let syntax = highlighter.detect_syntax(None, Some(path));
        assert_eq!(syntax.name, "Rust");
    }

    #[test]
    fn test_shebang_detection() {
        let highlighter = Highlighter::new(None, true).unwrap();
        let first_line = "#!/bin/bash";

        let syntax = highlighter.detect_syntax(Some(first_line), None);
        assert!(syntax.name.contains("Bash") || syntax.name.contains("Shell"));
    }

    #[test]
    fn test_line_highlighter() {
        let highlighter = Highlighter::new(None, true).unwrap();
        let path = Path::new("test.rs");
        let mut line_highlighter = highlighter.prepare_for_file(Some(path), None, None);

        let result = line_highlighter.highlight_line("fn main() {\n");
        assert!(result.is_ok());
    }

    #[test]
    fn test_language_specification() {
        let highlighter = Highlighter::new(None, true).unwrap();

        // 測試指定語言
        let mut line_highlighter = highlighter.prepare_for_file(None, None, Some("rust"));
        let result = line_highlighter.highlight_line("fn main() {\n");
        assert!(result.is_ok());

        // 測試不區分大小寫
        let mut line_highlighter2 = highlighter.prepare_for_file(None, None, Some("Rust"));
        let result2 = line_highlighter2.highlight_line("fn main() {\n");
        assert!(result2.is_ok());
    }
}
