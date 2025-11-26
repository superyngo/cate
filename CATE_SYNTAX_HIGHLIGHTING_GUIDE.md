# cate 專案語法高亮實作指引

## 專案概述

**cate** 是一個輕量級的 `cat` 替代品，專注於多編碼支援。本指引將協助你為 cate 添加語法高亮功能。

**當前狀態：**
- 程式碼量：384 行
- 核心功能：編碼檢測、文字輸出、行號顯示
- 語法高亮：無

**目標狀態：**
- 新增語法高亮功能
- 保持輕量特性
- 支援 20+ 種常用語言
- 二進位大小控制在 3MB 以內

---

## 一、實作方案總覽

### 方案選擇：直接依賴 syntect

**理由：**
- ✅ 實作簡單，侵入性最小
- ✅ 與 bat 使用相同品質的高亮引擎
- ✅ 一次性渲染場景，無需複雜狀態管理
- ✅ 2-4 小時即可完成

**技術棧：**
- `syntect 5.3.0` - 語法高亮引擎
- 自訂語法集 - 僅包含常用語言，減少二進位大小

---

## 二、建議保留的語言清單

### 完整語言列表（推薦）

為了平衡功能性與二進位大小，建議保留以下語言：

#### **系統程式語言（9 種）**
- Rust (.rs)
- Python (.py)
- JavaScript (.js)
- TypeScript (.ts)
- Go (.go)
- C (.c, .h)
- C++ (.cpp, .hpp, .cc, .cxx)
- Java (.java)
- C# (.cs)

#### **Shell 腳本（6 種）**
- Bash (.sh, .bash)
- PowerShell (.ps1, .psm1)
- Batch (.bat, .cmd)
- Shell Script (.zsh, .fish)
- Makefile (Makefile, .mk)
- Dockerfile (Dockerfile)

#### **標記/資料語言（8 種）**
- JSON (.json)
- YAML (.yml, .yaml)
- TOML (.toml)
- XML (.xml)
- HTML (.html, .htm)
- CSS (.css, .scss, .sass)
- Markdown (.md, .markdown)
- INI (.ini, .conf, .cfg)

#### **資料庫（2 種）**
- SQL (.sql)
- GraphQL (.graphql, .gql)

#### **其他常用（3 種）**
- Git Config (.gitignore, .gitattributes)
- Log Files (.log)
- Plain Text (.txt)

**總計：28 種語言/格式**

### 預估二進位影響

| 配置 | 語法數量 | 預估增加大小 |
|------|---------|-------------|
| 最小集（5 語言） | Rust, Python, JS, C, C++ | ~800 KB |
| 推薦集（28 語言） | 完整清單如上 | ~2.0 MB |
| bat 完整集（200+ 語言） | 所有支援的語言 | ~3.5 MB |

**建議：使用推薦集（28 語言），平衡實用性與大小**

---

## 三、實作步驟

### Step 1: 更新依賴配置

**檔案：`Cargo.toml`**

```toml
[package]
name = "cate"
version = "0.2.0"  # 升版號
edition = "2021"

[dependencies]
encoding_rs = "0.8"
pico-args = "0.5"
anyhow = "1.0"

# 新增：語法高亮支援
syntect = { version = "5.3.0", default-features = false, features = ["parsing"] }
once_cell = "1.19"  # 用於延遲初始化

[target.'cfg(windows)'.dependencies]
winapi = { version = "0.3", features = ["winnls"] }

[profile.release]
opt-level = "z"         # 優化二進位大小
lto = true              # Link-Time Optimization
strip = true            # 移除符號表
codegen-units = 1       # 單一編譯單元
panic = "abort"         # 減少 panic 處理程式碼

[features]
default = ["syntax-highlighting"]
syntax-highlighting = []  # 可選功能 flag
```

### Step 2: 建立語法高亮模組

**新建檔案：`src/highlighter.rs`**

