//! Execute a test plan and report what happened.
//!
//! Execution is separated from reporting so that `--json` can emit a single
//! document describing the real outcome, rather than announcing completion
//! before the tests have run.

use crate::analysis::dependency_mapper::TestPlan;
use crate::execution::test_runner;
use anyhow::Result;

/// What happened to one selected suite.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuiteOutcome {
    /// Human-readable suite name, e.g. `analysis::config` or `integration`.
    pub name: String,
    pub passed: bool,
    /// Set when the suite could not be run at all, as opposed to failing.
    pub error: Option<String>,
}

impl SuiteOutcome {
    fn label(&self) -> String {
        match &self.error {
            Some(e) => format!("{} ({})", self.name, e),
            None => self.name.clone(),
        }
    }
}

/// The result of executing a whole plan.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExecutionReport {
    pub suites: Vec<SuiteOutcome>,
}

impl ExecutionReport {
    pub fn passed(&self) -> bool {
        self.suites.iter().all(|s| s.passed)
    }

    pub fn failures(&self) -> Vec<&SuiteOutcome> {
        self.suites.iter().filter(|s| !s.passed).collect()
    }

    /// The error a failing run should surface, or `None` when everything passed.
    pub fn failure_message(&self) -> Option<String> {
        let failed = self.failures();
        if failed.is_empty() {
            return None;
        }
        Some(format!(
            "Failed tests: {}",
            failed
                .iter()
                .map(|s| s.label())
                .collect::<Vec<_>>()
                .join(", ")
        ))
    }
}

/// Run every suite the plan selected, printing progress unless `quiet`.
///
/// Returns the outcome of each suite rather than an error, so the caller can
/// decide how to report it. Use [`execute_test_plan`] for the plain
/// human-facing behaviour.
pub fn run_test_plan(plan: &TestPlan, quiet: bool) -> ExecutionReport {
    let mut report = ExecutionReport::default();

    if !plan.unit_tests.is_empty() {
        if !quiet {
            println!("🧪 Running targeted unit tests...");
        }
        let mut unit_tests: Vec<&String> = plan.unit_tests.iter().collect();
        unit_tests.sort();
        for test in unit_tests {
            if !quiet {
                println!("  Running: {test}");
            }
            report
                .suites
                .push(record(test, test_runner::run_unit_test(test), quiet));
        }
    }

    if plan.integration_tests {
        if !quiet {
            println!("🔗 Running integration tests for core changes...");
        }
        report.suites.push(record(
            "integration",
            test_runner::run_integration_tests(),
            quiet,
        ));
    }

    if !quiet {
        match report.failure_message() {
            None if report.suites.is_empty() => {
                println!("✅ No affected tests detected, nothing to run")
            }
            None => println!(
                "✅ Completed {} test suite(s) based on change analysis",
                report.suites.len()
            ),
            Some(_) => {}
        }
    }

    report
}

fn record(name: &str, result: Result<bool>, quiet: bool) -> SuiteOutcome {
    let outcome = match result {
        Ok(true) => SuiteOutcome {
            name: name.to_string(),
            passed: true,
            error: None,
        },
        Ok(false) => SuiteOutcome {
            name: name.to_string(),
            passed: false,
            error: None,
        },
        Err(e) => SuiteOutcome {
            name: name.to_string(),
            passed: false,
            error: Some(e.to_string()),
        },
    };
    if !quiet {
        let mark = if outcome.passed { "✅" } else { "❌" };
        println!("  {mark} {}", outcome.label());
    }
    outcome
}

/// Execute a test plan with human-readable output, failing if any suite failed.
pub fn execute_test_plan(plan: &TestPlan) -> Result<()> {
    let report = run_test_plan(plan, false);
    match report.failure_message() {
        Some(msg) => Err(anyhow::anyhow!(msg)),
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn plan_of(unit: &[&str], integration: bool) -> TestPlan {
        TestPlan {
            unit_tests: unit.iter().map(|s| s.to_string()).collect::<HashSet<_>>(),
            integration_tests: integration,
        }
    }

    #[test]
    fn an_empty_plan_runs_nothing_and_passes() {
        let report = run_test_plan(&plan_of(&[], false), true);

        assert_eq!(report.suites, vec![]);
        assert!(report.passed());
        assert_eq!(report.failure_message(), None);
        execute_test_plan(&TestPlan::default()).expect("an empty plan is not a failure");
    }

    #[test]
    fn a_report_summarises_every_failure_by_name() {
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
                SuiteOutcome {
                    name: "integration".into(),
                    passed: false,
                    error: Some("cargo not found".into()),
                },
            ],
        };

        assert!(!report.passed());
        assert_eq!(
            report
                .failures()
                .iter()
                .map(|s| s.name.as_str())
                .collect::<Vec<_>>(),
            vec!["b::two", "integration"]
        );
        assert_eq!(
            report.failure_message().unwrap(),
            "Failed tests: b::two, integration (cargo not found)"
        );
    }

    #[test]
    fn a_passing_report_has_no_failure_message() {
        let report = ExecutionReport {
            suites: vec![SuiteOutcome {
                name: "a::one".into(),
                passed: true,
                error: None,
            }],
        };

        assert!(report.passed());
        assert_eq!(report.failure_message(), None);
    }

    #[test]
    fn unit_suites_are_reported_in_a_stable_order() {
        // Selecting a module that matches no test still exits cargo 0, so this
        // exercises ordering without depending on the outcome.
        let report = run_test_plan(
            &plan_of(&["zzz_no_such_module", "aaa_no_such_module"], false),
            true,
        );

        assert_eq!(
            report
                .suites
                .iter()
                .map(|s| s.name.as_str())
                .collect::<Vec<_>>(),
            vec!["aaa_no_such_module", "zzz_no_such_module"]
        );
    }
}
