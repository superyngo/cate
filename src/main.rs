mod encoder;
mod printer;

use anyhow::Result;
use pico_args::Arguments;
use std::path::PathBuf;

#[derive(Debug)]
struct Args {
    file: Option<PathBuf>,
    encoding: Option<String>,
    show_line_numbers: bool,
    debug: bool,
}

impl Args {
    fn parse() -> Result<Self> {
        let mut pargs = Arguments::from_env();

        // 檢查是否有 --help
        if pargs.contains(["-h", "--help"]) {
            Self::print_help();
            std::process::exit(0);
        }

        // 檢查是否有 --version
        if pargs.contains(["-v", "--version"]) {
            Self::print_version();
            std::process::exit(0);
        }

        // 檢查是否要列出編碼
        if pargs.contains("--list-encodings") {
            encoder::list_encodings();
            std::process::exit(0);
        }

        let debug = pargs.contains("--debug");
        let show_line_numbers = pargs.contains(["-n", "--number"]);
        let encoding: Option<String> = pargs.opt_value_from_str(["-e", "--encoding"])?;

        // 獲取文件名（可選）
        let file: Option<PathBuf> = pargs.opt_free_from_str()?;

        // 檢查未處理的參數
        let remaining = pargs.finish();
        if !remaining.is_empty() {
            eprintln!("Warning: unused arguments {:?}", remaining);
        }

        Ok(Self {
            file,
            encoding,
            show_line_numbers,
            debug,
        })
    }

    fn print_version() {
        println!("cate {}", env!("CARGO_PKG_VERSION"));
    }

    fn print_help() {
        println!("cate - Display file contents with encoding support");
        println!();
        println!("USAGE:");
        println!("    cate [OPTIONS] [FILE]");
        println!();
        println!("ARGUMENTS:");
        println!("    [FILE]                  File to display (omit to read from stdin)");
        println!();
        println!("OPTIONS:");
        println!("    -h, --help              Show this help message");
        println!("    -v, --version           Show version information");
        println!("    -e, --encoding <ENC>    Specify encoding");
        println!("                            (utf-8, gbk, big5, shift-jis, etc.)");
        println!("    -n, --number            Show line numbers");
        println!("    --debug                 Enable debug mode");
        println!("    --list-encodings        List all supported encodings");
        println!();
        println!("ENCODING DETECTION:");
        println!("    Priority: UTF-8/BOM > User specified (-e) > System encoding");
        println!();
        println!("EXAMPLES:");
        println!("    cate file.txt                    # Display file with auto-detected encoding");
        println!("    cate file.txt -e gbk             # Display file using GBK encoding");
        println!("    cate file.txt -n                 # Display with line numbers");
        println!("    cate file.txt -e big5 -n         # Combine options");
        println!("    echo \"text\" | cate -e utf-8       # Read from stdin");
        println!("    cate --list-encodings            # Show supported encodings");
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

    // 讀取內容（從文件或 stdin）
    let (content, detected) = if let Some(ref file_path) = args.file {
        if args.debug {
            eprintln!("[DEBUG] Reading file: {:?}", file_path);
        }
        encoder::read_file_with_encoding(file_path, user_encoding, args.debug)?
    } else {
        if args.debug {
            eprintln!("[DEBUG] Reading from stdin");
        }
        encoder::read_stdin_with_encoding(user_encoding, args.debug)?
    };

    // Debug 信息
    if args.debug {
        eprintln!(
            "[DEBUG] Final encoding: {} (confidence: {:?})",
            detected.encoding.name(),
            detected.confidence
        );
        eprintln!("[DEBUG] Content length: {} bytes", content.len());
        eprintln!("[DEBUG] Line count: {}", content.lines().count());
        eprintln!("[DEBUG] ---");
    }

    // 打印內容
    printer::print_content(&content, args.show_line_numbers);

    Ok(())
}
