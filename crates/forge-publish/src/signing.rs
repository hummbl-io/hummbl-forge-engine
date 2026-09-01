//! Binary signing utilities.
//!
//! Provides cross-platform code signing for release binaries:
//! - Linux: GPG signature
//! - macOS: codesign + notarization
//! - Windows: signtool (authenticode)

use std::path::Path;
use std::process::Command;

/// Sign a binary for the current platform.
pub fn sign_binary(binary_path: &Path, config: &SigningConfig) -> anyhow::Result<SigningResult> {
    match config {
        SigningConfig::Gpg { key_id } => sign_gpg(binary_path, key_id),
        SigningConfig::MacOS {
            identity,
            notarize,
            apple_id,
            team_id,
        } => sign_macos(binary_path, identity, *notarize, apple_id, team_id),
        SigningConfig::Windows {
            cert_path,
            cert_password,
            timestamp_url,
        } => sign_windows(binary_path, cert_path, cert_password, timestamp_url),
    }
}

/// Signing configuration.
#[derive(Debug, Clone)]
pub enum SigningConfig {
    /// GPG signature (Linux)
    Gpg { key_id: String },
    /// macOS codesign + notarization
    MacOS {
        identity: String,
        notarize: bool,
        apple_id: Option<String>,
        team_id: Option<String>,
    },
    /// Windows Authenticode
    Windows {
        cert_path: String,
        cert_password: String,
        timestamp_url: String,
    },
}

/// Result of signing.
#[derive(Debug)]
pub struct SigningResult {
    pub method: String,
    pub signature_path: Option<std::path::PathBuf>,
    pub signed: bool,
}

/// Sign a binary with GPG.
fn sign_gpg(binary_path: &Path, key_id: &str) -> anyhow::Result<SigningResult> {
    let sig_path = binary_path.with_extension("sig");

    let output = Command::new("gpg")
        .arg("--detach-sign")
        .arg("--local-user")
        .arg(key_id)
        .arg("--output")
        .arg(&sig_path)
        .arg(binary_path)
        .output()
        .map_err(|e| anyhow::anyhow!("failed to run gpg: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("GPG signing failed: {stderr}");
    }

    tracing::info!(
        "GPG signed: {} → {}",
        binary_path.display(),
        sig_path.display()
    );

    Ok(SigningResult {
        method: "gpg".to_string(),
        signature_path: Some(sig_path),
        signed: true,
    })
}

/// Sign a binary on macOS with codesign.
fn sign_macos(
    binary_path: &Path,
    identity: &str,
    notarize: bool,
    apple_id: &Option<String>,
    team_id: &Option<String>,
) -> anyhow::Result<SigningResult> {
    // Codesign
    let output = Command::new("codesign")
        .arg("--sign")
        .arg(identity)
        .arg("--force")
        .arg(binary_path)
        .output()
        .map_err(|e| anyhow::anyhow!("failed to run codesign: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("codesign failed: {stderr}");
    }

    tracing::info!(
        "macOS codesigned: {} (identity: {})",
        binary_path.display(),
        identity
    );

    // Notarize if requested
    if notarize {
        if let (Some(apple_id), Some(team_id)) = (apple_id, team_id) {
            tracing::info!("Notarizing with Apple (this may take several minutes)...");

            let output = Command::new("xcrun")
                .arg("notarytool")
                .arg("submit")
                .arg(binary_path)
                .arg("--apple-id")
                .arg(apple_id)
                .arg("--team-id")
                .arg(team_id)
                .arg("--wait")
                .output()
                .map_err(|e| anyhow::anyhow!("failed to run notarytool: {e}"))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                tracing::warn!("Notarization failed: {stderr}");
            } else {
                tracing::info!("Notarization complete");
            }
        } else {
            tracing::warn!("Notarization requested but apple_id or team_id not provided");
        }
    }

    Ok(SigningResult {
        method: "macos-codesign".to_string(),
        signature_path: None,
        signed: true,
    })
}

/// Sign a binary on Windows with signtool.
fn sign_windows(
    binary_path: &Path,
    cert_path: &str,
    cert_password: &str,
    timestamp_url: &str,
) -> anyhow::Result<SigningResult> {
    let output = Command::new("signtool")
        .arg("sign")
        .arg("/f")
        .arg(cert_path)
        .arg("/p")
        .arg(cert_password)
        .arg("/t")
        .arg(timestamp_url)
        .arg(binary_path)
        .output()
        .map_err(|e| anyhow::anyhow!("failed to run signtool: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("signtool failed: {stderr}");
    }

    tracing::info!("Windows signed: {}", binary_path.display());

    Ok(SigningResult {
        method: "windows-authenticode".to_string(),
        signature_path: None,
        signed: true,
    })
}

/// Verify a GPG signature.
pub fn verify_gpg_signature(binary_path: &Path, sig_path: &Path) -> anyhow::Result<bool> {
    let output = Command::new("gpg")
        .arg("--verify")
        .arg(sig_path)
        .arg(binary_path)
        .output()
        .map_err(|e| anyhow::anyhow!("failed to run gpg: {e}"))?;

    Ok(output.status.success())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signing_config_variants() {
        let gpg = SigningConfig::Gpg {
            key_id: "ABC123".to_string(),
        };
        match &gpg {
            SigningConfig::Gpg { key_id } => assert_eq!(key_id, "ABC123"),
            _ => panic!("wrong variant"),
        }

        let macos = SigningConfig::MacOS {
            identity: "Developer ID".to_string(),
            notarize: true,
            apple_id: Some("dev@example.com".to_string()),
            team_id: Some("ABC123".to_string()),
        };
        match &macos {
            SigningConfig::MacOS { notarize, .. } => assert!(*notarize),
            _ => panic!("wrong variant"),
        }

        let windows = SigningConfig::Windows {
            cert_path: "cert.pfx".to_string(),
            cert_password: "pass".to_string(),
            timestamp_url: "http://timestamp.digicert.com".to_string(),
        };
        match &windows {
            SigningConfig::Windows { cert_path, .. } => assert_eq!(cert_path, "cert.pfx"),
            _ => panic!("wrong variant"),
        }
    }
}