```rust
use anyhow::{Result, Context};
use once_cell::sync::Lazy;
use syntect::parsing::{SyntaxSet, SyntaxSetBuilder, SyntaxDefinition};
use syntect::highlighting::{ThemeSet, Theme, Style};
use syntect::easy::HighlightLines;
use syntect::util::as_24_bit_terminal_escaped;
use std::path::Path;

/// 全域語法集（延遲載入）
static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(|| {
    load_custom_syntax_set().unwrap_or_else(|_| SyntaxSet::load_defaults_newlines())
});

/// 全域主題集（延遲載入）
static THEME_SET: Lazy<ThemeSet> = Lazy::new(ThemeSet::load_defaults);

/// 載入自訂語法集（僅包含常用語言）
fn load_custom_syntax_set() -> Result<SyntaxSet> {
    // 如果有自訂的 syntaxes.bin，從那裡載入
    // 否則使用預設集
    Ok(SyntaxSet::load_defaults_newlines())
}

/// 語法高亮器
pub struct Highlighter {
    theme: Theme,
    true_color: bool,
}

impl Highlighter {
    /// 建立新的高亮器
    pub fn new(theme_name: Option<&str>, true_color: bool) -> Result<Self> {
        let theme_name = theme_name.unwrap_or("Monokai Extended");
        let theme = THEME_SET.themes.get(theme_name)
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

        // 逐行高亮
        let mut highlighter = HighlightLines::new(syntax, &self.theme);
        let mut output = String::with_capacity(content.len() + content.len() / 4);

        for line in content.lines() {
            let ranges: Vec<(Style, &str)> = highlighter
                .highlight_line(line, &SYNTAX_SET)
                .context("Failed to highlight line")?;

            let escaped = if self.true_color {
                as_24_bit_terminal_escaped(&ranges[..], false)
            } else {
                self.as_8bit_terminal_escaped(&ranges[..])
            };

            output.push_str(&escaped);
            output.push('\n');
        }

        Ok(output)
    }

    /// 檢測檔案的語法類型
    fn detect_syntax<'a>(&self, content: &str, file_path: Option<&Path>) -> &'a syntect::parsing::SyntaxReference {
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
            // 簡化的 RGB -> 256 色映射
            let fg = style.foreground;
            let color_code = self.rgb_to_ansi256(fg.r, fg.g, fg.b);
            output.push_str(&format!("\x1b[38;5;{}m{}\x1b[0m", color_code, text));
        }

        output
    }

    /// RGB 轉 ANSI 256 色
    fn rgb_to_ansi256(&self, r: u8, g: u8, b: u8) -> u8 {
        // 簡化映射：使用 216 色立方體 (16-231)
        let r = (r as u16 * 5 / 255) as u8;
        let g = (g as u16 * 5 / 255) as u8;
        let b = (b as u16 * 5 / 255) as u8;
        16 + 36 * r + 6 * g + b
    }

    /// 列出可用主題
    pub fn available_themes() -> Vec<String> {
        THEME_SET.themes.keys().cloned().collect()
    }

    /// 列出可用語法
    pub fn available_syntaxes() -> Vec<String> {
        SYNTAX_SET.syntaxes()
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
```

### Step 3: 更新 printer.rs

**修改檔案：`src/printer.rs`**

```rust
use crate::highlighter::{Highlighter, supports_true_color};
use std::path::Path;

/// 列印檔案內容
pub fn print_content(
    content: &str,
    show_line_numbers: bool,
    file_path: Option<&Path>,
    enable_highlighting: bool,
    theme: Option<&str>,
) {
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
        print_with_line_numbers(&processed_content);
    } else {
        print!("{}", processed_content);
    }
}

fn print_with_line_numbers(content: &str) {
    let lines: Vec<&str> = content.lines().collect();
    let total_lines = lines.len();
    let width = total_lines.to_string().len();

    for (i, line) in lines.iter().enumerate() {
        println!("{:>width$} {}", i + 1, line, width = width);
    }

    // 處理沒有結尾換行的檔案
    if !content.ends_with('\n') && !content.is_empty() {
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_numbers() {
        let content = "line1\nline2\nline3";
        print_with_line_numbers(content);
    }
}
```

### Step 4: 更新 main.rs

**修改檔案：`src/main.rs`**

