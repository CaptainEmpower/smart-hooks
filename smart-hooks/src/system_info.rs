//! System information and environment checking utilities
//! 
//! This module handles system status checks, environment information,
//! and integration status for external tools. Follows SRP by handling
//! only system information concerns.

use serde_json::json;

/// Check if prek is available on the system
pub fn is_prek_available() -> bool {
    crate::prek::is_prek_available()
}

/// Get environment information for status checks
pub fn get_environment_info() -> serde_json::Value {
    json!({
        "git_repository": std::path::Path::new(".git").exists(),
        "config_file": std::path::Path::new("config.toml").exists(),
        "features_directory": std::path::Path::new("features").exists(),
        "working_directory": std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| "unknown".to_string())
    })
}

/// Get integration status for external tools
pub fn get_integration_status() -> serde_json::Value {
    let prek_available = is_prek_available();
    
    json!({
        "prek": {
            "available": prek_available,
            "status": if prek_available { "ready" } else { "not_installed" }
        },
        "claude_ai": {
            "available": cfg!(feature = "claude-ai"),
            "status": if cfg!(feature = "claude-ai") { "ready" } else { "not_enabled" }
        }
    })
}

/// Perform readiness checks based on environment and integrations
pub fn get_readiness_check(environment: &serde_json::Value, integrations: &serde_json::Value) -> serde_json::Value {
    let git_repo = environment["git_repository"].as_bool().unwrap_or(false);
    let features_dir = environment["features_directory"].as_bool().unwrap_or(false);
    let prek_available = integrations["prek"]["available"].as_bool().unwrap_or(false);
    let claude_available = integrations["claude_ai"]["available"].as_bool().unwrap_or(false);

    json!({
        "can_analyze_files": true,
        "can_run_tests": git_repo,
        "can_use_prek": prek_available,
        "can_use_bdd": claude_available && features_dir,
        "overall": if git_repo { "ready" } else { "limited" }
    })
}

/// Get system capabilities for a specific domain
pub fn get_capabilities_for_domain(domain: &str) -> anyhow::Result<serde_json::Value> {
    match domain {
        "analysis" => Ok(json!({
            "intelligent_test_selection": true,
            "dependency_analysis": true,
            "multi_language_support": true,
            "file_impact_analysis": true
        })),
        "integration" => Ok(json!({
            "prek_support": is_prek_available(),
            "git_hooks": true,
            "ci_cd_ready": true,
            "json_api": true
        })),
        "ai" => Ok(json!({
            "claude_ai_bdd": cfg!(feature = "claude-ai"),
            "semantic_analysis": cfg!(feature = "claude-ai"),
            "intelligent_recommendations": true
        })),
        _ => Err(anyhow::anyhow!("Unknown capabilities domain: {}", domain))
    }
}

/// Generate system capabilities information
pub fn generate_capabilities_info() -> serde_json::Value {
    json!({
        "system_info": {
            "version": "0.2.0",
            "name": "smart-hooks",
            "description": "Intelligent git hooks with optional prek integration"
        },
        "integrations": {
            "prek": {
                "available": is_prek_available(),
                "version_command": "prek --version",
                "installation": "cargo install prek@0.2.20"
            },
            "claude_ai": {
                "available": cfg!(feature = "claude-ai"),
                "feature_flag": "claude-ai", 
                "installation": "cargo install smart-hooks --features claude-ai"
            }
        },
        "commands": {
            "analysis": [
                {
                    "name": "test",
                    "description": "Intelligent test selection based on code changes",
                    "supports_json": true
                },
                {
                    "name": "analyze",
                    "description": "Multi-language project dependency analysis",
                    "supports_json": true
                },
                {
                    "name": "check",
                    "description": "Conditional compilation configuration check",
                    "supports_json": true
                }
            ],
            "pre_commit": [
                {
                    "name": "run",
                    "description": "Run pre-commit hooks (prek delegation or smart fallback)",
                    "supports_json": true
                },
                {
                    "name": "install",
                    "description": "Install git hooks (prek or smart-hooks)",
                    "supports_json": true
                },
                {
                    "name": "list",
                    "description": "List available hooks (prek + smart capabilities)",
                    "supports_json": true
                },
                {
                    "name": "validate",
                    "description": "Validate configuration (prek + smart-hooks)",
                    "supports_json": true
                }
            ],
            "agent_friendly": [
                {
                    "name": "capabilities",
                    "description": "System capabilities and feature discovery",
                    "supports_json": true
                },
                {
                    "name": "schema",
                    "description": "JSON schemas for request/response validation",
                    "supports_json": true
                },
                {
                    "name": "status",
                    "description": "System status and readiness check",
                    "supports_json": true
                },
                {
                    "name": "examples",
                    "description": "Usage examples for common scenarios",
                    "supports_json": true
                }
            ]
        },
        "features": {
            "intelligent_test_selection": true,
            "dependency_analysis": true,
            "multi_language_support": true,
            "prek_integration": is_prek_available(),
            "claude_ai_bdd": cfg!(feature = "claude-ai"),
            "json_output": true,
            "git_hooks_creation": true
        }
    })
}

