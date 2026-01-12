# Spec 173: Rust Native Modules

## Phase
8 - Electron Shell

## Spec ID
173

## Status
Planned

## Dependencies
- Spec 161 (Electron Main Process)
- Spec 174 (NAPI-RS Setup)

## Estimated Context
~11%

---

## Objective

Implement high-performance native modules in Rust using NAPI-RS for computationally intensive operations. These modules will be integrated into the Electron main process for tasks like file hashing, encryption, compression, and other CPU-bound operations.

---

## Acceptance Criteria

- [x] Rust native module compiles for all target platforms
- [x] Module integrates with Electron main process
- [x] Type-safe bindings between Rust and JavaScript
- [x] Async operations don't block the event loop
- [x] Error handling propagates correctly to JavaScript
- [x] Memory management is handled properly
- [x] Module supports cross-compilation
- [x] Performance benchmarks meet requirements
- [x] Graceful fallback to JS implementations

---

## Implementation Details

### Cargo.toml Configuration

```toml
# native/Cargo.toml
[package]
name = "tachikoma-native"
version = "0.1.0"
edition = "2021"
license = "MIT"
authors = ["Tachikoma Team"]

[lib]
crate-type = ["cdylib"]

[dependencies]
napi = { version = "2.14", features = ["async", "napi8", "serde-json"] }
napi-derive = "2.14"
tokio = { version = "1.34", features = ["rt-multi-thread", "sync", "fs", "io-util"] }
sha2 = "0.10"
sha3 = "0.10"
blake3 = "1.5"
aes-gcm = "0.10"
argon2 = "0.5"
rand = "0.8"
base64 = "0.21"
flate2 = "1.0"
zstd = "0.13"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
rayon = "1.8"

[build-dependencies]
napi-build = "2.1"

[profile.release]
lto = true
codegen-units = 1
opt-level = 3

[features]
default = []
```

### Build Script

```rust
// native/build.rs
extern crate napi_build;

fn main() {
    napi_build::setup();
}
```

### Main Library File

```rust
// native/src/lib.rs
#![deny(clippy::all)]

use napi::bindgen_prelude::*;
use napi_derive::napi;

mod crypto;
mod compression;
mod hash;
mod fs;
mod search;

// Re-export modules
pub use crypto::*;
pub use compression::*;
pub use hash::*;
pub use fs::*;
pub use search::*;

/// Initialize the native module
#[napi]
pub fn init() -> Result<()> {
    // Initialize thread pool for parallel operations
    rayon::ThreadPoolBuilder::new()
        .num_threads(num_cpus::get())
        .build_global()
        .map_err(|e| Error::from_reason(e.to_string()))?;

    Ok(())
}

/// Get version information
#[napi]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
```

### Hash Module

