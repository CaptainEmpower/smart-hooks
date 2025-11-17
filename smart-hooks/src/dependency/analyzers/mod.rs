//! Language-specific dependency analyzers for multi-language projects

pub mod php;
pub mod python;
pub mod rust;
pub mod typescript;

pub use php::PhpDependencyAnalyzer;
pub use python::PythonDependencyAnalyzer;
pub use rust::RustMultiLangAnalyzer;
pub use typescript::TypeScriptDependencyAnalyzer;