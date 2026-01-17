#!/bin/bash
#
# cs - Claude Code Session Manager
# Cross-platform installation script
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/bikramtuladhar/claude-code-resumer/main/install.sh | bash
#
# Supported platforms:
#   - macOS (Intel & Apple Silicon)
#   - Linux (x64 & ARM64)
#   - FreeBSD (x64)
#   - Android Termux (ARM64, ARM32, x64)
#   - iOS iSH (i686)
#

set -e

REPO="bikramtuladhar/claude-code-resumer"
BINARY_NAME="cs"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_banner() {
    echo -e "${BLUE}"
    echo "╔═══════════════════════════════════════════════════════════╗"
    echo "║           cs - Claude Code Session Manager                ║"
    echo "║              Cross-Platform Installer                     ║"
    echo "╚═══════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
}

info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

# Detect the operating system
detect_os() {
    local os=""

    if [ -n "$TERMUX_VERSION" ] || [ -d "/data/data/com.termux" ]; then
        os="android"
    elif [ -f "/proc/ish/version" ] || (uname -a 2>/dev/null | grep -qi "ish"); then
        os="ish"
    else
        case "$(uname -s)" in
            Darwin)
                os="macos"
                ;;
            Linux)
                os="linux"
                ;;
            FreeBSD)
                os="freebsd"
                ;;
            *)
                error "Unsupported operating system: $(uname -s)"
                ;;
        esac
    fi

    echo "$os"
}

# Detect the CPU architecture
detect_arch() {
    local arch=""
    local machine
    machine="$(uname -m)"

    case "$machine" in
        x86_64|amd64)
            arch="x64"
            ;;
        aarch64|arm64)
            arch="arm64"
            ;;
        armv7l|armv7|arm)
            arch="arm32"
            ;;
        i686|i386|i586)
            arch="i686"
            ;;
        *)
            error "Unsupported architecture: $machine"
            ;;
    esac

    echo "$arch"
}

# Get the appropriate binary name for the platform
get_binary_name() {
    local os="$1"
    local arch="$2"
    local binary=""

    case "$os" in
        macos)
            case "$arch" in
                arm64) binary="cs-macos-arm64" ;;
                x64)   binary="cs-macos-intel" ;;
                *)     error "Unsupported macOS architecture: $arch" ;;
            esac
            ;;
        linux)
            case "$arch" in
                x64)   binary="cs-linux-x64" ;;
                arm64) binary="cs-linux-arm64" ;;
                i686)  binary="cs-linux-i686-musl" ;;
                *)     error "Unsupported Linux architecture: $arch" ;;
            esac
            ;;
        freebsd)
            case "$arch" in
                x64) binary="cs-freebsd-x64" ;;
                *)   error "Unsupported FreeBSD architecture: $arch" ;;
            esac
            ;;
        android)
            case "$arch" in
                arm64) binary="cs-android-arm64" ;;
                arm32) binary="cs-android-arm32" ;;
                x64)   binary="cs-android-x64" ;;
                *)     error "Unsupported Android architecture: $arch" ;;
            esac
            ;;
        ish)
            # iSH is x86 Linux emulator
            binary="cs-linux-i686-musl"
            ;;
        *)
            error "Unsupported OS: $os"
            ;;
    esac

    echo "$binary"
}

# Get the installation directory
get_install_dir() {
    local os="$1"
    local install_dir=""

    case "$os" in
        android)
            # Termux uses $PREFIX/bin
            install_dir="${PREFIX:-/data/data/com.termux/files/usr}/bin"
            ;;
        ish)
            # iSH typically uses /usr/local/bin but may need different paths
            if [ -w "/usr/local/bin" ]; then
                install_dir="/usr/local/bin"
            elif [ -w "$HOME/.local/bin" ]; then
                install_dir="$HOME/.local/bin"
            else
                install_dir="$HOME/bin"
            fi
            ;;
        *)
            # Standard Unix paths
            if [ -w "/usr/local/bin" ]; then
                install_dir="/usr/local/bin"
            elif [ -w "$HOME/.local/bin" ]; then
                install_dir="$HOME/.local/bin"
            else
                install_dir="$HOME/bin"
            fi
            ;;
    esac

    echo "$install_dir"
}

