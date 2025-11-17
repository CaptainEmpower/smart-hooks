/// Multi-Language Project Analyzer
/// Demonstrates smart-hooks multi-language project discovery and dependency analysis
use clap::{Parser, Subcommand};
use smart_hooks::dependency::MultiLangDependencyAnalyzer;
use smart_hooks::project::{Language, MultiLangProjectDiscovery};
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "multi-lang-analyzer")]
#[command(about = "Analyze multi-language projects for intelligent testing")]
#[command(version = "1.0")]
struct Cli {
    /// Project directory to analyze
    #[arg(short, long, default_value = ".")]
    project_dir: PathBuf,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Discover programming languages in the project
    Discover {
        /// Show detailed language configurations
        #[arg(long)]
        detailed: bool,
    },
    /// Analyze dependencies across all languages
    Analyze {
        /// Focus on specific language (rust, typescript, python, php)
        #[arg(long)]
        language: Option<String>,

        /// Show dependency graph
        #[arg(long)]
        graph: bool,
    },
    /// Find tests that should run for given changed files
    Tests {
        /// Files that have changed (relative to project root)
        files: Vec<PathBuf>,

        /// Show test commands that would be executed
        #[arg(long)]
        show_commands: bool,
    },
    /// Show project summary across all languages
    Summary,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if cli.verbose {
        println!("🔍 Analyzing project at: {}", cli.project_dir.display());
    }

    // Discover the project structure
    let project_config = match MultiLangProjectDiscovery::discover(&cli.project_dir) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("❌ Failed to discover project structure: {}", e);
            eprintln!("   Make sure you're in a directory with supported programming languages");
            std::process::exit(1);
        }
    };

    if cli.verbose {
        println!(
            "📊 Project: {} (Primary language: {:?})",
            project_config.metadata.name, project_config.primary_language
        );
    }

    match cli.command {
        Commands::Discover { detailed } => {
            discover_languages(&project_config, detailed, cli.verbose)?;
        }
        Commands::Analyze { language, graph } => {
            analyze_dependencies(&project_config, language, graph, cli.verbose)?;
        }
        Commands::Tests {
            files,
            show_commands,
        } => {
            find_affected_tests(&project_config, &files, show_commands, cli.verbose)?;
        }
        Commands::Summary => {
            show_project_summary(&project_config, cli.verbose)?;
        }
    }

    Ok(())
}

fn discover_languages(
    project_config: &smart_hooks::project::MultiLangProjectConfig,
    detailed: bool,
    verbose: bool,
) -> anyhow::Result<()> {
    println!("\n🌐 Multi-Language Project Discovery");
    println!("===================================");

    println!("\n📋 **Detected Languages:**");
    for (i, lang_config) in project_config.languages.iter().enumerate() {
        let is_primary = lang_config.language == project_config.primary_language;
        let status = if is_primary {
            "🎯 Primary"
        } else {
            "📦 Detected"
        };

        println!("  {}. {:?} {}", i + 1, lang_config.language, status);

        if detailed || verbose {
            println!("     Package Manager: {:?}", lang_config.package_manager);
            println!("     Test Framework:  {:?}", lang_config.test_framework);
            println!(
                "     Source Dirs:     {} directories",
                lang_config.source_dirs.len()
            );

            if verbose {
                for dir in &lang_config.source_dirs {
                    println!("       - {}", dir.display());
                }
            }

            println!(
                "     File Patterns:   {}",
                lang_config.file_patterns.join(", ")
            );
            println!();
        }
    }

    println!("🏗️  **Build Configuration:**");
    println!(
        "     Build Tool: {:?}",
        project_config.build_config.build_tool
    );
    println!(
        "     Targets:    {}",
        project_config.build_config.targets.join(", ")
    );

    println!("\n🧪 **Test Strategy:**");
    println!(
        "     Selection Mode:        {:?}",
        project_config.test_strategy.selection_mode
    );
    println!(
        "     Cross-Language Tests:  {}",
        project_config.test_strategy.cross_language_testing
    );
    println!(
        "     Integration Tests:     {}",
        project_config.test_strategy.run_integration_tests
    );
    println!(
        "     E2E Tests:             {}",
        project_config.test_strategy.run_e2e_tests
    );

    if let Some(ai_config) = &project_config.test_strategy.ai_config {
        println!("\n🤖 **AI Configuration:**");
        println!(
            "     Confidence Threshold:  {:.1}%",
            ai_config.confidence_threshold * 100.0
        );
        println!(
            "     BDD Selection:         {}",
            ai_config.enable_bdd_selection
        );
        println!(
            "     Cross-Language Analysis: {}",
            ai_config.cross_language_analysis
        );
    }

    Ok(())
}

