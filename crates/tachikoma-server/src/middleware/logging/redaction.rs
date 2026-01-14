//! Sensitive data redaction utilities.

use std::collections::HashSet;

/// Headers that should be redacted in logs.
pub const SENSITIVE_HEADERS: &[&str] = &[
    "authorization",
    "cookie",
    "set-cookie",
    "x-api-key",
    "x-auth-token",
    "x-access-token",
    "x-csrf-token",
];

/// Fields that should be redacted in request bodies.
pub const SENSITIVE_FIELDS: &[&str] = &[
    "password",
    "password_confirm",
    "current_password",
    "new_password",
    "token",
    "secret",
    "api_key",
    "credit_card",
    "ssn",
    "access_token",
    "refresh_token",
];

/// Redact sensitive headers from a header map.
pub fn redact_headers(
    headers: &axum::http::HeaderMap,
    additional: &[String],
) -> Vec<(String, String)> {
    let sensitive: HashSet<&str> = SENSITIVE_HEADERS
        .iter()
        .copied()
        .chain(additional.iter().map(|s| s.as_str()))
        .collect();

    headers
        .iter()
        .map(|(name, value)| {
            let name_lower = name.as_str().to_lowercase();
            let value_str = if sensitive.contains(name_lower.as_str()) {
                "[REDACTED]".to_string()
            } else {
                value.to_str().unwrap_or("[non-utf8]").to_string()
            };
            (name.as_str().to_string(), value_str)
        })
        .collect()
}

/// Redact sensitive fields from a JSON value.
pub fn redact_json(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, val) in map.iter_mut() {
                let key_lower = key.to_lowercase();
                if SENSITIVE_FIELDS.iter().any(|f| key_lower.contains(f)) {
                    *val = serde_json::Value::String("[REDACTED]".to_string());
                } else {
                    redact_json(val);
                }
            }
        }
        serde_json::Value::Array(arr) => {
            for val in arr.iter_mut() {
                redact_json(val);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use axum::http::HeaderMap;

    #[test]
    fn test_redact_json() {
        let mut value = json!({
            "email": "user@example.com",
            "password": "secret123",
            "data": {
                "api_key": "key123",
                "name": "John Doe"
            },
            "nested": {
                "user": {
                    "current_password": "old_secret"
                }
            }
        });

        redact_json(&mut value);

        assert_eq!(value["email"], "user@example.com");
        assert_eq!(value["password"], "[REDACTED]");
        assert_eq!(value["data"]["api_key"], "[REDACTED]");
        assert_eq!(value["data"]["name"], "John Doe");
        assert_eq!(value["nested"]["user"]["current_password"], "[REDACTED]");
    }

    #[test]
    fn test_redact_json_array() {
        let mut value = json!([
            {"username": "user1", "password": "secret1"},
            {"username": "user2", "token": "abc123"}
        ]);

        redact_json(&mut value);

        assert_eq!(value[0]["username"], "user1");
        assert_eq!(value[0]["password"], "[REDACTED]");
        assert_eq!(value[1]["username"], "user2");
        assert_eq!(value[1]["token"], "[REDACTED]");
    }

    #[test]
    fn test_redact_headers() {
        let mut headers = HeaderMap::new();
        headers.insert("authorization", "Bearer token123".parse().unwrap());
        headers.insert("content-type", "application/json".parse().unwrap());
        headers.insert("x-api-key", "key123".parse().unwrap());
        headers.insert("user-agent", "TestAgent/1.0".parse().unwrap());

        let redacted = redact_headers(&headers, &[]);

        let auth_header = redacted.iter()
            .find(|(name, _)| name == "authorization")
            .map(|(_, value)| value)
            .unwrap();
        assert_eq!(auth_header, "[REDACTED]");

        let content_type = redacted.iter()
            .find(|(name, _)| name == "content-type")
            .map(|(_, value)| value)
            .unwrap();
        assert_eq!(content_type, "application/json");

        let api_key = redacted.iter()
            .find(|(name, _)| name == "x-api-key")
            .map(|(_, value)| value)
            .unwrap();
        assert_eq!(api_key, "[REDACTED]");
    }

    #[test]
    fn test_redact_headers_with_additional() {
        let mut headers = HeaderMap::new();
        headers.insert("authorization", "Bearer token123".parse().unwrap());
        headers.insert("x-custom-secret", "secret123".parse().unwrap());
        headers.insert("x-public-header", "public".parse().unwrap());

        let additional = vec!["x-custom-secret".to_string()];
        let redacted = redact_headers(&headers, &additional);

        let custom_secret = redacted.iter()
            .find(|(name, _)| name == "x-custom-secret")
            .map(|(_, value)| value)
            .unwrap();
        assert_eq!(custom_secret, "[REDACTED]");

        let public_header = redacted.iter()
            .find(|(name, _)| name == "x-public-header")
            .map(|(_, value)| value)
            .unwrap();
        assert_eq!(public_header, "public");
    }
}