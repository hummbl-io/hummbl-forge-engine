//! Integration tests for the Forge Engine CLI.
//!
//! These tests exercise the full pipeline end-to-end by creating
//! temporary projects, building, and verifying outputs.

use std::process::Command;

/// Helper: create a temp directory and return its path.
fn temp_dir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("forge-integration-{name}"));
    std::fs::remove_dir_all(&dir).ok();
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Helper: get the forge binary path.
fn forge_binary() -> std::path::PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    std::path::Path::new(manifest_dir)
        .join("../../target/debug/forge")
        .canonicalize()
        .unwrap_or_else(|_| std::path::Path::new(env!("CARGO_BIN_EXE_forge")).to_path_buf())
}

#[test]
fn test_forge_version_flag() {
    let output = Command::new(forge_binary())
        .arg("--version")
        .output()
        .expect("failed to run forge");

    assert!(output.status.success(), "forge --version failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("forge"),
        "version output should contain 'forge'"
    );
}

#[test]
fn test_forge_help_flag() {
    let output = Command::new(forge_binary())
        .arg("--help")
        .output()
        .expect("failed to run forge");

    assert!(output.status.success(), "forge --help failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("build"), "help should mention build");
    assert!(stdout.contains("publish"), "help should mention publish");
    assert!(stdout.contains("mod"), "help should mention mod");
    assert!(stdout.contains("version"), "help should mention version");
}

#[test]
fn test_forge_init_creates_project() {
    let dir = temp_dir("init");
    let output = Command::new(forge_binary())
        .arg("init")
        .arg("test-game")
        .arg("--engine")
        .arg("forge2d")
        .current_dir(&dir)
        .output()
        .expect("failed to run forge init");

    assert!(output.status.success(), "forge init failed");

    let project_dir = dir.join("test-game");
    assert!(project_dir.exists(), "project directory should exist");
    assert!(
        project_dir.join("forge.toml").exists(),
        "forge.toml should exist"
    );
    assert!(
        project_dir.join("assets").exists(),
        "assets directory should exist"
    );

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn test_forge_check_on_valid_project() {
    let dir = temp_dir("check");
    let init_output = Command::new(forge_binary())
        .arg("init")
        .arg("test-game")
        .current_dir(&dir)
        .output()
        .expect("failed to run forge init");

    assert!(init_output.status.success());

    let check_output = Command::new(forge_binary())
        .arg("check")
        .current_dir(dir.join("test-game"))
        .output()
        .expect("failed to run forge check");

    assert!(
        check_output.status.success(),
        "forge check should succeed on valid project"
    );

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn test_forge_info_on_valid_project() {
    let dir = temp_dir("info");
    Command::new(forge_binary())
        .arg("init")
        .arg("test-game")
        .current_dir(&dir)
        .output()
        .expect("failed to run forge init");

    let output = Command::new(forge_binary())
        .arg("info")
        .current_dir(dir.join("test-game"))
        .output()
        .expect("failed to run forge info");

    assert!(output.status.success(), "forge info should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("test-game") || stdout.contains("Test Game"),
        "info should show project name"
    );

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn test_forge_version_show() {
    let dir = temp_dir("version-show");
    Command::new(forge_binary())
        .arg("init")
        .arg("test-game")
        .current_dir(&dir)
        .output()
        .expect("failed to run forge init");

    let output = Command::new(forge_binary())
        .arg("version")
        .arg("show")
        .current_dir(dir.join("test-game"))
        .output()
        .expect("failed to run forge version show");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("0.1.0"),
        "should show initial version 0.1.0"
    );

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn test_forge_version_lock() {
    let dir = temp_dir("version-lock");
    Command::new(forge_binary())
        .arg("init")
        .arg("test-game")
        .current_dir(&dir)
        .output()
        .expect("failed to run forge init");

    let output = Command::new(forge_binary())
        .arg("version")
        .arg("lock")
        .current_dir(dir.join("test-game"))
        .output()
        .expect("failed to run forge version lock");

    assert!(output.status.success(), "forge version lock should succeed");

    let lockfile = dir.join("test-game/forge.lock");
    assert!(lockfile.exists(), "forge.lock should be created");

    let contents = std::fs::read_to_string(&lockfile).unwrap();
    assert!(
        contents.contains("format_version"),
        "lockfile should have format_version"
    );
    assert!(
        contents.contains("forge2d"),
        "lockfile should mention engine kind"
    );

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn test_forge_version_bump() {
    let dir = temp_dir("version-bump");
    Command::new(forge_binary())
        .arg("init")
        .arg("test-game")
        .current_dir(&dir)
        .output()
        .expect("failed to run forge init");

    // Bump minor version
    let output = Command::new(forge_binary())
        .arg("version")
        .arg("bump")
        .arg("minor")
        .current_dir(dir.join("test-game"))
        .output()
        .expect("failed to run forge version bump");

    assert!(output.status.success(), "forge version bump should succeed");

    // Verify the version was bumped
    let show_output = Command::new(forge_binary())
        .arg("version")
        .arg("show")
        .current_dir(dir.join("test-game"))
        .output()
        .expect("failed to run forge version show");

    let stdout = String::from_utf8_lossy(&show_output.stdout);
    assert!(
        stdout.contains("0.2.0"),
        "version should be bumped to 0.2.0"
    );

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn test_forge_mod_new_factorio() {
    let dir = temp_dir("mod-new");
    let output = Command::new(forge_binary())
        .arg("mod")
        .arg("new")
        .arg("test-factorio-mod")
        .arg("--game")
        .arg("factorio")
        .current_dir(&dir)
        .output()
        .expect("failed to run forge mod new");

    assert!(output.status.success(), "forge mod new should succeed");

    let mod_dir = dir.join("test-factorio-mod");
    assert!(mod_dir.join("info.json").exists(), "info.json should exist");
    assert!(mod_dir.join("data.lua").exists(), "data.lua should exist");
    assert!(
        mod_dir.join("control.lua").exists(),
        "control.lua should exist"
    );

    let info = std::fs::read_to_string(mod_dir.join("info.json")).unwrap();
    assert!(
        info.contains("test-factorio-mod"),
        "info.json should contain mod name"
    );
    assert!(
        info.contains("factorio_version"),
        "info.json should have factorio_version"
    );

    std::fs::remove_dir_all(&dir).ok();
}
