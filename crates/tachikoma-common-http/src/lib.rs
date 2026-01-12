//! HTTP client utilities for Tachikoma.
//! 
//! ## Example Usage
//! 
//! ```rust
//! use tachikoma_common_http::{RetryPolicy, with_retry};
//! use std::time::Duration;
//! 
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a retry policy
//! let policy = RetryPolicy::aggressive();
//! 
//! // Use retry with any operation that returns a RetryableError
//! let result = with_retry(&policy, || async {
//!     // Your HTTP operation here
//!     Ok::<&str, tachikoma_common_http::HttpError>("Success!")
//! }).await?;
//! 
//! assert_eq!(result, "Success!");
//! # Ok(())
//! # }
//! ```

pub mod client;
pub mod request;
pub mod response;
pub mod retry;

pub use client::{HttpClient, HttpConfig, HttpError, build_client};
pub use request::{RequestBuilder, JsonBody, headers};
pub use response::{parse_json, stream_response, ResponseError};
pub use retry::{RetryPolicy, RetryableError, with_retry};