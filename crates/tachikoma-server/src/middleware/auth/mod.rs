//! Authentication middleware for the Tachikoma API server.

pub mod extractor;
pub mod jwt;
pub mod layer;
pub mod types;

pub use extractor::{AdminAuth, Auth, MaybeAuth, RequireRole};
pub use jwt::{decode_token, encode_token, TokenDecoder};
pub use layer::{AuthLayer, AuthMiddleware};
pub use types::{ApiKeyAuth, AuthUser, Claims, TokenType};