/// Dependency impact analysis using multi-language project detection
use anyhow::{Context, Result};
use smart_hooks::dependency::multi_lang_analyzer::MultiLangDependencyAnalyzer;
use smart_hooks::project::{MultiLangProjectDiscovery, Language};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub async fn run(
    files: Vec<String>,
    _graph: bool,
    language_filter: Option<String>,
    verbose: bool,
) -> Result<()> {
    if verbose {
        println!("🔍 Smart Hooks Dependency Analysis");
        println!("Files to analyze: {}", files.len());
    }

    if files.is_empty() {
        println!("⏭️  No files to analyze");
        return Ok(());
    }

    // Discover the project configuration
    let project_config = MultiLangProjectDiscovery::discover(".")
        .context("Failed to discover project structure")?;

    if verbose {
        println!("📊 Project: {} (Primary language: {:?})", 
                project_config.metadata.name, 
                project_config.primary_language);
    }

    // Create analyzer
    let analyzer = MultiLangDependencyAnalyzer::new(project_config);

    // Convert string paths to PathBuf (for future use)
    let _file_paths: Vec<PathBuf> = files.iter().map(|f| PathBuf::from(f)).collect();

    // Group files by language for analysis
    let files_by_language = group_files_by_language(&files);

    if verbose {
        println!("\n📝 Files by Language:");
        for (lang, file_list) in &files_by_language {
            println!("   {:?}: {} files", lang, file_list.len());
        }
    }

    let total_dependencies = 0;
    let mut total_affected_tests = 0;

    // Analyze each language group
    for (language, file_list) in files_by_language {
        // Skip if language filter is specified and doesn't match
        if let Some(filter) = &language_filter {
            let filter_lang = parse_language_filter(filter);
            if let Some(filter_lang) = filter_lang {
                if language != filter_lang {
                    continue;
                }
            }
        }

        println!("\n🔬 Analyzing {:?} Dependencies", language);
        println!("{}", "=".repeat(50));

        // Analyze dependencies for each file
        for file in &file_list {
            let file_path = Path::new(file);
            
            if !file_path.exists() {
                if verbose {
                    println!("⚠️  File not found: {}", file);
                }
                continue;
            }

            println!("\n📄 File: {}", file);

            println!("   📄 Dependency analysis not yet implemented for individual files");
            println!("   💡 Use project-wide analysis instead");
        }

        // Find affected tests
        let file_paths_for_lang: Vec<PathBuf> = file_list.iter().map(|f| PathBuf::from(f)).collect();
        match analyzer.find_cross_language_affected_tests(&file_paths_for_lang) {
            Ok(tests) => {
                if !tests.is_empty() {
                    println!("\n🧪 Affected Tests ({} found):", tests.len());
                    total_affected_tests += tests.len();

                    for test in &tests {
                        let test_type_symbol = match &test.test_type {
                            smart_hooks::dependency::types::TestType::Unit { .. } => "🔬",
                            smart_hooks::dependency::types::TestType::Integration { .. } => "🔗",
                            smart_hooks::dependency::types::TestType::Doc { .. } => "📋",
                            smart_hooks::dependency::types::TestType::Benchmark { .. } => "⚖️",
                            smart_hooks::dependency::types::TestType::Custom { .. } => "🔧",
                        };
                        
                        println!("   {} {}: {}", test_type_symbol, test.test_type, test.name);
                        if verbose && !test.dependencies.is_empty() {
                            let dep_names: Vec<String> = test.dependencies.iter()
                                .map(|p| p.display().to_string())
                                .collect();
                            println!("      🔗 Dependencies: {}", dep_names.join(", "));
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("❌ Failed to find affected tests: {}", e);
            }
        }
    }

    // Overall summary
    println!("\n📋 Analysis Summary:");
    println!("{}", "=".repeat(50));
    println!("   📄 Files analyzed: {}", files.len());
    println!("   📦 Total dependencies: {}", total_dependencies);
    println!("   🧪 Affected tests: {}", total_affected_tests);

    if total_affected_tests > 0 {
        println!("\n💡 Recommended Actions:");
        println!("   • Run {} affected tests before committing", total_affected_tests);
        if total_dependencies > 10 {
            println!("   • Consider reducing coupling - {} dependencies found", total_dependencies);
        }
        println!("   • Review impact on dependent modules");
    }

    Ok(())
}

fn show_dependency_graph(
    dependencies: &[smart_hooks::dependency::types::Dependency],
    file_path: &Path,
    _verbose: bool
) {
    println!("      {} (current file)", file_path.display());
    
    for (i, dep) in dependencies.iter().enumerate() {
        let is_last = i == dependencies.len() - 1;
        let prefix = if is_last { "      └── " } else { "      ├── " };
        
        match &dep.dependency_type {
            smart_hooks::dependency::types::DependencyType::ModuleUse => {
                println!("{}{}", prefix, dep.name);
            }
            smart_hooks::dependency::types::DependencyType::FunctionCall => {
                println!("{}{}()", prefix, dep.name);
            }
            smart_hooks::dependency::types::DependencyType::TraitImpl => {
                println!("{}impl {}", prefix, dep.name);
            }
            smart_hooks::dependency::types::DependencyType::MacroUse => {
                println!("{}{}!", prefix, dep.name);
            }
            _ => {
                println!("{}{}", prefix, dep.name);
            }
        }
    }
}

fn group_files_by_language(files: &[String]) -> HashMap<Language, Vec<String>> {
    let mut files_by_language = HashMap::new();

    for file in files {
        if let Some(language) = detect_file_language(file) {
            files_by_language
                .entry(language)
                .or_insert_with(Vec::new)
                .push(file.clone());
        }
    }

    files_by_language
}

fn detect_file_language(file_path: &str) -> Option<Language> {
    let path = Path::new(file_path);
    if let Some(extension) = path.extension().and_then(|s| s.to_str()) {
        match extension {
            "rs" => Some(Language::Rust),
            "ts" | "tsx" => Some(Language::TypeScript),
            "js" | "jsx" | "mjs" => Some(Language::JavaScript),
            "py" | "pyx" | "pyi" => Some(Language::Python),
            "php" | "phtml" => Some(Language::PHP),
            "go" => Some(Language::Go),
            "java" => Some(Language::Java),
            "cs" => Some(Language::CSharp),
            _ => None,
        }
    } else {
        None
    }
}

fn parse_language_filter(filter: &str) -> Option<Language> {
    match filter.to_lowercase().as_str() {
        "rust" | "rs" => Some(Language::Rust),
        "typescript" | "ts" => Some(Language::TypeScript),
        "javascript" | "js" => Some(Language::JavaScript),
        "python" | "py" => Some(Language::Python),
        "php" => Some(Language::PHP),
        "go" => Some(Language::Go),
        "java" => Some(Language::Java),
        "csharp" | "cs" | "c#" => Some(Language::CSharp),
        _ => None,
    }
}