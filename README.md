# smart-hooks

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![Tests](https://img.shields.io/badge/tests-44%20passing-green.svg)]()
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**Intelligent git hooks that understand your code changes and automatically select the right tests to run.**

smart-hooks provides a sophisticated, AI-powered system for automatically selecting and running the right tests based on file changes. It uses structural pattern analysis and optional Claude AI integration to understand code changes and intelligently target relevant test suites.

## ✨ Features

### 🎯 **Intelligent Test Selection**
- **Smart dependency analysis** - Only runs tests affected by your changes
- **Configurable pattern matching** - Customize test selection rules for your project
- **Three-tier analysis**: Configuration → Static Analysis → Claude AI

### 🥒 **Cucumber Integration**
- **Semantic tag mapping** - File patterns → cucumber tags (`@business-logic`, `@api`, `@workflow`)
- **Automatic command generation** - Ready-to-run cucumber commands
- **Targeted test execution** - Run only relevant BDD scenarios

### 🏗️ **Domain-Agnostic Architecture**
- **Structural pattern recognition** - Based on software architecture, not business domain
- **Fully configurable** - No hard-coded business logic
- **Reusable across projects** - Works for any Rust project structure

### 🤖 **Claude AI Integration** (Optional)
- **Semantic code analysis** - Understands code intent, not just syntax
- **Intelligent feature selection** - Maps changes to behavioral requirements
- **Graceful fallback** - Works with or without Claude CLI

## 🚀 Quick Start

## 🚀 Installation

### macOS (Homebrew - Recommended)
```bash
brew tap CaptainEmpower/smart-hooks
brew install smart-hooks
```

### Quick Install (macOS/Linux)
```bash
curl -sSL https://raw.githubusercontent.com/CaptainEmpower/smart-hooks/main/install.sh | bash
```

### Manual Install (Any Platform)
```bash
cargo install --git https://github.com/CaptainEmpower/smart-hooks
```

### Library Usage
Add to your `Cargo.toml`:

```toml
[dev-dependencies]
smart-hooks = "0.2"
```

📖 **For detailed installation instructions, see [INSTALL.md](INSTALL.md)**

### Basic Usage

1. **Install pre-commit hooks:**
```bash
pip install pre-commit
pre-commit install
```

2. **Configure `.pre-commit-config.yaml`:**
```yaml
repos:
  - repo: local
    hooks:
      - id: smart-test-selector
        name: Smart Test Selection
        entry: cargo run --bin smart-test-selector --
        language: system
        files: '^src/.*\.rs$'
        pass_filenames: true
```

3. **Create configuration file** (`config.toml`):
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

## 📖 Usage Examples

### Smart Test Selection

```bash
# Automatically select tests based on changed files
cargo run --bin smart-test-selector -- src/core/processor.rs src/api/controller.rs

# Output:
# 🔍 Running unit tests for: processor, controller
# 🧪 Running integration tests for core changes
# 🎭 BDD recommendation: cucumber --tags "@business-logic or @api"
```

### Claude AI BDD Selection

```bash
# Intelligent BDD feature selection with Claude AI
cargo run --bin claude-bdd-selector --features claude-ai -- \
    --dry-run \
    --context "Payment processing system with fraud detection"

# Output:
# 🤖 Claude AI Analysis Results:
# 📋 Recommended Features: payment_validation.feature, fraud_detection.feature
# 🎯 Priority: high (confidence: 89%)
# 🏷️ Tags: @business-logic @critical @payment
```

### Configuration-Based Analysis

```rust
use smart_hooks::{TestSelectorConfig, select_bdd_features_static_with_config};

let config = TestSelectorConfig::from_file("project-config.toml")?;

// Get cucumber tags for a file
let tags = config.get_cucumber_tags_for_file("src/core/payment.rs");
// Returns: ["@business-logic", "@critical"]

// Get ready-to-run cucumber command
let cmd = config.get_cucumber_command_for_file("src/api/users.rs");
// Returns: "cucumber --tags '@api or @external'"
```

## 📁 Project Structure

```
smart-hooks/
├── src/
│   ├── analysis/           # Core analysis modules
│   │   ├── bdd/                   # BDD-specific analysis
│   │   │   ├── feature_discovery.rs  # Feature file discovery
│   │   │   ├── file_analyzer.rs      # BDD file analysis
│   │   │   ├── static_selector.rs    # Static pattern selection
│   │   │   └── types.rs              # BDD type definitions
│   │   ├── bdd_detector.rs        # Static BDD pattern detection
│   │   ├── claude_bdd_detector.rs # Claude AI integration
│   │   ├── bdd_feature_selector.rs # Intelligent feature selection
│   │   ├── config.rs              # Configuration management
│   │   ├── dependency_mapper.rs   # Test dependency analysis
│   │   └── file_analyzer.rs       # File content analysis
│   ├── dependency/         # Dependency analysis
│   │   ├── analyzers/             # Language-specific analyzers
│   │   ├── graph.rs               # Dependency graph
│   │   ├── multi_lang_analyzer.rs # Multi-language support
│   │   └── types.rs               # Dependency types
│   ├── execution/          # Test execution coordination
│   │   ├── plan_executor.rs       # Test plan execution
│   │   └── test_runner.rs         # Cargo command runner
│   ├── hotreload/          # Hot-reload capabilities
│   │   ├── cache/                 # Caching system
│   │   ├── engine.rs              # Hot-reload engine
│   │   ├── execution/             # Execution tracking
│   │   └── tracking/              # File change tracking
│   ├── project/            # Project discovery
│   │   ├── discovery/             # Language detection
│   │   ├── types.rs               # Project types
│   │   └── multi_lang_discovery.rs # Multi-language discovery
│   ├── utilities/          # Shared utilities
│   │   ├── file_utils.rs          # File operations
│   │   ├── impact_analyzer.rs     # Change impact assessment
│   │   └── module_utils.rs        # Module name extraction
│   ├── smart_test_selector.rs     # Main CLI binary
│   └── claude_bdd_selector.rs     # Claude AI CLI binary
├── features/              # BDD feature files
├── tests/                # Integration tests
├── example-config.toml   # Configuration template
└── README.md            # This file
```

## ⚙️ Configuration

### Structural Patterns

Define architectural patterns that should trigger BDD tests:

```toml
[bdd_structural_patterns]
# Core business logic
core_business_logic = ["/core/", "/service/", "/domain/", "/business/"]

# Application workflows
application_logic = ["/apply/", "/strategy/", "/handler/", "/processor/"]

# Error handling and validation
error_handling = ["/error", "/validate", "/types", "/exception/"]

# API interfaces
api_interfaces = ["/api/", "/controller/", "/endpoint/", "/web/"]

# Domain patterns (DDD, CQRS, etc.)
behavioral_patterns = ["/command/", "/event/", "/aggregate/", "/saga/"]
```

### Cucumber Tag Mapping

Map file patterns to semantic cucumber tags:

```toml
[bdd_pattern_tags]
# Business logic
"/core/" = ["@business-logic", "@critical"]
"/service/" = ["@business-logic", "@integration"]
"/domain/" = ["@business-logic", "@domain-rules"]

# API layer
"/api/" = ["@api", "@external"]
"/controller/" = ["@api", "@web"]
"/endpoint/" = ["@api", "@rest"]

# Error handling
"/error" = ["@error-handling", "@robustness"]
"/validate" = ["@error-handling", "@validation"]
```

### Test Selection Rules

Configure which files trigger which test types:

```toml
# Unit test patterns (file → module mapping)
[unit_test_patterns]
"processor.rs" = "processor"
"validator.rs" = "validator"

# Integration test patterns
integration_test_patterns = ["/core/", "/types.rs", "/error.rs"]

# BDD test patterns
bdd_test_patterns = ["/apply/", "/strategy/", "/service/"]
```

## 🤖 Claude AI Integration

### Setup

1. **Install Claude CLI:**
```bash
# Follow Claude CLI installation instructions
# https://docs.anthropic.com/claude/docs/claude-code
```

2. **Enable feature:**
```bash
cargo run --bin claude-bdd-selector --features claude-ai
```

3. **Usage:**
```bash
# Analyze staged changes
git add src/core/payment_processor.rs
cargo run --bin claude-bdd-selector --features claude-ai -- \
    --context "E-commerce platform with payment processing"
```

### Benefits

- **Semantic understanding** - Understands code intent beyond syntax
- **Context-aware** - Considers project domain and business logic
- **Intelligent recommendations** - Suggests specific test scenarios
- **Confidence scoring** - Provides reliability metrics

## 🧪 Testing

The crate includes comprehensive test coverage:

```bash
# Run all tests
cargo test --lib

# Run with Claude AI features
cargo test --lib --features claude-ai

# Run specific test modules
cargo test analysis::config
cargo test analysis::bdd_detector
```

**Test Coverage:**
- ✅ **44 unit tests** covering all core functionality
- ✅ **Static analysis** pattern detection
- ✅ **Configuration parsing** and validation
- ✅ **Tag mapping** and command generation
- ✅ **BDD feature selection** algorithms
- ✅ **Error handling** and edge cases

## 🏗️ Architecture

smart-hooks follows **Single Responsibility Principle** with each module ≤300 LOC:

### Analysis Layer
- **Structural pattern detection** (domain-agnostic)
- **Content analysis** (optional file reading)
- **AI-powered semantic analysis** (Claude integration)
- **Configuration management** (TOML-based)

### Execution Layer
- **Test plan creation** based on analysis
- **Cargo command execution** with proper error handling
- **Result coordination** and reporting

### Utilities Layer
- **File operations** with safety checks
- **Impact analysis** for change assessment
- **Module utilities** for Rust project navigation

## 🔄 Integration Examples

### With CI/CD

```yaml
# GitHub Actions
- name: Smart Test Selection
  run: |
    cargo run --bin smart-test-selector -- $(git diff --name-only HEAD~1)

# GitLab CI
script:
  - cargo run --bin claude-bdd-selector --features claude-ai -- --dry-run
```

### With Pre-commit

```yaml
# .pre-commit-config.yaml
repos:
  - repo: local
    hooks:
      - id: intelligent-bdd
        name: Intelligent BDD Selection
        entry: cargo run --bin claude-bdd-selector --features claude-ai
        files: '^src/.*\.rs$'
        language: system
```

### With Custom Scripts

```bash
#!/bin/bash
# run-smart-tests.sh

CHANGED_FILES=$(git diff --cached --name-only --diff-filter=AM | grep "\.rs$")

if [[ -n "$CHANGED_FILES" ]]; then
    echo "🔍 Analyzing changed files: $CHANGED_FILES"
    cargo run --bin smart-test-selector -- $CHANGED_FILES
fi
```

## 📚 API Reference

### Core Types

```rust
// Configuration
pub struct TestSelectorConfig {
    pub bdd_structural_patterns: Option<BddStructuralPatterns>,
    pub bdd_pattern_tags: Option<HashMap<String, Vec<String>>>,
    pub enable_content_analysis: bool,
}

// BDD Analysis
pub struct BddFeatureSelection {
    pub should_run_bdd: bool,
    pub confidence: f32,
    pub selected_features: Vec<SelectedFeature>,
    pub suggested_test_focus: Vec<String>,
}

// Test Planning
pub struct TestPlan {
    pub unit_tests: Vec<String>,
    pub integration_tests: bool,
    pub bdd_tests: bool,
}
```

### Key Functions

```rust
// Configuration-based analysis
pub fn select_bdd_features_static_with_config(
    staged_files: &[FileChange],
    available_features: &[String],
    config: &TestSelectorConfig,
) -> Result<BddFeatureSelection>

// Claude AI analysis (requires "claude-ai" feature)
pub async fn select_bdd_features_with_claude(
    staged_files: &[FileChange],
    available_features: &[String],
    project_context: &str,
) -> Result<BddFeatureSelection>

// Test plan creation
pub fn create_test_plan_with_config(
    files: &[String],
    config: &TestSelectorConfig,
) -> Result<TestPlan>
```

## 🤝 Contributing

1. **Fork the repository**
2. **Create a feature branch** (`git checkout -b feature/amazing-feature`)
3. **Follow SRP guidelines** (≤300 LOC per module)
4. **Add comprehensive tests**
5. **Update documentation**
6. **Submit a pull request**

### Development Guidelines

- **Single Responsibility Principle** - Each module has one clear purpose
- **Configuration over code** - Make patterns configurable, not hard-coded
- **Domain-agnostic design** - Avoid business-specific assumptions
- **Comprehensive testing** - Maintain >95% test coverage
- **Clear documentation** - Document architectural decisions

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **Anthropic Claude** for semantic code analysis capabilities
- **Cucumber** for excellent BDD testing framework
- **Rust community** for amazing tooling and ecosystem
- **git-mvh project** for inspiring the initial architecture
- **Multi-language support** for TypeScript, Python, PHP, and Rust projects

---

**Built with ❤️ for intelligent test automation across all projects**