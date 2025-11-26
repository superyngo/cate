use anyhow::{Context, Result};
use encoding_rs::Encoding;
use std::fs;
use std::io::Read;
use std::path::Path;

/// 編碼檢測結果
#[derive(Debug)]
pub struct DetectedEncoding {
    pub encoding: &'static Encoding,
    pub confidence: EncodingConfidence,
}

#[derive(Debug, PartialEq)]
pub enum EncodingConfidence {
    Certain, // BOM 或用戶指定
    High,    // UTF-8 檢測成功
    Low,     // 回退到系統編碼
}

/// 讀取文件內容並轉換為 UTF-8 字符串
pub fn read_file_with_encoding(
    path: &Path,
    user_encoding: Option<&'static Encoding>,
    debug: bool,
) -> Result<(String, DetectedEncoding)> {
    // 讀取文件的原始字節
    let bytes = fs::read(path).context("Failed to read file")?;

    // 編碼優先級：UTF-8/BOM > 用戶指定 > 系統編碼
    let detected = detect_encoding(&bytes, user_encoding, debug);

    if debug {
        eprintln!(
            "[DEBUG] Detected encoding: {} (confidence: {:?})",
            detected.encoding.name(),
            detected.confidence
        );
    }

    // 解碼為 UTF-8 字符串
    let (cow, _encoding_used, had_errors) = detected.encoding.decode(&bytes);

    if had_errors && debug {
        eprintln!("[DEBUG] Warning: Some characters could not be decoded properly");
    }

    Ok((cow.into_owned(), detected))
}

/// 從 stdin 讀取並轉換為 UTF-8 字符串
pub fn read_stdin_with_encoding(
    user_encoding: Option<&'static Encoding>,
    debug: bool,
) -> Result<(String, DetectedEncoding)> {
    let mut bytes = Vec::new();

    // 讀取 stdin，處理 Ctrl+C 中斷
    match std::io::stdin().read_to_end(&mut bytes) {
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {
            // Ctrl+C 被按下，正常退出
            std::process::exit(0);
        }
        Err(e) => return Err(e).context("Failed to read from stdin"),
    }

    let detected = detect_encoding(&bytes, user_encoding, debug);

    if debug {
        eprintln!(
            "[DEBUG] Detected encoding: {} (confidence: {:?})",
            detected.encoding.name(),
            detected.confidence
        );
    }

    let (cow, _encoding_used, had_errors) = detected.encoding.decode(&bytes);

    if had_errors && debug {
        eprintln!("[DEBUG] Warning: Some characters could not be decoded properly");
    }

    Ok((cow.into_owned(), detected))
}

/// 檢測文件編碼（優先級：UTF-8/BOM > 用戶指定 > 系統編碼）
fn detect_encoding(
    bytes: &[u8],
    user_encoding: Option<&'static Encoding>,
    debug: bool,
) -> DetectedEncoding {
    // 1. 檢查 BOM
    if let Some((encoding, _bom_length)) = Encoding::for_bom(bytes) {
        if debug {
            eprintln!("[DEBUG] BOM detected: {}", encoding.name());
        }
        return DetectedEncoding {
            encoding,
            confidence: EncodingConfidence::Certain,
        };
    }

    // 2. 檢查是否為有效的 UTF-8
    if std::str::from_utf8(bytes).is_ok() {
        if debug {
            eprintln!("[DEBUG] Valid UTF-8 detected");
        }
        return DetectedEncoding {
            encoding: encoding_rs::UTF_8,
            confidence: EncodingConfidence::High,
        };
    }

    // 3. 使用用戶指定的編碼
    if let Some(encoding) = user_encoding {
        if debug {
            eprintln!("[DEBUG] Using user-specified encoding: {}", encoding.name());
        }
        return DetectedEncoding {
            encoding,
            confidence: EncodingConfidence::Certain,
        };
    }

    // 4. 回退到系統編碼
    let system_encoding = get_system_encoding();
    if debug {
        eprintln!(
            "[DEBUG] Falling back to system encoding: {}",
            system_encoding.name()
        );
    }
    DetectedEncoding {
        encoding: system_encoding,
        confidence: EncodingConfidence::Low,
    }
}

/// 獲取系統預設編碼
#[cfg(target_os = "windows")]
fn get_system_encoding() -> &'static Encoding {
    use winapi::um::winnls::GetACP;

    let code_page = unsafe { GetACP() };

    match code_page {
        936 => encoding_rs::GBK, // 簡體中文
        950 => {
            // 繁體中文 Big5
            Encoding::for_label(b"big5").unwrap_or(encoding_rs::UTF_8)
        }
        932 => encoding_rs::SHIFT_JIS,     // 日文
        1252 => encoding_rs::WINDOWS_1252, // 西歐語言
        65001 => encoding_rs::UTF_8,       // UTF-8
        _ => encoding_rs::UTF_8,           // 預設 UTF-8
    }
}

#[cfg(not(target_os = "windows"))]
fn get_system_encoding() -> &'static Encoding {
    // Unix 系統通常使用 UTF-8
    // 可以通過環境變量 LANG 來檢測，但大多數現代系統都是 UTF-8
    if let Ok(lang) = std::env::var("LANG") {
        if lang.to_lowercase().contains("utf-8") || lang.to_lowercase().contains("utf8") {
            return encoding_rs::UTF_8;
        }
    }

    // 預設使用 UTF-8
    encoding_rs::UTF_8
}

/// 解析編碼名稱為 Encoding
pub fn parse_encoding(enc_str: &str) -> Result<&'static Encoding> {
    match enc_str.to_lowercase().as_str() {
        "utf-8" | "utf8" => Ok(encoding_rs::UTF_8),
        "utf-16le" | "utf16le" => Ok(encoding_rs::UTF_16LE),
        "utf-16be" | "utf16be" => Ok(encoding_rs::UTF_16BE),
        "gbk" | "cp936" => Ok(encoding_rs::GBK),
        "shift-jis" | "shift_jis" | "sjis" => Ok(encoding_rs::SHIFT_JIS),
        "big5" | "cp950" => Encoding::for_label(b"big5")
            .ok_or_else(|| anyhow::anyhow!("Big5 encoding not supported")),
        "cp1252" | "windows-1252" => Ok(encoding_rs::WINDOWS_1252),
        "iso-8859-1" | "latin1" => Ok(encoding_rs::WINDOWS_1252), // 類似
        _ => {
            // 嘗試查找其他編碼
            Encoding::for_label(enc_str.as_bytes())
                .ok_or_else(|| anyhow::anyhow!("Unsupported encoding: {}", enc_str))
        }
    }
}

/// 列出所有支持的常用編碼
pub fn list_encodings() {
    println!("Supported encodings:");
    println!();
    println!("Unicode:");
    println!("  utf-8, utf-16le, utf-16be");
    println!();
    println!("Chinese:");
    println!("  gbk (Simplified Chinese)");
    println!("  big5 (Traditional Chinese)");
    println!();
    println!("Japanese:");
    println!("  shift-jis (Shift_JIS)");
    println!();
    println!("Western European:");
    println!("  windows-1252, iso-8859-1");
    println!();
    println!("Other:");
    println!("  Use any encoding name supported by encoding_rs");
    println!("  (e.g., euc-jp, iso-8859-2, koi8-r, etc.)");
}
