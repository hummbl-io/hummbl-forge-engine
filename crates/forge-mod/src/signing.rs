//! Mod signing and verification (BLUE TEAM hardening).
//!
//! Addresses RED-TEAM-FINDING-2: unsigned mods are accepted.
//!
//! Mods can be signed with a GPG key. The signature is stored in a
//! `.sig` file alongside the mod. When a mod is installed, the
//! signature is verified against a set of trusted keys.

use std::path::Path;
use std::process::Command;

/// A trusted public key for mod verification.
#[derive(Debug, Clone)]
pub struct TrustedKey {
    /// Key ID (e.g. "ABC12345" or full fingerprint)
    pub key_id: String,
    /// Human-readable description
    pub description: String,
}

/// Mod signature verification result.
#[derive(Debug)]
pub enum VerificationResult {
    /// Signature is valid and matches a trusted key
    Valid { key_id: String },
    /// Mod is signed but the key is not in the trusted set
    UntrustedKey { key_id: String },
    /// Mod is not signed
    Unsigned,
    /// Signature is present but invalid (tampered or corrupted)
    Invalid { reason: String },
    /// Verification error
    Error { reason: String },
}

/// Verify a mod's signature.
///
/// Checks for a `.sig` file alongside the mod and verifies it
/// against the trusted keys.
pub fn verify_mod_signature(mod_path: &Path, trusted_keys: &[TrustedKey]) -> VerificationResult {
    // Look for signature file
    let sig_path = mod_path.with_extension("sig");
    let alt_sig_path = std::path::PathBuf::from(format!("{}.sig", mod_path.to_string_lossy()));
    let sig_path = if !sig_path.exists() {
        if !alt_sig_path.exists() {
            return VerificationResult::Unsigned;
        }
        alt_sig_path
    } else {
        sig_path
    };

    // Use gpg to verify the signature
    let output = Command::new("gpg")
        .arg("--verify")
        .arg(&sig_path)
        .arg(mod_path)
        .output();

    let output = match output {
        Ok(o) => o,
        Err(e) => {
            return VerificationResult::Error {
                reason: format!("failed to run gpg: {e}"),
            }
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return VerificationResult::Invalid {
            reason: stderr.to_string(),
        };
    }

    // Extract the key ID from gpg output
    let stderr = String::from_utf8_lossy(&output.stderr);
    let key_id = extract_key_id(&stderr);

    match key_id {
        Some(id) => {
            // Check if the key is trusted
            let trusted = trusted_keys.iter().any(|k| k.key_id == id);
            if trusted {
                VerificationResult::Valid { key_id: id }
            } else {
                VerificationResult::UntrustedKey { key_id: id }
            }
        }
        None => VerificationResult::Error {
            reason: "could not extract key ID from gpg output".to_string(),
        },
    }
}

/// Sign a mod with the given GPG key.
pub fn sign_mod(mod_path: &Path, key_id: &str) -> anyhow::Result<std::path::PathBuf> {
    let sig_path = std::path::PathBuf::from(format!("{}.sig", mod_path.to_string_lossy()));

    let output = Command::new("gpg")
        .arg("--detach-sign")
        .arg("--local-user")
        .arg(key_id)
        .arg("--output")
        .arg(&sig_path)
        .arg(mod_path)
        .output()
        .map_err(|e| anyhow::anyhow!("failed to run gpg: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("gpg signing failed: {stderr}");
    }

    Ok(sig_path)
}

/// Extract the key ID from gpg --verify output.
fn extract_key_id(gpg_output: &str) -> Option<String> {
    // gpg output typically contains: "Good signature from key 0xABC12345"
    // or "Signature made ... using RSA key ABC12345"
    for line in gpg_output.lines() {
        if line.contains("using") && line.contains("key") {
            // Parse "Signature made ... using RSA key ABC12345"
            if let Some(pos) = line.rfind("key ") {
                let rest = &line[pos + 4..];
                let key = rest.split_whitespace().next().unwrap_or("");
                if !key.is_empty() {
                    return Some(key.to_string());
                }
            }
        }
    }
    None
}

/// Default trusted keys (empty — must be configured by the user).
pub fn default_trusted_keys() -> Vec<TrustedKey> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unsigned_mod() {
        let tmp = std::env::temp_dir().join("forge-sign-test-unsigned");
        std::fs::write(&tmp, "test mod content").ok();
        let result = verify_mod_signature(&tmp, &[]);
        assert!(matches!(result, VerificationResult::Unsigned));
        std::fs::remove_file(&tmp).ok();
    }

    #[test]
    fn test_trusted_key_construction() {
        let key = TrustedKey {
            key_id: "ABC12345".to_string(),
            description: "Test key".to_string(),
        };
        assert_eq!(key.key_id, "ABC12345");
    }

    #[test]
    fn test_extract_key_id() {
        let output = "Signature made Mon 01 Jan 2024 using RSA key ABC12345\nGood signature";
        let id = extract_key_id(output);
        assert_eq!(id, Some("ABC12345".to_string()));
    }
}
