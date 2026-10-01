//! Exercise a root Cargo.toml through the real CLI and Cargo subprocess.

use std::process::Command;

#[test]
fn root_cargo_manifest_builds_and_runs_offline() {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "forge-relative-cargo-{}-{nonce}",
        std::process::id()
    ));
    std::fs::create_dir(&root).unwrap();
    std::fs::create_dir(root.join("src")).unwrap();
    std::fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"forge-relative-fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\n[workspace]\n",
    )
    .unwrap();
    std::fs::write(
        root.join("src/main.rs"),
        "fn main() { println!(\"owned fixture built and executed\"); }\n",
    )
    .unwrap();
    std::fs::write(
        root.join("forge.toml"),
        "version = \"0.1.0\"\n[project]\nname = \"relative-fixture\"\nid = \"com.hummbl.test.relative\"\n[engine]\nkind = \"forge2d\"\ncargo_toml = \"Cargo.toml\"\n",
    )
    .unwrap();
    // linux-native selects Cargo's host target on both Windows and Unix.
    // The adapter currently expects the fixture's default target directory.
    let output = Command::new(env!("CARGO_BIN_EXE_forge"))
        .args(["build", "--platform", "linux-native", "--profile", "debug"])
        .current_dir(&root)
        .env("CARGO_NET_OFFLINE", "true")
        .env_remove("CARGO_TARGET_DIR")
        .output()
        .expect("failed to launch the freshly built Forge CLI");
    assert!(
        output.status.success(),
        "root manifest build failed: {} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let binary = root.join("target/debug").join(format!(
        "forge-relative-fixture{}",
        std::env::consts::EXE_SUFFIX
    ));
    let run = Command::new(binary).output().unwrap();
    assert!(run.status.success());
    assert_eq!(
        String::from_utf8_lossy(&run.stdout).trim(),
        "owned fixture built and executed"
    );
    assert!(root.join("build-manifest.json").is_file());
    // Remove only this test's newly created, unique fixture directory.
    let canonical_root = root.canonicalize().unwrap();
    assert!(canonical_root.starts_with(std::env::temp_dir().canonicalize().unwrap()));
    std::fs::remove_dir_all(canonical_root).unwrap();
}