```rust
// native/src/hash.rs
use napi::bindgen_prelude::*;
use napi_derive::napi;
use sha2::{Sha256, Sha512, Digest};
use sha3::{Sha3_256, Sha3_512};
use blake3::Hasher as Blake3Hasher;
use std::fs::File;
use std::io::{Read, BufReader};
use std::path::Path;

#[napi]
pub enum HashAlgorithm {
    Sha256,
    Sha512,
    Sha3_256,
    Sha3_512,
    Blake3,
}

/// Hash a string
#[napi]
pub fn hash_string(data: String, algorithm: HashAlgorithm) -> Result<String> {
    let bytes = data.as_bytes();
    hash_bytes(bytes, algorithm)
}

/// Hash a buffer
#[napi]
pub fn hash_buffer(data: Buffer, algorithm: HashAlgorithm) -> Result<String> {
    hash_bytes(data.as_ref(), algorithm)
}

fn hash_bytes(data: &[u8], algorithm: HashAlgorithm) -> Result<String> {
    let hash = match algorithm {
        HashAlgorithm::Sha256 => {
            let mut hasher = Sha256::new();
            hasher.update(data);
            format!("{:x}", hasher.finalize())
        }
        HashAlgorithm::Sha512 => {
            let mut hasher = Sha512::new();
            hasher.update(data);
            format!("{:x}", hasher.finalize())
        }
        HashAlgorithm::Sha3_256 => {
            let mut hasher = Sha3_256::new();
            hasher.update(data);
            format!("{:x}", hasher.finalize())
        }
        HashAlgorithm::Sha3_512 => {
            let mut hasher = Sha3_512::new();
            hasher.update(data);
            format!("{:x}", hasher.finalize())
        }
        HashAlgorithm::Blake3 => {
            let mut hasher = Blake3Hasher::new();
            hasher.update(data);
            hasher.finalize().to_hex().to_string()
        }
    };

    Ok(hash)
}

/// Hash a file asynchronously
#[napi]
pub async fn hash_file(path: String, algorithm: HashAlgorithm) -> Result<String> {
    let path = Path::new(&path);

    tokio::task::spawn_blocking(move || {
        let file = File::open(path).map_err(|e| Error::from_reason(e.to_string()))?;
        let mut reader = BufReader::new(file);
        let mut buffer = vec![0u8; 8192];

        match algorithm {
            HashAlgorithm::Sha256 => {
                let mut hasher = Sha256::new();
                loop {
                    let count = reader.read(&mut buffer).map_err(|e| Error::from_reason(e.to_string()))?;
                    if count == 0 {
                        break;
                    }
                    hasher.update(&buffer[..count]);
                }
                Ok(format!("{:x}", hasher.finalize()))
            }
            HashAlgorithm::Blake3 => {
                let mut hasher = Blake3Hasher::new();
                loop {
                    let count = reader.read(&mut buffer).map_err(|e| Error::from_reason(e.to_string()))?;
                    if count == 0 {
                        break;
                    }
                    hasher.update(&buffer[..count]);
                }
                Ok(hasher.finalize().to_hex().to_string())
            }
            _ => {
                // Implement other algorithms similarly
                Err(Error::from_reason("Algorithm not implemented for file hashing"))
            }
        }
    })
    .await
    .map_err(|e| Error::from_reason(e.to_string()))?
}

/// Hash multiple files in parallel
#[napi]
pub async fn hash_files(paths: Vec<String>, algorithm: HashAlgorithm) -> Result<Vec<FileHash>> {
    use rayon::prelude::*;

    let results: Vec<_> = paths
        .par_iter()
        .map(|path| {
            let file = File::open(path)?;
            let mut reader = BufReader::new(file);
            let mut hasher = Blake3Hasher::new();
            let mut buffer = vec![0u8; 8192];

            loop {
                let count = reader.read(&mut buffer)?;
                if count == 0 {
                    break;
                }
                hasher.update(&buffer[..count]);
            }

            Ok::<_, std::io::Error>((path.clone(), hasher.finalize().to_hex().to_string()))
        })
        .collect();

    results
        .into_iter()
        .map(|r| {
            r.map(|(path, hash)| FileHash { path, hash })
                .map_err(|e| Error::from_reason(e.to_string()))
        })
        .collect()
}

#[napi(object)]
pub struct FileHash {
    pub path: String,
    pub hash: String,
}
```

### Crypto Module

