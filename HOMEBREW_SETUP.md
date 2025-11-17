# Setting Up Homebrew Tap

To make smart-hooks installable via `brew install smart-hooks`, follow these steps:

## 1. Create a Tap Repository

```bash
# Create a new repository named 'homebrew-smart-hooks'
gh repo create CaptainEmpower/homebrew-smart-hooks --public
cd ~/Code/GitHub
git clone https://github.com/CaptainEmpower/homebrew-smart-hooks.git
cd homebrew-smart-hooks
```

## 2. Add the Formula

```bash
# Copy the formula to the tap
mkdir -p Formula
cp /Users/sr/Code/GitHub/smart-hooks/Formula/smart-hooks.rb Formula/

# Create initial commit
git add Formula/smart-hooks.rb
git commit -m "feat: add smart-hooks formula"
git push origin main
```

## 3. Update Formula for Release

Before publishing, update the formula with a real release:

```ruby
class SmartHooks < Formula
  desc "Intelligent git hooks that understand your code changes and automatically select the right tests to run"
  homepage "https://github.com/CaptainEmpower/smart-hooks"
  url "https://github.com/CaptainEmpower/smart-hooks/archive/refs/tags/v0.2.0.tar.gz"
  sha256 "ACTUAL_SHA256_HASH_HERE"  # Generate after creating release
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    system "#{bin}/smart-test-selector", "--help"
  end
end
```

## 4. Create Release and Update SHA256

```bash
# In the main smart-hooks repository
git tag v0.2.0
git push origin v0.2.0

# Generate SHA256 for the release
curl -sSL https://github.com/CaptainEmpower/smart-hooks/archive/refs/tags/v0.2.0.tar.gz | sha256sum

# Update the formula with the real SHA256
```

## 5. Test the Tap

```bash
# Test locally
brew install --build-from-source Formula/smart-hooks.rb

# Test the tap
brew tap CaptainEmpower/smart-hooks
brew install smart-hooks
```

## 6. Usage After Setup

Once the tap is properly set up, users can install with:

```bash
brew tap CaptainEmpower/smart-hooks
brew install smart-hooks
```

## Current Status

✅ **Installed successfully via cargo**
- Main binary: `smart-test-selector`
- BDD binary: `claude-bdd-selector` (requires claude-ai feature)
- All 6 binaries available in `~/.cargo/bin/`

🚧 **Homebrew tap setup needed**
- Formula created: `/Formula/smart-hooks.rb`
- Need separate tap repository for proper brew installation
- Follow steps above to complete Homebrew integration