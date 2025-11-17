/// Auto-formatting implementation using multi-language project detection
use anyhow::{Context, Result};
use smart_hooks::project::{Language, MultiLangProjectDiscovery};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

pub async fn run(
    files: Vec<String>,
    check_only: bool,
    language_filter: Option<String>,
    verbose: bool,
) -> Result<()> {
    if verbose {
        println!("🎨 Smart Hooks Auto-Formatter");
        println!("Files to process: {}", files.len());
    }

    if files.is_empty() {
        println!("⏭️  No files to format");
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

    let mut total_formatted = 0;
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
            match format_files_for_language(&language, &file_list, check_only, verbose, lang_config)
                .await
            {
                Ok(count) => {
                    total_formatted += count;
                    if verbose && count > 0 {
                        println!("✅ Formatted {} {:?} files", count, language);
                    }
                }
                Err(e) => {
                    total_errors += 1;
                    eprintln!("❌ Failed to format {:?} files: {}", language, e);
                }
            }
        } else if verbose {
            println!("⚠️  No formatter configuration found for {:?}", language);
        }
    }

    // Summary
    if verbose || total_formatted > 0 || total_errors > 0 {
        println!("\n📋 Format Summary:");
        println!("   Formatted: {} files", total_formatted);
        if total_errors > 0 {
            println!("   Errors: {} languages", total_errors);
        }
        if check_only && total_formatted > 0 {
            println!(
                "   Check mode: Found {} files that need formatting",
                total_formatted
            );
            return Err(anyhow::anyhow!("Found files that need formatting"));
        }
    }

    Ok(())
}

async fn format_files_for_language(
    language: &Language,
    files: &[String],
    check_only: bool,
    verbose: bool,
    lang_config: &smart_hooks::project::LanguageConfig,
) -> Result<usize> {
    if files.is_empty() {
        return Ok(0);
    }

    // Get the format command from language config
    let format_command = if let Some(cmd) = &lang_config.commands.format_command {
        cmd.clone()
    } else {
        // Fallback to default commands
        match language {
            Language::Rust => vec!["cargo".to_string(), "fmt".to_string()],
            Language::TypeScript | Language::JavaScript => {
                vec![
                    "npx".to_string(),
                    "prettier".to_string(),
                    "--write".to_string(),
                ]
            }
            Language::Python => vec!["python".to_string(), "-m".to_string(), "black".to_string()],
            Language::PHP => vec!["vendor/bin/php-cs-fixer".to_string(), "fix".to_string()],
            _ => {
                if verbose {
                    println!("⚠️  No formatter available for {:?}", language);
                }
                return Ok(0);
            }
        }
    };

    // Modify command for check-only mode
    let mut final_command = format_command.clone();
    if check_only {
        match language {
            Language::Rust => {
                final_command.push("--check".to_string());
            }
            Language::TypeScript | Language::JavaScript => {
                // Replace --write with --check
                if let Some(pos) = final_command.iter().position(|x| x == "--write") {
                    final_command[pos] = "--check".to_string();
                }
            }
            Language::Python => {
                final_command.push("--check".to_string());
            }
            Language::PHP => {
                final_command.push("--dry-run".to_string());
            }
            _ => {}
        }
    }

    // For some formatters, we need to run on individual files
    // For others, we can run on all files at once
    match language {
        Language::Rust => {
            // Cargo fmt works on the whole project
            if verbose {
                println!("🔧 Running: {}", final_command.join(" "));
            }

            let mut cmd = Command::new(&final_command[0]);
            cmd.args(&final_command[1..]);

            let output = cmd.output().context("Failed to run cargo fmt")?;

            if !output.status.success() {
                if check_only {
                    return Err(anyhow::anyhow!("Code needs formatting"));
                } else {
                    return Err(anyhow::anyhow!(
                        "Formatting failed: {}",
                        String::from_utf8_lossy(&output.stderr)
                    ));
                }
            }

            Ok(files.len())
        }
        Language::TypeScript | Language::JavaScript | Language::Python | Language::PHP => {
            // Run on individual files
            let mut formatted_count = 0;

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
                    .with_context(|| format!("Failed to format file: {}", file))?;

                if output.status.success() {
                    formatted_count += 1;
                } else if check_only {
                    // In check mode, non-zero exit usually means formatting is needed
                    formatted_count += 1;
                } else {
                    eprintln!(
                        "⚠️  Failed to format {}: {}",
                        file,
                        String::from_utf8_lossy(&output.stderr)
                    );
                }
            }

            Ok(formatted_count)
        }
        _ => {
            if verbose {
                println!("⚠️  Formatting not implemented for {:?}", language);
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