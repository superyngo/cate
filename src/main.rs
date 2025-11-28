mod encoder;
mod highlighter;
mod printer;

use anyhow::Result;
use pico_args::Arguments;
use std::io::{IsTerminal, Write};
use std::path::PathBuf;

/// 顏色輸出模式
#[derive(Debug, Clone, Copy, PartialEq)]
enum ColorMode {
    Auto,   // 自動檢測（TTY 時啟用）
    Always, // 總是輸出顏色
    Never,  // 永不輸出顏色
}

impl ColorMode {
    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "auto" => Ok(ColorMode::Auto),
            "always" => Ok(ColorMode::Always),
            "never" => Ok(ColorMode::Never),
            _ => Err(anyhow::anyhow!(
                "Invalid color mode: '{}'. Use 'auto', 'always', or 'never'",
                s
            )),
        }
    }

    /// 根據模式和 TTY 狀態決定是否啟用顏色
    fn should_colorize(&self) -> bool {
        match self {
            ColorMode::Auto => std::io::stdout().is_terminal(),
            ColorMode::Always => true,
            ColorMode::Never => false,
        }
    }
}

struct Args {
    files: Vec<PathBuf>,
    encoding: Option<String>,
    show_line_numbers: bool,
    debug: bool,

    // 語法高亮選項
    no_highlight: bool,       // --no-highlight: 停用語法高亮
    theme: Option<String>,    // --theme: 指定主題
    language: Option<String>, // -l, --language: 指定語法語言
    color_mode: ColorMode,    // --color: 顏色輸出模式
}

impl Args {
    fn parse() -> Result<Self> {
        let mut args = Arguments::from_env();

        // 處理幫助和版本
        if args.contains(["-h", "--help"]) {
            print_help();
            std::process::exit(0);
        }

        if args.contains(["-v", "--version"]) {
            print_version();
            std::process::exit(0);
        }

        // 列出編碼
        if args.contains("--list-encodings") {
            encoder::list_encodings();
            std::process::exit(0);
        }

        // 列出主題
        if args.contains("--list-themes") {
            list_themes();
            std::process::exit(0);
        }

        // 列出語法
        if args.contains("--list-syntaxes") {
            list_syntaxes();
            std::process::exit(0);
        }

        // 解析 color mode
        let color_mode = if let Some(color_str) = args.opt_value_from_str::<_, String>("--color")? {
            ColorMode::from_str(&color_str)?
        } else {
            ColorMode::Auto // 默認為 auto
        };

        Ok(Args {
            encoding: args.opt_value_from_str(["-e", "--encoding"])?,
            show_line_numbers: args.contains(["-n", "--number"]),
            debug: args.contains("--debug"),

            // 語法高亮選項
            no_highlight: args.contains("--no-highlight"),
            theme: args.opt_value_from_str("--theme")?,
            language: args.opt_value_from_str(["-l", "--language"])?,
            color_mode,

            files: args.finish().into_iter().map(PathBuf::from).collect(),
        })
    }
}

