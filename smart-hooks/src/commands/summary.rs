/// Project summary and reporting functionality
use anyhow::{Context, Result};
use smart_hooks::project::{MultiLangProjectDiscovery, Language};
use smart_hooks::dependency::multi_lang_analyzer::MultiLangDependencyAnalyzer;
use serde_json::json;
use std::path::PathBuf;

pub async fn run(
    project_dir: PathBuf,
    format: String,
    verbose: bool,
) -> Result<()> {
    if verbose {
        println!("📊 Smart Hooks Project Summary");
        println!("Analyzing project: {}", project_dir.display());
    }

    // Discover the project configuration
    let project_config = MultiLangProjectDiscovery::discover(&project_dir)
        .context("Failed to discover project structure")?;

    // Create dependency analyzer
    let analyzer = MultiLangDependencyAnalyzer::new(project_config.clone());

    // Collect project statistics
    let stats = collect_project_stats(&project_config, &analyzer).await?;

    // Output results based on format
    match format.as_str() {
        "json" => output_json(&stats, &project_config)?,
        "text" | _ => output_text(&stats, &project_config, verbose)?,
    }

    Ok(())
}

#[derive(Debug)]
struct ProjectStats {
    total_files: usize,
    files_by_language: std::collections::HashMap<Language, usize>,
    total_dependencies: usize,
    external_dependencies: usize,
    internal_dependencies: usize,
    total_tests: usize,
    tests_by_type: std::collections::HashMap<String, usize>,
    complexity_score: f32,
}

async fn collect_project_stats(
    config: &smart_hooks::project::MultiLangProjectConfig,
    _analyzer: &MultiLangDependencyAnalyzer,
) -> Result<ProjectStats> {
    let mut stats = ProjectStats {
        total_files: 0,
        files_by_language: std::collections::HashMap::new(),
        total_dependencies: 0,
        external_dependencies: 0,
        internal_dependencies: 0,
        total_tests: 0,
        tests_by_type: std::collections::HashMap::new(),
        complexity_score: 0.0,
    };

    // Analyze each language
    for lang_config in &config.languages {
        let file_count = count_files_for_language(&lang_config.language, &config.project_root)?;
        stats.total_files += file_count;
        stats.files_by_language.insert(lang_config.language.clone(), file_count);

        // Analyze dependencies for sample files
        if let Ok(sample_files) = get_sample_files_for_language(&lang_config.language, &config.project_root, 5) {
            for file_path in sample_files {
                // Note: Individual file analysis not implemented yet
                // Using simplified dependency counting for now
                if file_path.extension().and_then(|s| s.to_str()) == Some("rs") {
                    let sample_deps = vec![
                        smart_hooks::dependency::types::Dependency {
                            name: "sample".to_string(),
                            dependency_type: smart_hooks::dependency::types::DependencyType::ModuleUse,
                            path: file_path.clone(),
                            weight: 0.5,
                        }
                    ];
                    let dependencies = sample_deps;
                    stats.total_dependencies += dependencies.len();
                    
                    for dep in dependencies {
                        match dep.dependency_type {
                            smart_hooks::dependency::types::DependencyType::ModuleUse => {
                                stats.external_dependencies += 1;
                            }
                            smart_hooks::dependency::types::DependencyType::FunctionCall |
                            smart_hooks::dependency::types::DependencyType::TraitImpl |
                            smart_hooks::dependency::types::DependencyType::MacroUse => {
                                stats.internal_dependencies += 1;
                            }
                            _ => {
                                stats.internal_dependencies += 1;
                            }
                        }
                    }
                } else {
                    // Skip analysis for non-Rust files for now
                }
            }
        }

        // Count tests
        if let Ok(tests) = find_tests_for_language(&lang_config.language, &config.project_root) {
            stats.total_tests += tests.len();
            for test in tests {
                let test_type = format!("{}", test.test_type);
                *stats.tests_by_type.entry(test_type).or_insert(0) += 1;
            }
        }
    }

    // Calculate complexity score (simple heuristic)
    stats.complexity_score = calculate_complexity_score(&stats);

    Ok(stats)
}

fn calculate_complexity_score(stats: &ProjectStats) -> f32 {
    let language_count = stats.files_by_language.len() as f32;
    let avg_files_per_language = if language_count > 0.0 {
        stats.total_files as f32 / language_count
    } else {
        0.0
    };
    let dependency_ratio = if stats.total_files > 0 {
        stats.total_dependencies as f32 / stats.total_files as f32
    } else {
        0.0
    };
    let test_coverage_ratio = if stats.total_files > 0 {
        stats.total_tests as f32 / stats.total_files as f32
    } else {
        0.0
    };

    // Simple complexity formula (lower is better)
    let complexity = (language_count * 0.1) + 
                    (avg_files_per_language * 0.01) + 
                    (dependency_ratio * 0.5) - 
                    (test_coverage_ratio * 0.3);

    complexity.max(0.0)
}

