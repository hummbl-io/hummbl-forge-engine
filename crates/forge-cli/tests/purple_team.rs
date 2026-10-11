//! Purple team synthesis tests — verify BLUE team fixes against RED team findings.
//!
//! These tests confirm that the security controls implemented by the BLUE
//! team actually mitigate the vulnerabilities found by the RED team.

use std::path::PathBuf;
use std::process::Command;

fn forge_binary() -> PathBuf {
    std::path::Path::new(env!("CARGO_BIN_EXE_forge")).to_path_buf()
}

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("forge-purpleteam-{name}"));
    std::fs::remove_dir_all(&dir).ok();
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

// === PURPLE-1: Lockfile integrity hash works ===

#[test]
fn purple_lockfile_integrity_blocks_tampering() {
    // Verify that the BLUE team integrity hash fix actually works
    use forge_version::lockfile::{EnginePin, Lockfile};

    let mut lockfile = Lockfile {
        format_version: 1,
        project_id: "com.hummbl.test".to_string(),
        project_version: "0.1.0".to_string(),
        engine: EnginePin {
            kind: "forge2d".to_string(),
            version: "0.1.0".to_string(),
            adapter_version: "0.1.0".to_string(),
        },
        mods: vec![],
        assets: vec![],
        generated_at: "1234567890".to_string(),
        integrity_hash: None,
    };

    // Set the integrity hash
    lockfile.integrity_hash = Some(lockfile.compute_integrity_hash());

    // 1. Original lockfile passes verification
    assert!(
        lockfile.verify_integrity(),
        "original lockfile should pass integrity check"
    );

    // 2. Tampered version fails
    lockfile.project_version = "99.99.99".to_string();
    assert!(
        !lockfile.verify_integrity(),
        "tampered version should fail integrity check"
    );

    // 3. Restore and verify it passes again
    lockfile.project_version = "0.1.0".to_string();
    assert!(
        lockfile.verify_integrity(),
        "restored lockfile should pass integrity check"
    );

    // 4. Tamper engine kind
    lockfile.engine.kind = "ue5".to_string();
    assert!(
        !lockfile.verify_integrity(),
        "tampered engine kind should fail integrity check"
    );
}

// === PURPLE-2: Symlink rejection in asset store works ===

#[test]
fn purple_symlink_rejected_by_asset_store() {
    #[cfg(unix)]
    use forge_asset::AssetStore;

    let dir = temp_dir("symlink-rejected");
    #[cfg(unix)]
    let store_dir = dir.join("store");
    #[cfg(unix)]
    let store = AssetStore::new(&store_dir);

    let evil_target = dir.join("evil-target.txt");
    std::fs::write(&evil_target, "secret data").unwrap();

    #[cfg(unix)]
    let symlink_path = dir.join("evil-symlink.txt");

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&evil_target, &symlink_path).unwrap();

        // BLUE TEAM FIX: asset store should reject symlinks
        let result = store.add(&symlink_path);
        assert!(
            result.is_err(),
            "PURPLE: asset store should reject symlinks after BLUE team fix"
        );

        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("symlink"),
            "PURPLE: error message should mention symlink"
        );
    }

    std::fs::remove_dir_all(&dir).ok();
}

// === PURPLE-3: Mod signing verification infrastructure exists ===

#[test]
fn purple_mod_signing_infrastructure_exists() {
    use forge_mod::signing::{default_trusted_keys, verify_mod_signature, TrustedKey};

    // Verify the signing infrastructure is in place
    let keys = default_trusted_keys();
    assert!(keys.is_empty(), "default trusted keys should be empty");

    // Verify TrustedKey can be constructed
    let key = TrustedKey {
        key_id: "ABC12345".to_string(),
        description: "Test key".to_string(),
    };
    assert_eq!(key.key_id, "ABC12345");

    // Verify unsigned mod returns Unsigned
    let tmp = std::env::temp_dir().join("forge-purple-sign-test");
    std::fs::write(&tmp, "test mod content").ok();
    let result = verify_mod_signature(&tmp, &[]);
    assert!(
        matches!(result, forge_mod::VerificationResult::Unsigned),
        "PURPLE: unsigned mod should return Unsigned"
    );
    std::fs::remove_file(&tmp).ok();
}

// === PURPLE-4: Sandbox path traversal protection is robust ===

#[test]
fn purple_sandbox_path_traversal_robust() {
    use forge_mod::manifest::ModPermission;
    use forge_mod::sdk::{PermissionChecker, SandboxConfig};

    let sandbox_root = std::env::temp_dir().join("forge-purple-sandbox");
    std::fs::create_dir_all(&sandbox_root).ok();

    let checker = PermissionChecker::new(
        SandboxConfig {
            strict_mode: true,
            allowed_read_root: sandbox_root.clone(),
            allowed_write_root: sandbox_root.clone(),
            allow_network: false,
            allow_code_execution: false,
        },
        vec![ModPermission::ReadFiles, ModPermission::WriteFiles],
    );

    // Test multiple path traversal variants
    let traversal_attempts = vec![
        "../../../etc/passwd",
        "../../../../etc/shadow",
        "./.././../etc/hosts",
        "subdir/../../../etc/passwd",
    ];

    // Windows-style traversal only works on Windows
    #[cfg(windows)]
    let traversal_attempts = {
        let mut attempts = traversal_attempts;
        attempts.push("..\\..\\..\\windows\\system32");
        attempts
    };

    for attempt in traversal_attempts {
        let result = checker.check_read(std::path::Path::new(attempt));
        assert!(
            result.is_err(),
            "PURPLE: path traversal '{attempt}' should be blocked"
        );
    }

    std::fs::remove_dir_all(&sandbox_root).ok();
}

