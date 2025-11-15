/// Claude-powered BDD Feature Selector
/// Automatically selects which BDD features to run based on staged git changes
use clap::Parser;
use smart_hooks::analysis::bdd::BddFeatureSelection;
#[cfg(all(not(test), feature = "claude-ai"))]
use smart_hooks::analysis::bdd::{BddFeatureSelector, BddFileAnalyzer};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "claude-bdd-selector")]
#[command(about = "Use Claude AI to intelligently select BDD features based on staged changes")]
struct Args {
    /// Project root directory (defaults to current directory)
    #[arg(long, short = 'd')]
    project_dir: Option<PathBuf>,

    /// Project context description for better Claude analysis
    #[arg(long, short = 'c')]
    context: Option<String>,

    /// Only show what would be selected (dry-run)
    #[arg(long)]
    dry_run: bool,

    /// Output format (text, json)
    #[arg(long, default_value = "text")]
    output: String,

    /// Minimum confidence threshold for running BDD tests
    #[arg(long, default_value = "0.5")]
    confidence_threshold: f32,
}

#[cfg(feature = "claude-ai")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let project_root = args
        .project_dir
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    // Get staged changes
    println!("🔍 Analyzing staged changes...");
    let staged_files = BddFileAnalyzer::analyze_staged_files()?;

    if staged_files.is_empty() {
        println!("📝 No staged changes found. Stage some files first with 'git add'");
        return Ok(());
    }

    println!("📂 Found {} staged file(s):", staged_files.len());
    for file in &staged_files {
        println!(
            "  {} {}",
            match file.change_type.as_str() {
                "added" => "➕",
                "modified" => "📝",
                "deleted" => "🗑️",
                _ => "📄",
            },
            file.file_path
        );
    }

    // Discover available BDD features
    println!("\n🎭 Discovering BDD features...");
    let available_features = BddFeatureSelector::discover_features(&project_root)?;

    if available_features.is_empty() {
        println!(
            "⚠️  No BDD features found in common directories (features/, tests/features/, etc.)"
        );
        println!("   Consider creating .feature files for Behavior-Driven Development");
        return Ok(());
    }

    println!("📋 Found {} BDD feature(s):", available_features.len());
    for feature in &available_features {
        println!("  🎭 {}", feature);
    }

    // Get project context
    let project_context = args
        .context
        .unwrap_or_else(|| format!("Rust project at {}", project_root.display()));

    // Use Claude AI to select features
    println!("\n🤖 Asking Claude AI to select relevant BDD features...");
    let selection = BddFeatureSelector::select_features_static(&staged_files, &available_features)?;

    // Output results
    match args.output.as_str() {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&selection)?);
        }
        _ => {
            print_text_output(&selection, args.confidence_threshold, args.dry_run)?;
        }
    }

    Ok(())
}

#[allow(dead_code)]
fn print_text_output(
    selection: &BddFeatureSelection,
    threshold: f32,
    dry_run: bool,
) -> anyhow::Result<()> {
    println!("\n🎯 Claude AI Analysis Results");
    println!("═══════════════════════════════");

    println!("\n📊 **Analysis Summary:**");
    println!(
        "   Should run BDD tests: {}",
        if selection.should_run_bdd {
            "✅ Yes"
        } else {
            "❌ No"
        }
    );
    println!("   Confidence: {:.1}%", selection.confidence * 100.0);
    println!("   Reasoning: {}", selection.reasoning);

    if !selection.selected_features.is_empty() {
        println!("\n🎭 **Selected BDD Features:**");
        for feature in &selection.selected_features {
            let priority_icon = match feature.priority.as_str() {
                "high" => "🔥",
                "medium" => "⚡",
                "low" => "📝",
                _ => "📋",
            };

            println!(
                "   {} {} ({})",
                priority_icon, feature.feature_file, feature.priority
            );
            println!("      Scenarios: {}", feature.scenarios.join(", "));
            println!("      Reason: {}", feature.reason);
            println!();
        }
    }

    if !selection.suggested_test_focus.is_empty() {
        println!("🎯 **Suggested Test Focus:**");
        for focus in &selection.suggested_test_focus {
            println!("   • {}", focus);
        }
        println!();
    }

    if !selection.risk_areas.is_empty() {
        println!("⚠️  **Risk Areas to Validate:**");
        for risk in &selection.risk_areas {
            println!("   • {}", risk);
        }
        println!();
    }

    // Execution recommendations
    if selection.should_run_bdd && selection.confidence >= threshold {
        println!("💡 **Recommended Action:**");

        if dry_run {
            println!("   [DRY RUN] Would execute the following BDD tests:");
        } else {
            println!("   Run these BDD tests before committing:");
        }

        for feature in &selection.selected_features {
            if feature.priority == "high" {
                if feature.scenarios.contains(&"all scenarios".to_string()) {
                    println!("   🔥 cucumber {}", feature.feature_file);
                } else {
                    for scenario in &feature.scenarios {
                        println!(
                            "   🔥 cucumber {} -n \"{}\"",
                            feature.feature_file, scenario
                        );
                    }
                }
            }
        }

        for feature in &selection.selected_features {
            if feature.priority == "medium" {
                if feature.scenarios.contains(&"all scenarios".to_string()) {
                    println!("   ⚡ cucumber {}", feature.feature_file);
                } else {
                    for scenario in &feature.scenarios {
                        println!(
                            "   ⚡ cucumber {} -n \"{}\"",
                            feature.feature_file, scenario
                        );
                    }
                }
            }
        }

        println!("\n   Or run all selected features:");
        let feature_list = selection
            .selected_features
            .iter()
            .map(|f| f.feature_file.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        println!("   🎭 cucumber {}", feature_list);
    } else if selection.confidence < threshold {
        println!("📝 **Low Confidence - Optional BDD Testing:**");
        println!(
            "   Confidence {:.1}% is below threshold {:.1}%",
            selection.confidence * 100.0,
            threshold * 100.0
        );
        println!("   Consider running a smoke test or skipping BDD for these changes");
    } else {
        println!("✅ **No BDD Testing Needed:**");
        println!("   Changes don't appear to require behavioral validation");
    }

    Ok(())
}

#[cfg(not(feature = "claude-ai"))]
fn main() -> anyhow::Result<()> {
    eprintln!("Claude BDD selector requires the 'claude-ai' feature to be enabled");
    eprintln!("Run with: cargo run --bin claude-bdd-selector --features claude-ai");
    std::process::exit(1);
}