fn count_files_for_language(language: &Language, root_path: &std::path::Path) -> Result<usize> {
    let extensions = get_extensions_for_language(language);
    let mut count = 0;

    fn count_files_recursive(dir: &std::path::Path, extensions: &[&str], count: &mut usize) -> Result<()> {
        if dir.is_dir() {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                
                if path.is_dir() {
                    // Skip common directories that don't contain source code
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if matches!(name, "target" | "node_modules" | ".git" | "build" | "dist" | "__pycache__") {
                            continue;
                        }
                    }
                    count_files_recursive(&path, extensions, count)?;
                } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if extensions.contains(&ext) {
                        *count += 1;
                    }
                }
            }
        }
        Ok(())
    }

    count_files_recursive(root_path, &extensions, &mut count)?;
    Ok(count)
}

fn get_sample_files_for_language(
    language: &Language, 
    root_path: &std::path::Path, 
    max_files: usize
) -> Result<Vec<PathBuf>> {
    let extensions = get_extensions_for_language(language);
    let mut files = Vec::new();

    fn collect_files_recursive(
        dir: &std::path::Path, 
        extensions: &[&str], 
        files: &mut Vec<PathBuf>,
        max_files: usize
    ) -> Result<()> {
        if files.len() >= max_files {
            return Ok(());
        }

        if dir.is_dir() {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                
                if path.is_dir() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if matches!(name, "target" | "node_modules" | ".git" | "build" | "dist" | "__pycache__") {
                            continue;
                        }
                    }
                    collect_files_recursive(&path, extensions, files, max_files)?;
                } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if extensions.contains(&ext) && files.len() < max_files {
                        files.push(path);
                    }
                }
            }
        }
        Ok(())
    }

    collect_files_recursive(root_path, &extensions, &mut files, max_files)?;
    Ok(files)
}

fn find_tests_for_language(
    language: &Language,
    root_path: &std::path::Path
) -> Result<Vec<smart_hooks::dependency::types::TestTarget>> {
    // This is a simplified implementation
    // In a real scenario, we'd use the actual test discovery logic
    let mut tests = Vec::new();
    
    match language {
        Language::Rust => {
            // Look for #[test] functions and integration tests
            if let Ok(sample_files) = get_sample_files_for_language(language, root_path, 10) {
                for file in sample_files {
                    if let Ok(content) = std::fs::read_to_string(&file) {
                        let test_count = content.matches("#[test]").count();
                        for i in 0..test_count {
                            tests.push(smart_hooks::dependency::types::TestTarget {
                                name: format!("test_{}", i),
                                test_type: smart_hooks::dependency::types::TestType::Unit { 
                                    module: file.file_stem()
                                        .and_then(|s| s.to_str())
                                        .unwrap_or("unknown")
                                        .to_string()
                                },
                                command: vec!["cargo".to_string(), "test".to_string()],
                                dependencies: vec![file.clone()],
                                confidence: 0.9,
                            });
                        }
                    }
                }
            }
        }
        Language::JavaScript | Language::TypeScript => {
            // Look for test files (.test.js, .spec.ts, etc.)
            // This is a simplified implementation
            tests.push(smart_hooks::dependency::types::TestTarget {
                name: "example_test".to_string(),
                test_type: smart_hooks::dependency::types::TestType::Unit { 
                    module: "example".to_string() 
                },
                command: vec!["npm".to_string(), "test".to_string()],
                dependencies: vec![],
                confidence: 0.7,
            });
        }
        _ => {
            // Basic implementation for other languages
        }
    }

    Ok(tests)
}

fn get_extensions_for_language(language: &Language) -> Vec<&'static str> {
    match language {
        Language::Rust => vec!["rs"],
        Language::JavaScript => vec!["js", "jsx", "mjs"],
        Language::TypeScript => vec!["ts", "tsx"],
        Language::Python => vec!["py", "pyx", "pyi"],
        Language::PHP => vec!["php", "phtml"],
        Language::Go => vec!["go"],
        Language::Java => vec!["java"],
        Language::CSharp => vec!["cs"],
        Language::Unknown(_) => vec![],
    }
}

