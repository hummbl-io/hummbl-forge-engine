# HUMMBL Forge Engine

A meta-engine layer for building, publishing, modding, and versioning games across multiple runtime engines.

## What This Is

Forge Engine sits above multiple runtime engines (UE5 for AAA 3D, Forge2D/Bevy for 2D/indie) and provides one unified pipeline for:

- **Building** games for any target platform
- **Publishing** to any store (Steam, itch.io, Xbox, Linux native, direct)
- **Modding** — both moddable-by-design for your games and a universal mod loader for existing games (Factorio, Minecraft, Skyrim)
- **Versioning** — engine, game, mod, and asset versions managed coherently with lockfiles
- **Assets** — content-addressable store, Blender→glTF export, bundling
- **Dev** — watch mode with auto-rebuild, test runner, doc generation

## Status

**Phases 1-10 complete.** Core engine is functional and proven end-to-end with a Bevy test game.

## Quick Start

```bash
# Build the CLI
cargo build

# Initialize a new game project
forge init "My Game" --engine forge2d

# Check project configuration
forge check

# Show project info
forge info

# Build for Linux
forge build --platform linux-native --profile release

# Publish (dry-run)
forge publish --target direct --dry-run

# Generate lockfile
forge version lock

# Bump version
forge version bump minor

# Development server with watch
forge dev --watch

# Run tests
forge test

# Generate docs
forge doc --open
```

## CLI Commands

| Command | Description |
|---------|-------------|
| `forge init <name> --engine <e>` | Initialize a new project |
| `forge build [--platform p] [--profile p]` | Build the game |
| `forge publish --target <t> [--dry-run]` | Publish to a store |
| `forge dev [--watch]` | Development server |
| `forge test` | Run project tests |
| `forge doc [--open]` | Generate documentation |
| `forge info` | Show project info |
| `forge check` | Validate project config |
| `forge version show` | Show current version |
| `forge version bump <part>` | Bump version (major/minor/patch/pre) |
| `forge version lock` | Generate forge.lock |
| `forge version check` | Check lockfile is current |
| `forge mod install <path> --game <g>` | Install a mod |
| `forge mod list` | List installed mods |
| `forge mod enable <id>` | Enable a mod |
| `forge mod disable <id>` | Disable a mod |
| `forge mod new <name> --game <g>` | Create a mod scaffold |

## Supported Engines

| Engine | Status | Profile |
|--------|--------|---------|
| Forge2D (Bevy) | Working, tested | 2D games, indie |
| UE5 | Implemented, untested | AAA 3D games |

## Supported Stores

| Store | Status |
|-------|--------|
| Direct (tarball) | Working |
| Linux Native (AppDir/AppImage) | Working |
| Steam (SteamPipe) | Implemented (needs Steamworks account) |
| itch.io (butler) | Implemented (needs butler + API key) |
| Xbox GDK | Scaffolded (needs Partner Center API) |

## Supported Game Profiles (Universal Mod Loader)

| Game | Status |
|------|--------|
| Factorio | Working (install, list, enable, disable, remove, scaffold) |
| Minecraft (Fabric/Forge/NeoForge) | Implemented (JAR parsing, enable/disable) |
| Skyrim AE | Implemented (ESP/ESM/ESL, directory mods) |
| Generic | Working (any game with directory-based mods) |

## Architecture

See [docs/ARCHITECTURE-PROPOSAL-v0.1.md](docs/proposals/ARCHITECTURE-PROPOSAL-v0.1.md) for the full architecture.

### Crate Structure

| Crate | Purpose |
|-------|---------|
| `forge-core` | Shared types, `forge.toml` config schema, error handling |
| `forge-cli` | The `forge` binary (clap-based CLI) |
| `forge-build` | Build pipeline, parallel builds, local build cache |
| `forge-publish` | Publish pipeline, store plugins, binary signing |
| `forge-mod` | Mod system (SDK + loader + universal mod loader profiles) |
| `forge-version` | Versioning, lockfile, changelog generation |
| `forge-asset` | Content-addressable asset store, Blender export, bundling |
| `forge-adapter` | Engine abstraction layer (trait + dispatch) |
| `forge-adapter/ue5` | UE5 adapter (via UAT subprocess) |
| `forge-adapter/forge2d` | Forge2D adapter (Bevy-based) |

### Decisions

- [ADR-001: Rust + Cargo Workspace](docs/adr/ADR-001-rust-cargo-workspace.md)
- [ADR-002: Bevy as Forge2D Foundation](docs/adr/ADR-002-bevy-forge2d-foundation.md)
- [ADR-003: UE5 Adapter via UAT Subprocess](docs/adr/ADR-003-ue5-adapter-uat-subprocess.md)
- [ADR-004: Apache 2.0 License](docs/adr/ADR-004-apache-2.0-license.md)

## Testing

```bash
# Run all unit tests
cargo test

# Run integration tests (requires forge binary)
cargo test --test integration

# Clippy
cargo clippy -- -D warnings

# Format check
cargo fmt -- --check
```

## License

Apache 2.0 — see [LICENSE](LICENSE).
