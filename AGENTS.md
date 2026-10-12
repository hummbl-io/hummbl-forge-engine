# AGENTS.md — hummbl-forge-engine

## Project

HUMMBL Forge Engine is a meta-engine layer for building, publishing, modding,
and versioning games across multiple runtime engines (UE5, Forge2D/Bevy).

## Authority

Per the operator's invariants (informed by the games repository AGENTS.md):

- **Operator (the operator)** retains: architectural authority, licensing
  decisions, publication authority, promotion authority, store account
  credentials.
- **Agents** may: write code, test, propose architecture changes via ADR,
  open PRs, draft documentation.
- **Agents may NOT**: choose the permanent architecture without operator
  approval, change the license, make public claims or releases, import
  third-party game code or assets without approval, merge own PRs, push
  directly to `main`.

## Setup

```bash
# Rust toolchain (stable)
rustup show

# Build
cargo build

# Test
cargo test

# Run CLI
cargo run -- --help
```

## Conventions

- **Language:** Rust (stable channel). Lua for mod scripting.
- **Format:** `cargo fmt` before commit.
- **Lint:** `cargo clippy -- -D warnings` must pass.
- **Commits:** Conventional Commits format.
- **Branch naming:** `type/agent/short-desc` (e.g. `feat/agent/ue5-adapter`)
- **PRs only:** No direct pushes to `main`. No force-push. No `--no-verify`.
- **GPG signing:** All commits GPG-signed per fleet policy.

## Architecture decisions

All foundational decisions require an ADR in `docs/adr/`. See:
- ADR-001: Rust + Cargo workspace
- ADR-002: Bevy as Forge2D foundation
- ADR-003: UE5 adapter via UAT subprocess
- ADR-004: Apache 2.0 license

## Crate structure

```
crates/
├── forge-core/       — types, config (forge.toml), error handling
├── forge-cli/        — the `forge` binary (clap)
├── forge-build/      — build pipeline
├── forge-publish/    — publish pipeline + store plugins
├── forge-mod/        — mod system (SDK + loader runtime)
├── forge-version/    — versioning + lockfile
├── forge-asset/      — asset store + Blender pipeline
└── forge-adapter/    — engine abstraction layer
    ├── ue5/          — UE5 adapter
    └── forge2d/      — Forge2D (Bevy) adapter
```

## Validation

Before merge:

```bash
cargo fmt -- --check
cargo clippy -- -D warnings
cargo test
```

## Store credentials

Store credentials (Steamworks, Xbox Partner Center, Humble) live in 1Password
or environment variables — NEVER in the repo. The publish pipeline reads from
vault/env at runtime.

## Off-limits

- Never commit secrets, API keys, or store credentials.
- Never push to `main` directly — PRs only.
- Never import third-party game code or assets without operator approval.
- Never modify the license without operator approval.
- Never publish releases without operator approval.