fn analyze_dependencies(
    project_config: &smart_hooks::project::MultiLangProjectConfig,
    language_filter: Option<String>,
    show_graph: bool,
    verbose: bool,
) -> anyhow::Result<()> {
    println!("\n🔗 Multi-Language Dependency Analysis");
    println!("=====================================");

    let analyzer = MultiLangDependencyAnalyzer::new(project_config.clone());

    println!("📊 Analyzing project dependencies...");
    let all_dependencies = analyzer.analyze_project_dependencies()?;

    if all_dependencies.is_empty() {
        println!("⚠️  No dependencies found. Make sure source files exist.");
        return Ok(());
    }

    // Filter by language if specified
    let target_language = if let Some(lang_str) = language_filter {
        match lang_str.to_lowercase().as_str() {
            "rust" => Some(Language::Rust),
            "typescript" | "ts" => Some(Language::TypeScript),
            "javascript" | "js" => Some(Language::JavaScript),
            "python" | "py" => Some(Language::Python),
            "php" => Some(Language::PHP),
            "go" => Some(Language::Go),
            "java" => Some(Language::Java),
            "csharp" | "c#" | "cs" => Some(Language::CSharp),
            _ => {
                eprintln!("⚠️  Unknown language: {}. Showing all languages.", lang_str);
                None
            }
        }
    } else {
        None
    };

    println!("\n📁 **Dependency Summary:**");
    let mut total_files = 0;
    let mut total_dependencies = 0;

    for (file_path, dependencies) in &all_dependencies {
        // Skip if filtering by language and this file doesn't match
        if let Some(target_lang) = &target_language {
            if let Some(ext) = file_path.extension().and_then(|s| s.to_str()) {
                let file_lang = match ext {
                    "rs" => Language::Rust,
                    "ts" | "tsx" => Language::TypeScript,
                    "js" | "jsx" | "mjs" => Language::JavaScript,
                    "py" | "pyx" | "pyi" => Language::Python,
                    "php" | "phtml" => Language::PHP,
                    "go" => Language::Go,
                    "java" => Language::Java,
                    "cs" => Language::CSharp,
                    _ => continue,
                };

                if file_lang != *target_lang {
                    continue;
                }
            }
        }

        total_files += 1;
        total_dependencies += dependencies.len();

        if verbose || show_graph {
            println!("  📄 {}", file_path.display());
            for dep in dependencies {
                println!("     └─ {} (weight: {:.1})", dep.name, dep.weight);
            }
            if !dependencies.is_empty() {
                println!();
            }
        }
    }

    println!("📈 **Statistics:**");
    println!("     Files analyzed:      {}", total_files);
    println!("     Total dependencies:  {}", total_dependencies);
    println!(
        "     Average deps/file:   {:.1}",
        if total_files > 0 {
            total_dependencies as f32 / total_files as f32
        } else {
            0.0
        }
    );

    // Show language breakdown
    println!("\n🌍 **By Language:**");
    let mut by_language = std::collections::HashMap::new();
    for (file_path, dependencies) in &all_dependencies {
        if let Some(ext) = file_path.extension().and_then(|s| s.to_str()) {
            let lang = match ext {
                "rs" => "Rust",
                "ts" | "tsx" => "TypeScript",
                "js" | "jsx" | "mjs" => "JavaScript",
                "py" | "pyx" | "pyi" => "Python",
                "php" | "phtml" => "PHP",
                "go" => "Go",
                "java" => "Java",
                "cs" => "C#",
                _ => "Other",
            };

            let entry = by_language.entry(lang).or_insert((0, 0));
            entry.0 += 1; // files
            entry.1 += dependencies.len(); // dependencies
        }
    }

    for (lang, (files, deps)) in by_language {
        println!("     {:<12} {} files, {} dependencies", lang, files, deps);
    }

    Ok(())
}

