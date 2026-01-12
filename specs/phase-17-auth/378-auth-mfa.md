# Spec 378: Multi-Factor Authentication

## Phase
17 - Authentication/Authorization

## Spec ID
378

## Status
Planned

## Dependencies
- Spec 366: Auth Types and Traits
- Spec 367: Auth Configuration
- Spec 368: Local User Auth

## Estimated Context
~11%

---

## Objective

Implement multi-factor authentication (MFA) to provide an additional layer of security beyond passwords. Support TOTP (Time-based One-Time Password), backup codes, and the infrastructure for future methods like SMS, email, or hardware keys.

---

## Acceptance Criteria

- [ ] Implement TOTP (RFC 6238) generation and validation
- [ ] Generate and validate backup codes
- [ ] Create `MfaManager` for MFA lifecycle
- [ ] Support MFA enrollment flow
- [ ] Support MFA verification during login
- [ ] Allow MFA recovery using backup codes
- [ ] Generate QR codes for TOTP setup
- [ ] Track MFA method usage
- [ ] Support disabling MFA with verification

---

## Implementation Details

### MFA Types and Manager

```rust
// src/auth/mfa.rs

use async_trait::async_trait;
use base32::Alphabet;
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use sha1::Sha1;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, instrument};

use crate::auth::{
    config::MfaConfig,
    events::{AuthEvent, AuthEventEmitter},
    types::*,
};

/// MFA enrollment data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaEnrollment {
    /// User ID
    pub user_id: UserId,

    /// Whether MFA is enabled
    pub enabled: bool,

    /// TOTP secret (encrypted)
    pub totp_secret: Option<String>,

    /// Backup codes (hashed)
    pub backup_codes: Vec<HashedBackupCode>,

    /// When MFA was enabled
    pub enabled_at: Option<DateTime<Utc>>,

    /// When MFA was last verified
    pub last_verified_at: Option<DateTime<Utc>>,

    /// Recovery email (if different from primary)
    pub recovery_email: Option<String>,

    /// Phone number for SMS (future)
    pub phone_number: Option<String>,
}

/// Hashed backup code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashedBackupCode {
    pub hash: String,
    pub used: bool,
    pub used_at: Option<DateTime<Utc>>,
}

/// TOTP setup data returned during enrollment
#[derive(Debug, Clone, Serialize)]
pub struct TotpSetup {
    /// Base32 encoded secret
    pub secret: String,

    /// URI for QR code generation
    pub uri: String,

    /// QR code as base64 PNG (optional)
    pub qr_code: Option<String>,

    /// Backup codes (only shown once!)
    pub backup_codes: Vec<String>,
}

/// MFA verification request
#[derive(Debug, Clone, Deserialize)]
pub struct MfaVerifyRequest {
    /// The verification code
    pub code: String,

    /// Type of code (totp, backup)
    pub code_type: MfaCodeType,
}

/// MFA code type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MfaCodeType {
    Totp,
    Backup,
    Sms,
    Email,
}

/// MFA challenge for pending verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaChallenge {
    pub id: String,
    pub user_id: UserId,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub attempts: u32,
    pub max_attempts: u32,
}

impl MfaChallenge {
    pub fn new(user_id: UserId, validity_secs: u64, max_attempts: u32) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            user_id,
            created_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::seconds(validity_secs as i64),
            attempts: 0,
            max_attempts,
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn is_valid(&self) -> bool {
        !self.is_expired() && self.attempts < self.max_attempts
    }
}

/// TOTP generator and validator
pub struct TotpProvider {
    /// Issuer name for TOTP URI
    issuer: String,
    /// Number of digits in code
    digits: u32,
    /// Time step in seconds
    step: u64,
    /// Number of time steps to allow for clock skew
    skew: u32,
}

impl TotpProvider {
    pub fn new(config: &MfaConfig) -> Self {
        Self {
            issuer: config.totp_issuer.clone(),
            digits: config.totp_digits,
            step: config.totp_step_secs,
            skew: 1, // Allow 1 step before/after
        }
    }

    /// Generate a new random secret
    pub fn generate_secret(&self) -> String {
        let mut rng = thread_rng();
        let secret: Vec<u8> = (0..20).map(|_| rng.gen()).collect();
        base32::encode(Alphabet::Rfc4648 { padding: false }, &secret)
    }

    /// Generate TOTP URI for QR code
    pub fn generate_uri(&self, secret: &str, account: &str) -> String {
        format!(
            "otpauth://totp/{}:{}?secret={}&issuer={}&digits={}&period={}",
            urlencoding::encode(&self.issuer),
            urlencoding::encode(account),
            secret,
            urlencoding::encode(&self.issuer),
            self.digits,
            self.step
        )
    }

    /// Generate current TOTP code (for testing/display)
    pub fn generate_code(&self, secret: &str) -> Result<String, MfaError> {
        let secret_bytes = base32::decode(Alphabet::Rfc4648 { padding: false }, secret)
            .ok_or(MfaError::InvalidSecret)?;

        let time = (Utc::now().timestamp() as u64) / self.step;
        self.generate_code_for_time(&secret_bytes, time)
    }

    /// Verify a TOTP code
    pub fn verify_code(&self, secret: &str, code: &str) -> Result<bool, MfaError> {
        let secret_bytes = base32::decode(Alphabet::Rfc4648 { padding: false }, secret)
            .ok_or(MfaError::InvalidSecret)?;

        let current_time = (Utc::now().timestamp() as u64) / self.step;

        // Check current time and allowed skew
        for offset in 0..=self.skew {
            // Check current - offset
            if offset > 0 {
                if let Ok(expected) = self.generate_code_for_time(&secret_bytes, current_time - offset as u64) {
                    if constant_time_eq(code.as_bytes(), expected.as_bytes()) {
                        return Ok(true);
                    }
                }
            }

            // Check current + offset
            if let Ok(expected) = self.generate_code_for_time(&secret_bytes, current_time + offset as u64) {
                if constant_time_eq(code.as_bytes(), expected.as_bytes()) {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// Generate TOTP code for a specific time counter
    fn generate_code_for_time(&self, secret: &[u8], time: u64) -> Result<String, MfaError> {
        let time_bytes = time.to_be_bytes();

        let mut mac = Hmac::<Sha1>::new_from_slice(secret)
            .map_err(|_| MfaError::InvalidSecret)?;
        mac.update(&time_bytes);
        let result = mac.finalize().into_bytes();

        // Dynamic truncation
        let offset = (result[result.len() - 1] & 0x0f) as usize;
        let binary = ((result[offset] & 0x7f) as u32) << 24
            | (result[offset + 1] as u32) << 16
            | (result[offset + 2] as u32) << 8
            | (result[offset + 3] as u32);

        let otp = binary % 10u32.pow(self.digits);
        Ok(format!("{:0>width$}", otp, width = self.digits as usize))
    }

    /// Generate QR code as base64 PNG
    #[cfg(feature = "qrcode")]
    pub fn generate_qr_code(&self, uri: &str) -> Result<String, MfaError> {
        use qrcode::QrCode;
        use image::Luma;

        let code = QrCode::new(uri).map_err(|_| MfaError::QrCodeError)?;
        let image = code.render::<Luma<u8>>().build();

        let mut png_bytes = Vec::new();
        image
            .write_to(&mut std::io::Cursor::new(&mut png_bytes), image::ImageOutputFormat::Png)
            .map_err(|_| MfaError::QrCodeError)?;

        Ok(base64::encode(&png_bytes))
    }

    #[cfg(not(feature = "qrcode"))]
    pub fn generate_qr_code(&self, _uri: &str) -> Result<String, MfaError> {
        Err(MfaError::QrCodeError)
    }
}

/// Constant-time comparison for security
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }
    result == 0
}

/// Backup code generator
pub struct BackupCodeGenerator {
    code_count: usize,
    code_length: usize,
}

impl BackupCodeGenerator {
    pub fn new(config: &MfaConfig) -> Self {
        Self {
            code_count: config.backup_code_count,
            code_length: config.backup_code_length,
        }
    }

    /// Generate new backup codes
    pub fn generate(&self) -> Vec<String> {
        let mut rng = thread_rng();
        let charset: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789"; // Avoiding confusing chars

        (0..self.code_count)
            .map(|_| {
                (0..self.code_length)
                    .map(|_| charset[rng.gen_range(0..charset.len())] as char)
                    .collect()
            })
            .collect()
    }

    /// Hash a backup code for storage
    pub fn hash_code(code: &str) -> String {
        use sha2::{Sha256, Digest};
        let normalized = code.to_uppercase().replace("-", "").replace(" ", "");
        let mut hasher = Sha256::new();
        hasher.update(normalized.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Verify and consume a backup code
    pub fn verify_and_consume(
        codes: &mut Vec<HashedBackupCode>,
        input: &str,
    ) -> bool {
        let input_hash = Self::hash_code(input);

        for code in codes.iter_mut() {
            if !code.used && constant_time_eq(code.hash.as_bytes(), input_hash.as_bytes()) {
                code.used = true;
                code.used_at = Some(Utc::now());
                return true;
            }
        }

        false
    }
}

/// MFA errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum MfaError {
    #[error("MFA not enabled")]
    NotEnabled,

    #[error("MFA already enabled")]
    AlreadyEnabled,

    #[error("Invalid TOTP secret")]
    InvalidSecret,

    #[error("Invalid verification code")]
    InvalidCode,

    #[error("Too many attempts")]
    TooManyAttempts,

    #[error("Challenge expired")]
    ChallengeExpired,

    #[error("No backup codes remaining")]
    NoBackupCodes,

    #[error("QR code generation failed")]
    QrCodeError,

    #[error("Storage error: {0}")]
    StorageError(String),
}

/// MFA Manager
pub struct MfaManager {
    storage: Arc<dyn MfaStorage>,
    totp_provider: TotpProvider,
    backup_generator: BackupCodeGenerator,
    event_emitter: Arc<dyn AuthEventEmitter>,
    config: MfaConfig,
    /// Active challenges
    challenges: RwLock<HashMap<String, MfaChallenge>>,
}

impl MfaManager {
    pub fn new(
        storage: Arc<dyn MfaStorage>,
        event_emitter: Arc<dyn AuthEventEmitter>,
        config: MfaConfig,
    ) -> Self {
        Self {
            storage,
            totp_provider: TotpProvider::new(&config),
            backup_generator: BackupCodeGenerator::new(&config),
            event_emitter,
            config,
            challenges: RwLock::new(HashMap::new()),
        }
    }

    /// Start MFA enrollment
    #[instrument(skip(self), fields(user_id = %user_id))]
    pub async fn start_enrollment(
        &self,
        user_id: UserId,
        account_name: &str,
    ) -> Result<TotpSetup, MfaError> {
        // Check if already enrolled
        if let Some(enrollment) = self.storage.get(user_id).await.map_err(|e| MfaError::StorageError(e.to_string()))? {
            if enrollment.enabled {
                return Err(MfaError::AlreadyEnabled);
            }
        }

        // Generate secret and backup codes
        let secret = self.totp_provider.generate_secret();
        let uri = self.totp_provider.generate_uri(&secret, account_name);
        let backup_codes = self.backup_generator.generate();

        // Hash backup codes for storage
        let hashed_codes: Vec<HashedBackupCode> = backup_codes
            .iter()
            .map(|c| HashedBackupCode {
                hash: BackupCodeGenerator::hash_code(c),
                used: false,
                used_at: None,
            })
            .collect();

        // Store enrollment (not yet enabled)
        let enrollment = MfaEnrollment {
            user_id,
            enabled: false,
            totp_secret: Some(secret.clone()),
            backup_codes: hashed_codes,
            enabled_at: None,
            last_verified_at: None,
            recovery_email: None,
            phone_number: None,
        };

        self.storage.save(&enrollment).await.map_err(|e| MfaError::StorageError(e.to_string()))?;

        // Generate QR code
        let qr_code = self.totp_provider.generate_qr_code(&uri).ok();

        Ok(TotpSetup {
            secret,
            uri,
            qr_code,
            backup_codes,
        })
    }

    /// Complete MFA enrollment by verifying first code
    #[instrument(skip(self, code), fields(user_id = %user_id))]
    pub async fn complete_enrollment(
        &self,
        user_id: UserId,
        code: &str,
    ) -> Result<(), MfaError> {
        let mut enrollment = self
            .storage
            .get(user_id)
            .await
            .map_err(|e| MfaError::StorageError(e.to_string()))?
            .ok_or(MfaError::NotEnabled)?;

        if enrollment.enabled {
            return Err(MfaError::AlreadyEnabled);
        }

        let secret = enrollment.totp_secret.as_ref().ok_or(MfaError::InvalidSecret)?;

        // Verify the code
        if !self.totp_provider.verify_code(secret, code)? {
            return Err(MfaError::InvalidCode);
        }

        // Enable MFA
        enrollment.enabled = true;
        enrollment.enabled_at = Some(Utc::now());
        enrollment.last_verified_at = Some(Utc::now());

        self.storage.save(&enrollment).await.map_err(|e| MfaError::StorageError(e.to_string()))?;

        self.event_emitter
            .emit(AuthEvent::MfaEnabled {
                user_id,
                timestamp: Utc::now(),
            })
            .await;

        info!("MFA enrollment completed");
        Ok(())
    }

    /// Create MFA challenge for login
    pub async fn create_challenge(&self, user_id: UserId) -> Result<MfaChallenge, MfaError> {
        let challenge = MfaChallenge::new(user_id, 300, 5); // 5 minutes, 5 attempts

        let mut challenges = self.challenges.write().await;
        challenges.insert(challenge.id.clone(), challenge.clone());

        Ok(challenge)
    }

    /// Verify MFA during login
    #[instrument(skip(self, code), fields(challenge_id = %challenge_id))]
    pub async fn verify(
        &self,
        challenge_id: &str,
        code: &str,
        code_type: MfaCodeType,
    ) -> Result<UserId, MfaError> {
        // Get and validate challenge
        let mut challenges = self.challenges.write().await;
        let challenge = challenges
            .get_mut(challenge_id)
            .ok_or(MfaError::ChallengeExpired)?;

        if !challenge.is_valid() {
            challenges.remove(challenge_id);
            return Err(if challenge.is_expired() {
                MfaError::ChallengeExpired
            } else {
                MfaError::TooManyAttempts
            });
        }

        challenge.attempts += 1;
        let user_id = challenge.user_id;

        // Get enrollment
        let mut enrollment = self
            .storage
            .get(user_id)
            .await
            .map_err(|e| MfaError::StorageError(e.to_string()))?
            .ok_or(MfaError::NotEnabled)?;

        if !enrollment.enabled {
            return Err(MfaError::NotEnabled);
        }

        // Verify based on code type
        let verified = match code_type {
            MfaCodeType::Totp => {
                let secret = enrollment.totp_secret.as_ref().ok_or(MfaError::InvalidSecret)?;
                self.totp_provider.verify_code(secret, code)?
            }
            MfaCodeType::Backup => {
                BackupCodeGenerator::verify_and_consume(&mut enrollment.backup_codes, code)
            }
            _ => return Err(MfaError::InvalidCode),
        };

        if !verified {
            if challenge.attempts >= challenge.max_attempts {
                challenges.remove(challenge_id);
                return Err(MfaError::TooManyAttempts);
            }
            return Err(MfaError::InvalidCode);
        }

        // Update enrollment
        enrollment.last_verified_at = Some(Utc::now());
        self.storage.save(&enrollment).await.map_err(|e| MfaError::StorageError(e.to_string()))?;

        // Remove challenge
        challenges.remove(challenge_id);

        self.event_emitter
            .emit(AuthEvent::MfaVerified {
                user_id,
                method: code_type,
                timestamp: Utc::now(),
            })
            .await;

        Ok(user_id)
    }

    /// Check if MFA is required for user
    pub async fn is_required(&self, user_id: UserId) -> bool {
        if self.config.required {
            return true;
        }

        match self.storage.get(user_id).await {
            Ok(Some(enrollment)) => enrollment.enabled,
            _ => false,
        }
    }

    /// Disable MFA (requires current MFA verification)
    #[instrument(skip(self, verification_code), fields(user_id = %user_id))]
    pub async fn disable(
        &self,
        user_id: UserId,
        verification_code: &str,
    ) -> Result<(), MfaError> {
        let mut enrollment = self
            .storage
            .get(user_id)
            .await
            .map_err(|e| MfaError::StorageError(e.to_string()))?
            .ok_or(MfaError::NotEnabled)?;

        if !enrollment.enabled {
            return Err(MfaError::NotEnabled);
        }

        // Verify current code
        let secret = enrollment.totp_secret.as_ref().ok_or(MfaError::InvalidSecret)?;
        if !self.totp_provider.verify_code(secret, verification_code)? {
            return Err(MfaError::InvalidCode);
        }

        // Disable MFA
        enrollment.enabled = false;
        enrollment.totp_secret = None;
        enrollment.backup_codes.clear();

        self.storage.save(&enrollment).await.map_err(|e| MfaError::StorageError(e.to_string()))?;

        self.event_emitter
            .emit(AuthEvent::MfaDisabled {
                user_id,
                timestamp: Utc::now(),
            })
            .await;

        info!("MFA disabled");
        Ok(())
    }

    /// Regenerate backup codes
    #[instrument(skip(self, verification_code), fields(user_id = %user_id))]
    pub async fn regenerate_backup_codes(
        &self,
        user_id: UserId,
        verification_code: &str,
    ) -> Result<Vec<String>, MfaError> {
        let mut enrollment = self
            .storage
            .get(user_id)
            .await
            .map_err(|e| MfaError::StorageError(e.to_string()))?
            .ok_or(MfaError::NotEnabled)?;

        if !enrollment.enabled {
            return Err(MfaError::NotEnabled);
        }

        // Verify current code
        let secret = enrollment.totp_secret.as_ref().ok_or(MfaError::InvalidSecret)?;
        if !self.totp_provider.verify_code(secret, verification_code)? {
            return Err(MfaError::InvalidCode);
        }

        // Generate new backup codes
        let new_codes = self.backup_generator.generate();
        enrollment.backup_codes = new_codes
            .iter()
            .map(|c| HashedBackupCode {
                hash: BackupCodeGenerator::hash_code(c),
                used: false,
                used_at: None,
            })
            .collect();

        self.storage.save(&enrollment).await.map_err(|e| MfaError::StorageError(e.to_string()))?;

        info!("Backup codes regenerated");
        Ok(new_codes)
    }

    /// Get remaining backup codes count
    pub async fn get_backup_codes_count(&self, user_id: UserId) -> Result<usize, MfaError> {
        let enrollment = self
            .storage
            .get(user_id)
            .await
            .map_err(|e| MfaError::StorageError(e.to_string()))?
            .ok_or(MfaError::NotEnabled)?;

        Ok(enrollment.backup_codes.iter().filter(|c| !c.used).count())
    }
}

/// MFA storage trait
#[async_trait]
pub trait MfaStorage: Send + Sync {
    async fn get(&self, user_id: UserId) -> AuthResult<Option<MfaEnrollment>>;
    async fn save(&self, enrollment: &MfaEnrollment) -> AuthResult<()>;
    async fn delete(&self, user_id: UserId) -> AuthResult<()>;
}

/// In-memory MFA storage
pub struct InMemoryMfaStorage {
    enrollments: RwLock<HashMap<UserId, MfaEnrollment>>,
}

impl InMemoryMfaStorage {
    pub fn new() -> Self {
        Self {
            enrollments: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl MfaStorage for InMemoryMfaStorage {
    async fn get(&self, user_id: UserId) -> AuthResult<Option<MfaEnrollment>> {
        let enrollments = self.enrollments.read().await;
        Ok(enrollments.get(&user_id).cloned())
    }

    async fn save(&self, enrollment: &MfaEnrollment) -> AuthResult<()> {
        let mut enrollments = self.enrollments.write().await;
        enrollments.insert(enrollment.user_id, enrollment.clone());
        Ok(())
    }

    async fn delete(&self, user_id: UserId) -> AuthResult<()> {
        let mut enrollments = self.enrollments.write().await;
        enrollments.remove(&user_id);
        Ok(())
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

    #[test]
    fn test_totp_generation() {
        let config = MfaConfig::default();
        let provider = TotpProvider::new(&config);

        let secret = provider.generate_secret();
        assert!(!secret.is_empty());

        let code = provider.generate_code(&secret).unwrap();
        assert_eq!(code.len(), 6);
    }

    #[test]
    fn test_totp_verification() {
        let config = MfaConfig::default();
        let provider = TotpProvider::new(&config);

        let secret = provider.generate_secret();
        let code = provider.generate_code(&secret).unwrap();

        assert!(provider.verify_code(&secret, &code).unwrap());
        assert!(!provider.verify_code(&secret, "000000").unwrap());
    }

    #[test]
    fn test_totp_uri_generation() {
        let config = MfaConfig::default();
        let provider = TotpProvider::new(&config);

        let uri = provider.generate_uri("SECRET123", "user@example.com");

        assert!(uri.starts_with("otpauth://totp/"));
        assert!(uri.contains("secret=SECRET123"));
        assert!(uri.contains("user%40example.com"));
    }

    #[test]
    fn test_backup_code_generation() {
        let config = MfaConfig::default();
        let generator = BackupCodeGenerator::new(&config);

        let codes = generator.generate();

        assert_eq!(codes.len(), config.backup_code_count);
        for code in &codes {
            assert_eq!(code.len(), config.backup_code_length);
        }
    }

    #[test]
    fn test_backup_code_verification() {
        let config = MfaConfig::default();
        let generator = BackupCodeGenerator::new(&config);

        let codes = generator.generate();
        let mut hashed: Vec<HashedBackupCode> = codes
            .iter()
            .map(|c| HashedBackupCode {
                hash: BackupCodeGenerator::hash_code(c),
                used: false,
                used_at: None,
            })
            .collect();

        // Valid code
        assert!(BackupCodeGenerator::verify_and_consume(&mut hashed, &codes[0]));

        // Code can't be reused
        assert!(!BackupCodeGenerator::verify_and_consume(&mut hashed, &codes[0]));

        // Invalid code
        assert!(!BackupCodeGenerator::verify_and_consume(&mut hashed, "INVALID"));
    }

    #[tokio::test]
    async fn test_mfa_enrollment_flow() {
        let storage = Arc::new(InMemoryMfaStorage::new());
        let events = Arc::new(NoOpEventEmitter);
        let config = MfaConfig::default();
        let manager = MfaManager::new(storage, events, config);

        let user_id = UserId::new();

        // Start enrollment
        let setup = manager.start_enrollment(user_id, "test@example.com").await.unwrap();
        assert!(!setup.secret.is_empty());
        assert!(!setup.backup_codes.is_empty());

        // Generate valid code
        let code = manager.totp_provider.generate_code(&setup.secret).unwrap();

        // Complete enrollment
        manager.complete_enrollment(user_id, &code).await.unwrap();

        // Verify MFA is required
        assert!(manager.is_required(user_id).await);
    }

    #[tokio::test]
    async fn test_mfa_challenge_verification() {
        let storage = Arc::new(InMemoryMfaStorage::new());
        let events = Arc::new(NoOpEventEmitter);
        let config = MfaConfig::default();
        let manager = MfaManager::new(storage.clone(), events, config);

        let user_id = UserId::new();

        // Setup MFA
        let setup = manager.start_enrollment(user_id, "test@example.com").await.unwrap();
        let code = manager.totp_provider.generate_code(&setup.secret).unwrap();
        manager.complete_enrollment(user_id, &code).await.unwrap();

        // Create challenge
        let challenge = manager.create_challenge(user_id).await.unwrap();

        // Verify with valid code
        let new_code = manager.totp_provider.generate_code(&setup.secret).unwrap();
        let result = manager.verify(&challenge.id, &new_code, MfaCodeType::Totp).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), user_id);
    }

    #[test]
    fn test_constant_time_eq() {
        assert!(constant_time_eq(b"hello", b"hello"));
        assert!(!constant_time_eq(b"hello", b"world"));
        assert!(!constant_time_eq(b"hello", b"hell"));
    }

    struct NoOpEventEmitter;
    #[async_trait]
    impl AuthEventEmitter for NoOpEventEmitter {
        async fn emit(&self, _: AuthEvent) {}
    }
}
```

---

## Related Specs

- **Spec 366**: Auth Types - Uses MfaType enum
- **Spec 367**: Auth Configuration - Uses MfaConfig
- **Spec 368**: Local Auth - Integrates MFA with login
- **Spec 381**: Audit Logging - Logs MFA events
- **Spec 384**: Auth Events - Emits MFA events
