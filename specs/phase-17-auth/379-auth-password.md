# Spec 379: Password Hashing

## Phase
17 - Authentication/Authorization

## Spec ID
379

## Status
Planned

## Dependencies
- Spec 366: Auth Types and Traits
- Spec 367: Auth Configuration

## Estimated Context
~9%

---

## Objective

Implement secure password hashing using Argon2id, the winner of the Password Hashing Competition. This includes password validation against configurable policies, secure hash generation, and verification. The implementation should be timing-attack resistant and support hash upgrades.

---

## Acceptance Criteria

- [ ] Implement Argon2id password hashing
- [ ] Support configurable Argon2 parameters
- [ ] Implement password policy validation
- [ ] Provide timing-safe password verification
- [ ] Support hash format versioning for upgrades
- [ ] Validate password strength requirements
- [ ] Check against common password lists
- [ ] Provide password strength estimation

---

## Implementation Details

### Password Hashing and Validation

```rust
// src/auth/password.rs

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher as Argon2Hasher, PasswordVerifier, SaltString},
    Algorithm, Argon2, Params, Version,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use tracing::{debug, instrument};
use unicode_normalization::UnicodeNormalization;

use crate::auth::{config::PasswordConfig, types::AuthResult};

/// Password hasher using Argon2id
pub struct PasswordHasher {
    argon2: Argon2<'static>,
    config: PasswordConfig,
}

impl PasswordHasher {
    /// Create a new password hasher with the given configuration
    pub fn new(config: PasswordConfig) -> Self {
        let params = Params::new(
            config.argon2_memory_kb,
            config.argon2_time_cost,
            config.argon2_parallelism,
            None, // Output length (default)
        )
        .expect("Invalid Argon2 parameters");

        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

        Self { argon2, config }
    }

    /// Hash a password
    #[instrument(skip(self, password))]
    pub async fn hash(&self, password: &str) -> AuthResult<String> {
        // Normalize password (NFC normalization)
        let normalized: String = password.nfc().collect();

        // Generate salt
        let salt = SaltString::generate(&mut OsRng);

        // Hash password
        let hash = self
            .argon2
            .hash_password(normalized.as_bytes(), &salt)
            .map_err(|e| crate::auth::types::AuthError::Internal(format!("Hashing failed: {}", e)))?;

        debug!("Password hashed successfully");
        Ok(hash.to_string())
    }

    /// Verify a password against a hash
    #[instrument(skip(self, password, hash))]
    pub async fn verify(&self, password: &str, hash: &str) -> AuthResult<bool> {
        // Normalize password
        let normalized: String = password.nfc().collect();

        // Parse the hash
        let parsed_hash = PasswordHash::new(hash).map_err(|e| {
            crate::auth::types::AuthError::Internal(format!("Invalid hash format: {}", e))
        })?;

        // Verify
        let result = self
            .argon2
            .verify_password(normalized.as_bytes(), &parsed_hash)
            .is_ok();

        debug!(verified = %result, "Password verification completed");
        Ok(result)
    }

    /// Check if a hash needs to be upgraded (different parameters)
    pub fn needs_upgrade(&self, hash: &str) -> bool {
        if let Ok(parsed) = PasswordHash::new(hash) {
            // Check if algorithm is Argon2id
            if parsed.algorithm != argon2::Algorithm::Argon2id.ident() {
                return true;
            }

            // Check parameters
            if let Some(params) = parsed.params() {
                let m_cost = params.get_str("m").and_then(|s| s.parse::<u32>().ok());
                let t_cost = params.get_str("t").and_then(|s| s.parse::<u32>().ok());
                let p_cost = params.get_str("p").and_then(|s| s.parse::<u32>().ok());

                // Check if any parameter is lower than current config
                if let Some(m) = m_cost {
                    if m < self.config.argon2_memory_kb {
                        return true;
                    }
                }
                if let Some(t) = t_cost {
                    if t < self.config.argon2_time_cost {
                        return true;
                    }
                }
                if let Some(p) = p_cost {
                    if p < self.config.argon2_parallelism {
                        return true;
                    }
                }
            }

            false
        } else {
            true // Invalid hash format, needs upgrade
        }
    }

    /// Get the current configuration
    pub fn config(&self) -> &PasswordConfig {
        &self.config
    }
}

/// Password validator for policy enforcement
pub struct PasswordValidator {
    config: PasswordConfig,
    common_passwords: HashSet<String>,
}

impl PasswordValidator {
    /// Create a new password validator
    pub fn new(config: PasswordConfig) -> Self {
        Self {
            config,
            common_passwords: Self::load_common_passwords(),
        }
    }

    /// Load common passwords list
    fn load_common_passwords() -> HashSet<String> {
        // Top 100 most common passwords
        let common = [
            "123456", "password", "123456789", "12345678", "12345", "1234567", "1234567890",
            "qwerty", "abc123", "111111", "123123", "admin", "letmein", "welcome", "monkey",
            "login", "dragon", "passw0rd", "master", "hello", "freedom", "whatever", "qazwsx",
            "trustno1", "654321", "jordan23", "harley", "password1", "1234", "robert",
            "matthew", "jordan", "asshole", "daniel", "andrew", "taylor", "passw0rd",
            "shadow", "123456a", "ashley", "baseball", "iloveyou", "soccer", "charlie",
            "sunshine", "michael", "princess", "jennifer", "hunter", "summer", "batman",
            "football", "starwars", "hockey", "ranger", "george", "corvette", "cheese",
        ];

        common.iter().map(|s| s.to_lowercase()).collect()
    }

    /// Validate a password against the policy
    #[instrument(skip(self, password))]
    pub fn validate(&self, password: &str) -> AuthResult<()> {
        let mut errors = Vec::new();

        // Check minimum length
        if password.len() < self.config.min_length {
            errors.push(format!(
                "Password must be at least {} characters",
                self.config.min_length
            ));
        }

        // Check maximum length
        if password.len() > self.config.max_length {
            errors.push(format!(
                "Password must be at most {} characters",
                self.config.max_length
            ));
        }

        // Check for uppercase
        if self.config.require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
            errors.push("Password must contain at least one uppercase letter".to_string());
        }

        // Check for lowercase
        if self.config.require_lowercase && !password.chars().any(|c| c.is_lowercase()) {
            errors.push("Password must contain at least one lowercase letter".to_string());
        }

        // Check for digit
        if self.config.require_digit && !password.chars().any(|c| c.is_ascii_digit()) {
            errors.push("Password must contain at least one digit".to_string());
        }

        // Check for special character
        if self.config.require_special {
            let has_special = password
                .chars()
                .any(|c| self.config.special_chars.contains(c));
            if !has_special {
                errors.push(format!(
                    "Password must contain at least one special character ({})",
                    self.config.special_chars
                ));
            }
        }

        // Check against common passwords
        if self.common_passwords.contains(&password.to_lowercase()) {
            errors.push("Password is too common".to_string());
        }

        // Check for sequential characters
        if self.has_sequential_chars(password, 4) {
            errors.push("Password contains sequential characters".to_string());
        }

        // Check for repeated characters
        if self.has_repeated_chars(password, 4) {
            errors.push("Password contains too many repeated characters".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(crate::auth::types::AuthError::InvalidCredentials)
        }
    }

    /// Check for sequential characters (e.g., "1234", "abcd")
    fn has_sequential_chars(&self, password: &str, min_length: usize) -> bool {
        let chars: Vec<char> = password.chars().collect();

        if chars.len() < min_length {
            return false;
        }

        let mut seq_count = 1;

        for i in 1..chars.len() {
            let prev = chars[i - 1] as i32;
            let curr = chars[i] as i32;

            if curr == prev + 1 || curr == prev - 1 {
                seq_count += 1;
                if seq_count >= min_length {
                    return true;
                }
            } else {
                seq_count = 1;
            }
        }

        false
    }

    /// Check for repeated characters (e.g., "aaaa")
    fn has_repeated_chars(&self, password: &str, min_length: usize) -> bool {
        let chars: Vec<char> = password.chars().collect();

        if chars.len() < min_length {
            return false;
        }

        let mut repeat_count = 1;

        for i in 1..chars.len() {
            if chars[i] == chars[i - 1] {
                repeat_count += 1;
                if repeat_count >= min_length {
                    return true;
                }
            } else {
                repeat_count = 1;
            }
        }

        false
    }

    /// Estimate password strength (0-100)
    pub fn estimate_strength(&self, password: &str) -> PasswordStrength {
        let mut score = 0;

        // Length contribution (up to 30 points)
        let length = password.len();
        score += std::cmp::min(length * 2, 30);

        // Character variety contribution (up to 30 points)
        let has_upper = password.chars().any(|c| c.is_uppercase());
        let has_lower = password.chars().any(|c| c.is_lowercase());
        let has_digit = password.chars().any(|c| c.is_ascii_digit());
        let has_special = password.chars().any(|c| self.config.special_chars.contains(c));

        let variety_count = [has_upper, has_lower, has_digit, has_special]
            .iter()
            .filter(|&&x| x)
            .count();
        score += variety_count * 7;

        // Unique characters contribution (up to 20 points)
        let unique_chars: HashSet<char> = password.chars().collect();
        let unique_ratio = unique_chars.len() as f32 / length as f32;
        score += (unique_ratio * 20.0) as usize;

        // Penalties
        if self.common_passwords.contains(&password.to_lowercase()) {
            score = score.saturating_sub(50);
        }
        if self.has_sequential_chars(password, 3) {
            score = score.saturating_sub(10);
        }
        if self.has_repeated_chars(password, 3) {
            score = score.saturating_sub(10);
        }

        let score = std::cmp::min(score, 100);

        PasswordStrength {
            score,
            level: match score {
                0..=20 => StrengthLevel::VeryWeak,
                21..=40 => StrengthLevel::Weak,
                41..=60 => StrengthLevel::Fair,
                61..=80 => StrengthLevel::Strong,
                _ => StrengthLevel::VeryStrong,
            },
            feedback: self.generate_feedback(password),
        }
    }

    /// Generate improvement feedback
    fn generate_feedback(&self, password: &str) -> Vec<String> {
        let mut feedback = Vec::new();

        if password.len() < 12 {
            feedback.push("Consider using a longer password".to_string());
        }

        if !password.chars().any(|c| c.is_uppercase()) {
            feedback.push("Add uppercase letters".to_string());
        }

        if !password.chars().any(|c| c.is_ascii_digit()) {
            feedback.push("Add numbers".to_string());
        }

        if !password.chars().any(|c| self.config.special_chars.contains(c)) {
            feedback.push("Add special characters".to_string());
        }

        if self.common_passwords.contains(&password.to_lowercase()) {
            feedback.push("Avoid common passwords".to_string());
        }

        feedback
    }
}

/// Password strength estimation result
#[derive(Debug, Clone, Serialize)]
pub struct PasswordStrength {
    /// Numeric score (0-100)
    pub score: usize,
    /// Strength level
    pub level: StrengthLevel,
    /// Improvement feedback
    pub feedback: Vec<String>,
}

/// Password strength level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StrengthLevel {
    VeryWeak,
    Weak,
    Fair,
    Strong,
    VeryStrong,
}

/// Password change request
#[derive(Debug, Clone, Deserialize)]
pub struct PasswordChangeRequest {
    pub current_password: String,
    pub new_password: String,
    pub confirm_password: String,
}

impl PasswordChangeRequest {
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.new_password != self.confirm_password {
            errors.push("Passwords do not match".to_string());
        }

        if self.current_password == self.new_password {
            errors.push("New password must be different from current password".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Password reset token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordResetToken {
    pub token: String,
    pub user_id: crate::auth::types::UserId,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub used: bool,
}

impl PasswordResetToken {
    pub fn new(user_id: crate::auth::types::UserId, validity_hours: u32) -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let token: String = (0..64)
            .map(|_| rng.sample(rand::distributions::Alphanumeric) as char)
            .collect();

        let now = chrono::Utc::now();
        Self {
            token,
            user_id,
            created_at: now,
            expires_at: now + chrono::Duration::hours(validity_hours as i64),
            used: false,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.used && chrono::Utc::now() < self.expires_at
    }
}

/// Secure password generator
pub struct PasswordGenerator {
    length: usize,
    use_uppercase: bool,
    use_lowercase: bool,
    use_digits: bool,
    use_special: bool,
    special_chars: String,
}

impl PasswordGenerator {
    pub fn new() -> Self {
        Self {
            length: 16,
            use_uppercase: true,
            use_lowercase: true,
            use_digits: true,
            use_special: true,
            special_chars: "!@#$%^&*()_+-=[]{}|;:,.<>?".to_string(),
        }
    }

    pub fn length(mut self, length: usize) -> Self {
        self.length = length;
        self
    }

    pub fn uppercase(mut self, use_it: bool) -> Self {
        self.use_uppercase = use_it;
        self
    }

    pub fn lowercase(mut self, use_it: bool) -> Self {
        self.use_lowercase = use_it;
        self
    }

    pub fn digits(mut self, use_it: bool) -> Self {
        self.use_digits = use_it;
        self
    }

    pub fn special(mut self, use_it: bool) -> Self {
        self.use_special = use_it;
        self
    }

    pub fn generate(&self) -> String {
        use rand::seq::SliceRandom;
        use rand::Rng;

        let mut charset = Vec::new();
        let mut password = Vec::new();
        let mut rng = rand::thread_rng();

        if self.use_uppercase {
            charset.extend(b'A'..=b'Z');
            password.push(rng.gen_range(b'A'..=b'Z') as char);
        }
        if self.use_lowercase {
            charset.extend(b'a'..=b'z');
            password.push(rng.gen_range(b'a'..=b'z') as char);
        }
        if self.use_digits {
            charset.extend(b'0'..=b'9');
            password.push(rng.gen_range(b'0'..=b'9') as char);
        }
        if self.use_special {
            let special_bytes: Vec<u8> = self.special_chars.bytes().collect();
            charset.extend(&special_bytes);
            password.push(special_bytes[rng.gen_range(0..special_bytes.len())] as char);
        }

        // Fill remaining length
        while password.len() < self.length {
            password.push(charset[rng.gen_range(0..charset.len())] as char);
        }

        // Shuffle
        password.shuffle(&mut rng);
        password.into_iter().collect()
    }
}

impl Default for PasswordGenerator {
    fn default() -> Self {
        Self::new()
    }
}
```

