/// Auto-linting implementation using multi-language project detection
use anyhow::{Context, Result};
use smart_hooks::project::{Language, MultiLangProjectDiscovery};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

pub async fn run(
    files: Vec<String>,
    language_filter: Option<String>,
    fix: bool,
    verbose: bool,
) -> Result<()> {
    if verbose {
        println!("🧹 Smart Hooks Auto-Linter");
        println!("Files to process: {}", files.len());
    }

    if files.is_empty() {
        println!("⏭️  No files to lint");
        return Ok(());
    }

    // Discover the project configuration
    let project_config =
        MultiLangProjectDiscovery::discover(".").context("Failed to discover project structure")?;

    if verbose {
        println!(
            "📊 Project: {} (Primary language: {:?})",
            project_config.metadata.name, project_config.primary_language
        );
    }

    // Group files by language
    let files_by_language = group_files_by_language(&files);

    if verbose {
        for (lang, file_list) in &files_by_language {
            println!(
                "📝 {} files for {:?}: {}",
                file_list.len(),
                lang,
                file_list.len()
            );
        }
    }

    let mut total_linted = 0;
    let mut total_errors = 0;

    // Process each language group
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

        // Find the language configuration
        if let Some(lang_config) = project_config
            .languages
            .iter()
            .find(|c| c.language == language)
        {
            match lint_files_for_language(&language, &file_list, fix, verbose, lang_config).await {
                Ok(count) => {
                    total_linted += count;
                    if verbose && count > 0 {
                        println!("✅ Linted {} {:?} files", count, language);
                    }
                }
                Err(e) => {
                    total_errors += 1;
                    eprintln!("❌ Failed to lint {:?} files: {}", language, e);
                }
            }
        } else if verbose {
            println!("⚠️  No linter configuration found for {:?}", language);
        }
    }

    // Summary
    if verbose || total_linted > 0 || total_errors > 0 {
        println!("\n📋 Lint Summary:");
        println!("   Linted: {} files", total_linted);
        if total_errors > 0 {
            println!("   Errors: {} languages", total_errors);
        }
    }

    Ok(())
}

async fn lint_files_for_language(
    language: &Language,
    files: &[String],
    fix: bool,
    verbose: bool,
    lang_config: &smart_hooks::project::LanguageConfig,
) -> Result<usize> {
    if files.is_empty() {
        return Ok(0);
    }

    // Get the lint command from language config
    let lint_command = if let Some(cmd) = &lang_config.commands.lint_command {
        cmd.clone()
    } else {
        // Fallback to default commands
        match language {
            Language::Rust => vec!["cargo".to_string(), "clippy".to_string()],
            Language::TypeScript | Language::JavaScript => {
                vec!["npx".to_string(), "eslint".to_string()]
            }
            Language::Python => vec!["python".to_string(), "-m".to_string(), "flake8".to_string()],
            Language::PHP => vec!["vendor/bin/phpcs".to_string()],
            _ => {
                if verbose {
                    println!("⚠️  No linter available for {:?}", language);
                }
                return Ok(0);
            }
        }
    };

    // Modify command for fix mode
    let mut final_command = lint_command.clone();
    if fix {
        match language {
            Language::Rust => {
                final_command.push("--fix".to_string());
            }
            Language::TypeScript | Language::JavaScript => {
                final_command.push("--fix".to_string());
            }
            Language::Python => {
                // flake8 doesn't have auto-fix, use autopep8 instead
                final_command = vec![
                    "python".to_string(),
                    "-m".to_string(),
                    "autopep8".to_string(),
                    "--in-place".to_string(),
                ];
            }
            Language::PHP => {
                // Use php-cs-fixer instead of phpcs for fixing
                final_command = vec!["vendor/bin/php-cs-fixer".to_string(), "fix".to_string()];
            }
            _ => {}
        }
    }

    // For some linters, we need to run on individual files
    // For others, we can run on all files at once
    match language {
        Language::Rust => {
            // Cargo clippy works on the whole project
            if verbose {
                println!("🔧 Running: {}", final_command.join(" "));
            }

            let mut cmd = Command::new(&final_command[0]);
            cmd.args(&final_command[1..]);
            cmd.arg("--");
            cmd.arg("-D");
            cmd.arg("warnings");

            let output = cmd.output().context("Failed to run cargo clippy")?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!("⚠️  Clippy found issues:\n{}", stderr);
                if !fix {
                    return Err(anyhow::anyhow!("Linting found issues"));
                }
            }

            Ok(files.len())
        }
        Language::TypeScript | Language::JavaScript | Language::Python | Language::PHP => {
            // Run on individual files
            let mut linted_count = 0;
            let mut has_errors = false;

            for file in files {
                if !Path::new(file).exists() {
                    if verbose {
                        println!("⚠️  File not found: {}", file);
                    }
                    continue;
                }

                let mut cmd = Command::new(&final_command[0]);
                cmd.args(&final_command[1..]);
                cmd.arg(file);

                if verbose {
                    println!("🔧 Running: {} {}", final_command.join(" "), file);
                }

                let output = cmd
                    .output()
                    .with_context(|| format!("Failed to lint file: {}", file))?;

                if output.status.success() {
                    linted_count += 1;
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    let stdout = String::from_utf8_lossy(&output.stdout);

                    if fix {
                        // In fix mode, try to apply fixes but still show warnings
                        linted_count += 1;
                        if verbose && (!stderr.is_empty() || !stdout.is_empty()) {
                            println!("🔧 Fixed issues in {}", file);
                            if !stdout.is_empty() {
                                println!("   Output: {}", stdout.trim());
                            }
                        }
                    } else {
                        has_errors = true;
                        eprintln!("⚠️  Lint issues in {}", file);
                        if !stderr.is_empty() {
                            eprintln!("   Error: {}", stderr.trim());
                        }
                        if !stdout.is_empty() {
                            eprintln!("   Output: {}", stdout.trim());
                        }
                    }
                }
            }

            if has_errors && !fix {
                return Err(anyhow::anyhow!("Found linting issues"));
            }

            Ok(linted_count)
        }
        _ => {
            if verbose {
                println!("⚠️  Linting not implemented for {:?}", language);
            }
            Ok(0)
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