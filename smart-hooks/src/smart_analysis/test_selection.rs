//! Turn a list of changed files into a test plan, then run it.

use anyhow::Result;
use serde_json::json;

use smart_hooks::TestPlan;

/// Analyse `files`, build a test plan and execute it.
///
/// With `dry_run`, the plan is reported and nothing is executed — useful for
/// seeing what a hook *would* run before trusting it with a commit.
pub fn run_smart_test_selector(files: Vec<String>, json_output: bool, dry_run: bool) -> Result<()> {
    if files.is_empty() {
        report_empty(json_output)?;
        return Ok(());
    }

    if !json_output {
        println!(
            "🔍 Analysing {} changed file(s) for affected tests...",
            files.len()
        );
    }

    let test_plan = smart_hooks::create_test_plan(&files)?;

    if json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&plan_json(&test_plan, &files, dry_run))?
        );
    } else if dry_run {
        print_plan(&test_plan);
    }

    if dry_run {
        return Ok(());
    }

    smart_hooks::execute_test_plan(&test_plan)
}

fn report_empty(json_output: bool) -> Result<()> {
    if json_output {
        let empty = json!({
            "status": "skipped",
            "message": "No files to analyse",
            "files_count": 0,
            "test_plan": null,
        });
        println!("{}", serde_json::to_string_pretty(&empty)?);
    } else {
        println!("No files to analyse, skipping test selection");
    }
    Ok(())
}

fn plan_json(plan: &TestPlan, files: &[String], dry_run: bool) -> serde_json::Value {
    let mut unit_tests: Vec<&String> = plan.unit_tests.iter().collect();
    unit_tests.sort();

    json!({
        "status": if dry_run { "planned" } else { "completed" },
        "files_analysed": files.len(),
        "test_plan": {
            "unit_tests": unit_tests,
            "integration_tests": plan.integration_tests,
            "bdd_tests": plan.bdd_tests,
        },
    })
}

fn print_plan(plan: &TestPlan) {
    if plan.unit_tests.is_empty() && !plan.integration_tests && !plan.bdd_tests {
        println!("  (no tests selected)");
        return;
    }

    let mut unit_tests: Vec<&String> = plan.unit_tests.iter().collect();
    unit_tests.sort();
    for test in unit_tests {
        println!("  unit: {test}");
    }
    if plan.integration_tests {
        println!("  integration: all");
    }
    if plan.bdd_tests {
        println!("  bdd: all");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn plan(unit: &[&str], integration: bool, bdd: bool) -> TestPlan {
        TestPlan {
            unit_tests: unit.iter().map(|s| s.to_string()).collect::<HashSet<_>>(),
            integration_tests: integration,
            bdd_tests: bdd,
        }
    }

    #[test]
    fn empty_file_list_runs_nothing_and_succeeds() {
        run_smart_test_selector(vec![], true, false).expect("an empty file list is not an error");
    }

    #[test]
    fn dry_run_reports_the_plan_without_executing_it() {
        // `src/does_not_exist.rs` selects nothing, so a non-dry run would also
        // be a no-op; the assertion that matters is that dry_run returns Ok
        // without invoking cargo.
        run_smart_test_selector(vec!["src/does_not_exist.rs".to_string()], true, true)
            .expect("dry run must not fail");
    }

    #[test]
    fn plan_json_sorts_unit_tests_and_marks_a_dry_run_as_planned() {
        let value = plan_json(
            &plan(&["b::two", "a::one"], true, false),
            &["x.rs".into()],
            true,
        );

        assert_eq!(value["status"], "planned");
        assert_eq!(value["files_analysed"], 1);
        assert_eq!(
            value["test_plan"]["unit_tests"],
            json!(["a::one", "b::two"])
        );
        assert_eq!(value["test_plan"]["integration_tests"], json!(true));
        assert_eq!(value["test_plan"]["bdd_tests"], json!(false));
    }

    #[test]
    fn plan_json_marks_an_executed_run_as_completed() {
        let value = plan_json(&plan(&[], false, false), &[], false);

        assert_eq!(value["status"], "completed");
        assert_eq!(value["test_plan"]["unit_tests"], json!([]));
    }
}
