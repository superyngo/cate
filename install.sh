#!/usr/bin/env bash
# cate Remote Installation Script for Linux/macOS
# Usage: curl -fsSL https://raw.githubusercontent.com/superyngo/cate/main/install.sh | bash

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

print_success() { echo -e "${GREEN}$1${NC}"; }
print_info() { echo -e "${CYAN}$1${NC}"; }
print_error() { echo -e "${RED}$1${NC}"; }
print_warning() { echo -e "${YELLOW}$1${NC}"; }

# Configuration
APP_NAME="cate"
REPO="superyngo/cate"
INSTALL_DIR="$HOME/.local/bin"
BIN_PATH="$INSTALL_DIR/$APP_NAME"

# Detect OS and Architecture
detect_platform() {
    local os=$(uname -s | tr '[:upper:]' '[:lower:]')
    local arch=$(uname -m)

    case "$os" in
        linux*)
            OS="linux"
            ;;
        darwin*)
            OS="darwin"
            ;;
        *)
            print_error "Unsupported OS: $os"
            exit 1
            ;;
    esac

    case "$arch" in
        x86_64|amd64)
            ARCH="x86_64"
            ;;
        aarch64|arm64)
            ARCH="aarch64"
            ;;
        *)
            print_error "Unsupported architecture: $arch"
            exit 1
            ;;
    esac

    print_info "Detected platform: $OS-$ARCH"
}

# Get latest release from GitHub API
get_latest_release() {
    local api_url="https://api.github.com/repos/$REPO/releases/latest"

    print_info "Fetching latest release information..."

    if command -v curl >/dev/null 2>&1; then
        RELEASE_DATA=$(curl -fsSL "$api_url")
    elif command -v wget >/dev/null 2>&1; then
        RELEASE_DATA=$(wget -qO- "$api_url")
    else
        print_error "Error: Neither curl nor wget is available. Please install one of them."
        exit 1
    fi

    # Extract version tag
    VERSION=$(echo "$RELEASE_DATA" | grep '"tag_name":' | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/')

    if [ -z "$VERSION" ]; then
        print_error "Failed to get latest version"
        exit 1
    fi

    print_success "Latest version: $VERSION"
}

# Download and extract release
download_release() {
    # The release workflow creates files named:
    # - Linux: cate-linux-x86_64.tar.gz, cate-linux-aarch64.tar.gz, etc.
    # - macOS: cate-macos-x86_64.tar.gz, cate-macos-aarch64.tar.gz

    local os_name=""
    if [ "$OS" = "darwin" ]; then
        os_name="macos"
    else
        os_name="linux"
    fi

    local asset_name="$APP_NAME-$os_name-$ARCH.tar.gz"

    # Extract download URL from release data
    local download_url=$(echo "$RELEASE_DATA" | grep "\"browser_download_url\".*$asset_name" | sed -E 's/.*"browser_download_url": *"([^"]+)".*/\1/')

    if [ -z "$download_url" ]; then
        print_error "Could not find release asset: $asset_name"
        print_info "Available assets:"
        echo "$RELEASE_DATA" | grep '"name":' | sed -E 's/.*"name": *"([^"]+)".*/  - \1/'
        echo ""
        print_info "Looking for: $asset_name"
        exit 1
    fi

    print_info "Download URL: $download_url"
    print_info ""

    # Create temporary directory
    local temp_dir=$(mktemp -d)
    trap "rm -rf '$temp_dir'" EXIT

    # Download archive
    local archive_path="$temp_dir/$asset_name"
    print_info "Downloading $APP_NAME..."

    if command -v curl >/dev/null 2>&1; then
        curl -fsSL -o "$archive_path" "$download_url"
    else
        wget -qO "$archive_path" "$download_url"
    fi

    print_success "Downloaded successfully!"

    # Extract archive
    print_info "Extracting archive..."
    tar -xzf "$archive_path" -C "$temp_dir"

    # Find binary
    local binary=$(find "$temp_dir" -name "$APP_NAME" -type f | head -n 1)

    if [ ! -f "$binary" ]; then
        print_error "Could not find $APP_NAME binary in archive"
        exit 1
    fi

    # Create installation directory
    mkdir -p "$INSTALL_DIR"

    # Install binary
    print_info "Installing to: $INSTALL_DIR"
    cp "$binary" "$BIN_PATH"
    chmod +x "$BIN_PATH"

    print_success "Binary installed successfully!"
}

# Configure PATH
configure_path() {
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        print_warning "Warning: $INSTALL_DIR is not in your PATH"
        echo ""
        print_info "To add it to your PATH, add this line to your shell configuration file:"

        # Detect shell and suggest appropriate config file
        local shell_config=""
        if [ -n "$BASH_VERSION" ]; then
            shell_config="$HOME/.bashrc"
            print_info "  echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.bashrc"
            print_info "  source ~/.bashrc"
        elif [ -n "$ZSH_VERSION" ]; then
            shell_config="$HOME/.zshrc"
            print_info "  echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.zshrc"
            print_info "  source ~/.zshrc"
        else
            # Try to detect default shell
            local user_shell=$(basename "$SHELL")
            case "$user_shell" in
                bash)
                    shell_config="$HOME/.bashrc"
                    ;;
                zsh)
                    shell_config="$HOME/.zshrc"
                    ;;
                *)
                    shell_config="$HOME/.profile"
                    ;;
            esac
            print_info "  echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> $shell_config"
            print_info "  source $shell_config"
        fi

        echo ""

        # Auto-add to PATH (if running interactively)
        if [ -t 0 ]; then
            read -p "Do you want to add it to your PATH automatically? (y/N) " -n 1 -r
            echo ""
            if [[ $REPLY =~ ^[Yy]$ ]]; then
                if [ -n "$shell_config" ] && [ -f "$shell_config" ]; then
                    if ! grep -q "export PATH=\"\$HOME/.local/bin:\$PATH\"" "$shell_config"; then
                        echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$shell_config"
                        print_success "Added to $shell_config"
                        print_warning "Please run: source $shell_config"
                    else
                        print_info "PATH already configured in $shell_config"
                    fi
                else
                    print_warning "Could not detect shell config file. Please add manually."
                fi
            fi
        fi
    else
        print_info "Already in PATH"
    fi
}

