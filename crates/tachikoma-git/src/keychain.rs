//! Platform keychain integration.

use crate::{GitCredential, GitResult, GitError};

/// Keychain service name.
const SERVICE_NAME: &str = "tachikoma-git";

/// Store credential in system keychain.
#[cfg(target_os = "macos")]
pub fn store_in_keychain(account: &str, credential: &GitCredential) -> GitResult<()> {
    use security_framework::passwords::set_generic_password;

    let password = match credential {
        GitCredential::UserPassword { password, .. } => password.clone(),
        GitCredential::Token { token, .. } => token.clone(),
        _ => return Err(GitError::InvalidOperation {
            message: "Only password/token credentials can be stored in keychain".to_string(),
        }),
    };

    set_generic_password(SERVICE_NAME, account, password.as_bytes())
        .map_err(|e| GitError::InvalidOperation {
            message: format!("Failed to store in keychain: {}", e),
        })?;

    Ok(())
}

/// Get credential from system keychain.
#[cfg(target_os = "macos")]
pub fn get_from_keychain(account: &str) -> GitResult<Option<String>> {
    use security_framework::passwords::get_generic_password;

    match get_generic_password(SERVICE_NAME, account) {
        Ok(password) => {
            let password = String::from_utf8(password)
                .map_err(|_| GitError::InvalidOperation {
                    message: "Invalid UTF-8 in stored password".to_string(),
                })?;
            Ok(Some(password))
        }
        Err(_) => Ok(None),
    }
}

/// Delete credential from system keychain.
#[cfg(target_os = "macos")]
pub fn delete_from_keychain(account: &str) -> GitResult<()> {
    use security_framework::passwords::delete_generic_password;

    let _ = delete_generic_password(SERVICE_NAME, account);
    Ok(())
}

/// Store credential in system keychain (Windows).
#[cfg(target_os = "windows")]
pub fn store_in_keychain(account: &str, credential: &GitCredential) -> GitResult<()> {
    use winapi::um::wincred::{CredWriteW, CREDENTIALW, CRED_PERSIST_LOCAL_MACHINE, CRED_TYPE_GENERIC};
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;

    let password = match credential {
        GitCredential::UserPassword { password, .. } => password.clone(),
        GitCredential::Token { token, .. } => token.clone(),
        _ => return Err(GitError::InvalidOperation {
            message: "Only password/token credentials can be stored in keychain".to_string(),
        }),
    };

    let target_name = format!("{}:{}", SERVICE_NAME, account);
    let target_name_wide: Vec<u16> = OsString::from(&target_name)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    
    let password_bytes = password.as_bytes();
    
    let mut credential = CREDENTIALW {
        Flags: 0,
        Type: CRED_TYPE_GENERIC,
        TargetName: target_name_wide.as_ptr() as *mut u16,
        Comment: std::ptr::null_mut(),
        LastWritten: unsafe { std::mem::zeroed() },
        CredentialBlobSize: password_bytes.len() as u32,
        CredentialBlob: password_bytes.as_ptr() as *mut u8,
        Persist: CRED_PERSIST_LOCAL_MACHINE,
        AttributeCount: 0,
        Attributes: std::ptr::null_mut(),
        TargetAlias: std::ptr::null_mut(),
        UserName: std::ptr::null_mut(),
    };

    let result = unsafe { CredWriteW(&mut credential, 0) };
    if result == 0 {
        return Err(GitError::InvalidOperation {
            message: "Failed to store credential in Windows credential store".to_string(),
        });
    }

    Ok(())
}

/// Get credential from system keychain (Windows).
#[cfg(target_os = "windows")]
pub fn get_from_keychain(account: &str) -> GitResult<Option<String>> {
    use winapi::um::wincred::{CredReadW, CredFree, PCREDENTIALW, CRED_TYPE_GENERIC};
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;

    let target_name = format!("{}:{}", SERVICE_NAME, account);
    let target_name_wide: Vec<u16> = OsString::from(&target_name)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let mut credential: PCREDENTIALW = std::ptr::null_mut();
    let result = unsafe {
        CredReadW(
            target_name_wide.as_ptr(),
            CRED_TYPE_GENERIC,
            0,
            &mut credential,
        )
    };

    if result == 0 {
        return Ok(None);
    }

    let password = unsafe {
        let blob_ptr = (*credential).CredentialBlob;
        let blob_size = (*credential).CredentialBlobSize as usize;
        let slice = std::slice::from_raw_parts(blob_ptr, blob_size);
        String::from_utf8(slice.to_vec())
            .map_err(|_| GitError::InvalidOperation {
                message: "Invalid UTF-8 in stored password".to_string(),
            })?
    };

    unsafe { CredFree(credential as *mut _) };
    Ok(Some(password))
}

/// Delete credential from system keychain (Windows).
#[cfg(target_os = "windows")]
pub fn delete_from_keychain(account: &str) -> GitResult<()> {
    use winapi::um::wincred::{CredDeleteW, CRED_TYPE_GENERIC};
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;

    let target_name = format!("{}:{}", SERVICE_NAME, account);
    let target_name_wide: Vec<u16> = OsString::from(&target_name)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let _ = unsafe {
        CredDeleteW(target_name_wide.as_ptr(), CRED_TYPE_GENERIC, 0)
    };

    Ok(())
}