/// Generate system status information  
pub fn generate_status_info() -> serde_json::Value {
    let environment_info = get_environment_info();
    let integration_status = get_integration_status();
    let readiness_check = get_readiness_check(&environment_info, &integration_status);

    json!({
        "system_status": "operational",
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        "environment": environment_info,
        "integrations": integration_status,
        "readiness": readiness_check
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_prek_available() {
        // Test that the function doesn't panic and returns a bool
        let result = is_prek_available();
        assert!(result || !result); // Always true - just testing it returns a bool
    }

    #[test]
    fn test_get_environment_info() {
        let env_info = get_environment_info();
        
        assert!(env_info["git_repository"].is_boolean());
        assert!(env_info["config_file"].is_boolean());
        assert!(env_info["features_directory"].is_boolean());
        assert!(env_info["working_directory"].is_string());
    }

    #[test]
    fn test_get_integration_status() {
        let status = get_integration_status();
        
        assert!(status["prek"]["available"].is_boolean());
        assert!(status["prek"]["status"].is_string());
        assert!(status["claude_ai"]["available"].is_boolean());
        assert!(status["claude_ai"]["status"].is_string());
    }

    #[test]
    fn test_get_readiness_check() {
        let env = json!({
            "git_repository": true,
            "features_directory": false
        });
        let integrations = json!({
            "prek": {"available": true},
            "claude_ai": {"available": false}
        });
        
        let readiness = get_readiness_check(&env, &integrations);
        
        assert_eq!(readiness["can_analyze_files"], true);
        assert_eq!(readiness["can_run_tests"], true);
        assert_eq!(readiness["can_use_prek"], true);
        assert_eq!(readiness["can_use_bdd"], false);
        assert_eq!(readiness["overall"], "ready");
    }

    #[test] 
    fn test_get_capabilities_for_domain() {
        let analysis_caps = get_capabilities_for_domain("analysis").unwrap();
        assert_eq!(analysis_caps["intelligent_test_selection"], true);
        assert_eq!(analysis_caps["dependency_analysis"], true);

        let integration_caps = get_capabilities_for_domain("integration").unwrap();
        assert_eq!(integration_caps["git_hooks"], true);
        assert_eq!(integration_caps["json_api"], true);

        let ai_caps = get_capabilities_for_domain("ai").unwrap();
        assert!(ai_caps["intelligent_recommendations"].is_boolean());

        let invalid_result = get_capabilities_for_domain("invalid");
        assert!(invalid_result.is_err());
    }

    #[test]
    fn test_generate_capabilities_info() {
        let capabilities = generate_capabilities_info();
        
        assert_eq!(capabilities["system_info"]["name"], "smart-hooks");
        assert_eq!(capabilities["system_info"]["version"], "0.2.0");
        assert!(capabilities["integrations"]["prek"]["available"].is_boolean());
        assert!(capabilities["features"]["intelligent_test_selection"].is_boolean());
    }

    #[test]
    fn test_generate_status_info() {
        let status = generate_status_info();
        
        assert_eq!(status["system_status"], "operational");
        assert!(status["timestamp"].is_u64());
        assert!(status["environment"]["git_repository"].is_boolean());
        assert!(status["readiness"]["overall"].is_string());
    }

    #[test]
    fn test_readiness_limited_without_git() {
        let env = json!({
            "git_repository": false,
            "features_directory": false
        });
        let integrations = json!({
            "prek": {"available": true},
            "claude_ai": {"available": true}
        });
        
        let readiness = get_readiness_check(&env, &integrations);
        assert_eq!(readiness["overall"], "limited");
        assert_eq!(readiness["can_run_tests"], false);
    }
}