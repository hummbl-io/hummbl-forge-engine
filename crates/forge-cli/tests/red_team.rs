//! Red team security audit — adversarial testing of Forge Engine.
//!
//! These tests probe for known vulnerability classes:
//! - Path traversal in mod installation
//! - Command injection in build/publish pipelines
//! - Symlink attacks on asset store
//! - Lockfile tampering
//! - Mod sandbox escapes
//! - Supply chain risks (unsigned mods, unverified downloads)

use std::path::PathBuf;
use std::process::Command;

fn forge_binary() -> PathBuf {
    std::path::Path::new(env!("CARGO_BIN_EXE_forge")).to_path_buf()
}

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("forge-redteam-{name}"));
    std::fs::remove_dir_all(&dir).ok();
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

// === PATH TRAVERSAL TESTS ===

#[test]
fn red_path_traversal_mod_install() {
    // Try to install a mod with a path traversal name
    let dir = temp_dir("path-traversal");
    Command::new(forge_binary())
        .arg("init")
        .arg("test-game")
        .current_dir(&dir)
        .output()
        .unwrap();

    // Create a malicious mod with ../../../ in the name
    let mod_dir = dir.join("../../../tmp/forge-redteam-traversal-evil");
    std::fs::create_dir_all(&mod_dir).ok();
    std::fs::write(
        mod_dir.join("info.json"),
        r#"{"name": "evil", "version": "1.0.0", "factorio_version": "1.0"}"#,
    )
    .ok();
    std::fs::write(mod_dir.join("data.lua"), "-- evil").ok();

    // This should NOT write outside the mods directory
    let output = Command::new(forge_binary())
        .arg("mod")
        .arg("install")
        .arg("../../../tmp/forge-redteam-traversal-evil")
        .arg("--game")
        .arg("factorio")
        .current_dir(dir.join("test-game"))
        .output()
        .unwrap();

    // Document the behavior — BLUE team should add explicit path validation
    let _ = output.status.success();

    std::fs::remove_dir_all(&dir).ok();
    std::fs::remove_dir_all(&mod_dir).ok();
}

// === COMMAND INJECTION TESTS ===

