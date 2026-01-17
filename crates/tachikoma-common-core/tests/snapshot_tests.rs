//! Snapshot tests for core types.

use serde::{Deserialize, Serialize};
use tachikoma_test_harness::{assert_json, assert_yaml, assert_debug, with_redactions};

#[derive(Debug, Serialize, Deserialize)]
struct ApiResponse {
    status: String,
    data: ResponseData,
    metadata: Metadata,
}

#[derive(Debug, Serialize, Deserialize)]
struct ResponseData {
    items: Vec<Item>,
    total: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct Item {
    id: String,
    name: String,
    value: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct Metadata {
    version: String,
    timestamp: String,
}

fn create_test_response() -> ApiResponse {
    ApiResponse {
        status: "success".into(),
        data: ResponseData {
            items: vec![
                Item { id: "1".into(), name: "First".into(), value: 100 },
                Item { id: "2".into(), name: "Second".into(), value: 200 },
            ],
            total: 2,
        },
        metadata: Metadata {
            version: "1.0.0".into(),
            timestamp: "2024-01-15T10:30:00Z".into(),
        },
    }
}

#[test]
fn test_api_response_json_snapshot() {
    let response = create_test_response();
    assert_json!(response);
}

#[test]
fn test_api_response_yaml_snapshot() {
    let response = create_test_response();
    assert_yaml!(response);
}

#[test]
fn test_api_response_with_redactions() {
    let response = create_test_response();

    with_redactions(&[
        (".metadata.timestamp", "[timestamp]"),
        (".data.items[].id", "[id]"),
    ], || {
        insta::assert_json_snapshot!("api_response_redacted", response);
    });
}

#[test]
fn test_named_snapshots() {
    let items = vec![
        Item { id: "a".into(), name: "Alpha".into(), value: 1 },
        Item { id: "b".into(), name: "Beta".into(), value: 2 },
    ];

    assert_json!("item_list", items);
}

#[test]
fn test_inline_snapshot() {
    let simple = serde_json::json!({
        "key": "value",
        "number": 42
    });

    insta::assert_json_snapshot!(simple, @r###"
    {
      "key": "value",
      "number": 42
    }
    "###);
}

mod error_snapshots {
    use super::*;

    #[derive(Debug, Serialize)]
    struct ErrorResponse {
        code: String,
        message: String,
        details: Option<String>,
    }

    #[test]
    fn test_error_not_found() {
        let error = ErrorResponse {
            code: "NOT_FOUND".into(),
            message: "Resource not found".into(),
            details: Some("The requested item does not exist".into()),
        };
        assert_json!("error_not_found", error);
    }

    #[test]
    fn test_error_validation() {
        let error = ErrorResponse {
            code: "VALIDATION_ERROR".into(),
            message: "Invalid input".into(),
            details: None,
        };
        assert_json!("error_validation", error);
    }
}