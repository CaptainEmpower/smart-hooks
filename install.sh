#!/bin/bash
set -e

# Smart Hooks Installation Script for macOS/Linux
# Usage: curl -sSL https://raw.githubusercontent.com/CaptainEmpower/smart-hooks/main/install.sh | bash

REPO="CaptainEmpower/smart-hooks"
BIN_DIR="$HOME/.local/bin"
LATEST_RELEASE_URL="https://api.github.com/repos/$REPO/releases/latest"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if cargo is installed
check_cargo() {
    if ! command -v cargo &> /dev/null; then
        print_error "Cargo (Rust) is not installed. Please install Rust first:"
        echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        exit 1
    fi
}

# Install via cargo
install_via_cargo() {
    print_status "Installing smart-hooks via cargo..."
    cargo install --git https://github.com/$REPO
    print_status "✅ Installation complete!"
    print_status "smart-hooks binaries are now available:"
    echo "  • smart-test-selector"
    echo "  • claude-bdd-selector"
}

# Install via Homebrew (if available)
install_via_homebrew() {
    if command -v brew &> /dev/null; then
        print_status "Homebrew detected. You can also install via:"
        echo "  brew tap CaptainEmpower/smart-hooks"
        echo "  brew install smart-hooks"
        echo ""
        print_warning "For now, installing via cargo..."
    fi
}

# Create bin directory if it doesn't exist
mkdir -p "$BIN_DIR"

# Check prerequisites
check_cargo

# Show installation options
install_via_homebrew

# Install via cargo
install_via_cargo

# Add to PATH instructions
if [[ ":$PATH:" != *":$BIN_DIR:"* ]]; then
    print_warning "Make sure $BIN_DIR is in your PATH:"
    echo "  echo 'export PATH=\"$BIN_DIR:\$PATH\"' >> ~/.bashrc"
    echo "  echo 'export PATH=\"$BIN_DIR:\$PATH\"' >> ~/.zshrc"
fi

print_status "🎉 Smart Hooks installation completed!"
print_status "Run 'smart-test-selector --help' to get started"