```rust
// native/src/crypto.rs
use napi::bindgen_prelude::*;
use napi_derive::napi;
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rand::RngCore;

#[napi(object)]
pub struct EncryptedData {
    pub ciphertext: Buffer,
    pub nonce: Buffer,
    pub tag: Buffer,
}

#[napi(object)]
pub struct KeyDerivationResult {
    pub hash: String,
    pub salt: String,
}

/// Encrypt data using AES-256-GCM
#[napi]
pub fn encrypt(data: Buffer, key: Buffer) -> Result<EncryptedData> {
    if key.len() != 32 {
        return Err(Error::from_reason("Key must be 32 bytes for AES-256"));
    }

    let cipher = Aes256Gcm::new_from_slice(key.as_ref())
        .map_err(|e| Error::from_reason(e.to_string()))?;

    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, data.as_ref())
        .map_err(|e| Error::from_reason(e.to_string()))?;

    // AES-GCM appends the tag to the ciphertext
    let tag_start = ciphertext.len() - 16;
    let (ct, tag) = ciphertext.split_at(tag_start);

    Ok(EncryptedData {
        ciphertext: Buffer::from(ct.to_vec()),
        nonce: Buffer::from(nonce_bytes.to_vec()),
        tag: Buffer::from(tag.to_vec()),
    })
}

/// Decrypt data using AES-256-GCM
#[napi]
pub fn decrypt(encrypted: EncryptedData, key: Buffer) -> Result<Buffer> {
    if key.len() != 32 {
        return Err(Error::from_reason("Key must be 32 bytes for AES-256"));
    }

    let cipher = Aes256Gcm::new_from_slice(key.as_ref())
        .map_err(|e| Error::from_reason(e.to_string()))?;

    let nonce = Nonce::from_slice(encrypted.nonce.as_ref());

    // Reconstruct ciphertext with tag
    let mut ciphertext_with_tag = encrypted.ciphertext.to_vec();
    ciphertext_with_tag.extend_from_slice(encrypted.tag.as_ref());

    let plaintext = cipher
        .decrypt(nonce, ciphertext_with_tag.as_ref())
        .map_err(|_| Error::from_reason("Decryption failed"))?;

    Ok(Buffer::from(plaintext))
}

/// Generate a random encryption key
#[napi]
pub fn generate_key(length: u32) -> Buffer {
    let mut key = vec![0u8; length as usize];
    OsRng.fill_bytes(&mut key);
    Buffer::from(key)
}

/// Hash a password using Argon2id
#[napi]
pub async fn hash_password(password: String) -> Result<KeyDerivationResult> {
    tokio::task::spawn_blocking(move || {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| Error::from_reason(e.to_string()))?
            .to_string();

        Ok(KeyDerivationResult {
            hash: password_hash,
            salt: salt.to_string(),
        })
    })
    .await
    .map_err(|e| Error::from_reason(e.to_string()))?
}

/// Verify a password against a hash
#[napi]
pub async fn verify_password(password: String, hash: String) -> Result<bool> {
    tokio::task::spawn_blocking(move || {
        let parsed_hash = PasswordHash::new(&hash)
            .map_err(|e| Error::from_reason(e.to_string()))?;

        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    })
    .await
    .map_err(|e| Error::from_reason(e.to_string()))?
}
```

### Compression Module

