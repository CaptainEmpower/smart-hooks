# smart-hooks Design Document

## Table of Contents

1. [Overview](#overview)
2. [Architecture Principles](#architecture-principles)
3. [System Architecture](#system-architecture)
4. [Design Patterns](#design-patterns)
5. [Data Flow](#data-flow)
6. [Configuration System](#configuration-system)
7. [AI Integration](#ai-integration)
8. [Testing Strategy](#testing-strategy)
9. [Performance Considerations](#performance-considerations)
10. [Security Model](#security-model)
11. [Extension Points](#extension-points)
12. [Decision Records](#decision-records)

## Overview

smart-hooks is a sophisticated git hook system designed to intelligently select and execute tests based on code changes. The system combines static analysis, configurable pattern matching, and optional AI-powered semantic analysis to provide efficient, targeted test execution for any project.

### Goals

- **Intelligence**: Understand code changes and select relevant tests
- **Efficiency**: Minimize test execution time while maintaining coverage
- **Flexibility**: Work across different project domains and structures
- **Reliability**: Provide consistent, predictable behavior
- **Extensibility**: Support future enhancements and customizations

### Non-Goals

- **Domain-specific logic**: Avoid hard-coding business rules
- **Test execution**: Focus on selection, not running tests directly
- **Git operations**: Delegate git operations to external tools
- **Complex AI**: Keep AI integration optional and bounded

## Architecture Principles

### 1. Single Responsibility Principle (SRP)

Every module has exactly one reason to change:

```
analysis/bdd_detector.rs        → Static BDD pattern detection (240 LOC)
analysis/config.rs              → Configuration management (372 LOC)
analysis/bdd_feature_selector.rs → Feature selection logic (456 LOC)
execution/plan_executor.rs      → Test execution coordination (87 LOC)
```

**Constraint**: Each module ≤300 LOC (target), ≤500 LOC (hard limit)

### 2. Configuration Over Code

All behavioral patterns are externalized to configuration:

```rust
// ❌ WRONG: Hard-coded patterns
if file_path.contains("payment") || file_path.contains("billing") {
    return vec!["payment_tests.feature"];
}

// ✅ RIGHT: Configuration-driven
if config.matches_structural_pattern(file_path) {
    return config.get_cucumber_tags_for_file(file_path);
}
```

### 3. Domain Agnostic Design

The system recognizes **software architecture patterns**, not business domains:

```toml
# ✅ Architectural patterns (reusable)
core_business_logic = ["/core/", "/service/", "/domain/"]
api_interfaces = ["/api/", "/controller/", "/endpoint/"]

# ❌ Domain patterns (not reusable)
payment_logic = ["/payment/", "/billing/", "/invoice/"]
```

### 4. Graceful Degradation

The system provides multiple fallback layers:

```
1. Configuration-based patterns → Fast, predictable
2. Static code analysis → Moderate intelligence
3. Claude AI analysis → High intelligence (optional)
```

### 5. Immutable Data Flow

Data flows through pure functions without side effects:

```rust
FileChange → Analysis → TestPlan → Execution
     ↓            ↓         ↓         ↓
  Immutable  Pure Func  Immutable  Effect
```

## System Architecture

### Layered Architecture

```
┌─────────────────────────────────────────────────────┐
│                   CLI Layer                         │
│  smart_test_selector.rs  │  claude_bdd_selector.rs  │
├─────────────────────────────────────────────────────┤
│                 Execution Layer                     │
│  plan_executor.rs        │  test_runner.rs          │
├─────────────────────────────────────────────────────┤
│                 Analysis Layer                      │
│ ┌─────────────────┬─────────────────┬──────────────┐ │
│ │ Static Analysis │ Config Analysis │ AI Analysis  │ │
│ │ bdd_detector    │ config.rs       │ claude_bdd   │ │
│ │ file_analyzer   │ dependency_map  │ detector     │ │
│ └─────────────────┴─────────────────┴──────────────┘ │
├─────────────────────────────────────────────────────┤
│                Utilities Layer                      │
│  file_utils.rs   │  impact_analyzer.rs  │ module_utils │
└─────────────────────────────────────────────────────┘
```

### Module Responsibilities

#### Analysis Layer
- **`bdd_detector.rs`**: Static pattern detection using regex and heuristics
- **`claude_bdd_detector.rs`**: AI-powered semantic analysis via Claude CLI
- **`bdd_feature_selector.rs`**: Intelligent feature selection algorithms
- **`config.rs`**: Configuration parsing and structural pattern matching
- **`dependency_mapper.rs`**: Test dependency analysis and plan creation
- **`file_analyzer.rs`**: File content analysis and functionality detection

#### Execution Layer
- **`plan_executor.rs`**: Coordinates test plan execution
- **`test_runner.rs`**: Executes cargo commands safely

#### Utilities Layer
- **`file_utils.rs`**: Safe file operations and content reading
- **`impact_analyzer.rs`**: Change impact assessment
- **`module_utils.rs`**: Rust module name extraction

### Data Structures

#### Core Types

```rust
pub struct TestSelectorConfig {
    /// Pattern-based test selection rules
    pub unit_test_patterns: HashMap<String, String>,
    pub integration_test_patterns: Vec<String>,
    pub bdd_test_patterns: Vec<String>,

    /// Structural analysis patterns
    pub bdd_structural_patterns: Option<BddStructuralPatterns>,
    pub bdd_pattern_tags: Option<HashMap<String, Vec<String>>>,

    /// Analysis configuration
    pub enable_content_analysis: bool,
}

pub struct BddStructuralPatterns {
    pub core_business_logic: Vec<String>,
    pub application_logic: Vec<String>,
    pub error_handling: Vec<String>,
    pub api_interfaces: Vec<String>,
    pub behavioral_patterns: Vec<String>,
}

pub struct BddFeatureSelection {
    pub should_run_bdd: bool,
    pub confidence: f32,
    pub reasoning: String,
    pub selected_features: Vec<SelectedFeature>,
    pub suggested_test_focus: Vec<String>,
    pub risk_areas: Vec<String>,
}

pub struct TestPlan {
    pub unit_tests: Vec<String>,
    pub integration_tests: bool,
    pub bdd_tests: bool,
    pub impact_level: ImpactLevel,
}
```

#### Change Representation

```rust
pub struct FileChange {
    pub file_path: String,
    pub change_type: String,    // "modified", "added", "deleted"
    pub diff_summary: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImpactLevel {
    None,       // No testing needed
    Low,        // Unit tests only
    Medium,     // Unit + integration tests
    High,       // Full test suite including BDD
    Dependency, // Reverse dependency analysis needed
}
```

## Design Patterns

### 1. Strategy Pattern

Multiple analysis strategies with common interface:

```rust
trait BddAnalyzer {
    fn analyze(&self, files: &[FileChange]) -> Result<BddFeatureSelection>;
}

struct StaticAnalyzer { config: TestSelectorConfig }
struct ClaudeAnalyzer { config: TestSelectorConfig }
struct HybridAnalyzer { static_analyzer: StaticAnalyzer, claude_analyzer: ClaudeAnalyzer }
```

### 2. Builder Pattern

Configuration construction:

```rust
let config = TestSelectorConfig::default()
    .with_content_analysis()
    .with_claude_integration()
    .with_custom_patterns(patterns);
```

### 3. Command Pattern

Test execution as commands:

```rust
pub struct CargoCommand {
    pub command: String,
    pub args: Vec<String>,
    pub working_dir: Option<PathBuf>,
}

impl CargoCommand {
    pub fn execute(&self) -> Result<CommandResult> { ... }
}
```

### 4. Factory Pattern

Analysis strategy creation:

```rust
pub struct AnalyzerFactory;

impl AnalyzerFactory {
    pub fn create_analyzer(config: &TestSelectorConfig) -> Box<dyn BddAnalyzer> {
        if config.claude_enabled() {
            Box::new(HybridAnalyzer::new(config))
        } else {
            Box::new(StaticAnalyzer::new(config))
        }
    }
}
```

### 5. Template Method Pattern

Test plan execution workflow:

```rust
pub trait TestExecutor {
    fn validate_plan(&self, plan: &TestPlan) -> Result<()>;
    fn execute_unit_tests(&self, tests: &[String]) -> Result<()>;
    fn execute_integration_tests(&self) -> Result<()>;
    fn execute_bdd_tests(&self) -> Result<()>;

    // Template method
    fn execute_plan(&self, plan: &TestPlan) -> Result<()> {
        self.validate_plan(plan)?;
        if !plan.unit_tests.is_empty() {
            self.execute_unit_tests(&plan.unit_tests)?;
        }
        if plan.integration_tests {
            self.execute_integration_tests()?;
        }
        if plan.bdd_tests {
            self.execute_bdd_tests()?;
        }
        Ok(())
    }
}
```

## Data Flow

### 1. Input Processing

```
Git Changes → File List → FileChange Structs
     ↓
[git diff --cached --name-status]
     ↓
["M src/core/payment.rs", "A src/api/users.rs"]
     ↓
[FileChange { file_path: "src/core/payment.rs", change_type: "modified" }]
```

### 2. Analysis Pipeline

```
FileChange[] → Configuration → Pattern Analysis → Feature Selection
     ↓               ↓              ↓                 ↓
Input Files     TOML Config    Structural        BddFeatureSelection
                               Patterns
```

### 3. Test Plan Creation

```
Analysis Results → Test Plan → Execution Commands → Results
      ↓               ↓              ↓              ↓
BddFeatureSelection  TestPlan   CargoCommand[]   TestResults
```

### Detailed Flow Diagram

```
┌─────────────┐    ┌──────────────┐    ┌─────────────────┐
│ Git Changes │───→│ File Scanner │───→│ Change Analyzer │
└─────────────┘    └──────────────┘    └─────────────────┘
                                                 │
                                                 ▼
┌─────────────────┐    ┌─────────────────┐    ┌──────────────┐
│ Test Executor   │◄───│ Plan Generator  │◄───│ Config Loader│
└─────────────────┘    └─────────────────┘    └──────────────┘
                                 │
                                 ▼
                    ┌─────────────────────────┐
                    │    Analysis Engine      │
                    ├─────────┬───────┬───────┤
                    │ Static  │Config │Claude │
                    │Analysis │Pattern│  AI   │
                    └─────────┴───────┴───────┘
```

## Configuration System

### Configuration Hierarchy

1. **Default Configuration** (in code)
2. **Project Configuration** (`config.toml`)
3. **User Configuration** (`~/.smart-hooks/config.toml`)
4. **Environment Variables** (`SMART_HOOKS_*`)
5. **Command Line Arguments** (`--config-file`)

### Configuration Schema

```toml
# Basic settings
enable_content_analysis = false
claude_integration = true

# File pattern mappings
[unit_test_patterns]
"processor.rs" = "processor"
"validator.rs" = "validator"

integration_test_patterns = ["/core/", "/types.rs"]
bdd_test_patterns = ["/apply/", "/strategy/"]

# Structural patterns for domain-agnostic analysis
[bdd_structural_patterns]
core_business_logic = ["/core/", "/service/", "/domain/"]
application_logic = ["/apply/", "/strategy/", "/handler/"]
error_handling = ["/error", "/validate", "/types"]
api_interfaces = ["/api/", "/controller/", "/endpoint/"]
behavioral_patterns = ["/command/", "/event/", "/aggregate/"]

# Cucumber tag mapping for semantic test selection
[bdd_pattern_tags]
"/core/" = ["@business-logic", "@critical"]
"/service/" = ["@business-logic", "@integration"]
"/api/" = ["@api", "@external"]
"/error" = ["@error-handling", "@robustness"]
```

### Configuration Validation

```rust
impl TestSelectorConfig {
    pub fn validate(&self) -> Result<()> {
        // Validate pattern syntax
        for pattern in &self.bdd_test_patterns {
            validate_pattern(pattern)?;
        }

        // Check for conflicting settings
        if self.enable_content_analysis && self.unit_test_patterns.is_empty() {
            warn!("Content analysis enabled but no unit test patterns defined");
        }

        // Validate tag mapping
        if let Some(tags) = &self.bdd_pattern_tags {
            for (pattern, tag_list) in tags {
                validate_pattern(pattern)?;
                validate_tags(tag_list)?;
            }
        }

        Ok(())
    }
}
```

## AI Integration

### Claude Integration Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌──────────────┐
│ Claude CLI      │◄───│ JSON Protocol   │◄───│ Rust Process │
│ (External)      │    │ (stdin/stdout)  │    │ (Our Code)   │
└─────────────────┘    └─────────────────┘    └──────────────┘
```

### Prompt Engineering

The system uses structured prompts for consistent AI analysis:

```rust
fn build_claude_prompt(file_path: &Path, content: &str, context: &str) -> String {
    format!(r#"
Analyze this Rust code file ({}) for Behavior-Driven Development (BDD) testing needs.

Context: {}

Code:
```rust
{}
```

Please analyze and return JSON with this structure:
{{
    "should_have_bdd_tests": boolean,
    "confidence": float (0.0-1.0),
    "reasoning": "explanation",
    "suggested_scenarios": ["scenario1", "scenario2"],
    "user_facing_features": ["feature descriptions"],
    "business_rules": ["rule descriptions"],
    "integration_points": ["external dependencies"]
}}

Focus on identifying code that represents BEHAVIOR rather than implementation details.
Consider: user-facing APIs, business logic, state mutations, error handling, integrations.
"#, file_path.display(), context, content)
}
```

### Fallback Strategy

```rust
pub async fn analyze_hybrid(file_path: &Path, content: &str) -> Result<BddAnalysis> {
    // 1. Try Claude AI analysis
    #[cfg(feature = "claude-ai")]
    {
        match analyze_with_claude(file_path, content).await {
            Ok(analysis) => return Ok(analysis),
            Err(e) => {
                warn!("Claude AI analysis failed, falling back to static: {}", e);
            }
        }
    }

    // 2. Fallback to static analysis
    analyze_with_static_patterns(file_path, content)
}
```

### Error Handling

```rust
#[derive(Debug)]
pub enum ClaudeError {
    CommandNotFound,           // Claude CLI not installed
    InvalidResponse(String),   // Malformed JSON response
    RateLimited,              // API rate limiting
    NetworkError(String),     // Connection issues
    ParseError(serde_json::Error), // JSON parsing failure
}

impl ClaudeError {
    pub fn is_recoverable(&self) -> bool {
        matches!(self, ClaudeError::RateLimited | ClaudeError::NetworkError(_))
    }
}
```

## Testing Strategy

### Test Architecture

```
Unit Tests (44 tests)
├── analysis/
│   ├── bdd_detector (6 tests)
│   ├── config (7 tests)
│   ├── bdd_feature_selector (4 tests)
│   └── dependency_mapper (8 tests)
├── execution/ (2 tests)
├── utilities/ (17 tests)
└── integration/ (0 tests - using parent crate)
```

### Test Categories

#### 1. Unit Tests
- **Pure function testing** - No side effects
- **Mock external dependencies** - Claude CLI, file system
- **Comprehensive coverage** - All code paths
- **Fast execution** - <100ms total

#### 2. Integration Tests
- **End-to-end workflows** - File changes → Test selection
- **Configuration integration** - TOML parsing → behavior
- **Command generation** - Config → cucumber commands

#### 3. Property Tests
- **Pattern matching** - Verify patterns match expected files
- **Configuration validation** - Invalid configs rejected
- **Tag generation** - File paths → consistent tag sets

### Test Utilities

```rust
// Test fixture creation
pub fn create_test_config() -> TestSelectorConfig {
    TestSelectorConfig {
        bdd_structural_patterns: Some(BddStructuralPatterns {
            core_business_logic: vec!["/core/".to_string()],
            // ... other patterns
        }),
        // ... other fields
    }
}

// Mock file changes
pub fn mock_file_changes() -> Vec<FileChange> {
    vec![
        FileChange {
            file_path: "src/core/processor.rs".to_string(),
            change_type: "modified".to_string(),
            diff_summary: Some("Added validation logic".to_string()),
        },
    ]
}

// Assert test plan contents
pub fn assert_test_plan_contains_bdd(plan: &TestPlan) {
    assert!(plan.bdd_tests, "Expected BDD tests to be enabled");
    assert_eq!(plan.impact_level, ImpactLevel::High);
}
```

## Performance Considerations

### Analysis Performance

```rust
// Lazy evaluation for expensive operations
pub struct LazyBddAnalyzer {
    static_analyzer: OnceCell<StaticAnalyzer>,
    claude_analyzer: OnceCell<ClaudeAnalyzer>,
}

impl LazyBddAnalyzer {
    pub fn analyze(&self, files: &[FileChange]) -> Result<BddFeatureSelection> {
        // Try fast static analysis first
        let static_result = self.static_analyzer()
            .get_or_init(|| StaticAnalyzer::new(&self.config))
            .analyze(files)?;

        // Only use Claude if static analysis is uncertain
        if static_result.confidence < 0.7 {
            return self.claude_analyzer()
                .get_or_init(|| ClaudeAnalyzer::new(&self.config))
                .analyze(files);
        }

        Ok(static_result)
    }
}
```

### Caching Strategy

```rust
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

pub struct AnalysisCache {
    cache: HashMap<String, CachedResult>,
    ttl: Duration,
}

struct CachedResult {
    result: BddFeatureSelection,
    timestamp: SystemTime,
    file_hash: u64, // For invalidation
}

impl AnalysisCache {
    pub fn get(&self, file_path: &str, content_hash: u64) -> Option<BddFeatureSelection> {
        let cached = self.cache.get(file_path)?;

        // Check if cache is still valid
        if cached.file_hash != content_hash ||
           cached.timestamp.elapsed().ok()? > self.ttl {
            return None;
        }

        Some(cached.result.clone())
    }
}
```

### Memory Management

- **Streaming file processing** - Don't load all files into memory
- **Pattern compilation** - Compile regex patterns once
- **Reference sharing** - Use `Arc<T>` for shared configuration
- **Bounded collections** - Limit analysis result sizes

```rust
pub struct BoundedAnalysis {
    pub max_features: usize,
    pub max_scenarios: usize,
    pub max_reasoning_length: usize,
}

impl BoundedAnalysis {
    pub fn apply_limits(&self, mut selection: BddFeatureSelection) -> BddFeatureSelection {
        selection.selected_features.truncate(self.max_features);
        selection.reasoning.truncate(self.max_reasoning_length);
        for feature in &mut selection.selected_features {
            feature.scenarios.truncate(self.max_scenarios);
        }
        selection
    }
}
```

## Security Model

### Input Validation

```rust
pub fn validate_file_path(path: &str) -> Result<()> {
    // Prevent directory traversal
    if path.contains("..") || path.starts_with("/") {
        return Err(SecurityError::InvalidPath(path.to_string()));
    }

    // Ensure it's a source file
    if !path.ends_with(".rs") {
        return Err(SecurityError::UnsupportedFileType(path.to_string()));
    }

    // Check path length
    if path.len() > 512 {
        return Err(SecurityError::PathTooLong(path.len()));
    }

    Ok(())
}
```

### Command Injection Prevention

```rust
pub struct SafeCargoCommand {
    subcommand: CargoSubcommand,
    args: Vec<SafeArg>,
}

#[derive(Debug, Clone)]
pub enum CargoSubcommand {
    Test, Check, Clippy, Doc
}

#[derive(Debug, Clone)]
pub struct SafeArg {
    value: String,
    validated: bool,
}

impl SafeCargoCommand {
    pub fn test() -> Self {
        Self {
            subcommand: CargoSubcommand::Test,
            args: Vec::new(),
        }
    }

    pub fn with_package(mut self, package: &str) -> Result<Self> {
        validate_package_name(package)?;
        self.args.push(SafeArg {
            value: format!("--package={}", package),
            validated: true,
        });
        Ok(self)
    }
}
```

### Data Sanitization

```rust
pub fn sanitize_claude_response(response: &str) -> Result<String> {
    // Remove potential script injection
    let cleaned = response
        .replace("<script>", "")
        .replace("</script>", "")
        .replace("javascript:", "");

    // Validate JSON structure
    let _: serde_json::Value = serde_json::from_str(&cleaned)?;

    Ok(cleaned)
}
```

### File Access Control

```rust
pub struct SecureFileAccess {
    allowed_paths: HashSet<PathBuf>,
    max_file_size: usize,
}

impl SecureFileAccess {
    pub fn read_file(&self, path: &Path) -> Result<String> {
        // Check if path is allowed
        let canonical_path = path.canonicalize()?;
        if !self.is_allowed_path(&canonical_path) {
            return Err(SecurityError::UnauthorizedAccess(path.to_path_buf()));
        }

        // Check file size
        let metadata = std::fs::metadata(&canonical_path)?;
        if metadata.len() > self.max_file_size as u64 {
            return Err(SecurityError::FileTooLarge(metadata.len()));
        }

        // Read with timeout
        std::fs::read_to_string(&canonical_path)
            .map_err(SecurityError::FileReadError)
    }
}
```

## Extension Points

### Custom Analyzers

```rust
pub trait CustomAnalyzer: Send + Sync {
    fn name(&self) -> &'static str;
    fn analyze(&self, changes: &[FileChange]) -> Result<AnalysisResult>;
    fn priority(&self) -> u8; // Higher = runs first
}

pub struct AnalyzerRegistry {
    analyzers: Vec<Box<dyn CustomAnalyzer>>,
}

impl AnalyzerRegistry {
    pub fn register<A: CustomAnalyzer + 'static>(&mut self, analyzer: A) {
        self.analyzers.push(Box::new(analyzer));
    }

    pub fn analyze_all(&self, changes: &[FileChange]) -> Result<Vec<AnalysisResult>> {
        self.analyzers
            .iter()
            .map(|analyzer| analyzer.analyze(changes))
            .collect()
    }
}
```

### Plugin System

```rust
// Plugin trait
pub trait TestSelectorPlugin {
    fn initialize(&mut self, config: &TestSelectorConfig) -> Result<()>;
    fn on_file_changed(&self, change: &FileChange) -> Result<Option<TestRecommendation>>;
    fn on_analysis_complete(&self, results: &AnalysisResults) -> Result<()>;
}

// Plugin manager
pub struct PluginManager {
    plugins: Vec<Box<dyn TestSelectorPlugin>>,
}

impl PluginManager {
    pub fn load_plugin<P: TestSelectorPlugin + 'static>(&mut self, plugin: P) {
        self.plugins.push(Box::new(plugin));
    }

    pub fn notify_file_changed(&self, change: &FileChange) -> Result<Vec<TestRecommendation>> {
        self.plugins
            .iter()
            .filter_map(|plugin| plugin.on_file_changed(change).transpose())
            .collect()
    }
}
```

### Configuration Extensions

```rust
// Custom configuration sections
#[derive(Debug, Serialize, Deserialize)]
pub struct ExtendedConfig {
    #[serde(flatten)]
    pub base: TestSelectorConfig,

    pub plugins: Option<HashMap<String, toml::Value>>,
    pub custom_analyzers: Option<Vec<String>>,
    pub notification_hooks: Option<Vec<NotificationConfig>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NotificationConfig {
    pub trigger: String,        // "test_failure", "analysis_complete"
    pub action: String,         // "slack_message", "email", "webhook"
    pub config: toml::Value,    // Action-specific config
}
```

## Decision Records

### DR001: Single Responsibility Principle

**Decision**: Each module must have exactly one responsibility and ≤300 LOC.

**Context**: Initial code was monolithic and hard to maintain.

**Consequences**:
- ✅ Easy to understand and modify individual modules
- ✅ Better test coverage and isolation
- ✅ Clear ownership and responsibilities
- ❌ More files to navigate
- ❌ Potential over-engineering for simple cases

### DR002: Configuration Over Code

**Decision**: All behavioral patterns must be configurable via TOML files.

**Context**: Hard-coded patterns made the system domain-specific and inflexible.

**Consequences**:
- ✅ Reusable across different project types
- ✅ Customizable without code changes
- ✅ Domain-agnostic design
- ❌ More complex initial setup
- ❌ Configuration validation overhead

### DR003: Cucumber Tag Integration

**Decision**: Use cucumber tags rather than feature names for test selection.

**Context**: Feature names are too specific; tags provide semantic meaning.

**Consequences**:
- ✅ Semantic test categorization
- ✅ Flexible test combinations
- ✅ Better cucumber integration
- ✅ Scalable tag taxonomy
- ❌ Requires cucumber knowledge
- ❌ Tag maintenance overhead

### DR004: Optional Claude AI Integration

**Decision**: Make Claude AI analysis optional with graceful fallback.

**Context**: Not all users have Claude CLI access; system must work without it.

**Consequences**:
- ✅ Works in all environments
- ✅ No external dependencies required
- ✅ Incremental intelligence adoption
- ❌ More complex conditional compilation
- ❌ Multiple code paths to maintain

### DR005: Immutable Data Structures

**Decision**: Use immutable data structures throughout the analysis pipeline.

**Context**: Ensure predictable behavior and easy testing.

**Consequences**:
- ✅ Thread-safe by default
- ✅ Predictable behavior
- ✅ Easy to test and reason about
- ❌ Higher memory usage
- ❌ Performance overhead for large data sets

### DR006: Error Recovery Strategy

**Decision**: Implement comprehensive error recovery with detailed error types.

**Context**: Pre-commit hooks should never block development workflow.

**Consequences**:
- ✅ Robust error handling
- ✅ Clear error messages
- ✅ Graceful degradation
- ❌ More complex error handling code
- ❌ Potential over-engineering

---

**Document Version**: 1.0
**Last Updated**: 2024-11-13
**Authors**: smart-hooks development team