use crate::utilities::file_utils;
use anyhow::Result;
/// BDD Test Detection through Static Analysis
/// Identifies code patterns that suggest behavior-driven testing needs
use std::path::Path;

#[derive(Debug, Default, PartialEq)]
pub struct BehaviorPatterns {
    pub has_user_facing_apis: bool,
    pub has_state_mutations: bool,
    pub has_business_logic: bool,
    pub has_error_scenarios: bool,
    pub has_integration_points: bool,
    pub confidence_score: f32,
}

impl BehaviorPatterns {
    /// Determine if this code warrants BDD testing based on detected patterns
    pub fn should_run_bdd_tests(&self, threshold: f32) -> bool {
        self.confidence_score >= threshold
    }

    /// Get recommended BDD scenarios based on detected patterns
    pub fn suggested_scenarios(&self) -> Vec<&'static str> {
        let mut scenarios = Vec::new();

        if self.has_user_facing_apis {
            scenarios.push("API contract adherence");
            scenarios.push("Input validation scenarios");
        }

        if self.has_state_mutations {
            scenarios.push("State transition scenarios");
            scenarios.push("Concurrent access scenarios");
        }

        if self.has_business_logic {
            scenarios.push("Business rule validation");
            scenarios.push("Edge case handling");
        }

        if self.has_error_scenarios {
            scenarios.push("Error handling workflows");
            scenarios.push("Recovery scenarios");
        }

        if self.has_integration_points {
            scenarios.push("Integration failure scenarios");
            scenarios.push("External service mocking");
        }

        scenarios
    }
}

/// Analyze Rust code for behavioral testing indicators
pub fn detect_behavior_patterns(file_path: &Path) -> Result<BehaviorPatterns> {
    let content = file_utils::read_file_content(file_path)?;

    let mut patterns = BehaviorPatterns::default();

    // Detect user-facing APIs (public functions with complex signatures)
    patterns.has_user_facing_apis = detect_user_apis(&content);

    // Detect state mutations (mutable references, interior mutability)
    patterns.has_state_mutations = detect_state_mutations(&content);

    // Detect business logic (domain-specific operations)
    patterns.has_business_logic = detect_business_logic(&content);

    // Detect error scenarios (Result types, custom errors)
    patterns.has_error_scenarios = detect_error_scenarios(&content);

    // Detect integration points (external dependencies, I/O)
    patterns.has_integration_points = detect_integration_points(&content);

    // Calculate confidence score
    patterns.confidence_score = calculate_confidence_score(&patterns);

    Ok(patterns)
}

fn detect_user_apis(content: &str) -> bool {
    // Look for public functions that take complex parameters
    let has_pub_functions = content.contains("pub fn") || content.contains("pub async fn");
    let has_complex_params =
        content.contains("&mut") || content.contains("Option<") || content.contains("Result<");
    let has_builder_pattern = content.contains(".with_") || content.contains(".build()");

    has_pub_functions && (has_complex_params || has_builder_pattern)
}

fn detect_state_mutations(content: &str) -> bool {
    content.contains("&mut ")
        || content.contains("RefCell")
        || content.contains("Mutex")
        || content.contains("RwLock")
        || content.contains(".set(")
        || content.contains(".insert(")
        || content.contains(".remove(")
        || content.contains(".clear(")
}

fn detect_business_logic(content: &str) -> bool {
    // Look for domain-specific patterns
    let has_validation = content.contains("validate") || content.contains("check_");
    let has_processing =
        content.contains("process") || content.contains("execute") || content.contains("apply");
    let has_workflows =
        content.contains("workflow") || content.contains("strategy") || content.contains("policy");
    let has_rules = content.contains("rule")
        || content.contains("constraint")
        || content.contains("requirement");

    has_validation || has_processing || has_workflows || has_rules
}

