//! Project discovery module components
//!
//! This module contains focused sub-modules for multi-language project discovery:
//!
//! - `language_detector`: Language detection utilities
//! - `metadata_extractor`: Project metadata extraction
//! - `build_detector`: Build configuration detection
//! - `config_creator`: Language configuration creation

pub mod build_detector;
pub mod config_creator;
pub mod language_detector;
pub mod metadata_extractor;
pub mod project_discovery;

// Re-export main types for external use
pub use build_detector::BuildDetector;
pub use config_creator::ConfigCreator;
pub use language_detector::LanguageDetector;
pub use metadata_extractor::MetadataExtractor;
pub use project_discovery::ProjectDiscovery;