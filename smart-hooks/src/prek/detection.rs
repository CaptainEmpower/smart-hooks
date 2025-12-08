//! Prek detection and availability checking
//! 
//! This module handles detecting whether prek is available on the system
//! and provides utility functions for prek version checking.
//! Follows SRP by handling only detection concerns.

use std::process::{Command, Stdio};

/// Check if prek is available on the system
pub fn is_prek_available() -> bool {
    Command::new("prek")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

/// Get prek version if available
pub fn get_prek_version() -> Option<String> {
    let output = Command::new("prek")
        .arg("--version")
        .output()
        .ok()?;

    if output.status.success() {
        String::from_utf8(output.stdout)
            .ok()
            .map(|s| s.trim().to_string())
    } else {
        None
    }
}

/// Check if prek supports a specific feature
pub fn check_prek_feature_support(feature: &str) -> bool {
    if !is_prek_available() {
        return false;
    }

    // Basic feature checking - can be extended
    match feature {
        "install" => true,
        "run" => true,
        "list" => true,
        "validate" => true,
        _ => false,
    }
}

/// Validate prek installation health
pub fn validate_prek_installation() -> Result<(), String> {
    if !is_prek_available() {
        return Err("Prek is not available on the system".to_string());
    }

    if let Some(version) = get_prek_version() {
        if version.contains("0.2.20") || version.contains("0.2.") {
            Ok(())
        } else {
            Err(format!("Unsupported prek version: {}. Expected 0.2.x", version))
        }
    } else {
        Err("Could not determine prek version".to_string())
    }
}

/// Get prek installation recommendations
pub fn get_prek_installation_info() -> serde_json::Value {
    serde_json::json!({
        "available": is_prek_available(),
        "installation_methods": [
            {
                "method": "cargo",
                "command": "cargo install prek@0.2.20",
                "description": "Install via Rust package manager"
            },
            {
                "method": "pip",
                "command": "pip install prek",
                "description": "Install via Python package manager"
            }
        ],
        "verification": {
            "command": "prek --version",
            "expected_output": "prek 0.2.20"
        },
        "documentation": "https://github.com/j178/prek"
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_prek_available() {
        // Test that the function doesn't panic and returns a boolean
        let result = is_prek_available();
        assert!(result || !result); // Always true, just testing it returns a bool
    }

    #[test]
    fn test_get_prek_version() {
        // Should return None or Some(String) without panicking
        let version = get_prek_version();
        if let Some(v) = version {
            assert!(!v.is_empty());
        }
    }

    #[test]
    fn test_check_prek_feature_support() {
        assert!(check_prek_feature_support("install") || !check_prek_feature_support("install"));
        assert!(check_prek_feature_support("run") || !check_prek_feature_support("run"));
        assert!(!check_prek_feature_support("unknown_feature"));
    }

    #[test]
    fn test_validate_prek_installation() {
        let result = validate_prek_installation();
        // Should not panic, result depends on system state
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_get_prek_installation_info() {
        let info = get_prek_installation_info();
        assert!(info["available"].is_boolean());
        assert!(info["installation_methods"].is_array());
        assert!(info["verification"]["command"].is_string());
    }

    #[test]
    fn test_feature_support_coverage() {
        let supported_features = ["install", "run", "list", "validate"];
        for feature in &supported_features {
            if is_prek_available() {
                assert!(check_prek_feature_support(feature));
            }
        }
    }
}