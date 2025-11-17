# cate

A lightweight CLI tool to display file contents with encoding support - like `cat` with better encoding handling.

## Features

- 🌍 **Multi-encoding Support**: UTF-8, GBK, Big5, Shift-JIS, and more
- 🔍 **Smart Encoding Detection**: Auto-detects UTF-8/BOM, falls back gracefully
- 📝 **Line Numbers**: Optional line number display
- 🚀 **Lightweight**: Optimized for minimal binary size
- 🔧 **Debug Mode**: Detailed encoding detection information
- 📋 **Stdin Support**: Read from pipes and redirects

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
# Display file with auto-detected encoding
cate file.txt

# Specify encoding
cate file.txt -e gbk

# Show line numbers
cate file.txt -n

# Combine options
cate file.txt -e big5 -n

# Read from stdin
echo "Hello World" | cate

# Debug mode (show encoding detection info)
cate file.txt --debug

# List all supported encodings
cate --list-encodings
```

## Options

```
-h, --help              Show help message
-v, --version           Show version information
-e, --encoding <ENC>    Specify encoding (utf-8, gbk, big5, shift-jis, etc.)
-n, --number            Show line numbers
--debug                 Enable debug mode
--list-encodings        List all supported encodings
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

## Examples

### Display GBK-encoded file
```bash
cate chinese_file.txt -e gbk
```

### Show Big5 file with line numbers
```bash
cate traditional_chinese.txt -e big5 -n
```

### Debug encoding detection
```bash
cate mystery_file.txt --debug
```

Output:
```
[DEBUG] Reading file: "mystery_file.txt"
[DEBUG] Valid UTF-8 detected
[DEBUG] Detected encoding: UTF-8 (confidence: High)
[DEBUG] Final encoding: UTF-8 (confidence: High)
[DEBUG] Content length: 1234 bytes
[DEBUG] Line count: 42
[DEBUG] ---
... file content ...
```

### Use with pipes
```bash
# Convert encoding and display
iconv -f gbk -t utf-8 input.txt | cate

# Display with line numbers
cat file.txt | cate -n
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

## Why cate?

The name "cate" combines "cat" (the Unix command) with "encoding", making it easy to remember and type.

### Advantages over regular `cat`:
- ✅ Automatic encoding detection and conversion
- ✅ Support for non-UTF-8 files (GBK, Big5, Shift-JIS, etc.)
- ✅ Debug mode for troubleshooting encoding issues
- ✅ Built-in line numbering

## License

MIT License - see LICENSE file for details

## Author

wen

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.