fn find_affected_tests(
    project_config: &smart_hooks::project::MultiLangProjectConfig,
    changed_files: &[PathBuf],
    show_commands: bool,
    verbose: bool,
) -> anyhow::Result<()> {
    println!("\n🎯 Cross-Language Test Impact Analysis");
    println!("======================================");

    if changed_files.is_empty() {
        println!("⚠️  No files provided. Specify files that have changed.");
        return Ok(());
    }

    // Convert to absolute paths relative to project root
    let mut absolute_files = Vec::new();
    for file in changed_files {
        let absolute = if file.is_absolute() {
            file.clone()
        } else {
            project_config.project_root.join(file)
        };
        absolute_files.push(absolute);
    }

    println!("📝 **Changed Files:**");
    for (i, file) in absolute_files.iter().enumerate() {
        let lang = detect_file_language(file);
        println!("  {}. {} ({:?})", i + 1, file.display(), lang);
    }

    let analyzer = MultiLangDependencyAnalyzer::new(project_config.clone());

    println!("\n🔍 Finding affected tests...");
    let test_targets = analyzer.find_cross_language_affected_tests(&absolute_files)?;

    if test_targets.is_empty() {
        println!("✅ No specific tests found for the changed files.");
        println!(
            "   Consider running the full test suite or adding more specific test discovery rules."
        );
        return Ok(());
    }

    println!("\n🧪 **Affected Tests:**");
    for (i, target) in test_targets.iter().enumerate() {
        println!(
            "  {}. {} (confidence: {:.1}%)",
            i + 1,
            target.name,
            target.confidence * 100.0
        );

        if verbose {
            match &target.test_type {
                smart_hooks::dependency::TestType::Unit { module } => {
                    println!("     Type: Unit test (module: {})", module);
                }
                smart_hooks::dependency::TestType::Integration { test_file } => {
                    println!("     Type: Integration test ({})", test_file);
                }
                smart_hooks::dependency::TestType::Doc { source_file } => {
                    println!("     Type: Documentation test ({})", source_file);
                }
                smart_hooks::dependency::TestType::Benchmark { bench_name } => {
                    println!("     Type: Benchmark ({})", bench_name);
                }
                smart_hooks::dependency::TestType::Custom { command } => {
                    println!("     Type: Custom ({})", command);
                }
            }
        }

        if show_commands || verbose {
            println!("     Command: {}", target.command.join(" "));
        }

        if verbose {
            println!("     Dependencies: {} files", target.dependencies.len());
            for dep in &target.dependencies {
                println!("       - {}", dep.display());
            }
            println!();
        }
    }

    if show_commands {
        println!("\n⚡ **Recommended Test Commands:**");
        for target in &test_targets {
            if target.confidence >= 0.7 {
                println!("  🔥 {}", target.command.join(" "));
            } else if target.confidence >= 0.5 {
                println!("  ⚡ {}", target.command.join(" "));
            } else {
                println!("  📝 {}", target.command.join(" "));
            }
        }

        println!("\n💡 **Legend:**");
        println!("     🔥 High confidence (≥70%)");
        println!("     ⚡ Medium confidence (≥50%)");
        println!("     📝 Low confidence (<50%)");
    }

    Ok(())
}

