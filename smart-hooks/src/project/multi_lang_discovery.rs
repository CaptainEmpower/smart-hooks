/// Multi-language project discovery
/// Detects and configures support for multiple programming languages in a project
use anyhow::Result;
use std::path::Path;

use super::discovery::{BuildDetector, ConfigCreator, LanguageDetector, MetadataExtractor};
use super::multi_lang_types::*;

/// Multi-language project discovery service
pub struct MultiLangProjectDiscovery;

impl MultiLangProjectDiscovery {
    /// Discover project configuration for all supported languages
    pub fn discover<P: AsRef<Path>>(project_root: P) -> Result<MultiLangProjectConfig> {
        let root = project_root.as_ref().to_path_buf();

        // Detect all languages in the project
        let languages = LanguageDetector::detect_languages(&root)?;

        if languages.is_empty() {
            return Err(anyhow::anyhow!(
                "No supported programming languages detected in project"
            ));
        }

        // Determine primary language (most files or explicit configuration)
        let primary_language = LanguageDetector::determine_primary_language(&languages, &root)?;

        // Create language-specific configurations
        let mut language_configs = Vec::new();
        for lang in &languages {
            if let Ok(config) = ConfigCreator::create_language_config(lang.clone(), &root) {
                language_configs.push(config);
            }
        }

        // Extract project metadata
        let metadata = Self::extract_project_metadata(&root, &primary_language)?;

        // Determine build configuration
        let build_config = BuildDetector::detect_build_config(&root, &primary_language)?;

        Ok(MultiLangProjectConfig {
            project_root: root,
            primary_language,
            languages: language_configs,
            metadata,
            test_strategy: TestStrategy::default(),
            build_config,
        })
    }

    /// Extract project metadata from various sources
    fn extract_project_metadata(
        project_root: &Path,
        primary_language: &Language,
    ) -> Result<ProjectMetadata> {
        // Try to extract from language-specific files
        match primary_language {
            Language::Rust => MetadataExtractor::extract_rust_metadata(project_root),
            Language::TypeScript | Language::JavaScript => {
                MetadataExtractor::extract_node_metadata(project_root)
            }
            Language::Python => MetadataExtractor::extract_python_metadata(project_root),
            Language::PHP => MetadataExtractor::extract_php_metadata(project_root),
            _ => {
                // Generic fallback
                Ok(MetadataExtractor::extract_generic_metadata(project_root))
            }
        }
    }
}