// === PURPLE-5: Command injection protection confirmed ===

#[test]
fn purple_command_injection_confirmed_safe() {
    let dir = temp_dir("cmd-injection-confirmed");
    let target = "/tmp/forge-purple-cmd-target";
    std::fs::write(target, "safe").ok();

    // Multiple injection attempts
    let attempts = vec![
        "test; rm -f /tmp/forge-purple-cmd-target",
        "test && rm -f /tmp/forge-purple-cmd-target",
        "test | rm -f /tmp/forge-purple-cmd-target",
        "test$(rm -f /tmp/forge-purple-cmd-target)",
        "test`rm -f /tmp/forge-purple-cmd-target`",
    ];

    for evil_name in &attempts {
        let _ = Command::new(forge_binary())
            .arg("init")
            .arg(evil_name)
            .current_dir(&dir)
            .output()
            .unwrap();

        assert!(
            std::path::Path::new(target).exists(),
            "PURPLE: command injection with '{evil_name}' should not execute"
        );
    }

    std::fs::remove_file(target).ok();
    std::fs::remove_dir_all(&dir).ok();
}

// === PURPLE-6: Permission sandbox is comprehensive ===

#[test]
fn purple_permission_sandbox_comprehensive() {
    use forge_mod::manifest::ModPermission;
    use forge_mod::sdk::{PermissionChecker, SandboxConfig};

    let sandbox_root = std::env::temp_dir().join("forge-purple-perm-sandbox");
    std::fs::create_dir_all(&sandbox_root).ok();

    // Test 1: Mod with only ReadFiles cannot write
    let read_only = PermissionChecker::new(
        SandboxConfig {
            strict_mode: true,
            allowed_read_root: sandbox_root.clone(),
            allowed_write_root: sandbox_root.clone(),
            allow_network: false,
            allow_code_execution: false,
        },
        vec![ModPermission::ReadFiles],
    );

    assert!(read_only
        .check_read(std::path::Path::new("test.txt"))
        .is_ok());
    assert!(read_only
        .check_write(std::path::Path::new("test.txt"))
        .is_err());
    assert!(read_only.check_network().is_err());
    assert!(read_only.check_execute().is_err());

    // Test 2: Mod with ReadFiles + WriteFiles but not network/code-exec
    let read_write = PermissionChecker::new(
        SandboxConfig {
            strict_mode: true,
            allowed_read_root: sandbox_root.clone(),
            allowed_write_root: sandbox_root.clone(),
            allow_network: false,
            allow_code_execution: false,
        },
        vec![ModPermission::ReadFiles, ModPermission::WriteFiles],
    );

    assert!(read_write
        .check_read(std::path::Path::new("test.txt"))
        .is_ok());
    assert!(read_write
        .check_write(std::path::Path::new("test.txt"))
        .is_ok());
    assert!(read_write.check_network().is_err());
    assert!(read_write.check_execute().is_err());

    std::fs::remove_dir_all(&sandbox_root).ok();
}

// === PURPLE-7: Circular dependency detection is robust ===

#[test]
fn purple_circular_dependency_detection_robust() {
    use forge_mod::manifest::{ModDependency, ModInfo, ModManifest, ModTarget};
    use forge_mod::sdk::DependencyResolver;

    let target = ModTarget {
        game: "test".to_string(),
        compatible_versions: "^1.0.0".to_string(),
    };

    // Three-way cycle: A → B → C → A
    let mod_a = ModManifest {
        mod_info: ModInfo {
            id: "mod-a".to_string(),
            name: "Mod A".to_string(),
            version: "1.0.0".to_string(),
            author: "test".to_string(),
            description: String::new(),
        },
        target: target.clone(),
        dependencies: vec![ModDependency {
            id: "mod-b".to_string(),
            version: "^1.0.0".to_string(),
            optional: false,
        }],
        hooks: vec![],
        permissions: vec![],
    };

    let mod_b = ModManifest {
        mod_info: ModInfo {
            id: "mod-b".to_string(),
            name: "Mod B".to_string(),
            version: "1.0.0".to_string(),
            author: "test".to_string(),
            description: String::new(),
        },
        target: target.clone(),
        dependencies: vec![ModDependency {
            id: "mod-c".to_string(),
            version: "^1.0.0".to_string(),
            optional: false,
        }],
        hooks: vec![],
        permissions: vec![],
    };

    let mod_c = ModManifest {
        mod_info: ModInfo {
            id: "mod-c".to_string(),
            name: "Mod C".to_string(),
            version: "1.0.0".to_string(),
            author: "test".to_string(),
            description: String::new(),
        },
        target: target.clone(),
        dependencies: vec![ModDependency {
            id: "mod-a".to_string(),
            version: "^1.0.0".to_string(),
            optional: false,
        }],
        hooks: vec![],
        permissions: vec![],
    };

    let resolver = DependencyResolver::new(vec![mod_a, mod_b, mod_c]);
    let result = resolver.resolve();
    assert!(
        result.is_err(),
        "PURPLE: three-way circular dependency should be detected"
    );
}