```rust
mod encoder;
mod printer;
mod highlighter;  // 新增

use pico_args::Arguments;
use std::path::PathBuf;
use anyhow::{Result, Context};

struct Args {
    files: Vec<PathBuf>,
    encoding: Option<String>,
    show_line_numbers: bool,
    debug: bool,

    // 新增：語法高亮選項
    no_highlight: bool,      // --no-highlight: 停用語法高亮
    theme: Option<String>,   // --theme: 指定主題
    list_themes: bool,       // --list-themes: 列出可用主題
    list_syntaxes: bool,     // --list-syntaxes: 列出可用語法
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

        // 新增：列出主題
        if args.contains("--list-themes") {
            list_themes();
            std::process::exit(0);
        }

        // 新增：列出語法
        if args.contains("--list-syntaxes") {
            list_syntaxes();
            std::process::exit(0);
        }

        Ok(Args {
            encoding: args.opt_value_from_str(["-e", "--encoding"])?,
            show_line_numbers: args.contains(["-n", "--number"]),
            debug: args.contains("--debug"),

            // 新增選項
            no_highlight: args.contains("--no-highlight"),
            theme: args.opt_value_from_str("--theme")?,
            list_themes: false,
            list_syntaxes: false,

            files: args.finish(),
        })
    }
}

fn main() -> Result<()> {
    let args = Args::parse()?;

    // 如果沒有檔案且 stdin 不是 pipe，顯示幫助
    if args.files.is_empty() && atty::is(atty::Stream::Stdin) {
        print_help();
        return Ok(());
    }

    // 處理 stdin
    if args.files.is_empty() {
        let content = encoder::read_stdin_with_encoding(args.encoding.as_deref())?;

        printer::print_content(
            &content,
            args.show_line_numbers,
            None,
            !args.no_highlight,  // 預設啟用高亮
            args.theme.as_deref(),
        );

        return Ok(());
    }

    // 處理檔案
    for (i, file_path) in args.files.iter().enumerate() {
        if args.debug {
            eprintln!("Reading file: {}", file_path.display());
        }

        let content = encoder::read_file_with_encoding(file_path, args.encoding.as_deref())
            .context(format!("Failed to read file: {}", file_path.display()))?;

        printer::print_content(
            &content,
            args.show_line_numbers,
            Some(file_path.as_path()),
            !args.no_highlight,
            args.theme.as_deref(),
        );

        // 多個檔案間加分隔
        if i < args.files.len() - 1 {
            println!();
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
    println!();
    println!("SYNTAX HIGHLIGHTING:");
    println!("    --no-highlight          Disable syntax highlighting");
    println!("    --theme <THEME>         Set color theme (default: Monokai Extended)");
    println!("    --list-themes           List all available themes");
    println!("    --list-syntaxes         List all supported languages");
    println!();
    println!("EXAMPLES:");
    println!("    cate file.rs                    # Display Rust file with syntax highlighting");
    println!("    cate --theme 'Solarized (dark)' file.py");
    println!("    cate -n --no-highlight file.txt # Show line numbers without highlighting");
    println!("    cate -e gbk chinese.txt         # Specify GBK encoding");
    println!("    cat file.js | cate              # Read from stdin");
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
    println!("Supported languages:");
    let mut syntaxes = highlighter::Highlighter::available_syntaxes();
    syntaxes.sort();

    // 分類顯示
    println!("\nProgramming Languages:");
    for syntax in syntaxes.iter().filter(|s| {
        ["Rust", "Python", "JavaScript", "TypeScript", "Go", "C", "C++", "Java", "C#"]
            .contains(&s.as_str())
    }) {
        println!("  {}", syntax);
    }

    println!("\nScripting:");
    for syntax in syntaxes.iter().filter(|s| {
        ["Bash", "PowerShell", "Batch File", "Makefile", "Dockerfile"]
            .contains(&s.as_str())
    }) {
        println!("  {}", syntax);
    }

    println!("\nData & Markup:");
    for syntax in syntaxes.iter().filter(|s| {
        ["JSON", "YAML", "TOML", "XML", "HTML", "CSS", "Markdown"]
            .contains(&s.as_str())
    }) {
        println!("  {}", syntax);
    }
}
```

### Step 5: 建構與測試

```bash
# 1. 更新依賴
cd D:\Users\user\Documents\cate
cargo update

# 2. 編譯（開發模式）
cargo build

# 3. 測試基本功能
cargo run -- --help
cargo run -- --list-themes
cargo run -- --list-syntaxes

# 4. 測試語法高亮
cargo run -- src/main.rs
cargo run -- Cargo.toml
echo "fn main() { println!(\"test\"); }" | cargo run

# 5. 測試不同主題
cargo run -- --theme "Solarized (dark)" src/main.rs

# 6. 測試停用高亮
cargo run -- --no-highlight src/main.rs

# 7. Release 建構
cargo build --release

# 8. 檢查二進位大小
ls -lh target/release/cate.exe
```