#[test]
fn red_command_injection_in_project_name() {
    let dir = temp_dir("cmd-injection");
    let evil_name = "test; rm -rf /tmp/forge-redteam-cmd-injection-target";
    let target_file = "/tmp/forge-redteam-cmd-injection-target";
    std::fs::write(target_file, "should not be deleted").ok();

    let _output = Command::new(forge_binary())
        .arg("init")
        .arg(evil_name)
        .current_dir(&dir)
        .output()
        .unwrap();

    // The project name should be treated as a literal string, not executed
    assert!(
        std::path::Path::new(target_file).exists(),
        "CRITICAL: command injection in project name — file was deleted!"
    );

    std::fs::remove_file(target_file).ok();
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn red_command_injection_in_engine_flag() {
    let dir = temp_dir("cmd-injection-engine");
    let target_file = "/tmp/forge-redteam-engine-injection-target";
    std::fs::write(target_file, "should not be deleted").ok();

    let evil_engine = "forge2d; rm -f /tmp/forge-redteam-engine-injection-target";
    let _output = Command::new(forge_binary())
        .arg("init")
        .arg("test")
        .arg("--engine")
        .arg(evil_engine)
        .current_dir(&dir)
        .output()
        .unwrap();

    assert!(
        std::path::Path::new(target_file).exists(),
        "CRITICAL: command injection in --engine flag!"
    );

    std::fs::remove_file(target_file).ok();
    std::fs::remove_dir_all(&dir).ok();
}

// === LOCKFILE TAMPERING TESTS ===

#[test]
fn red_lockfile_tampering_detection() {
    let dir = temp_dir("lockfile-tamper");
    Command::new(forge_binary())
        .arg("init")
        .arg("test-game")
        .current_dir(&dir)
        .output()
        .unwrap();

    // Generate lockfile
    Command::new(forge_binary())
        .arg("version")
        .arg("lock")
        .current_dir(dir.join("test-game"))
        .output()
        .unwrap();

    let lockfile = dir.join("test-game/forge.lock");
    let original = std::fs::read_to_string(&lockfile).unwrap();

    // Tamper with the lockfile — change the version
    let tampered = original.replace("0.1.0", "99.99.99");
    std::fs::write(&lockfile, &tampered).unwrap();

    // Check should detect the tampering
    let output = Command::new(forge_binary())
        .arg("version")
        .arg("check")
        .current_dir(dir.join("test-game"))
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let detected = stdout.contains("out of date")
        || stdout.contains("mismatch")
        || stdout.contains("stale")
        || stderr.contains("out of date")
        || stderr.contains("mismatch")
        || !output.status.success();

    // RED TEAM FINDING: lockfile tampering is NOT currently detected.
    // The version check compares the lockfile against the current project
    // state, but a version field change in the lockfile is not caught
    // because the check only verifies the engine/mod hashes, not the
    // project version field.
    //
    // BLUE TEAM ACTION: Add a hash/MAC of the project version to the
    // lockfile so tampering with the version field is detectable.
    if !detected {
        eprintln!(
            "RED TEAM FINDING: lockfile tampering NOT detected — \
             version check does not catch modified version field in lockfile"
        );
    }

    std::fs::remove_dir_all(&dir).ok();
}

// === MOD SANDBOX ESCAPE TESTS ===

#[test]
fn red_mod_sandbox_path_traversal() {
    use forge_mod::manifest::ModPermission;
    use forge_mod::sdk::{PermissionChecker, SandboxConfig};

    let sandbox_root = std::env::temp_dir().join("forge-redteam-sandbox");
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

    // Attempt path traversal via check_read
    let result = checker.check_read(std::path::Path::new("../../../etc/passwd"));
    assert!(
        result.is_err(),
        "CRITICAL: sandbox allows path traversal to /etc/passwd!"
    );

    // Attempt absolute path outside root
    let result = checker.check_read(std::path::Path::new("/etc/shadow"));
    assert!(
        result.is_err(),
        "CRITICAL: sandbox allows absolute path to /etc/shadow!"
    );

    std::fs::remove_dir_all(&sandbox_root).ok();
}

#[test]
fn red_mod_sandbox_network_denied() {
    use forge_mod::manifest::ModPermission;
    use forge_mod::sdk::{PermissionChecker, SandboxConfig};

    let checker = PermissionChecker::new(
        SandboxConfig {
            strict_mode: true,
            allowed_read_root: std::path::PathBuf::from("/tmp"),
            allowed_write_root: std::path::PathBuf::from("/tmp"),
            allow_network: false,
            allow_code_execution: false,
        },
        vec![ModPermission::ReadFiles],
    );

    let result = checker.check_network();
    assert!(
        result.is_err(),
        "CRITICAL: sandbox allows network when it should be denied!"
    );
}

#[test]
fn red_mod_sandbox_code_exec_denied() {
    use forge_mod::manifest::ModPermission;
    use forge_mod::sdk::{PermissionChecker, SandboxConfig};

    let checker = PermissionChecker::new(
        SandboxConfig {
            strict_mode: true,
            allowed_read_root: std::path::PathBuf::from("/tmp"),
            allowed_write_root: std::path::PathBuf::from("/tmp"),
            allow_network: false,
            allow_code_execution: false,
        },
        vec![ModPermission::ReadFiles],
    );

    let result = checker.check_execute();
    assert!(
        result.is_err(),
        "CRITICAL: sandbox allows code execution when it should be denied!"
    );
}

#[test]
fn red_mod_sandbox_write_outside_root() {
    use forge_mod::manifest::ModPermission;
    use forge_mod::sdk::{PermissionChecker, SandboxConfig};

    let sandbox_root = std::env::temp_dir().join("forge-redteam-sandbox-write");
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

    // Attempt to write outside the sandbox
    let result = checker.check_write(std::path::Path::new("/tmp/forge-redteam-evil-write"));
    assert!(
        result.is_err(),
        "CRITICAL: sandbox allows write outside allowed root!"
    );

    std::fs::remove_dir_all(&sandbox_root).ok();
}

// === SUPPLY CHAIN TESTS ===

#[test]
fn red_unsigned_mod_accepted() {
    // Document whether unsigned mods are accepted (they currently are)
    // BLUE team should add mod signing verification
    let dir = temp_dir("unsigned-mod");
    Command::new(forge_binary())
        .arg("init")
        .arg("test-game")
        .current_dir(&dir)
        .output()
        .unwrap();

    let mod_dir = dir.join("unsigned-mod");
    std::fs::create_dir_all(&mod_dir).unwrap();
    std::fs::write(
        mod_dir.join("info.json"),
        r#"{"name": "unsigned", "version": "1.0.0", "factorio_version": "1.0"}"#,
    )
    .unwrap();
    std::fs::write(mod_dir.join("data.lua"), "-- unsigned mod").unwrap();

    let output = Command::new(forge_binary())
        .arg("mod")
        .arg("install")
        .arg(&mod_dir)
        .arg("--game")
        .arg("factorio")
        .current_dir(dir.join("test-game"))
        .output()
        .unwrap();

    // Currently unsigned mods ARE accepted — this is a finding
    let accepted = output.status.success();
    eprintln!("FINDING: unsigned mod accepted = {accepted} (should be false in production)");

    std::fs::remove_dir_all(&dir).ok();
}

// === SYMLINK ATTACK TESTS ===

#[test]
fn red_symlink_attack_asset_store() {
    use forge_asset::AssetStore;

    let dir = temp_dir("symlink-attack");
    let store_dir = dir.join("store");
    let store = AssetStore::new(&store_dir);

    let evil_target = dir.join("evil-target.txt");
    std::fs::write(&evil_target, "secret data").unwrap();

    let symlink_path = dir.join("evil-symlink.txt");
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&evil_target, &symlink_path).unwrap();
        let result = store.add(&symlink_path);
        eprintln!(
            "FINDING: symlink to external file {}",
            if result.is_ok() {
                "accepted (potential info leak)"
            } else {
                "rejected (safe)"
            }
        );
    }
    #[cfg(not(unix))]
    {
        let _ = store;
        let _ = symlink_path;
    }

    std::fs::remove_dir_all(&dir).ok();
}

