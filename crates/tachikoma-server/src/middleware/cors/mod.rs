//! CORS (Cross-Origin Resource Sharing) middleware.

pub mod builder;
pub mod config;
pub mod layer;

pub use builder::CorsBuilder;
pub use config::{AllowedHeaders, AllowedOrigins, CorsConfig};
pub use layer::{CorsLayer, CorsMiddleware};