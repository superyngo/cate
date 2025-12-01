# cate

A lightweight CLI tool to display file contents with encoding support and syntax highlighting - like `cat` but better.

## Features

- 🎨 **Syntax Highlighting**: Automatic syntax highlighting for 219 languages
- 🌍 **Multi-encoding Support**: UTF-8, GBK, Big5, Shift-JIS, and more
- 🔍 **Smart Detection**: Auto-detects file type and encoding
- 🎭 **Multiple Themes**: 7 built-in color themes
- 📝 **Line Numbers**: Optional line number display
- 🚀 **Lightweight & Fast**: Only 2.1 MB binary size with streaming architecture
- ⚡ **High Performance**: Instant output for large files with line-by-line processing
- 🔧 **Debug Mode**: Detailed encoding detection information
- 📋 **Stdin Support**: Read from pipes and redirects with language specification
- 🎯 **Manual Language Selection**: Override auto-detection with `-l/--language` flag

## Installation

### Quick Install (Recommended)

#### Linux/macOS
```bash
curl -fsSL https://raw.githubusercontent.com/superyngo/cate/main/install.sh | bash
```

#### Windows (PowerShell)
```powershell
irm https://raw.githubusercontent.com/superyngo/cate/main/install.ps1 | iex
```

The installation script will:
- Automatically detect your platform and architecture
- Download the latest release from GitHub
- Install the binary to the appropriate location
- Add it to your PATH automatically

### Manual Installation

#### From Binary Releases