fn detect_error_scenarios(content: &str) -> bool {
    let has_results = content.contains("Result<") && content.contains("Error");
    let has_custom_errors = content.contains("impl") && content.contains("Error");
    let has_error_handling =
        content.contains(".map_err(") || content.contains("match") && content.contains("Err(");
    let has_fallback = content.contains("unwrap_or") || content.contains("unwrap_or_else");

    has_results || has_custom_errors || has_error_handling || has_fallback
}

fn detect_integration_points(content: &str) -> bool {
    // Look for external integrations
    let has_io = content.contains("std::fs")
        || content.contains("std::process")
        || content.contains("tokio");
    let has_network =
        content.contains("reqwest") || content.contains("hyper") || content.contains("url");
    let has_database =
        content.contains("sql") || content.contains("diesel") || content.contains("rusqlite");
    let has_external_calls = content.contains(".spawn(") || content.contains(".await");

    has_io || has_network || has_database || has_external_calls
}

fn calculate_confidence_score(patterns: &BehaviorPatterns) -> f32 {
    let mut score: f32 = 0.0;

    if patterns.has_user_facing_apis {
        score += 0.3;
    }
    if patterns.has_state_mutations {
        score += 0.25;
    }
    if patterns.has_business_logic {
        score += 0.2;
    }
    if patterns.has_error_scenarios {
        score += 0.15;
    }
    if patterns.has_integration_points {
        score += 0.1;
    }

    score.min(1.0) // Cap at 1.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_file(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file
    }

    #[test]
    fn test_user_api_detection() {
        let content = r#"
        pub fn execute_move(options: &MoveOptions) -> Result<MoveResult> {
            // Complex public API
        }
        "#;

        assert!(detect_user_apis(content));
        assert!(!detect_user_apis("fn private_helper() {}"));
    }

    #[test]
    fn test_state_mutation_detection() {
        let content = r#"
        pub fn update_state(&mut self, value: String) {
            self.data.insert(key, value);
        }
        "#;

        assert!(detect_state_mutations(content));
        assert!(!detect_state_mutations(
            "fn pure_function(x: i32) -> i32 { x * 2 }"
        ));
    }

    #[test]
    fn test_business_logic_detection() {
        let content = r#"
        pub fn validate_move_constraints(&self) -> Result<()> {
            // Business rule validation
        }
        "#;

        assert!(detect_business_logic(content));
        assert!(!detect_business_logic(
            "fn add(a: i32, b: i32) -> i32 { a + b }"
        ));
    }

    #[test]
    fn test_confidence_scoring() {
        let mut patterns = BehaviorPatterns {
            has_user_facing_apis: true,
            has_business_logic: true,
            has_error_scenarios: true,
            ..Default::default()
        };

        patterns.confidence_score = calculate_confidence_score(&patterns);
        assert_eq!(patterns.confidence_score, 0.65); // 0.3 + 0.2 + 0.15
        assert!(patterns.should_run_bdd_tests(0.5));
    }

    #[test]
    fn test_end_to_end_analysis() {
        let content = r#"
        pub struct FileMover {
            repository: GitRepository,
        }

        impl FileMover {
            pub fn execute_move(&mut self, options: &MoveOptions) -> Result<MoveResult, MoveError> {
                self.validate_constraints(&options)?;
                self.process_move(&options)
                    .map_err(|e| MoveError::ProcessingFailed(e))
            }

            fn validate_constraints(&self, options: &MoveOptions) -> Result<()> {
                // Business logic validation
            }
        }
        "#;

        let file = create_test_file(content);
        let patterns = detect_behavior_patterns(file.path()).unwrap();

        assert!(patterns.has_user_facing_apis);
        assert!(patterns.has_business_logic);
        assert!(patterns.has_error_scenarios);
        assert!(patterns.should_run_bdd_tests(0.4));

        let scenarios = patterns.suggested_scenarios();
        assert!(scenarios.contains(&"API contract adherence"));
        assert!(scenarios.contains(&"Business rule validation"));
    }
}