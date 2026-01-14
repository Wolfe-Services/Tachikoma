//! JWT encoding and decoding utilities.

use super::types::Claims;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};

/// Encode claims into a JWT token.
pub fn encode_token(claims: &Claims, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
    encode(
        &Header::default(),
        claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

/// Decode and validate a JWT token.
pub fn decode_token(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let mut validation = Validation::default();
    validation.validate_exp = true;

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )?;

    Ok(token_data.claims)
}

/// Token decoder for use in extractors.
pub struct TokenDecoder {
    secret: String,
}

impl TokenDecoder {
    pub fn new(secret: String) -> Self {
        Self { secret }
    }

    pub fn decode(&self, token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        decode_token(token, &self.secret)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_encode_decode_roundtrip() {
        let secret = "test_secret_key_32_chars_long!!";
        let claims = Claims::new_access(
            Uuid::new_v4(),
            "test@example.com",
            vec!["user".into()],
            3600,
        );

        let token = encode_token(&claims, secret).unwrap();
        let decoded = decode_token(&token, secret).unwrap();

        assert_eq!(decoded.sub, claims.sub);
        assert_eq!(decoded.email, claims.email);
    }
}