Download the appropriate binary for your platform from the [releases page](https://github.com/superyngo/cate/releases).

##### Linux/macOS
```bash
# Extract the tarball
tar xzf cate-*.tar.gz

# Move to PATH (optional)
sudo mv cate /usr/local/bin/

# Make executable (if needed)
chmod +x /usr/local/bin/cate
```

##### Windows
Simply download `cate-windows-x86_64.exe` and rename it to `cate.exe`. Add it to your PATH for easy access.

### From Source

```bash
# Clone the repository
git clone https://github.com/superyngo/cate.git
cd cate

# Build with cargo
cargo build --release

# Binary will be in target/release/cate (or cate.exe on Windows)
```

## Uninstallation

If you installed using the installation scripts, you can uninstall with:

### Linux/macOS
```bash
curl -fsSL https://raw.githubusercontent.com/superyngo/cate/main/install.sh | bash -s uninstall
```

### Windows (PowerShell)
```powershell
irm https://raw.githubusercontent.com/superyngo/cate/main/install.ps1 | iex -Uninstall
```

For manual installations, simply remove the binary from your system.

## Usage

```bash
# Display file with syntax highlighting (default)
cate file.rs

# Display multiple files
cate file1.rs file2.py file3.js

# Show line numbers with highlighting
cate file.rs -n

# Specify encoding
cate file.txt -e gbk

# Use different theme
cate file.py --theme "Solarized (dark)"

# Disable syntax highlighting
cate file.txt --no-highlight

# Read from stdin with language specification
cat script.sh | cate -l bash
echo 'fn main() { println!("Hello"); }' | cate -l rust

# Override file extension detection
cate -l python file.txt

# List available themes
cate --list-themes

# List supported languages
cate --list-syntaxes

# List supported encodings
cate --list-encodings

# Debug mode (show encoding detection info)
cate file.txt --debug
```

## Options

```
-h, --help              Show help message
-v, --version           Show version information
-e, --encoding <ENC>    Specify encoding (utf-8, gbk, big5, shift-jis, etc.)
-n, --number            Show line numbers
--debug                 Enable debug mode
--list-encodings        List all supported encodings

Syntax Highlighting:
--no-highlight          Disable syntax highlighting
--theme <THEME>         Set color theme (default: base16-eighties.dark)
-l, --language <LANG>   Specify syntax language (e.g., rust, python, js)
--list-themes           List all available themes
--list-syntaxes         List all supported languages
```

## Encoding Detection

The tool uses the following priority for encoding detection:

1. **UTF-8/BOM**: If file has BOM or is valid UTF-8
2. **User Specified**: Encoding specified with `-e` flag
3. **System Encoding**: Falls back to system default encoding

## Supported Encodings

### Unicode
- UTF-8, UTF-16LE, UTF-16BE

### Chinese
- GBK (Simplified Chinese)
- Big5 (Traditional Chinese)

### Japanese
- Shift-JIS

### Western European
- Windows-1252, ISO-8859-1

### Others
Any encoding supported by the `encoding_rs` crate (e.g., EUC-JP, ISO-8859-2, KOI8-R, etc.)

## Supported Languages

Syntax highlighting is supported for 219 languages including:

### Programming Languages
- Rust, Python, JavaScript, TypeScript
- Go, C, C++, Java, C#

### Scripting
- Bash, PowerShell, Batch File
- Makefile, Dockerfile

### Data & Markup
- JSON, YAML, TOML, XML
- HTML, CSS, Markdown

### Themes
- base16-eighties.dark (default)
- Solarized (dark/light)
- InspiredGitHub
- base16-ocean.dark
- base16-mocha.dark
- And more...

## Examples

### Syntax Highlighting

```bash
# Display Rust file with syntax highlighting
cate src/main.rs

# Use a different theme
cate app.py --theme "Solarized (dark)"

# Show line numbers with highlighting
cate config.toml -n

# Display multiple source files
cate src/*.rs
```

### Encoding Support

```bash
# Display GBK-encoded file
cate chinese_file.txt -e gbk

# Show Big5 file with line numbers
cate traditional_chinese.txt -e big5 -n

# Combine encoding and highlighting
cate source_code.py -e gbk -n
```

### Advanced Usage

```bash
# Debug encoding detection
cate mystery_file.txt --debug

# Use with pipes and specify language
cat unknown_script | cate -l python
echo 'fn main() { println!("Test"); }' | cate -l rust

# Language specification (case-insensitive, supports extensions)
cat script | cate -l Rust    # Works with any case
cat script | cate -l rs      # Works with file extension
cat script | cate -l js      # JavaScript by extension

# Override file extension detection
cate config.txt -l yaml      # Treat .txt as YAML

# Convert encoding and display with highlighting
iconv -f gbk -t utf-8 input.py | cate -l python

# Disable highlighting for plain text
cate log.txt --no-highlight
```

## Building

### Development Build
```bash
cargo build
```

### Release Build (Optimized)
```bash
cargo build --release
```

The release build is heavily optimized for size using:
- LTO (Link-Time Optimization)
- Size optimization (`opt-level = "z"`)
- Symbol stripping
- Single codegen unit

## Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture
```

## Performance

**cate** uses a streaming architecture inspired by [bat](https://github.com/sharkdp/bat) for optimal performance:

- ⚡ **Line-by-line processing**: Files are processed and output incrementally
- 🚀 **Instant output**: First lines appear immediately, even for large files
- 💾 **Low memory usage**: Only buffers single lines, not entire files
- 🎯 **Stateful highlighting**: Correctly handles multi-line syntax (comments, strings, etc.)
- 🛡️ **Long line protection**: Automatically skips highlighting for lines >16KB

### Performance Example
```bash
# 10,000 line file processes in ~0.3 seconds with instant first-line output
cate large_file.rs

# Use --no-highlight to disable syntax highlighting for faster plain text output
cate large_file.txt --no-highlight
```

## Why cate?

The name "cate" combines "cat" (the Unix command) with "encoding", making it easy to remember and type.

### Advantages over regular `cat`:
- ✅ **Syntax highlighting** for 219 programming languages
- ✅ **7 beautiful themes** with true color support
- ✅ **Streaming architecture** for instant output on large files
- ✅ Automatic encoding detection and conversion
- ✅ Support for non-UTF-8 files (GBK, Big5, Shift-JIS, etc.)
- ✅ Manual language selection with `-l` flag
- ✅ Debug mode for troubleshooting encoding issues
- ✅ Built-in line numbering
- ✅ Only 2.1 MB binary size

## License

MIT License - see LICENSE file for details

## Third-Party Resources

This project uses syntax definitions from the [bat](https://github.com/sharkdp/bat) project, which are licensed under MIT License / Apache License 2.0. The syntax definitions are originally derived from Sublime Text packages (MIT License).

For complete third-party license information, see [THIRD-PARTY-LICENSES.md](THIRD-PARTY-LICENSES.md).

### Acknowledgements

- **bat project** - For the excellent syntax definition collection
- **Sublime Text community** - For maintaining the original syntax definitions
- **syntect** - For the syntax highlighting engine

## Author

wen

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.