---

## 四、進階優化（可選）

### 4.1 自訂語法集建構

如果想進一步減少二進位大小，可以建立自訂的語法集：

**新建檔案：`build.rs`**

```rust
use std::path::Path;
use syntect::parsing::SyntaxSetBuilder;
use syntect::dumps::dump_to_file;

fn main() {
    println!("cargo:rerun-if-changed=assets/syntaxes");

    // 只載入指定語法
    let mut builder = SyntaxSetBuilder::new();

    let syntaxes_dir = Path::new("assets/syntaxes");
    if syntaxes_dir.exists() {
        builder.add_from_folder(syntaxes_dir, true).unwrap();

        let syntax_set = builder.build();
        dump_to_file(&syntax_set, "assets/syntaxes.bin").unwrap();

        println!("Built custom syntax set with {} syntaxes", syntax_set.syntaxes().len());
    }
}
```

然後在 `Cargo.toml` 加入：

```toml
[build-dependencies]
syntect = "5.3.0"
```

### 4.2 功能 Feature Flag

允許使用者選擇是否編譯語法高亮：

```toml
[features]
default = ["syntax-highlighting"]
syntax-highlighting = ["syntect", "once_cell"]

[dependencies]
syntect = { version = "5.3.0", optional = true, default-features = false, features = ["parsing"] }
once_cell = { version = "1.19", optional = true }
```

在程式碼中使用條件編譯：

```rust
#[cfg(feature = "syntax-highlighting")]
mod highlighter;

#[cfg(feature = "syntax-highlighting")]
use highlighter::Highlighter;

// 在 printer.rs 中
pub fn print_content(
    content: &str,
    show_line_numbers: bool,
    file_path: Option<&Path>,
    #[cfg(feature = "syntax-highlighting")]
    enable_highlighting: bool,
    #[cfg(feature = "syntax-highlighting")]
    theme: Option<&str>,
) {
    #[cfg(feature = "syntax-highlighting")]
    let processed_content = if enable_highlighting {
        // ... 高亮邏輯
    } else {
        content.to_string()
    };

    #[cfg(not(feature = "syntax-highlighting"))]
    let processed_content = content.to_string();

    // ... 剩餘邏輯
}
```

編譯不含高亮的版本：

```bash
cargo build --release --no-default-features
```

### 4.3 快取優化

對於多次使用相同主題，可以快取 Highlighter：

```rust
use std::sync::Arc;
use once_cell::sync::OnceCell;

static DEFAULT_HIGHLIGHTER: OnceCell<Arc<Highlighter>> = OnceCell::new();

pub fn get_default_highlighter() -> Arc<Highlighter> {
    DEFAULT_HIGHLIGHTER
        .get_or_init(|| {
            Arc::new(Highlighter::new(None, supports_true_color()).unwrap())
        })
        .clone()
}
```

---

## 五、測試清單

實作完成後，請測試以下場景：

### 基本功能
- [ ] `cate --help` - 顯示幫助
- [ ] `cate --version` - 顯示版本
- [ ] `cate --list-themes` - 列出主題
- [ ] `cate --list-syntaxes` - 列出語法

### 語法高亮
- [ ] `cate file.rs` - Rust 檔案高亮
- [ ] `cate file.py` - Python 檔案高亮
- [ ] `cate file.js` - JavaScript 檔案高亮
- [ ] `cate Cargo.toml` - TOML 檔案高亮
- [ ] `cate package.json` - JSON 檔案高亮
- [ ] `cate script.sh` - Shell 腳本高亮
- [ ] `cate script.ps1` - PowerShell 高亮
- [ ] `cate Dockerfile` - Dockerfile 高亮
- [ ] `cate Makefile` - Makefile 高亮

### 選項組合
- [ ] `cate -n file.rs` - 行號 + 高亮
- [ ] `cate --no-highlight file.rs` - 停用高亮
- [ ] `cate --theme "Solarized (dark)" file.py` - 自訂主題
- [ ] `echo "fn main() {}" | cate` - stdin 高亮

### 編碼支援
- [ ] `cate -e gbk chinese.txt` - GBK 編碼 + 高亮
- [ ] `cate -e big5 traditional.txt` - Big5 編碼 + 高亮

