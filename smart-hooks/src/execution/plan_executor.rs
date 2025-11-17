use crate::analysis::dependency_mapper::TestPlan;
use crate::execution::test_runner;
use anyhow::Result;

/// Test plan execution
/// Focused on coordinating test execution based on analysis results
/// Execute a test plan, running all required tests
pub fn execute_test_plan(plan: &TestPlan) -> Result<()> {
    let mut tests_run = 0;
    let mut failed_tests = Vec::new();

    // Run unit tests
    if !plan.unit_tests.is_empty() {
        println!("🧪 Running targeted unit tests...");
        for test in &plan.unit_tests {
            println!("  Running: {}", test);
            match test_runner::run_unit_test(test) {
                Ok(true) => {
                    println!("  ✅ {}", test);
                    tests_run += 1;
                }
                Ok(false) => {
                    println!("  ❌ {}", test);
                    failed_tests.push(format!("Unit test: {}", test));
                }
                Err(e) => {
                    println!("  ❌ {} (error: {})", test, e);
                    failed_tests.push(format!("Unit test: {} ({})", test, e));
                }
            }
        }
    }

    // Run integration tests
    if plan.integration_tests {
        println!("🔗 Running integration tests for core changes...");
        match test_runner::run_integration_tests() {
            Ok(true) => {
                println!("  ✅ Integration tests");
                tests_run += 1;
            }
            Ok(false) => {
                println!("  ❌ Integration tests");
                failed_tests.push("Integration tests".to_string());
            }
            Err(e) => {
                println!("  ❌ Integration tests (error: {})", e);
                failed_tests.push(format!("Integration tests ({})", e));
            }
        }
    }

    // Run BDD tests
    if plan.bdd_tests {
        println!("🎭 Running BDD tests for behavioral changes...");
        match test_runner::run_bdd_tests() {
            Ok(true) => {
                println!("  ✅ BDD tests");
                tests_run += 1;
            }
            Ok(false) => {
                println!("  ❌ BDD tests");
                failed_tests.push("BDD tests".to_string());
            }
            Err(e) => {
                println!("  ❌ BDD tests (error: {})", e);
                failed_tests.push(format!("BDD tests ({})", e));
            }
        }
    }

    // Report results
    if failed_tests.is_empty() {
        if tests_run == 0 {
            println!("✅ No functionality changes detected, skipping behavioral tests");
        } else {
            println!(
                "✅ Completed {} test suite(s) based on change analysis",
                tests_run
            );
        }
        Ok(())
    } else {
        let error_msg = format!("Failed tests: {}", failed_tests.join(", "));
        Err(anyhow::anyhow!(error_msg))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_execute_empty_plan() {
        let plan = TestPlan::default();
        let result = execute_test_plan(&plan);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_plan_with_tests() {
        let mut unit_tests = HashSet::new();
        unit_tests.insert("nonexistent::test".to_string());

        let plan = TestPlan {
            unit_tests,
            integration_tests: false,
            bdd_tests: false,
        };

        // This will fail because the test doesn't exist, but should handle gracefully
        let result = execute_test_plan(&plan);
        assert!(result.is_err());
    }
}