```rust
// native/src/compression.rs
use napi::bindgen_prelude::*;
use napi_derive::napi;
use flate2::read::{GzDecoder, GzEncoder};
use flate2::Compression;
use std::io::{Read, Write};

#[napi]
pub enum CompressionAlgorithm {
    Gzip,
    Zstd,
}

#[napi]
pub enum CompressionLevel {
    Fast,
    Default,
    Best,
}

/// Compress data
#[napi]
pub fn compress(
    data: Buffer,
    algorithm: CompressionAlgorithm,
    level: Option<CompressionLevel>,
) -> Result<Buffer> {
    let level = level.unwrap_or(CompressionLevel::Default);

    match algorithm {
        CompressionAlgorithm::Gzip => {
            let compression = match level {
                CompressionLevel::Fast => Compression::fast(),
                CompressionLevel::Default => Compression::default(),
                CompressionLevel::Best => Compression::best(),
            };

            let mut encoder = GzEncoder::new(data.as_ref(), compression);
            let mut compressed = Vec::new();
            encoder
                .read_to_end(&mut compressed)
                .map_err(|e| Error::from_reason(e.to_string()))?;

            Ok(Buffer::from(compressed))
        }
        CompressionAlgorithm::Zstd => {
            let level = match level {
                CompressionLevel::Fast => 1,
                CompressionLevel::Default => 3,
                CompressionLevel::Best => 19,
            };

            let compressed = zstd::encode_all(data.as_ref(), level)
                .map_err(|e| Error::from_reason(e.to_string()))?;

            Ok(Buffer::from(compressed))
        }
    }
}

/// Decompress data
#[napi]
pub fn decompress(data: Buffer, algorithm: CompressionAlgorithm) -> Result<Buffer> {
    match algorithm {
        CompressionAlgorithm::Gzip => {
            let mut decoder = GzDecoder::new(data.as_ref());
            let mut decompressed = Vec::new();
            decoder
                .read_to_end(&mut decompressed)
                .map_err(|e| Error::from_reason(e.to_string()))?;

            Ok(Buffer::from(decompressed))
        }
        CompressionAlgorithm::Zstd => {
            let decompressed = zstd::decode_all(data.as_ref())
                .map_err(|e| Error::from_reason(e.to_string()))?;

            Ok(Buffer::from(decompressed))
        }
    }
}

/// Compress a file asynchronously
#[napi]
pub async fn compress_file(
    input_path: String,
    output_path: String,
    algorithm: CompressionAlgorithm,
) -> Result<()> {
    tokio::task::spawn_blocking(move || {
        let input = std::fs::read(&input_path)
            .map_err(|e| Error::from_reason(e.to_string()))?;

        let compressed = match algorithm {
            CompressionAlgorithm::Gzip => {
                let mut encoder = GzEncoder::new(input.as_slice(), Compression::default());
                let mut compressed = Vec::new();
                encoder.read_to_end(&mut compressed)
                    .map_err(|e| Error::from_reason(e.to_string()))?;
                compressed
            }
            CompressionAlgorithm::Zstd => {
                zstd::encode_all(input.as_slice(), 3)
                    .map_err(|e| Error::from_reason(e.to_string()))?
            }
        };

        std::fs::write(&output_path, compressed)
            .map_err(|e| Error::from_reason(e.to_string()))?;

        Ok(())
    })
    .await
    .map_err(|e| Error::from_reason(e.to_string()))?
}
```

### TypeScript Bindings

```typescript
// src/electron/main/native/index.ts
import { join } from 'path';
import { app } from 'electron';

// Load native module
let native: NativeModule | null = null;

interface NativeModule {
  init(): void;
  getVersion(): string;

  // Hash functions
  hashString(data: string, algorithm: HashAlgorithm): string;
  hashBuffer(data: Buffer, algorithm: HashAlgorithm): string;
  hashFile(path: string, algorithm: HashAlgorithm): Promise<string>;
  hashFiles(paths: string[], algorithm: HashAlgorithm): Promise<FileHash[]>;

  // Crypto functions
  encrypt(data: Buffer, key: Buffer): EncryptedData;
  decrypt(encrypted: EncryptedData, key: Buffer): Buffer;
  generateKey(length: number): Buffer;
  hashPassword(password: string): Promise<KeyDerivationResult>;
  verifyPassword(password: string, hash: string): Promise<boolean>;

  // Compression functions
  compress(data: Buffer, algorithm: CompressionAlgorithm, level?: CompressionLevel): Buffer;
  decompress(data: Buffer, algorithm: CompressionAlgorithm): Buffer;
  compressFile(inputPath: string, outputPath: string, algorithm: CompressionAlgorithm): Promise<void>;
}

enum HashAlgorithm {
  Sha256 = 'Sha256',
  Sha512 = 'Sha512',
  Sha3_256 = 'Sha3_256',
  Sha3_512 = 'Sha3_512',
  Blake3 = 'Blake3',
}

enum CompressionAlgorithm {
  Gzip = 'Gzip',
  Zstd = 'Zstd',
}

enum CompressionLevel {
  Fast = 'Fast',
  Default = 'Default',
  Best = 'Best',
}

interface FileHash {
  path: string;
  hash: string;
}

interface EncryptedData {
  ciphertext: Buffer;
  nonce: Buffer;
  tag: Buffer;
}

interface KeyDerivationResult {
  hash: string;
  salt: string;
}

function loadNativeModule(): NativeModule | null {
  try {
    const modulePath = app.isPackaged
      ? join(process.resourcesPath, 'native', `tachikoma-native.${process.platform}.node`)
      : join(__dirname, '../../../../native/index.node');

    const module = require(modulePath);
    module.init();
    return module;
  } catch (error) {
    console.error('Failed to load native module:', error);
    return null;
  }
}

export function getNative(): NativeModule | null {
  if (!native) {
    native = loadNativeModule();
  }
  return native;
}

export function isNativeAvailable(): boolean {
  return getNative() !== null;
}

// Export types
export {
  HashAlgorithm,
  CompressionAlgorithm,
  CompressionLevel,
  FileHash,
  EncryptedData,
  KeyDerivationResult,
  NativeModule,
};
```

