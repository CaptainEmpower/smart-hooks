//! Turn a list of changed files into a test plan, then run it.

use anyhow::Result;
use serde_json::{json, Value};

use smart_hooks::execution::plan_executor::{run_test_plan, ExecutionReport};
use smart_hooks::TestPlan;

/// Analyse `files`, build a test plan and execute it.
///
/// With `dry_run`, the plan is reported and nothing is executed — useful for
/// seeing what a hook *would* run before trusting it with a commit.
///
/// Under `json_output` exactly one JSON document reaches stdout, emitted after
/// execution so its status reflects what actually happened. Progress output
/// from the executor is suppressed so stdout stays parseable.
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

    if dry_run {
        if json_output {
            println!(
                "{}",
                serde_json::to_string_pretty(&plan_json(&test_plan, &files))?
            );
        } else {
            print_plan(&test_plan);
        }
        return Ok(());
    }

    let report = run_test_plan(&test_plan, json_output);

    if json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&result_json(&test_plan, &files, &report))?
        );
    }

    match report.failure_message() {
        Some(msg) => Err(anyhow::anyhow!(msg)),
        None => Ok(()),
    }
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

fn plan_value(plan: &TestPlan) -> Value {
    let mut unit_tests: Vec<&String> = plan.unit_tests.iter().collect();
    unit_tests.sort();
    json!({
        "unit_tests": unit_tests,
        "integration_tests": plan.integration_tests,
    })
}

fn plan_json(plan: &TestPlan, files: &[String]) -> Value {
    json!({
        "status": "planned",
        "files_analysed": files.len(),
        "test_plan": plan_value(plan),
    })
}

fn result_json(plan: &TestPlan, files: &[String], report: &ExecutionReport) -> Value {
    let suites: Vec<Value> = report
        .suites
        .iter()
        .map(|s| {
            json!({
                "name": s.name,
                "passed": s.passed,
                "error": s.error,
            })
        })
        .collect();

    json!({
        "status": if report.passed() { "completed" } else { "failed" },
        "files_analysed": files.len(),
        "test_plan": plan_value(plan),
        "suites": suites,
        "failure": report.failure_message(),
    })
}

fn print_plan(plan: &TestPlan) {
    if plan.unit_tests.is_empty() && !plan.integration_tests {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use smart_hooks::execution::plan_executor::SuiteOutcome;
    use std::collections::HashSet;

    fn plan(unit: &[&str], integration: bool) -> TestPlan {
        TestPlan {
            unit_tests: unit.iter().map(|s| s.to_string()).collect::<HashSet<_>>(),
            integration_tests: integration,
        }
    }

    #[test]
    fn empty_file_list_runs_nothing_and_succeeds() {
        run_smart_test_selector(vec![], true, false).expect("an empty file list is not an error");
    }

    #[test]
    fn dry_run_reports_the_plan_without_executing_it() {
        run_smart_test_selector(vec!["src/does_not_exist.rs".to_string()], true, true)
            .expect("dry run must not fail");
    }

    #[test]
    fn plan_json_sorts_unit_tests_and_is_marked_planned() {
        let value = plan_json(&plan(&["b::two", "a::one"], true), &["x.rs".into()]);

        assert_eq!(value["status"], "planned");
        assert_eq!(value["files_analysed"], 1);
        assert_eq!(
            value["test_plan"]["unit_tests"],
            json!(["a::one", "b::two"])
        );
        assert_eq!(value["test_plan"]["integration_tests"], json!(true));
    }

    #[test]
    fn result_json_reports_completed_only_when_every_suite_passed() {
        let report = ExecutionReport {
            suites: vec![SuiteOutcome {
                name: "a::one".into(),
                passed: true,
                error: None,
            }],
        };

        let value = result_json(&plan(&["a::one"], false), &["x.rs".into()], &report);

        assert_eq!(value["status"], "completed");
        assert_eq!(value["failure"], Value::Null);
        assert_eq!(
            value["suites"],
            json!([{"name": "a::one", "passed": true, "error": null}])
        );
    }

    #[test]
    fn result_json_reports_failure_rather_than_completion() {
        let report = ExecutionReport {
            suites: vec![
                SuiteOutcome {
                    name: "a::one".into(),
                    passed: true,
                    error: None,
                },
                SuiteOutcome {
                    name: "b::two".into(),
                    passed: false,
                    error: None,
                },
            ],
        };

        let value = result_json(
            &plan(&["a::one", "b::two"], false),
            &["x.rs".into()],
            &report,
        );

        assert_eq!(value["status"], "failed");
        assert_eq!(value["failure"], json!("Failed tests: b::two"));
        assert_eq!(value["suites"][1]["passed"], json!(false));
    }
}