fn show_project_summary(
    project_config: &smart_hooks::project::MultiLangProjectConfig,
    verbose: bool,
) -> anyhow::Result<()> {
    println!("\n📊 Multi-Language Project Summary");
    println!("=================================");

    println!("\n🏷️  **Project Metadata:**");
    println!("     Name:         {}", project_config.metadata.name);
    println!("     Version:      {}", project_config.metadata.version);
    if let Some(desc) = &project_config.metadata.description {
        println!("     Description:  {}", desc);
    }
    if let Some(repo) = &project_config.metadata.repository {
        println!("     Repository:   {}", repo);
    }

    println!("\n🌍 **Language Ecosystem:**");
    println!(
        "     Primary Language:     {:?}",
        project_config.primary_language
    );
    println!(
        "     Total Languages:      {}",
        project_config.languages.len()
    );
    println!(
        "     Build Tool:           {:?}",
        project_config.build_config.build_tool
    );

    // Show quick stats for each language
    for lang_config in &project_config.languages {
        println!("\n     📦 {:?}", lang_config.language);
        println!(
            "        Package Manager:  {:?}",
            lang_config.package_manager
        );
        println!("        Test Framework:   {:?}", lang_config.test_framework);
        println!(
            "        Source Dirs:      {}",
            lang_config.source_dirs.len()
        );

        // Count files
        let mut file_count = 0;
        for source_dir in &lang_config.source_dirs {
            if let Ok(entries) = std::fs::read_dir(source_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                            let lang_extensions = lang_config.language.file_extensions();
                            if lang_extensions.contains(&ext) {
                                file_count += 1;
                            }
                        }
                    }
                }
            }
        }
        println!("        Estimated Files:  {}", file_count);
    }

    // Analyze dependencies quickly
    println!("\n🔗 **Dependency Analysis:**");
    let analyzer = MultiLangDependencyAnalyzer::new(project_config.clone());

    match analyzer.analyze_project_dependencies() {
        Ok(deps) => {
            let total_files = deps.len();
            let total_deps: usize = deps.values().map(|d| d.len()).sum();

            println!("     Files with dependencies: {}", total_files);
            println!("     Total dependencies:      {}", total_deps);
            if total_files > 0 {
                println!(
                    "     Average deps per file:   {:.1}",
                    total_deps as f32 / total_files as f32
                );
            }
        }
        Err(e) if verbose => {
            println!("     ⚠️  Could not analyze dependencies: {}", e);
        }
        Err(_) => {
            println!("     ⚠️  Dependency analysis not available");
        }
    }

    println!("\n✨ **Smart Hooks Capabilities:**");
    println!("     ✅ Multi-language project discovery");
    println!("     ✅ Cross-language dependency analysis");
    println!("     ✅ Intelligent test selection");
    println!("     ✅ Language-specific build tool detection");
    println!("     ✅ Unified testing workflow");

    if project_config.test_strategy.cross_language_testing {
        println!("     🌟 Cross-language impact analysis enabled");
    }

    if project_config.test_strategy.ai_config.is_some() {
        println!("     🤖 AI-powered test selection available");
    }

    Ok(())
}

fn detect_file_language(file_path: &Path) -> Language {
    if let Some(extension) = file_path.extension().and_then(|s| s.to_str()) {
        match extension {
            "rs" => Language::Rust,
            "ts" | "tsx" => Language::TypeScript,
            "js" | "jsx" | "mjs" => Language::JavaScript,
            "py" | "pyx" | "pyi" => Language::Python,
            "php" | "phtml" => Language::PHP,
            "go" => Language::Go,
            "java" => Language::Java,
            "cs" => Language::CSharp,
            _ => Language::Unknown(extension.to_string()),
        }
    } else {
        Language::Unknown("no-extension".to_string())
    }
}