// === FORGE.TOML INJECTION TESTS ===

#[test]
fn red_toml_injection_in_config() {
    let dir = temp_dir("toml-injection");
    Command::new(forge_binary())
        .arg("init")
        .arg("test-game")
        .current_dir(&dir)
        .output()
        .unwrap();

    let forge_toml = dir.join("test-game/forge.toml");
    let malicious_toml = r#"
[project]
name = "test-game"
version = "0.1.0"

[engine]
kind = "forge2d"
cargo_toml = "../../../etc/passwd"
"#;
    std::fs::write(&forge_toml, malicious_toml).unwrap();

    let output = Command::new(forge_binary())
        .arg("check")
        .current_dir(dir.join("test-game"))
        .output()
        .unwrap();

    eprintln!(
        "FINDING: malicious cargo_toml path — check exited {}",
        output.status
    );

    std::fs::remove_dir_all(&dir).ok();
}

// === DEPENDENCY CONFUSION TEST ===

#[test]
fn red_dependency_resolution_cycle() {
    use forge_mod::manifest::{ModDependency, ModInfo, ModManifest, ModTarget};
    use forge_mod::sdk::DependencyResolver;

    let target = ModTarget {
        game: "test".to_string(),
        compatible_versions: "^1.0.0".to_string(),
    };

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
            id: "mod-a".to_string(),
            version: "^1.0.0".to_string(),
            optional: false,
        }],
        hooks: vec![],
        permissions: vec![],
    };

    let resolver = DependencyResolver::new(vec![mod_a, mod_b]);
    let result = resolver.resolve();
    assert!(
        result.is_err(),
        "CRITICAL: circular dependency not detected!"
    );
}

// === PERMISSION ESCALATION TEST ===

#[test]
fn red_permission_escalation_no_perms() {
    use forge_mod::manifest::ModPermission;
    use forge_mod::sdk::{PermissionChecker, SandboxConfig};

    // A mod with NO permissions should not be able to read, write, network, or exec
    let checker = PermissionChecker::new(
        SandboxConfig {
            strict_mode: true,
            allowed_read_root: std::path::PathBuf::from("/tmp"),
            allowed_write_root: std::path::PathBuf::from("/tmp"),
            allow_network: false,
            allow_code_execution: false,
        },
        vec![], // no permissions
    );

    assert!(checker
        .check_read(std::path::Path::new("/tmp/test"))
        .is_err());
    assert!(checker
        .check_write(std::path::Path::new("/tmp/test"))
        .is_err());
    assert!(checker.check_network().is_err());
    assert!(checker.check_execute().is_err());
}
