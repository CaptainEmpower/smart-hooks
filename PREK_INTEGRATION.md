# Prek Integration Guide

**Smart-hooks + Prek: Intelligent Pre-commit Hooks with Enterprise-Grade Infrastructure**

This document describes the complete integration between smart-hooks and [prek](https://github.com/j178/prek), providing users with both intelligent code analysis and mature pre-commit infrastructure.

## Overview

Smart-hooks implements a **hybrid CLI delegation approach** to integrate with prek v0.2.20, providing:

- ✅ **Zero coupling** - No local dependencies or workspace conflicts
- ✅ **Intelligent fallback** - Works perfectly with or without prek
- ✅ **Stable integration** - Uses prek@0.2.20 stable release
- ✅ **Best of both worlds** - Combines prek's maturity with smart-hooks' intelligence

## Architecture

### Hybrid CLI Delegation

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   smart-hooks   │───▶│ Prek Detection  │───▶│ CLI Delegation  │
│                 │    │                 │    │                 │
├─────────────────┤    ├─────────────────┤    ├─────────────────┤
│ Enhanced CLI    │    │ Auto-detect     │    │ With prek:      │
│ - run           │    │ prek --version  │    │   Execute prek  │
│ - install       │    │                 │    │                 │
│ - list          │    │ Graceful        │    │ Without prek:   │
│ - validate      │    │ Fallback        │    │   Smart hooks   │
│                 │    │                 │    │   Intelligence  │
├─────────────────┤    └─────────────────┘    └─────────────────┘
│ Smart Features  │
│ - test          │
│ - bdd           │
│ - analyze       │
│ - check         │
└─────────────────┘
```

### Integration Points

| Command | With Prek Available | Without Prek | Smart Fallback |
|---------|-------------------|--------------|----------------|
| `run` | Delegates to prek CLI | Smart analysis | ✅ Intelligent test selection |
| `install` | Uses prek installation | Creates git hooks | ✅ Smart pre-commit hook |
| `list` | Shows prek + smart hooks | Shows smart capabilities | ✅ Complete feature list |
| `validate` | Uses prek validation | Smart config validation | ✅ Configuration analysis |

## Installation Options

### Option 1: Full Integration (Recommended)

Get both prek's mature infrastructure and smart-hooks' intelligence:

```bash
# Install both tools
cargo install prek@0.2.20
cargo install smart-hooks

# Verify installation
prek --version    # Should show v0.2.20
smart-hooks --version
```

### Option 2: Smart-hooks Standalone

Get intelligent hooks without prek dependency:

```bash
# Install smart-hooks only
cargo install smart-hooks

# Still provides full functionality with intelligent fallbacks
smart-hooks --version
```

### Option 3: Development Setup

For development and testing:

```bash
# Clone and build
git clone https://github.com/CaptainEmpower/smart-hooks
cd smart-hooks
cargo build --release

# Optional: Install companion prek
cargo install prek@0.2.20
```

## Usage Guide

### Basic Commands

#### Hook Execution

```bash
# Run all applicable hooks
smart-hooks run

# Run specific hooks
smart-hooks run trailing-whitespace check-yaml

# Run on specific files
smart-hooks run --files src/main.rs src/lib.rs

# Run on all files
smart-hooks run --all-files
```

**Behavior:**
- **With prek**: Delegates to `prek run` with same arguments
- **Without prek**: Uses smart analysis to determine which tests to run
- **Fallback**: If prek fails, automatically falls back to smart analysis

#### Hook Installation

```bash
# Install pre-commit hooks
smart-hooks install pre-commit

# Install other hook types
smart-hooks install pre-push
```

**Behavior:**
- **With prek**: Uses prek's installation system with .pre-commit-config.yaml
- **Without prek**: Creates intelligent git hooks in `.git/hooks/`

#### Hook Discovery

```bash
# List available hooks
smart-hooks list
```

**Output with prek:**
```
📋 Available prek hooks:
  trailing-whitespace
  end-of-file-fixer
  check-yaml
  check-json
  [... more prek hooks]

🧠 Additional smart-hooks capabilities:
  🧠 smart-test-selector - Intelligent test selection
  🧠 bdd-feature-selector - AI-powered BDD feature selection  
  🧠 dependency-analyzer - Multi-language dependency analysis
  🧠 hotreload-optimizer - Hot-reload performance optimization
```

**Output without prek:**
```
⚠️  Prek not found. Showing smart-hooks capabilities:
📋 Smart-hooks capabilities:
  🧠 smart-test-selector - Intelligent test selection based on code changes
  🧠 bdd-feature-selector - AI-powered BDD feature selection
  🧠 dependency-analyzer - Multi-language dependency analysis
  🧠 hotreload-optimizer - Hot-reload performance optimization
```

#### Configuration Validation

```bash
# Validate configuration
smart-hooks validate
```

**With prek**: Validates `.pre-commit-config.yaml` using prek
**Without prek**: Validates smart-hooks configuration and project setup

### Smart-hooks Original Features

All original smart-hooks capabilities are preserved and enhanced:

#### Intelligent Test Selection
```bash
# Analyze specific files for test selection
smart-hooks test src/main.rs src/lib.rs

# Analyze all staged files
smart-hooks test $(git diff --cached --name-only --diff-filter=AM)
```

#### BDD Feature Analysis (requires claude-ai feature)
```bash
# AI-powered BDD feature selection
smart-hooks bdd src/core/payment.rs

# Example output:
# 🤖 Claude AI Analysis Results:
# 📋 Recommended Features: payment_validation.feature
# 🎯 Priority: high (confidence: 89%)
# 🏷️ Tags: @business-logic @critical @payment
```

#### Multi-language Analysis
```bash
# Analyze project dependencies
smart-hooks analyze --verbose

# Check conditional compilation
smart-hooks check
```

## Configuration

### Smart-hooks Configuration (config.toml)

Create `config.toml` in your project root:

```toml
enable_content_analysis = false

[bdd_structural_patterns]
core_business_logic = ["/core/", "/service/", "/domain/"]
application_logic = ["/apply/", "/strategy/", "/handler/"]
error_handling = ["/error", "/validate", "/types"]
api_interfaces = ["/api/", "/controller/", "/endpoint/"]

[bdd_pattern_tags]
"/core/" = ["@business-logic", "@critical"]
"/api/" = ["@api", "@external"]
"/error" = ["@error-handling", "@robustness"]
```

### Prek Configuration (.pre-commit-config.yaml)

When prek is available, you can use standard pre-commit configuration:

```yaml
repos:
  - repo: local
    hooks:
      - id: smart-test-selector
        name: Smart Test Selector
        entry: smart-hooks test
        language: system
        files: \.rs$
      - id: trailing-whitespace
        name: Trim Trailing Whitespace
        entry: trailing-whitespace-fixer
        language: system
        types: [text]
```

### Git Hook Integration (Without Prek)

When prek is not available, smart-hooks creates intelligent git hooks:

```bash
# Install smart git hooks
smart-hooks install pre-commit
```

This creates `.git/hooks/pre-commit`:
```bash
#!/bin/sh
# Smart-hooks pre-commit integration
# Automatically generated by smart-hooks

# Get staged files
STAGED_FILES=$(git diff --cached --name-only --diff-filter=AM)

if [ -n "$STAGED_FILES" ]; then
    echo "🧠 Running smart-hooks analysis..."
    smart-hooks test $STAGED_FILES
else
    echo "ℹ️  No staged files to analyze"
fi
```

## Integration Examples

### CI/CD Integration

#### GitHub Actions
```yaml
name: Smart Hooks CI
on: [push, pull_request]

jobs:
  smart-hooks:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      
      # Install smart-hooks (prek optional)
      - run: cargo install smart-hooks
      - run: cargo install prek@0.2.20  # Optional but recommended
      
      # Run intelligent analysis
      - run: smart-hooks validate
      - run: smart-hooks run --all-files
```

#### GitLab CI
```yaml
smart-hooks:
  image: rust:latest
  script:
    - cargo install smart-hooks prek@0.2.20
    - smart-hooks validate
    - smart-hooks run --all-files
```

### Team Workflow

#### Setup for New Projects
```bash
# 1. Install tools
cargo install smart-hooks prek@0.2.20

# 2. Initialize configuration
smart-hooks validate  # Creates default config if needed

# 3. Install hooks
smart-hooks install pre-commit

# 4. Test setup
echo "test change" >> README.md
git add README.md
git commit -m "test"  # Triggers smart-hooks analysis
```

#### Migration from pre-commit

**From existing pre-commit setup:**
```bash
# 1. Install smart-hooks
cargo install smart-hooks prek@0.2.20

# 2. Your existing .pre-commit-config.yaml works as-is
smart-hooks validate

# 3. Enhanced with smart analysis
smart-hooks run --all-files

# 4. Optional: Add smart hooks to config
# Add smart-test-selector hook to .pre-commit-config.yaml
```

## CLI Reference

### Command Structure
```
smart-hooks <COMMAND>

Commands:
  run       Run hooks (delegates to prek when available)
  install   Install git hooks (uses prek or creates smart hooks)
  list      List available hooks (shows prek + smart capabilities)
  validate  Validate configuration (uses prek or smart validation)
  test      Intelligent test selection (smart-hooks feature)
  bdd       BDD feature selection (smart-hooks + AI)
  analyze   Multi-language analysis (smart-hooks feature)
  check     Conditional compilation check (smart-hooks feature)
```

### Environment Variables

```bash
# Optional: Disable prek delegation (force smart-hooks mode)
export SMART_HOOKS_NO_PREK=1
smart-hooks run  # Will use smart analysis even if prek is available

# Optional: Prek path (if not in PATH)
export PREK_PATH=/custom/path/to/prek
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Hook execution failed |
| 2 | Configuration error |
| 3 | File not found |
| 4 | Git repository not found |

## Troubleshooting

### Common Issues

#### "Prek not found" Warning
```bash
⚠️  Prek not found. Install with:
    cargo install prek@0.2.20
    # or: pip install prek
```

**Solution**: Install prek or use smart-hooks standalone (it works perfectly without prek).

#### Hooks Not Triggering
```bash
# Check hook installation
ls -la .git/hooks/pre-commit

# Reinstall hooks
smart-hooks install pre-commit

# Test manually
smart-hooks run --files README.md
```

#### Configuration Conflicts
```bash
# Validate current setup
smart-hooks validate

# Check for conflicting configurations
ls -la .pre-commit-config.yaml config.toml
```

### Debug Mode

```bash
# Enable verbose logging
RUST_LOG=debug smart-hooks run --all-files

# Check prek detection
smart-hooks list  # Shows which mode is active
```

### Performance Optimization

#### For Large Repositories
```bash
# Use smart file filtering
smart-hooks run --files $(git diff --name-only HEAD~1)

# Enable hot-reload features
smart-hooks analyze --verbose
```

#### For CI/CD
```bash
# Cache installations
cargo install --locked smart-hooks prek@0.2.20

# Parallel execution
smart-hooks run --all-files &
smart-hooks analyze &
wait
```

## Best Practices

### Recommended Setup

1. **Install both tools** for maximum functionality
2. **Use standard .pre-commit-config.yaml** for team compatibility  
3. **Add smart-hooks specific hooks** for enhanced analysis
4. **Configure project-specific patterns** in config.toml
5. **Test setup regularly** with `smart-hooks validate`

### Team Guidelines

1. **Document installation** in project README
2. **Commit configuration files** (.pre-commit-config.yaml, config.toml)
3. **Use consistent hook IDs** across team
4. **Regular validation** in CI/CD pipelines

### Security Considerations

- Smart-hooks never logs sensitive information
- Prek delegation uses standard CLI interfaces
- Git hooks run in project context only
- Configuration files should be version controlled

## Version Compatibility

| smart-hooks | prek | Status |
|-------------|------|--------|
| 0.2.x | 0.2.20 | ✅ Fully supported |
| 0.2.x | 0.2.x | ✅ Compatible |
| 0.2.x | None | ✅ Standalone mode |

## Contributing

### Development Setup

```bash
# Clone repository
git clone https://github.com/CaptainEmpower/smart-hooks
cd smart-hooks

# Install dependencies
cargo build

# Install test tools
cargo install prek@0.2.20

# Run tests
cargo test --lib --features claude-ai
```

### Testing Integration

```bash
# Test with prek
cargo run -- list

# Test without prek (temporarily move it)
mv ~/.cargo/bin/prek ~/.cargo/bin/prek.bak
cargo run -- list
mv ~/.cargo/bin/prek.bak ~/.cargo/bin/prek
```

## Support

- **Issues**: [GitHub Issues](https://github.com/CaptainEmpower/smart-hooks/issues)
- **Discussions**: [GitHub Discussions](https://github.com/CaptainEmpower/smart-hooks/discussions)  
- **Prek Issues**: [Prek Repository](https://github.com/j178/prek/issues)

## License

This integration maintains compatibility with both project licenses:
- Smart-hooks: MIT License
- Prek: MIT License

---

**Built with ❤️ for intelligent, enterprise-grade pre-commit workflows**