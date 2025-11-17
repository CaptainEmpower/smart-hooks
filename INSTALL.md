# Installation Guide

## macOS Installation

### Option 1: Homebrew (Recommended)

```bash
# Add the tap
brew tap CaptainEmpower/smart-hooks

# Install smart-hooks
brew install smart-hooks
```

### Option 2: One-line Install Script

```bash
curl -sSL https://raw.githubusercontent.com/CaptainEmpower/smart-hooks/main/install.sh | bash
```

### Option 3: Manual Cargo Install

```bash
# Requires Rust/Cargo to be installed
cargo install --git https://github.com/CaptainEmpower/smart-hooks
```

## Linux Installation

### Option 1: Install Script

```bash
curl -sSL https://raw.githubusercontent.com/CaptainEmpower/smart-hooks/main/install.sh | bash
```

### Option 2: Cargo Install

```bash
# Install Rust first if not already installed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install smart-hooks
cargo install --git https://github.com/CaptainEmpower/smart-hooks
```

## Windows Installation

### Option 1: Cargo Install (WSL2 Recommended)

```powershell
# Install Rust first
# Download from: https://rustup.rs/

# Install smart-hooks
cargo install --git https://github.com/CaptainEmpower/smart-hooks
```

## Verification

After installation, verify that smart-hooks is working:

```bash
# Check main binary
smart-test-selector --help

# Check BDD selector
claude-bdd-selector --help
```

## Available Binaries

- **`smart-test-selector`** - Main intelligent test selector
- **`claude-bdd-selector`** - BDD feature selector with Claude AI integration

## Prerequisites

- **Rust 1.70+** (for cargo install method)
- **Git** (for repository integration)

## Updating

### Homebrew
```bash
brew upgrade smart-hooks
```

### Cargo
```bash
cargo install --git https://github.com/CaptainEmpower/smart-hooks --force
```

## Uninstalling

### Homebrew
```bash
brew uninstall smart-hooks
```

### Cargo
```bash
cargo uninstall smart-hooks
```