### 錯誤處理
- [ ] `cate nonexistent.rs` - 檔案不存在
- [ ] `cate --theme "Invalid" file.rs` - 無效主題（應回退預設）
- [ ] `cate /dev/null` - 空檔案

---

## 六、效能與大小優化檢查

```bash
# 1. 檢查 Release 二進位大小
ls -lh target/release/cate.exe

# 目標：< 3 MB

# 2. 啟動速度測試
hyperfine "target/release/cate src/main.rs"

# 目標：< 100ms

# 3. 大檔案效能測試
seq 1 10000 > large.txt
hyperfine "target/release/cate large.txt"

# 4. 比較有無高亮的速度差異
hyperfine "target/release/cate --no-highlight src/main.rs" \
          "target/release/cate src/main.rs"
```

---

## 七、疑難排解

### 問題 1: 編譯錯誤 - syntect 版本不相容

**錯誤訊息：**
```
error: failed to select a version for `syntect`
```

**解決方案：**
```bash
cargo update -p syntect
cargo clean
cargo build
```

### 問題 2: 某些語言無法高亮

**檢查步驟：**
1. 確認語法名稱：`cate --list-syntaxes`
2. 檢查副檔名映射
3. 嘗試手動指定語法（未來功能）

### 問題 3: 終端顏色顯示異常

**可能原因：**
- 終端不支援 24-bit 色彩

**解決方案：**
```bash
# 設定環境變數強制 8-bit 模式
export COLORTERM=

# 或在程式碼中降級處理
```

### 問題 4: 二進位過大 (>5 MB)

**優化步驟：**
1. 確認 `Cargo.toml` 中的 release 設定
2. 執行 `strip target/release/cate.exe`
3. 使用 `cargo bloat --release` 分析大小
4. 考慮使用自訂語法集（見 4.1 節）

---

## 八、後續改進方向

完成基本實作後，可以考慮以下改進：

1. **自動色彩降級** - 根據終端能力自動選擇色彩模式
2. **主題檔案支援** - 允許使用者載入自訂 .tmTheme 檔案
3. **語法手動指定** - 新增 `--language` 參數
4. **分頁支援** - 整合 pager（類似 bat）
5. **Git 差異高亮** - 顯示修改行（類似 bat）
6. **設定檔** - `~/.caterc` 儲存預設選項
7. **效能優化** - 大檔案的增量渲染
8. **Windows 終端相容性** - 改善 Windows Console 支援

---

## 九、參考資源

- [syntect 官方文件](https://docs.rs/syntect/)
- [bat 原始碼](https://github.com/sharkdp/bat)
- [Sublime Text 語法定義格式](https://www.sublimetext.com/docs/syntax.html)
- [TextMate 語法](https://macromates.com/manual/en/language_grammars)

---

## 附錄 A：完整的語法對應表

| 語言 | 副檔名 | Syntect 語法名稱 |
|------|--------|-----------------|
| Rust | .rs | Rust |
| Python | .py | Python |
| JavaScript | .js | JavaScript |
| TypeScript | .ts | TypeScript |
| Go | .go | Go |
| C | .c, .h | C |
| C++ | .cpp, .hpp, .cc, .cxx | C++ |
| Java | .java | Java |
| C# | .cs | C# |
| Bash | .sh, .bash | Bash |
| PowerShell | .ps1 | PowerShell |
| Batch | .bat, .cmd | Batch File |
| JSON | .json | JSON |
| YAML | .yml, .yaml | YAML |
| TOML | .toml | TOML |
| XML | .xml | XML |
| HTML | .html, .htm | HTML |
| CSS | .css | CSS |
| Markdown | .md | Markdown |
| SQL | .sql | SQL |
| Makefile | Makefile | Makefile |
| Dockerfile | Dockerfile | Dockerfile |

---

## 附錄 B：推薦主題清單

**亮色主題：**
- Solarized (light)
- InspiredGitHub
- Monokai Extended Light

**暗色主題：**
- Monokai Extended (預設)
- Solarized (dark)
- base16-ocean.dark
- Dracula
- Nord
- OneHalfDark

---

**預估開發時間：2-4 小時**
**預估測試時間：1 小時**
**總計：3-5 小時**

祝實作順利！如遇問題請參考疑難排解章節。