fn main() -> Result<()> {
    let args = Args::parse()?;

    // 解析用戶指定的編碼
    let user_encoding = if let Some(ref enc_str) = args.encoding {
        Some(encoder::parse_encoding(enc_str)?)
    } else {
        None
    };

    // 處理 stdin
    if args.files.is_empty() {
        if args.debug {
            eprintln!("[DEBUG] Reading from stdin");
        }
        let (content, detected) = encoder::read_stdin_with_encoding(user_encoding, args.debug)?;

        if args.debug {
            eprintln!(
                "[DEBUG] Final encoding: {} (confidence: {:?})",
                detected.encoding.name(),
                detected.confidence
            );
            eprintln!("[DEBUG] Content length: {} bytes", content.len());
            eprintln!("[DEBUG] ---");
        }

        // 決定是否啟用語法高亮（考慮 --no-highlight 和 TTY 檢測）
        let enable_highlighting = !args.no_highlight && args.color_mode.should_colorize();

        // 使用 Cursor 將字符串轉為 BufRead
        let reader = std::io::Cursor::new(content);
        printer::print_content_streaming(
            reader,
            args.show_line_numbers,
            None,
            enable_highlighting,
            args.theme.as_deref(),
            args.language.as_deref(),
        )?;

        return Ok(());
    }

    // 處理檔案
    for (i, file_path) in args.files.iter().enumerate() {
        if args.debug {
            eprintln!("[DEBUG] Reading file: {:?}", file_path);
        }

        let (content, detected) =
            encoder::read_file_with_encoding(file_path, user_encoding, args.debug)?;

        if args.debug {
            eprintln!(
                "[DEBUG] Final encoding: {} (confidence: {:?})",
                detected.encoding.name(),
                detected.confidence
            );
            eprintln!("[DEBUG] Content length: {} bytes", content.len());
            eprintln!("[DEBUG] ---");
        }

        // 決定是否啟用語法高亮（考慮 --no-highlight 和 TTY 檢測）
        let enable_highlighting = !args.no_highlight && args.color_mode.should_colorize();

        // 使用 Cursor 將字符串轉為 BufRead
        let reader = std::io::Cursor::new(content);
        printer::print_content_streaming(
            reader,
            args.show_line_numbers,
            Some(file_path.as_path()),
            enable_highlighting,
            args.theme.as_deref(),
            args.language.as_deref(),
        )?;

        // 多個檔案間加分隔
        if i < args.files.len() - 1 {
            // 使用 println! 來檢查並忽略 broken pipe
            if let Err(e) = writeln!(std::io::stdout()) {
                if e.kind() == std::io::ErrorKind::BrokenPipe {
                    break; // 提早退出迴圈
                }
                return Err(e.into());
            }
        }
    }

    Ok(())
}
fn print_help() {
    println!("cate - cat with encoding support and syntax highlighting");
    println!();
    println!("USAGE:");
    println!("    cate [OPTIONS] [FILES]...");
    println!();
    println!("OPTIONS:");
    println!("    -h, --help              Print this help message");
    println!("    -v, --version           Print version information");
    println!("    -e, --encoding <ENC>    Specify input encoding (utf-8, gbk, big5, etc.)");
    println!("    -n, --number            Show line numbers");
    println!("    --debug                 Show debug information");
    println!("    --list-encodings        List all supported encodings");
    println!();
    println!("SYNTAX HIGHLIGHTING:");
    println!("    --no-highlight          Disable syntax highlighting");
    println!("    --theme <THEME>         Set color theme (default: base16-eighties.dark)");
    println!("    -l, --language <LANG>   Specify syntax language (e.g., rust, python)");
    println!("    --color <WHEN>          When to use colors: auto, always, never (default: auto)");
    println!("    --list-themes           List all available themes");
    println!("    --list-syntaxes         List all supported languages");
    println!();
    println!("EXAMPLES:");
    println!("    cate file.rs                    # Display Rust file with syntax highlighting");
    println!("    cate --theme 'Solarized (dark)' file.py");
    println!("    cate -n --no-highlight file.txt # Show line numbers without highlighting");
    println!("    cate -e gbk chinese.txt         # Specify GBK encoding");
    println!("    cat file.js | cate              # Read from stdin");
    println!("    cat script | cate -l python     # Specify language for stdin");
    println!("    cate file.rs > output.txt       # Auto-disable colors when redirected");
    println!("    cate file.rs --color=always | less -R  # Force colors for pager");
    println!();
    println!("SUPPORTED ENCODINGS:");
    encoder::list_encodings();
}

fn print_version() {
    println!("cate {}", env!("CARGO_PKG_VERSION"));
}

fn list_themes() {
    println!("Available themes:");
    let mut themes = highlighter::Highlighter::available_themes();
    themes.sort();
    for theme in themes {
        println!("  {}", theme);
    }
}

fn list_syntaxes() {
    // 像 bat 一樣顯示所有語法
    let syntaxes = highlighter::Highlighter::available_syntaxes();

    for syntax in syntaxes {
        println!("{}", syntax);
    }
}
