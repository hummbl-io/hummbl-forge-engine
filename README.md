# HUMMBL Forge Engine

A meta-engine layer for building, publishing, modding, and versioning games across multiple runtime engines.

## What This Is

Forge Engine sits above multiple runtime engines (UE5 for AAA 3D, Forge2D/Bevy for 2D/indie) and provides one unified pipeline for:

- **Building** games for any target platform
- **Publishing** to any store (Steam, Humble, Xbox, Linux native)
- **Modding** — both moddable-by-design for your games and a universal mod loader for existing games
- **Versioning** — engine, game, mod, and asset versions managed coherently

## Status

**Phase 0: Foundation** — scaffold and architecture. Not yet functional.

## Quick Start

```bash
# Build the CLI
cargo build

# Initialize a new game project
cargo run -- init "My Game" --engine forge2d

# Check project configuration
cargo run -- check

# Show project info
cargo run -- info
```

## Architecture

See [docs/ARCHITECTURE-PROPOSAL-v0.1.md](docs/ARCHITECTURE-PROPOSAL-v0.1.md) for the full architecture.

### Crate Structure

| Crate | Purpose |
|-------|---------|
| `forge-core` | Shared types, `forge.toml` config schema, error handling |
| `forge-cli` | The `forge` binary (clap-based CLI) |
| `forge-build` | Build pipeline (project + platform → packaged build) |
| `forge-publish` | Publish pipeline (build artifact + store → upload) |
| `forge-mod` | Mod system (SDK + loader + universal mod loader) |
| `forge-version` | Versioning + lockfile generation |
| `forge-asset` | Content-addressable asset store + Blender pipeline |
| `forge-adapter` | Engine abstraction layer (trait + dispatch) |
| `forge-adapter/ue5` | UE5 adapter (via UAT subprocess) |
| `forge-adapter/forge2d` | Forge2D adapter (Bevy-based) |

### Decisions

- [ADR-001: Rust + Cargo Workspace](docs/adr/ADR-001-rust-cargo-workspace.md)
- [ADR-002: Bevy as Forge2D Foundation](docs/adr/ADR-002-bevy-forge2d-foundation.md)
- [ADR-003: UE5 Adapter via UAT Subprocess](docs/adr/ADR-003-ue5-adapter-uat-subprocess.md)
- [ADR-004: Apache 2.0 License](docs/adr/ADR-004-apache-2.0-license.md)

## License

Apache 2.0 — see [LICENSE](LICENSE).
