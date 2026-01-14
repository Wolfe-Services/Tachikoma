//! Pagination support for list endpoints.

use serde::{Deserialize, Serialize};

/// Paginated response wrapper.
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    /// List of items.
    pub items: Vec<T>,
    /// Pagination metadata.
    pub pagination: PaginationMeta,
    /// Optional HATEOAS links.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<PaginationLinks>,
}

/// Pagination metadata.
#[derive(Debug, Serialize)]
pub struct PaginationMeta {
    /// Current page number (1-indexed).
    pub page: u32,
    /// Items per page.
    pub per_page: u32,
    /// Total number of items.
    pub total_items: u64,
    /// Total number of pages.
    pub total_pages: u32,
    /// Whether there's a next page.
    pub has_next: bool,
    /// Whether there's a previous page.
    pub has_prev: bool,
}

/// HATEOAS pagination links.
#[derive(Debug, Serialize)]
pub struct PaginationLinks {
    /// Link to current page.
    #[serde(rename = "self")]
    pub current: String,
    /// Link to first page.
    pub first: String,
    /// Link to last page.
    pub last: String,
    /// Link to next page (if exists).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
    /// Link to previous page (if exists).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev: Option<String>,
}

impl<T> PaginatedResponse<T> {
    /// Create a paginated response.
    pub fn new(items: Vec<T>, page: u32, per_page: u32, total_items: u64) -> Self {
        let total_pages = ((total_items as f64) / (per_page as f64)).ceil() as u32;
        let has_next = page < total_pages;
        let has_prev = page > 1;

        Self {
            items,
            pagination: PaginationMeta {
                page,
                per_page,
                total_items,
                total_pages,
                has_next,
                has_prev,
            },
            links: None,
        }
    }

    /// Add HATEOAS links.
    pub fn with_links(mut self, base_url: &str) -> Self {
        let pagination = &self.pagination;
        let build_url = |p: u32| format!("{}?page={}&per_page={}", base_url, p, pagination.per_page);

        self.links = Some(PaginationLinks {
            current: build_url(pagination.page),
            first: build_url(1),
            last: build_url(pagination.total_pages.max(1)),
            next: if pagination.has_next {
                Some(build_url(pagination.page + 1))
            } else {
                None
            },
            prev: if pagination.has_prev {
                Some(build_url(pagination.page - 1))
            } else {
                None
            },
        });

        self
    }
}

/// Pagination request parameters.
#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_per_page")]
    pub per_page: u32,
}

fn default_page() -> u32 { 1 }
fn default_per_page() -> u32 { 20 }

impl PaginationParams {
    /// Get the offset for database queries.
    pub fn offset(&self) -> u64 {
        ((self.page.saturating_sub(1)) * self.per_page) as u64
    }

    /// Get the limit, capped at maximum.
    pub fn limit(&self) -> u32 {
        self.per_page.min(100)
    }
}