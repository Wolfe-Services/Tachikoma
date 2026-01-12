//! HTTP response types.

use serde::de::DeserializeOwned;
use futures_util::Stream;

/// Parse a JSON response.
pub async fn parse_json<T: DeserializeOwned>(
    response: reqwest::Response,
) -> Result<T, ResponseError> {
    let status = response.status();
    let bytes = response.bytes().await.map_err(ResponseError::Read)?;

    serde_json::from_slice(&bytes).map_err(|e| ResponseError::Parse {
        status: status.as_u16(),
        body: String::from_utf8_lossy(&bytes).to_string(),
        source: e,
    })
}

/// Stream response content.
pub fn stream_response(
    response: reqwest::Response,
) -> impl Stream<Item = Result<bytes::Bytes, reqwest::Error>> {
    response.bytes_stream()
}

/// Response parsing errors.
#[derive(Debug, thiserror::Error)]
pub enum ResponseError {
    #[error("failed to read response body: {0}")]
    Read(#[source] reqwest::Error),

    #[error("failed to parse JSON (status {status}): {source}")]
    Parse {
        status: u16,
        body: String,
        #[source]
        source: serde_json::Error,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestData {
        message: String,
        value: i32,
    }

    #[tokio::test]
    async fn test_response_error_display() {
        let json_error = serde_json::from_str::<TestData>("invalid json").unwrap_err();
        let parse_error = ResponseError::Parse {
            status: 400,
            body: "invalid json".to_string(),
            source: json_error,
        };

        let error_string = format!("{}", parse_error);
        assert!(error_string.contains("failed to parse JSON"));
        assert!(error_string.contains("status 400"));
    }
}