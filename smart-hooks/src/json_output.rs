//! JSON output utilities for structured responses
//! 
//! This module provides utilities for generating consistent JSON output
//! across all smart-hooks commands. Follows SRP by handling only
//! JSON formatting and output concerns.

use anyhow::Result;
use serde_json::{json, Value};

/// Standard JSON response structure for success cases
#[derive(Debug)]
pub struct JsonResponse {
    pub status: String,
    pub data: Value,
    pub metadata: Option<Value>,
}

impl JsonResponse {
    /// Create a new success response
    pub fn success(data: Value) -> Self {
        Self {
            status: "success".to_string(),
            data,
            metadata: None,
        }
    }

    /// Create a new success response with metadata
    pub fn success_with_metadata(data: Value, metadata: Value) -> Self {
        Self {
            status: "success".to_string(),
            data,
            metadata: Some(metadata),
        }
    }

    /// Create an error response
    pub fn error(error_message: &str) -> Self {
        Self {
            status: "error".to_string(),
            data: json!({ "error": error_message }),
            metadata: None,
        }
    }

    /// Convert to JSON string
    pub fn to_json_string(&self) -> Result<String> {
        let mut response = json!({
            "status": self.status,
            "data": self.data
        });

        if let Some(metadata) = &self.metadata {
            response["metadata"] = metadata.clone();
        }

        Ok(serde_json::to_string_pretty(&response)?)
    }
}

/// Output either JSON or human-readable format based on flag
pub fn output_result(json_output: bool, json_data: Value, human_message: &str) -> Result<()> {
    if json_output {
        let response = JsonResponse::success(json_data);
        println!("{}", response.to_json_string()?);
    } else {
        println!("{}", human_message);
    }
    Ok(())
}

/// Output an error in JSON or human-readable format
pub fn output_error(json_output: bool, error_message: &str) -> Result<()> {
    if json_output {
        let response = JsonResponse::error(error_message);
        println!("{}", response.to_json_string()?);
    } else {
        eprintln!("❌ {}", error_message);
    }
    Ok(())
}

/// Create a status JSON object with common fields
pub fn create_status_json(
    status: &str,
    message: Option<&str>,
    details: Option<Value>
) -> Value {
    let mut status_obj = json!({
        "status": status,
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    });

    if let Some(msg) = message {
        status_obj["message"] = json!(msg);
    }

    if let Some(details_obj) = details {
        status_obj["details"] = details_obj;
    }

    status_obj
}

/// Create a progress JSON object for long-running operations
pub fn create_progress_json(
    stage: &str,
    current: usize,
    total: usize,
    details: Option<Value>
) -> Value {
    let mut progress = json!({
        "stage": stage,
        "progress": {
            "current": current,
            "total": total,
            "percentage": if total > 0 { (current * 100) / total } else { 0 }
        },
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    });

    if let Some(details_obj) = details {
        progress["details"] = details_obj;
    }

    progress
}

/// Format file list for JSON output
pub fn format_files_json(files: &[String]) -> Value {
    json!({
        "files": files,
        "count": files.len()
    })
}

/// Format command results for JSON output
pub fn format_command_result_json(
    command: &str,
    success: bool,
    output: Option<&str>,
    execution_time_ms: Option<u64>
) -> Value {
    let mut result = json!({
        "command": command,
        "success": success,
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    });

    if let Some(out) = output {
        result["output"] = json!(out);
    }

    if let Some(time) = execution_time_ms {
        result["execution_time_ms"] = json!(time);
    }

    result
}

/// Format test plan for JSON output
pub fn format_test_plan_json(
    unit_tests: &[String],
    integration_tests: bool,
    bdd_tests: bool,
    additional_info: Option<Value>
) -> Value {
    let mut test_plan = json!({
        "unit_tests": unit_tests,
        "integration_tests": integration_tests,
        "bdd_tests": bdd_tests,
        "summary": {
            "unit_test_count": unit_tests.len(),
            "has_integration_tests": integration_tests,
            "has_bdd_tests": bdd_tests
        }
    });

    if let Some(info) = additional_info {
        test_plan["additional_info"] = info;
    }

    test_plan
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_json_response_success() {
        let data = json!({"test": "value"});
        let response = JsonResponse::success(data.clone());
        
        assert_eq!(response.status, "success");
        assert_eq!(response.data, data);
        assert!(response.metadata.is_none());
    }

    #[test]
    fn test_json_response_success_with_metadata() {
        let data = json!({"test": "value"});
        let metadata = json!({"version": "1.0"});
        let response = JsonResponse::success_with_metadata(data.clone(), metadata.clone());
        
        assert_eq!(response.status, "success");
        assert_eq!(response.data, data);
        assert_eq!(response.metadata.unwrap(), metadata);
    }

    #[test]
    fn test_json_response_error() {
        let error_msg = "Test error";
        let response = JsonResponse::error(error_msg);
        
        assert_eq!(response.status, "error");
        assert_eq!(response.data["error"], json!(error_msg));
    }

    #[test]
    fn test_json_response_to_string() {
        let data = json!({"test": "value"});
        let response = JsonResponse::success(data);
        let json_string = response.to_json_string().unwrap();
        
        assert!(json_string.contains("\"status\": \"success\""));
        assert!(json_string.contains("\"test\": \"value\""));
    }

    #[test]
    fn test_create_status_json() {
        let status_json = create_status_json("processing", Some("Working on it"), None);
        
        assert_eq!(status_json["status"], "processing");
        assert_eq!(status_json["message"], "Working on it");
        assert!(status_json["timestamp"].is_u64());
    }

    #[test]
    fn test_create_progress_json() {
        let progress_json = create_progress_json("analyzing", 5, 10, None);
        
        assert_eq!(progress_json["stage"], "analyzing");
        assert_eq!(progress_json["progress"]["current"], 5);
        assert_eq!(progress_json["progress"]["total"], 10);
        assert_eq!(progress_json["progress"]["percentage"], 50);
    }

    #[test]
    fn test_format_files_json() {
        let files = vec!["file1.rs".to_string(), "file2.rs".to_string()];
        let files_json = format_files_json(&files);
        
        assert_eq!(files_json["count"], 2);
        assert_eq!(files_json["files"], json!(files));
    }

    #[test]
    fn test_format_command_result_json() {
        let result_json = format_command_result_json("test", true, Some("output"), Some(1500));
        
        assert_eq!(result_json["command"], "test");
        assert_eq!(result_json["success"], true);
        assert_eq!(result_json["output"], "output");
        assert_eq!(result_json["execution_time_ms"], 1500);
    }

    #[test]
    fn test_format_test_plan_json() {
        let unit_tests = vec!["test1".to_string(), "test2".to_string()];
        let test_plan_json = format_test_plan_json(&unit_tests, true, false, None);
        
        assert_eq!(test_plan_json["unit_tests"], json!(unit_tests));
        assert_eq!(test_plan_json["integration_tests"], true);
        assert_eq!(test_plan_json["bdd_tests"], false);
        assert_eq!(test_plan_json["summary"]["unit_test_count"], 2);
    }

    #[test]
    fn test_progress_with_zero_total() {
        let progress_json = create_progress_json("starting", 0, 0, None);
        assert_eq!(progress_json["progress"]["percentage"], 0);
    }
}