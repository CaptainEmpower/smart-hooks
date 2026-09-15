//! CLI surface tests for the narrowed binary.
//!
//! These assert **exit codes**, not just output. The defect recorded in
//! ADR 0001 — a hook that printed a failure and exited 0 — was reachable only
//! because nothing in the suite ever looked at an exit status.

use serde_json::Value;
use std::path::PathBuf;
use std::process::{Command, Output};

fn binary() -> PathBuf {
    // target/debug/deps/<test binary> -> target/debug/smart-hooks
    let mut path = std::env::current_exe().expect("test binary path");
    path.pop();
    if path.ends_with("deps") {
        path.pop();
    }
    path.join(format!("smart-hooks{}", std::env::consts::EXE_SUFFIX))
}

fn run(args: &[&str]) -> Output {
    Command::new(binary())
        .args(args)
        .output()
        .expect("failed to run the smart-hooks binary")
}

fn stdout_of(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout was not valid UTF-8")
}

#[test]
fn no_files_succeeds_and_selects_nothing() {
    let output = run(&["test", "--json"]);

    assert_eq!(output.status.code(), Some(0));

    let value: Value = serde_json::from_str(&stdout_of(&output)).expect("expected JSON on stdout");
    assert_eq!(value["status"], "skipped");
    assert_eq!(value["files_count"], 0);
    assert_eq!(value["test_plan"], Value::Null);
}

#[test]
fn a_repository_relative_path_selects_the_matching_module() {
    // The path shape a hook runner passes. Before the fix in this PR, module
    // extraction required a leading "/src/" and this selected nothing.
    let output = run(&["test", "--dry-run", "--json", "src/analysis/config.rs"]);

    assert_eq!(output.status.code(), Some(0));

    let value: Value = serde_json::from_str(&stdout_of(&output)).expect("expected JSON on stdout");
    assert_eq!(value["status"], "planned");
    assert_eq!(value["files_analysed"], 1);
    assert_eq!(
        value["test_plan"]["unit_tests"],
        serde_json::json!(["analysis::config"])
    );
}

#[test]
fn dry_run_reports_a_plan_without_running_cargo() {
    let output = run(&["test", "--dry-run", "src/analysis/config.rs"]);

    assert_eq!(output.status.code(), Some(0));

    let stdout = stdout_of(&output);
    assert!(
        stdout.contains("unit: analysis::config"),
        "expected the plan to be printed, got: {stdout}"
    );
    // `execute_test_plan` announces the tests it runs; a dry run must not.
    assert!(
        !stdout.contains("Running:"),
        "dry run must not execute the plan, got: {stdout}"
    );
}

#[test]
fn non_rust_files_select_nothing() {
    let output = run(&["test", "--dry-run", "--json", "README.md", "Cargo.toml"]);

    assert_eq!(output.status.code(), Some(0));

    let value: Value = serde_json::from_str(&stdout_of(&output)).expect("expected JSON on stdout");
    assert_eq!(
        value["test_plan"]["unit_tests"],
        serde_json::json!(Vec::<String>::new())
    );
    assert_eq!(value["test_plan"]["integration_tests"], false);
}

#[test]
fn contradictory_global_flags_fail_with_a_non_zero_exit() {
    let output = run(&["--quiet", "--verbose", "test"]);

    assert_ne!(
        output.status.code(),
        Some(0),
        "quiet + verbose must not exit 0"
    );
    let stderr = String::from_utf8(output.stderr).expect("stderr was not valid UTF-8");
    assert!(
        stderr.contains("Cannot specify both --quiet and --verbose"),
        "unexpected stderr: {stderr}"
    );
}

#[test]
fn removed_subcommands_exit_non_zero_rather_than_silently_passing() {
    // ADR 0001/0002 removed these. A hook entry still naming one must fail
    // loudly; the old CLI parsed `test selective` as a filename and passed.
    for removed in [
        "run", "install", "list", "validate", "lint", "format", "summary", "analyze",
    ] {
        let output = run(&[removed]);
        assert_eq!(
            output.status.code(),
            Some(2),
            "`smart-hooks {removed}` should be a usage error"
        );
    }
}

#[test]
fn json_mode_emits_exactly_one_document_after_running_the_tests() {
    // Regression for the review on #8: the JSON path used to print a
    // "completed" document *before* execution, after which the executor wrote
    // progress lines to stdout — so stdout was neither parseable nor truthful.
    let output = run(&["test", "--json", "src/utilities/module_utils.rs"]);

    let stdout = stdout_of(&output);
    let value: Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("stdout must be one JSON document, got {e}:\n{stdout}"));

    // Status is the real outcome, and every suite is accounted for.
    assert!(
        value["status"] == "completed" || value["status"] == "failed",
        "unexpected status: {}",
        value["status"]
    );
    assert_eq!(
        value["status"] == "completed",
        output.status.code() == Some(0),
        "status must agree with the exit code"
    );
    assert!(value["suites"].is_array(), "suites missing: {value}");
    assert!(
        !stdout.contains("Running:"),
        "human progress leaked into JSON output:\n{stdout}"
    );
}

#[test]
fn structural_paths_do_not_schedule_an_absent_test_target() {
    // Regression for the review on #8: a path under `src/core/` used to set a
    // BDD flag whose executor ran `cargo test --test cucumber_tests`, a target
    // this repository does not define — failing the hook on a valid commit.
    let output = run(&["test", "--dry-run", "--json", "src/core/engine.rs"]);

    assert_eq!(output.status.code(), Some(0));
    let value: Value = serde_json::from_str(&stdout_of(&output)).expect("expected JSON");
    assert!(
        value["test_plan"].get("bdd_tests").is_none(),
        "BDD selection is gone; plan still advertises it: {}",
        value["test_plan"]
    );
}