# Installation function
install_cate() {
    print_info "=== cate Installation Script ==="
    echo ""

    detect_platform
    get_latest_release
    download_release
    configure_path

    echo ""
    print_success "Installation completed successfully!"
    echo ""
    print_info "Installed version: $VERSION"
    print_info "Installation path: $BIN_PATH"
    echo ""
    print_info "Usage:"
    print_info "  $APP_NAME <filename>       - Display file with auto-detected encoding"
    print_info "  $APP_NAME -e <encoding> <filename> - Specify encoding"
    print_info "  $APP_NAME -n <filename>    - Show line numbers"
    print_info "  $APP_NAME --help           - Show help"
    print_info "  $APP_NAME --version        - Show version"
    echo ""
    print_info "To uninstall, run:"
    print_info "  curl -fsSL https://raw.githubusercontent.com/$REPO/main/install.sh | bash -s uninstall"
}

# Uninstallation function
uninstall_cate() {
    print_info "=== cate Uninstallation Script ==="
    echo ""

    # Remove binary
    if [ -f "$BIN_PATH" ]; then
        print_info "Removing binary..."
        rm -f "$BIN_PATH"
        print_success "Binary removed"
    else
        print_info "Binary not found (already removed?)"
    fi

    echo ""
    print_success "Uninstallation completed!"
    print_info ""
    print_info "Note: This script does not automatically remove PATH modifications."
    print_info "If you added $INSTALL_DIR to your PATH, you may want to remove it manually."
}

# Main
ACTION="${1:-install}"

case "$ACTION" in
    install)
        install_cate
        ;;
    uninstall)
        uninstall_cate
        ;;
    *)
        print_error "Unknown action: $ACTION"
        print_info "Usage: $0 [install|uninstall]"
        exit 1
        ;;
esac
