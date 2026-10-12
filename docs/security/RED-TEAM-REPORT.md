# RED TEAM Security Audit Report — HUMMBL Forge Engine

**Date:** 2026-09-21
**Auditor:** an agent (automated)
**Scope:** All Forge Engine crates, CLI, mod system, asset pipeline, publish pipeline
**Method:** Adversarial testing via `crates/forge-cli/tests/red_team.rs` (13 tests)

## Test Results

| # | Test | Result | Severity |
|---|------|--------|----------|
| 1 | `red_command_injection_in_project_name` | PASS (no injection) | — |
| 2 | `red_command_injection_in_engine_flag` | PASS (no injection) | — |
| 3 | `red_mod_sandbox_path_traversal` | PASS (blocked) | — |
| 4 | `red_mod_sandbox_network_denied` | PASS (blocked) | — |
| 5 | `red_mod_sandbox_code_exec_denied` | PASS (blocked) | — |
| 6 | `red_mod_sandbox_write_outside_root` | PASS (blocked) | — |
| 7 | `red_permission_escalation_no_perms` | PASS (denied) | — |
| 8 | `red_dependency_resolution_cycle` | PASS (detected) | — |
| 9 | `red_path_traversal_mod_install` | DOCUMENTED | MEDIUM |
| 10 | `red_lockfile_tampering_detection` | FINDING | MEDIUM |
| 11 | `red_unsigned_mod_accepted` | FINDING | HIGH |
| 12 | `red_symlink_attack_asset_store` | DOCUMENTED | LOW |
| 13 | `red_toml_injection_in_config` | DOCUMENTED | MEDIUM |

## Findings Detail

### FINDING-1: Lockfile tampering not detected (MEDIUM)

**Test:** `red_lockfile_tampering_detection`
**Description:** The `forge version check` command does not detect when the
`version` field in `forge.lock` has been manually modified. The check only
compares engine/mod hashes against the current project state, not the project
version field in the lockfile itself.

**Impact:** An attacker with write access to `forge.lock` could modify the
recorded version without detection, potentially causing downstream consumers
to believe they have a different version than actually built.

**BLUE TEAM ACTION:** Add a hash/MAC of the project version (and other
metadata fields) to the lockfile so any field modification is detectable.

### FINDING-2: Unsigned mods are accepted (HIGH)

**Test:** `red_unsigned_mod_accepted`
**Description:** The mod loader accepts and installs mods without verifying
any digital signature. Any mod directory with a valid `info.json` is accepted.

**Impact:** A malicious mod could be installed without any provenance
verification. This is the highest-severity finding because mods can contain
arbitrary Lua code (Factorio) or JAR files (Minecraft).

**BLUE TEAM ACTION:** Implement mod signing (GPG or custom signature scheme).
Add a `--trusted-keys` flag and verify mod signatures before installation.

### FINDING-3: Path traversal in mod install path (MEDIUM)

**Test:** `red_path_traversal_mod_install`
**Description:** The mod install command accepts relative paths with `../../../`
without explicit validation. The test documents the current behavior — the
path is resolved relative to the project directory, which could potentially
write outside the mods directory.

**Impact:** A user tricked into installing a mod from a path like
`../../../tmp/evil` could have files written outside the expected mods
directory.

**BLUE TEAM ACTION:** Add explicit path canonicalization and containment
checks in the mod install command. Reject paths that resolve outside the
project directory.

### FINDING-4: Symlink handling in asset store (LOW)

**Test:** `red_symlink_attack_asset_store`
**Description:** The asset store does not explicitly check for symlinks. A
symlink pointing to a file outside the store directory would be hashed
(reading the target content) rather than rejected.

**Impact:** Low — the content hash would still be deterministic, but an
attacker could potentially use symlinks to include sensitive file contents
in the asset store.

**BLUE TEAM ACTION:** Add symlink detection in `AssetStore::add()` — either
reject symlinks or canonicalize and verify the target is within the store.

### FINDING-5: Malicious forge.toml paths (MEDIUM)

**Test:** `red_toml_injection_in_config`
**Description:** The `forge check` command does not validate that paths in
`forge.toml` (like `cargo_toml`) are within the project directory. A
malicious `forge.toml` could point to files outside the project.

**Impact:** If a user is tricked into running forge commands in a directory
with a malicious `forge.toml`, the engine could read files outside the
project directory.

**BLUE TEAM ACTION:** Add path validation in the config loader — reject
`forge.toml` paths that resolve outside the project root.

## What Passed (Working Defenses)

- **Command injection:** Project names and engine flags with shell
  metacharacters are treated as literal strings. No shell execution occurs.
- **Sandbox path traversal:** The `PermissionChecker` correctly blocks
  `../../../etc/passwd` and absolute paths outside the allowed root.
- **Sandbox network/code-exec:** Network access and code execution are
  correctly denied when not explicitly allowed.
- **Permission escalation:** A mod with no permissions cannot perform any
  restricted operation.
- **Circular dependencies:** The dependency resolver correctly detects
  circular mod dependencies and rejects them.

## Recommendations for BLUE TEAM

1. **Mod signing** (addresses FINDING-2, HIGH)
2. **Lockfile integrity hash** (addresses FINDING-1, MEDIUM)
3. **Path validation in mod install** (addresses FINDING-3, MEDIUM)
4. **Symlink rejection in asset store** (addresses FINDING-4, LOW)
5. **Config path validation** (addresses FINDING-5, MEDIUM)