/// Store credential in system keychain (Linux).
#[cfg(target_os = "linux")]
pub fn store_in_keychain(account: &str, credential: &GitCredential) -> GitResult<()> {
    // Try to use secret-tool if available
    use std::process::Command;

    let password = match credential {
        GitCredential::UserPassword { password, .. } => password.clone(),
        GitCredential::Token { token, .. } => token.clone(),
        _ => return Err(GitError::InvalidOperation {
            message: "Only password/token credentials can be stored in keychain".to_string(),
        }),
    };

    let output = Command::new("secret-tool")
        .args(&["store", "--label", &format!("Tachikoma Git: {}", account)])
        .args(&["service", SERVICE_NAME])
        .args(&["account", account])
        .arg("--password")
        .arg(&password)
        .output();

    match output {
        Ok(output) if output.status.success() => Ok(()),
        Ok(output) => Err(GitError::InvalidOperation {
            message: format!(
                "secret-tool failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ),
        }),
        Err(_) => Err(GitError::InvalidOperation {
            message: "secret-tool not available. Install libsecret-tools package.".to_string(),
        }),
    }
}

/// Get credential from system keychain (Linux).
#[cfg(target_os = "linux")]
pub fn get_from_keychain(account: &str) -> GitResult<Option<String>> {
    use std::process::Command;

    let output = Command::new("secret-tool")
        .args(&["lookup"])
        .args(&["service", SERVICE_NAME])
        .args(&["account", account])
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let password = String::from_utf8(output.stdout)
                .map_err(|_| GitError::InvalidOperation {
                    message: "Invalid UTF-8 in stored password".to_string(),
                })?
                .trim()
                .to_string();
            Ok(Some(password))
        }
        Ok(_) => Ok(None),
        Err(_) => Ok(None),
    }
}

/// Delete credential from system keychain (Linux).
#[cfg(target_os = "linux")]
pub fn delete_from_keychain(account: &str) -> GitResult<()> {
    use std::process::Command;

    let _ = Command::new("secret-tool")
        .args(&["clear"])
        .args(&["service", SERVICE_NAME])
        .args(&["account", account])
        .output();

    Ok(())
}

// Stubs for unsupported platforms
#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
pub fn store_in_keychain(_account: &str, _credential: &GitCredential) -> GitResult<()> {
    Err(GitError::InvalidOperation {
        message: "Keychain not supported on this platform".to_string(),
    })
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
pub fn get_from_keychain(_account: &str) -> GitResult<Option<String>> {
    Ok(None)
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
pub fn delete_from_keychain(_account: &str) -> GitResult<()> {
    Ok(())
}

/// Enhanced credential store with keychain integration.
pub struct KeychainCredentialStore {
    memory_store: crate::credentials::CredentialStore,
    use_keychain: bool,
}

impl KeychainCredentialStore {
    /// Create a new keychain credential store.
    pub fn new() -> Self {
        Self {
            memory_store: crate::credentials::CredentialStore::new(),
            use_keychain: true,
        }
    }

    /// Create store without keychain integration.
    pub fn memory_only() -> Self {
        Self {
            memory_store: crate::credentials::CredentialStore::new(),
            use_keychain: false,
        }
    }

    /// Store a credential.
    pub fn store(&mut self, pattern: impl Into<String>, credential: GitCredential) -> GitResult<()> {
        let pattern = pattern.into();
        
        // Store in memory
        self.memory_store.store(&pattern, credential.clone());
        
        // Also store in keychain if supported
        if self.use_keychain {
            if let Err(e) = store_in_keychain(&pattern, &credential) {
                tracing::warn!("Failed to store credential in keychain: {}", e);
            }
        }
        
        Ok(())
    }

    /// Get a credential.
    pub fn get(&self, url: &str) -> GitResult<Option<GitCredential>> {
        // Try memory first
        if let Some(cred) = self.memory_store.get(url) {
            return Ok(Some(cred.clone()));
        }

        // Try keychain
        if self.use_keychain {
            // Try direct match first
            if let Ok(Some(password)) = get_from_keychain(url) {
                return Ok(Some(GitCredential::token(password)));
            }

            // Try pattern matching
            for pattern in self.memory_store.patterns() {
                if crate::credentials::url_matches_pattern(url, &pattern) {
                    if let Ok(Some(password)) = get_from_keychain(&pattern) {
                        return Ok(Some(GitCredential::token(password)));
                    }
                }
            }
        }

        Ok(None)
    }

    /// Remove a credential.
    pub fn remove(&mut self, pattern: &str) -> GitResult<Option<GitCredential>> {
        let credential = self.memory_store.remove(pattern);
        
        if self.use_keychain {
            if let Err(e) = delete_from_keychain(pattern) {
                tracing::warn!("Failed to remove credential from keychain: {}", e);
            }
        }
        
        Ok(credential)
    }

    /// Clear all credentials.
    pub fn clear(&mut self) -> GitResult<()> {
        if self.use_keychain {
            for pattern in self.memory_store.patterns() {
                if let Err(e) = delete_from_keychain(&pattern) {
                    tracing::warn!("Failed to remove credential from keychain: {}", e);
                }
            }
        }
        
        self.memory_store.clear();
        Ok(())
    }
}

impl Default for KeychainCredentialStore {
    fn default() -> Self {
        Self::new()
    }
}