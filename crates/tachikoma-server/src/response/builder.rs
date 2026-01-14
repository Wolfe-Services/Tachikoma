//! Response builder utilities.

use axum::{
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use super::types::{ApiResponse, ResponseMeta};

/// Builder for constructing API responses.
pub struct ResponseBuilder<T> {
    status: StatusCode,
    data: Option<T>,
    meta: ResponseMeta,
    headers: Vec<(header::HeaderName, String)>,
}

impl<T: serde::Serialize> ResponseBuilder<T> {
    /// Create a new response builder.
    pub fn new(data: T) -> Self {
        Self {
            status: StatusCode::OK,
            data: Some(data),
            meta: ResponseMeta::now(),
            headers: Vec::new(),
        }
    }

    /// Set HTTP status code.
    pub fn status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }

    /// Add request ID to metadata.
    pub fn request_id(mut self, id: impl Into<String>) -> Self {
        self.meta.request_id = Some(id.into());
        self
    }

    /// Add API version to metadata.
    pub fn api_version(mut self, version: impl Into<String>) -> Self {
        self.meta.api_version = Some(version.into());
        self
    }

    /// Add a custom header.
    pub fn header(mut self, name: header::HeaderName, value: impl Into<String>) -> Self {
        self.headers.push((name, value.into()));
        self
    }

    /// Build the response.
    pub fn build(self) -> Response {
        let response = ApiResponse::success_with_meta(self.data.unwrap(), self.meta);
        let mut res = (self.status, Json(response)).into_response();

        for (name, value) in self.headers {
            if let Ok(v) = value.parse() {
                res.headers_mut().insert(name, v);
            }
        }

        res
    }
}

/// Create a 201 Created response.
pub fn created<T: serde::Serialize>(data: T) -> Response {
    ResponseBuilder::new(data)
        .status(StatusCode::CREATED)
        .build()
}

/// Create a 204 No Content response.
pub fn no_content() -> Response {
    StatusCode::NO_CONTENT.into_response()
}

/// Create a 202 Accepted response.
pub fn accepted<T: serde::Serialize>(data: T) -> Response {
    ResponseBuilder::new(data)
        .status(StatusCode::ACCEPTED)
        .build()
}