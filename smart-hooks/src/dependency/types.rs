/// Core types for dependency analysis
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    /// Name of the dependency (module, crate, or file)
    pub name: String,
    /// Type of dependency relationship
    pub dependency_type: DependencyType,
    /// Path to the dependency file
    pub path: PathBuf,
    /// Weight/importance of this dependency (0.0 to 1.0)
    pub weight: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyType {
    /// Direct module dependency (use statement)
    ModuleUse,
    /// Function call dependency
    FunctionCall,
    /// Trait implementation dependency
    TraitImpl,
    /// Macro usage dependency
    MacroUse,
    /// Test dependency (tests that should run when this changes)
    TestDependency,
    /// Integration test dependency
    IntegrationTest,
    /// Documentation dependency
    DocTest,
}

impl std::fmt::Display for DependencyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DependencyType::ModuleUse => write!(f, "ModuleUse"),
            DependencyType::FunctionCall => write!(f, "FunctionCall"),
            DependencyType::TraitImpl => write!(f, "TraitImpl"),
            DependencyType::MacroUse => write!(f, "MacroUse"),
            DependencyType::TestDependency => write!(f, "TestDependency"),
            DependencyType::IntegrationTest => write!(f, "IntegrationTest"),
            DependencyType::DocTest => write!(f, "DocTest"),
        }
    }
}

impl std::fmt::Display for TestType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestType::Unit { module } => write!(f, "Unit({})", module),
            TestType::Integration { test_file } => write!(f, "Integration({})", test_file),
            TestType::Doc { source_file } => write!(f, "Doc({})", source_file),
            TestType::Benchmark { bench_name } => write!(f, "Benchmark({})", bench_name),
            TestType::Custom { command } => write!(f, "Custom({})", command),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestTarget {
    /// Name of the test target
    pub name: String,
    /// Type of test (unit, integration, doc, etc.)
    pub test_type: TestType,
    /// Command to execute this test
    pub command: Vec<String>,
    /// Files that this test depends on
    pub dependencies: Vec<PathBuf>,
    /// Confidence that this test should be run (0.0 to 1.0)
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestType {
    /// Unit test within a module
    Unit { module: String },
    /// Integration test
    Integration { test_file: String },
    /// Documentation test
    Doc { source_file: String },
    /// Benchmark test
    Benchmark { bench_name: String },
    /// Custom test command
    Custom { command: String },
}