# Check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Download file using curl or wget
download_file() {
    local url="$1"
    local output="$2"

    if command_exists curl; then
        curl -fsSL "$url" -o "$output"
    elif command_exists wget; then
        wget -q "$url" -O "$output"
    else
        error "Neither curl nor wget found. Please install one of them."
    fi
}

# Get the latest release version
get_latest_version() {
    local version=""
    local api_url="https://api.github.com/repos/${REPO}/releases/latest"

    if command_exists curl; then
        version=$(curl -fsSL "$api_url" 2>/dev/null | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')
    elif command_exists wget; then
        version=$(wget -qO- "$api_url" 2>/dev/null | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')
    fi

    if [ -z "$version" ]; then
        error "Failed to fetch latest version. Please check your internet connection."
    fi

    echo "$version"
}

# Main installation function
main() {
    print_banner

    # Detect platform
    info "Detecting platform..."
    local os
    local arch
    os=$(detect_os)
    arch=$(detect_arch)

    info "OS: $os"
    info "Architecture: $arch"

    # Get binary name
    local binary_name
    binary_name=$(get_binary_name "$os" "$arch")
    info "Binary: $binary_name"

    # Get latest version
    info "Fetching latest version..."
    local version
    version=$(get_latest_version)
    info "Latest version: $version"

    # Build download URL
    local download_url="https://github.com/${REPO}/releases/download/${version}/${binary_name}"
    info "Download URL: $download_url"

    # Get install directory
    local install_dir
    install_dir=$(get_install_dir "$os")

    # Create install directory if it doesn't exist
    if [ ! -d "$install_dir" ]; then
        info "Creating directory: $install_dir"
        mkdir -p "$install_dir"
    fi

    # Download binary
    info "Downloading $binary_name..."
    local temp_file
    temp_file=$(mktemp)

    if ! download_file "$download_url" "$temp_file"; then
        rm -f "$temp_file"
        error "Failed to download binary. Please check your internet connection."
    fi

    # Install binary
    local install_path="${install_dir}/${BINARY_NAME}"

    info "Installing to $install_path..."

    # Try to install, use sudo if needed
    if [ -w "$install_dir" ]; then
        mv "$temp_file" "$install_path"
        chmod +x "$install_path"
    else
        info "Requesting sudo access to install to $install_dir..."
        sudo mv "$temp_file" "$install_path"
        sudo chmod +x "$install_path"
    fi

    # Verify installation
    if [ -x "$install_path" ]; then
        success "cs has been installed to $install_path"
        echo ""

        # Check if install_dir is in PATH
        if ! echo "$PATH" | tr ':' '\n' | grep -qx "$install_dir"; then
            warn "Note: $install_dir is not in your PATH"
            echo ""
            echo "Add it to your PATH by running:"
            echo ""
            case "$os" in
                android)
                    echo "  # Already in PATH for Termux"
                    ;;
                *)
                    echo "  echo 'export PATH=\"$install_dir:\$PATH\"' >> ~/.bashrc"
                    echo "  source ~/.bashrc"
                    echo ""
                    echo "Or for zsh:"
                    echo "  echo 'export PATH=\"$install_dir:\$PATH\"' >> ~/.zshrc"
                    echo "  source ~/.zshrc"
                    ;;
            esac
            echo ""
        fi

        # Show version
        echo "Installed version:"
        "$install_path" --version 2>/dev/null || echo "  $version"
        echo ""

        # Usage hint
        echo -e "${GREEN}Quick start:${NC}"
        echo "  cd your-project"
        echo "  cs              # Create/resume session for current folder+branch"
        echo ""
        echo "For more info: cs --help"
        echo ""
    else
        error "Installation failed. Binary is not executable."
    fi
}

# Run main function
main "$@"
