//! Request data transfer objects.

use serde::Deserialize;
use validator::Validate;

/// Filter parameters for list endpoints.
#[derive(Debug, Deserialize, Default)]
pub struct FilterParams {
    /// Status filter.
    #[serde(default)]
    pub status: Option<String>,
    /// Search query.
    #[serde(default)]
    pub q: Option<String>,
    /// Date from filter.
    #[serde(default)]
    pub from: Option<chrono::DateTime<chrono::Utc>>,
    /// Date to filter.
    #[serde(default)]
    pub to: Option<chrono::DateTime<chrono::Utc>>,
    /// Tags filter (comma-separated).
    #[serde(default)]
    pub tags: Option<String>,
}

impl FilterParams {
    /// Parse tags into a vector.
    pub fn tags_vec(&self) -> Vec<String> {
        self.tags
            .as_ref()
            .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default()
    }
}

/// Sort parameters for list endpoints.
#[derive(Debug, Deserialize)]
pub struct SortParams {
    #[serde(default = "default_sort_by")]
    pub sort_by: String,
    #[serde(default)]
    pub sort_order: SortOrder,
}

fn default_sort_by() -> String { "created_at".into() }

#[derive(Debug, Deserialize, Default, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    #[default]
    Desc,
    Asc,
}

impl SortOrder {
    pub fn as_sql(&self) -> &'static str {
        match self {
            Self::Asc => "ASC",
            Self::Desc => "DESC",
        }
    }
}

/// Mission creation request.
#[derive(Debug, Deserialize, Validate)]
pub struct CreateMissionRequest {
    #[validate(length(min = 1, max = 200))]
    pub title: String,
    #[validate(length(max = 2000))]
    pub description: Option<String>,
    #[validate(length(min = 1))]
    pub spec_id: String,
    pub config: Option<serde_json::Value>,
}

/// Mission update request.
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateMissionRequest {
    #[validate(length(min = 1, max = 200))]
    pub title: Option<String>,
    #[validate(length(max = 2000))]
    pub description: Option<String>,
    pub config: Option<serde_json::Value>,
}