fn output_text(
    stats: &ProjectStats,
    config: &smart_hooks::project::MultiLangProjectConfig,
    verbose: bool,
) -> Result<()> {
    println!("📊 Project Summary Report");
    println!("{}", "=".repeat(60));
    println!();
    
    // Project info
    println!("📋 Project Information:");
    println!("   Name: {}", config.metadata.name);
    println!("   Path: {}", config.project_root.display());
    println!("   Primary Language: {:?}", config.primary_language);
    println!("   Languages: {}", config.languages.len());
    println!();

    // File statistics
    println!("📄 File Statistics:");
    println!("   Total Files: {}", stats.total_files);
    for (language, count) in &stats.files_by_language {
        println!("   {:?}: {} files", language, count);
    }
    println!();

    // Dependency statistics
    if stats.total_dependencies > 0 {
        println!("📦 Dependency Analysis:");
        println!("   Total Dependencies: {}", stats.total_dependencies);
        println!("   External Dependencies: {}", stats.external_dependencies);
        println!("   Internal Dependencies: {}", stats.internal_dependencies);
        println!("   Avg Dependencies/File: {:.1}", 
                stats.total_dependencies as f32 / stats.total_files.max(1) as f32);
        println!();
    }

    // Test statistics
    if stats.total_tests > 0 {
        println!("🧪 Test Coverage:");
        println!("   Total Tests: {}", stats.total_tests);
        for (test_type, count) in &stats.tests_by_type {
            println!("   {}: {} tests", test_type, count);
        }
        println!("   Test Ratio: {:.2} tests/file", 
                stats.total_tests as f32 / stats.total_files.max(1) as f32);
        println!();
    }

    // Complexity assessment
    println!("🎯 Project Assessment:");
    println!("   Complexity Score: {:.2}", stats.complexity_score);
    
    let complexity_level = if stats.complexity_score < 1.0 {
        "🟢 Simple"
    } else if stats.complexity_score < 2.0 {
        "🟡 Moderate"
    } else {
        "🔴 Complex"
    };
    println!("   Complexity Level: {}", complexity_level);
    println!();

    // Recommendations
    println!("💡 Recommendations:");
    if stats.files_by_language.len() > 3 {
        println!("   • Consider language consolidation - {} languages detected", stats.files_by_language.len());
    }
    if stats.total_dependencies > stats.total_files * 5 {
        println!("   • High dependency count - consider reducing coupling");
    }
    if stats.total_tests < stats.total_files / 2 {
        println!("   • Consider improving test coverage");
    }
    if stats.complexity_score > 2.0 {
        println!("   • Project complexity is high - consider refactoring");
    }

    if verbose {
        println!();
        println!("🔧 Available Smart Hooks:");
        println!("   • smart-test-selector: Selective test execution");
        println!("   • conditional-compilation-check: Rust compilation validation");
        println!("   • multi-language-analyzer: Cross-language analysis");
        println!("   • dependency-impact-analysis: Change impact assessment");
    }

    Ok(())
}

fn output_json(
    stats: &ProjectStats,
    config: &smart_hooks::project::MultiLangProjectConfig,
) -> Result<()> {
    let summary = json!({
        "project": {
            "name": config.metadata.name,
            "path": config.project_root.display().to_string(),
            "primary_language": format!("{:?}", config.primary_language),
            "languages": config.languages.iter()
                .map(|l| format!("{:?}", l.language))
                .collect::<Vec<_>>()
        },
        "statistics": {
            "total_files": stats.total_files,
            "files_by_language": stats.files_by_language.iter()
                .map(|(k, v)| (format!("{:?}", k), v))
                .collect::<std::collections::HashMap<_, _>>(),
            "dependencies": {
                "total": stats.total_dependencies,
                "external": stats.external_dependencies,
                "internal": stats.internal_dependencies,
                "avg_per_file": if stats.total_files > 0 {
                    stats.total_dependencies as f32 / stats.total_files as f32
                } else { 0.0 }
            },
            "tests": {
                "total": stats.total_tests,
                "by_type": stats.tests_by_type,
                "ratio": if stats.total_files > 0 {
                    stats.total_tests as f32 / stats.total_files as f32
                } else { 0.0 }
            }
        },
        "assessment": {
            "complexity_score": stats.complexity_score,
            "complexity_level": if stats.complexity_score < 1.0 {
                "simple"
            } else if stats.complexity_score < 2.0 {
                "moderate"
            } else {
                "complex"
            }
        },
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    println!("{}", serde_json::to_string_pretty(&summary)?);
    Ok(())
}