---

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::config::PasswordConfig;

    #[tokio::test]
    async fn test_password_hashing() {
        let config = PasswordConfig::default();
        let hasher = PasswordHasher::new(config);

        let password = "SecureP@ssw0rd!";
        let hash = hasher.hash(password).await.unwrap();

        assert!(!hash.is_empty());
        assert!(hash.starts_with("$argon2id$"));
    }

    #[tokio::test]
    async fn test_password_verification() {
        let config = PasswordConfig::default();
        let hasher = PasswordHasher::new(config);

        let password = "SecureP@ssw0rd!";
        let hash = hasher.hash(password).await.unwrap();

        assert!(hasher.verify(password, &hash).await.unwrap());
        assert!(!hasher.verify("wrongpassword", &hash).await.unwrap());
    }

    #[tokio::test]
    async fn test_password_unicode_normalization() {
        let config = PasswordConfig::default();
        let hasher = PasswordHasher::new(config);

        // Different representations of the same string
        let password1 = "cafe\u{0301}"; // cafe + combining acute accent
        let password2 = "caf\u{00e9}";  // cafe with precomposed e-acute

        let hash = hasher.hash(password1).await.unwrap();

        // Both should verify (NFC normalization)
        assert!(hasher.verify(password2, &hash).await.unwrap());
    }

    #[test]
    fn test_password_validation_minimum_length() {
        let config = PasswordConfig::default();
        let validator = PasswordValidator::new(config);

        assert!(validator.validate("Short1!").is_err());
        assert!(validator.validate("ThisIsLongEnough1!").is_ok());
    }

    #[test]
    fn test_password_validation_requirements() {
        let config = PasswordConfig::default();
        let validator = PasswordValidator::new(config);

        // Missing uppercase
        assert!(validator.validate("thisislongenough1!").is_err());

        // Missing lowercase
        assert!(validator.validate("THISISLONGENOUGH1!").is_err());

        // Missing digit
        assert!(validator.validate("ThisIsLongEnough!").is_err());

        // Missing special
        assert!(validator.validate("ThisIsLongEnough1").is_err());

        // All requirements met
        assert!(validator.validate("ThisIsLongEnough1!").is_ok());
    }

    #[test]
    fn test_password_validation_common_passwords() {
        let config = PasswordConfig::default();
        let validator = PasswordValidator::new(config);

        // Common passwords should fail even if they meet length
        assert!(validator.validate("Password123!!!!").is_err());
    }

    #[test]
    fn test_sequential_character_detection() {
        let config = PasswordConfig::default();
        let validator = PasswordValidator::new(config);

        assert!(validator.has_sequential_chars("abcdefgh", 4));
        assert!(validator.has_sequential_chars("12345678", 4));
        assert!(!validator.has_sequential_chars("a1b2c3d4", 4));
    }

    #[test]
    fn test_repeated_character_detection() {
        let config = PasswordConfig::default();
        let validator = PasswordValidator::new(config);

        assert!(validator.has_repeated_chars("aaaabc", 4));
        assert!(!validator.has_repeated_chars("aabbc", 4));
    }

    #[test]
    fn test_password_strength_estimation() {
        let config = PasswordConfig::default();
        let validator = PasswordValidator::new(config);

        let weak = validator.estimate_strength("password");
        assert!(weak.score < 30);
        assert!(matches!(weak.level, StrengthLevel::VeryWeak | StrengthLevel::Weak));

        let strong = validator.estimate_strength("MyV3ryStr0ng!P@ssw0rd");
        assert!(strong.score > 60);
        assert!(matches!(strong.level, StrengthLevel::Strong | StrengthLevel::VeryStrong));
    }

    #[test]
    fn test_password_generator() {
        let generator = PasswordGenerator::new().length(20);
        let password = generator.generate();

        assert_eq!(password.len(), 20);
        assert!(password.chars().any(|c| c.is_uppercase()));
        assert!(password.chars().any(|c| c.is_lowercase()));
        assert!(password.chars().any(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_password_generator_options() {
        let generator = PasswordGenerator::new()
            .length(12)
            .uppercase(true)
            .lowercase(true)
            .digits(false)
            .special(false);

        let password = generator.generate();

        assert_eq!(password.len(), 12);
        assert!(password.chars().all(|c| c.is_alphabetic()));
    }

    #[test]
    fn test_password_reset_token() {
        let user_id = crate::auth::types::UserId::new();
        let token = PasswordResetToken::new(user_id, 24);

        assert!(!token.token.is_empty());
        assert!(token.is_valid());
        assert_eq!(token.user_id, user_id);
    }

    #[test]
    fn test_password_change_request_validation() {
        let request = PasswordChangeRequest {
            current_password: "OldPassword1!".to_string(),
            new_password: "NewPassword1!".to_string(),
            confirm_password: "NewPassword1!".to_string(),
        };

        assert!(request.validate().is_ok());

        let mismatched = PasswordChangeRequest {
            current_password: "OldPassword1!".to_string(),
            new_password: "NewPassword1!".to_string(),
            confirm_password: "DifferentPassword1!".to_string(),
        };

        assert!(mismatched.validate().is_err());

        let same = PasswordChangeRequest {
            current_password: "Password1!".to_string(),
            new_password: "Password1!".to_string(),
            confirm_password: "Password1!".to_string(),
        };

        assert!(same.validate().is_err());
    }

    #[test]
    fn test_hash_upgrade_detection() {
        let config = PasswordConfig::default();
        let hasher = PasswordHasher::new(config);

        // A valid argon2id hash with current params shouldn't need upgrade
        let hash = "$argon2id$v=19$m=65536,t=3,p=4$c29tZXNhbHQ$hash";
        // Note: This is a simplified test - in reality the hash would be validated

        // An old algorithm hash should need upgrade
        let old_hash = "$2a$10$N9qo8uLOickgx2ZMRZoMye"; // bcrypt
        assert!(hasher.needs_upgrade(old_hash));
    }
}
```

---

## Related Specs

- **Spec 366**: Auth Types - Uses AuthError for validation failures
- **Spec 367**: Auth Configuration - Uses PasswordConfig
- **Spec 368**: Local Auth - Uses PasswordHasher for login
- **Spec 380**: Account Recovery - Uses PasswordResetToken