---

## Testing Requirements

### Rust Unit Tests

```rust
// native/src/tests.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_string() {
        let result = hash_string("hello".to_string(), HashAlgorithm::Sha256).unwrap();
        assert_eq!(
            result,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn test_encrypt_decrypt() {
        let key = generate_key(32);
        let data = Buffer::from("secret message".as_bytes().to_vec());

        let encrypted = encrypt(data.clone(), key.clone()).unwrap();
        let decrypted = decrypt(encrypted, key).unwrap();

        assert_eq!(data.as_ref(), decrypted.as_ref());
    }

    #[test]
    fn test_compress_decompress() {
        let data = Buffer::from("hello world".repeat(100).as_bytes().to_vec());

        let compressed = compress(data.clone(), CompressionAlgorithm::Gzip, None).unwrap();
        let decompressed = decompress(compressed, CompressionAlgorithm::Gzip).unwrap();

        assert_eq!(data.as_ref(), decompressed.as_ref());
    }

    #[tokio::test]
    async fn test_hash_password() {
        let result = hash_password("mypassword".to_string()).await.unwrap();
        let verified = verify_password("mypassword".to_string(), result.hash).await.unwrap();

        assert!(verified);
    }
}
```

### TypeScript Integration Tests

```typescript
// src/electron/main/native/__tests__/native.test.ts
import { describe, it, expect, beforeAll } from 'vitest';
import { getNative, isNativeAvailable, HashAlgorithm } from '../index';

describe('Native Module', () => {
  beforeAll(() => {
    // Skip tests if native module is not available
    if (!isNativeAvailable()) {
      console.warn('Native module not available, skipping tests');
    }
  });

  it('should load native module', () => {
    const native = getNative();
    expect(native).not.toBeNull();
  });

  it('should hash strings', () => {
    const native = getNative();
    if (!native) return;

    const hash = native.hashString('hello', HashAlgorithm.Sha256);
    expect(hash).toBe(
      '2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824'
    );
  });

  it('should encrypt and decrypt data', () => {
    const native = getNative();
    if (!native) return;

    const key = native.generateKey(32);
    const data = Buffer.from('secret message');

    const encrypted = native.encrypt(data, key);
    const decrypted = native.decrypt(encrypted, key);

    expect(decrypted.toString()).toBe('secret message');
  });

  it('should compress and decompress data', () => {
    const native = getNative();
    if (!native) return;

    const data = Buffer.from('hello world'.repeat(100));
    const compressed = native.compress(data, 'Gzip');
    const decompressed = native.decompress(compressed, 'Gzip');

    expect(decompressed.toString()).toBe(data.toString());
    expect(compressed.length).toBeLessThan(data.length);
  });
});
```

---

## Related Specs

- Spec 161: Electron Main Process
- Spec 174: NAPI-RS Setup
- Spec 175: Build Configuration
