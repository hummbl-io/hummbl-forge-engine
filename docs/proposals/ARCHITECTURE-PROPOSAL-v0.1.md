# HUMMBL Forge Engine — Architecture Proposal v0.1

**Status:** PROPOSAL — awaiting operator approval. Not canonical.
**Author:** an agent (Principal AI Agent)
**Authority:** Operator (the operator) retains all architectural, licensing, and publication authority per the games repository AGENTS.md invariants.

---

## 1. What This Is

HUMMBL Forge Engine is a **meta-engine layer** — a unified toolchain that sits above multiple runtime engines and provides one pipeline for building, publishing, modding, and versioning games across all target platforms.

### The problem it solves

You want to:
- Build games in UE5 (AAA 3D) AND in a lighter engine (2D/indie/experimental)
- Publish to Linux native, Steam, Humble, Xbox/Microsoft, and more
- Make every game you build moddable by design
- Load mods into existing games you own (Skyrim, Minecraft, Factorio, etc.)
- Version and evolve engines, games, mods, and assets coherently over years

Today each of those is a separate toolchain with no shared pipeline. Forge Engine is the shared layer.

### What it is NOT

- Not a replacement for UE5. UE5 stays as the rendering/runtime core for AAA 3D work on Windows.
- Not a from-scratch engine (yet). The custom lightweight engine for 2D/indie is a component, not the whole system.
- Not a mod tool for a single game. It's a cross-game mod platform.
- Not a publishing service. It's a build/publish pipeline that targets multiple stores.

---

## 2. Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                    FORGE ENGINE LAYER                     │
│                                                           │
│  ┌─────────┐  ┌──────────┐  ┌─────────┐  ┌───────────┐  │
│  │  Build  │  │ Publish  │  │  Mod    │  │  Version  │  │
│  │ Pipeline│  │ Pipeline │  │ System  │  │  System   │  │
│  └────┬────┘  └────┬─────┘  └────┬────┘  └─────┬─────┘  │
│       │            │              │              │        │
│  ┌────┴────────────┴──────────────┴──────────────┴────┐  │
│  │              Engine Abstraction Layer               │  │
│  │                                                     │  │
│  │  ┌──────────┐    ┌──────────┐    ┌──────────────┐  │  │
│  │  │ UE5      │    │ Forge2D  │    │  Future      │  │  │
│  │  │ Adapter  │    │ Adapter  │    │  Adapters    │  │  │
│  │  └────┬─────┘    └────┬─────┘    └──────────────┘  │  │
│  └───────┼───────────────┼───────────────────────────┘  │
│          │               │                               │
└──────────┼───────────────┼───────────────────────────────┘
           │               │
     ┌─────┴─────┐   ┌─────┴─────┐
     │   UE5     │   │  Forge2D  │
     │ (Windows)   │   │ (custom,  │
     │           │   │  lightweight)
     └───────────┘   └───────────┘
```

### Layers (top to bottom)

1. **Forge Engine Layer** — the meta-layer you own. Four subsystems, one abstraction layer.
2. **Engine Abstraction Layer** — adapters that translate Forge commands into engine-specific operations. UE5 adapter talks to Unreal Automation Tool / UBT. Forge2D adapter talks to the custom engine directly.
3. **Runtime Engines** — UE5 on Windows for AAA 3D. Forge2D (custom, to be built) for 2D/indie/experimental. Future adapters possible (Godot, etc.).

---

## 3. The Four Subsystems

### 3.1 Build Pipeline

Takes a game project + target platform → produces a packaged, tested, signed build.

```
Game Project (forge.toml)
    │
    ├── forge build --engine ue5 --platform linux-steam
    ├── forge build --engine ue5 --platform windows-xbox
    ├── forge build --engine forge2d --platform linux-native
    ├── forge build --engine forge2d --platform web
    │
    ▼
Build Artifact (signed, manifest, tested)
```

**Key design decisions:**
- `forge.toml` is the single source of truth for a game project (engine, targets, assets, mod policy, version)
- Build pipeline is engine-agnostic at the top, engine-specific at the adapter
- UE5 builds invoke Unreal Automation Tool (UAT) / UnrealBuildTool (UBT) via the UE5 adapter
- Forge2D builds invoke the custom engine's build system directly
- Every build produces: binary + manifest + checksum + test receipt
- Reproducible builds: pinned toolchain versions, deterministic asset processing

### 3.2 Publish Pipeline

Takes a build artifact + target store → produces a store-ready package and uploads it.

```
Build Artifact
    │
    ├── forge publish --target steam
    │     → SteamPipe upload (depot, build, live branch)
    ├── forge publish --target humble
    │     → Humble widget/package format
    ├── forge publish --target xbox
    │     → Microsoft Partner Center upload (.msixvc package)
    ├── forge publish --target linux-native
    │     → .AppImage / .deb / .rpm / Flatpak
    │
    ▼
Store-ready package + upload receipt
```

**Key design decisions:**
- Each store target is a plugin (steam, humble, xbox, linux-native, flatpak, itch)
- Store credentials live in a vault (1Password integration via existing fleet MCP), never in the repo
- Dry-run mode: produces the package without uploading
- Publish receipts: what was uploaded, when, to which store, with what build hash
- Steam: wraps SteamPipe / steamcmd
- Xbox: wraps Microsoft Game Development Kit (GDK) packaging tools
- Humble: produces the appropriate bundle/widget format
- Linux native: AppImage (universal), .deb (Debian/Ubuntu), .rpm (Fedora/SUSE), Flatpak (sandboxed)

### 3.3 Mod System (two parts)

#### 3.3a Moddable-by-Design SDK (for games you build)

Games built on Forge Engine are moddable by construction:

```
Mod SDK
├── Mod Manifest (mod.toml)
│   ├── id, name, author, version
│   ├── target game + compatible versions
│   ├── dependencies (other mods)
│   ├── content hooks (what this mod replaces/adds)
│   └── permissions (what the mod is allowed to do)
├── Asset Hot-Loading
│   ├── drop assets into mod folder → live in-game
│   ├── supports UE5 assets (via adapter) and Forge2D assets
│   └── versioned asset overrides
├── Script/Logic Hooks
│   ├── event-driven hook system (on_load, on_tick, on_player_action, etc.)
│   ├── sandboxed script execution (Lua for Forge2D, Blueprint/C++ for UE5)
│   └── mod API surface (what mods can call)
└── Mod Loader Runtime
    ├── loads mods at startup in dependency order
    ├── resolves conflicts (load order, priority, overrides)
    ├── sandboxes mods (permissions enforced)
    └── reports mod errors without crashing the game
```

#### 3.3b Universal Mod Loader (for existing games you own)

A cross-game mod manager that hooks into games you didn't build:

```
Universal Mod Loader
├── Game Profiles
│   ├── Skyrim: SKSE/SE plugin loading, BSA archives, Papyrus scripts
│   ├── Minecraft: Java agent / mod loader (Fabric/Forge/Quilt profile)
│   ├── Factorio: mod directory + scenario loading
│   ├── Stardew Valley: SMAPI integration
│   └── [extensible — each game is a profile plugin]
├── Mod Library
│   ├── local mod collection (any source: Nexus, Steam Workshop, manual)
│   ├── mod metadata cache (name, version, source, hash)
│   └── conflict detection across mods for the same game
├── Load Order Manager
│   ├── per-game load order rules
│   ├── automatic sorting where possible
│   ├── manual override with validation
│   └── profile switching (multiple mod configurations per game)
└── Deployment
    ├── installs mods to the correct game directory
    ├── creates backups of vanilla state
    ├── one-click enable/disable per mod
    └── clean uninstall (restores vanilla)
```

**Key design decisions:**
- The universal loader does NOT modify game executables (uses each game's supported mod mechanism)
- Each game profile is a plugin — adding a new game means writing a profile, not modifying the core
- Mod sources are agnostic: Nexus, Steam Workshop, manual download — the loader manages them uniformly
- Safety first: backups, validation, clean uninstall. No bricking games.

### 3.4 Version & Evolution System

Manages versions across engines, games, mods, and assets as a coherent whole.

```
Version System
├── Engine Versions
│   ├── UE5: tracked via Epic Games Launcher versions + .uproject engine association
│   ├── Forge2D: semver, own release cycle
│   └── Adapter compatibility matrix (which adapter version works with which engine version)
├── Game Versions
│   ├── semver per game
│   ├── changelog generation from conventional commits
│   ├── platform-specific version pins (Steam build ID ↔ git tag)
│   └── save-file compatibility (migration scripts between game versions)
├── Mod Versions
│   ├── mod semver + compatible game version range
│   ├── mod dependency resolution (semver ranges)
│   └── mod update distribution (workshop, direct, or Forge-hosted)
├── Asset Versions
│   ├── content-addressable asset store (hash-based)
│   ├── asset deduplication across games
│   ├── asset lineage (which game uses which asset version)
│   └── Blender pipeline integration (source .blend → exported asset → hash)
└── Cross-Cutting
    ├── `forge.toml` pins all versions (engine, adapter, game, mods)
    ├── `forge lock` generates a lockfile (exact versions, like Cargo.lock)
    └── `forge upgrade` bumps versions with compatibility checks
```

---

## 4. Tech Stack Proposal

### Meta-layer (Forge Engine itself)

| Component | Choice | Rationale |
|-----------|--------|-----------|
| Core language | **Rust** | Fast, safe, cross-platform, excellent tooling (cargo), already on Windows. CLI + library from one codebase. |
| CLI | **Rust + clap** | Single binary, fast startup, no runtime dependency. `forge build`, `forge publish`, `forge mod install`, etc. |
| Config format | **TOML** | `forge.toml` for projects, `mod.toml` for mods. Human-readable, well-tooled in Rust. |
| Build orchestration | **Rust + subprocess** | Forge CLI invokes engine-specific build tools (UAT, cargo, etc.) as subprocesses. |
| Scripting (mod API) | **Lua** (Forge2D) / **Blueprint+C++** (UE5) | Lua is the game industry standard for mod scripting. UE5 mods use native UE5 scripting via adapter. |
| Asset pipeline | **Rust + Blender CLI** | Blender headless export via Python scripts. Forge orchestrates. |
| Store SDK integration | **Rust wrappers** | SteamPipe (steamcmd subprocess), Xbox GDK (CLI tools), Humble (HTTP API). |
| Testing | **Rust + cargo test** + engine-native | Unit tests for Forge core. Integration tests invoke real builds. |
| CI | **GitHub Actions** (self-hosted on Windows) | Windows has the GPU, UE5, and toolchain. |

### Forge2D (custom lightweight engine)

| Component | Choice | Rationale |
|-----------|--------|-----------|
| Language | **Rust** | Same language as the meta-layer. Shared types, no FFI boundary. |
| Rendering | **wgpu** | Cross-platform (Vulkan/Metal/DX12/WebGPU). Runs on Linux, Windows, web. |
| Physics | **rapier2d** | Pure Rust, good performance, maintained. |
| Audio | **kira** or **rodio** | Rust-native, low-latency. |
| Scripting | **Lua** via `mlua` | Mod scripting, hot-reloadable. |
| ECS | **bevy_ecs** (standalone) or **hecs** | Entity-component-system for game objects. |
| Asset format | **Custom** (content-addressed) | Forge asset store integration. |

**Note on Bevy:** Bevy is a full Rust game engine that could serve as Forge2D's foundation rather than building from scratch. This is a build-vs-buy decision for your approval. Options:
- **Option A:** Build Forge2D from scratch using wgpu/rapier/mlua directly (maximum control, more work)
- **Option B:** Use Bevy as Forge2D's core, wrap it in the Forge adapter (faster, less control, Bevy is MIT-licensed)
- **Recommendation:** Option B — Bevy is mature, MIT-licensed, and you get a working 2D engine immediately. Your value is in the meta-layer, not in reinventing 2D rendering.

### UE5 Adapter

| Component | Choice | Rationale |
|-----------|--------|-----------|
| Communication | **Rust → subprocess → UAT/UBT** | Unreal Automation Tool is Epic's official build pipeline. No need to reinvent. |
| Project management | **`.uproject` file manipulation** | Read/write UE5 project files programmatically. |
| Plugin generation | **UE5 plugin templates** | Forge generates UE5 plugins for mod support, build hooks, etc. |
| Asset pipeline | **Blender → FBX/glTF → UE5 import** | Blender headless export, UE5 automated import via commandlet. |

---

## 5. Repository Structure (proposed)

```
hummbl-forge-engine/          (this repo — the meta-layer)
├── Cargo.toml                (workspace)
├── forge.toml                (engine's own forge.toml — dogfooding)
├── crates/
│   ├── forge-core/           (types, config, error handling)
│   ├── forge-cli/            (the `forge` binary)
│   ├── forge-build/          (build pipeline)
│   ├── forge-publish/        (publish pipeline + store plugins)
│   ├── forge-mod/            (mod system: SDK + loader runtime)
│   ├── forge-version/        (versioning + lockfile)
│   ├── forge-asset/          (asset store + Blender pipeline)
│   └── forge-adapter/        (engine abstraction layer)
│       ├── ue5/              (UE5 adapter)
│       └── forge2d/          (Forge2D adapter)
├── plugins/
│   ├── publish-steam/        (Steam publishing plugin)
│   ├── publish-humble/       (Humble publishing plugin)
│   ├── publish-xbox/         (Xbox publishing plugin)
│   ├── publish-linux/        (Linux native packaging plugin)
│   └── mod-game-skyrim/      (Universal mod loader: Skyrim profile)
│   ├── mod-game-minecraft/   (Universal mod loader: Minecraft profile)
│   └── mod-game-factorio/    (Universal mod loader: Factorio profile)
├── docs/
│   ├── ARCHITECTURE-PROPOSAL-v0.1.md  (this file)
│   ├── adr/                  (architecture decision records)
│   └── guides/               (user-facing guides)
├── tests/
│   └── integration/          (end-to-end build/publish/mod tests)
└── .github/
    └── workflows/            (CI on self-hosted Windows runner)
```

**Forge2D** lives in this repo initially (as `crates/forge-adapter/forge2d/` or a sibling crate). If it grows large enough, it can split into its own repo later.

---

## 6. Target Platforms

| Platform | Build Target | Publish Target | Status |
|----------|-------------|----------------|--------|
| Linux (native) | x86_64-linux | AppImage, .deb, .rpm, Flatpak | Phase 1 |
| Steam (Linux) | x86_64-linux-steam | SteamPipe depot | Phase 1 |
| Steam (Windows) | x86_64-windows-steam | SteamPipe depot | Phase 1 |
| Humble (Linux) | x86_64-linux | Humble widget/package | Phase 2 |
| Xbox/Microsoft | x86_64-windows-xbox | MSIXVC via GDK | Phase 2 |
| Web | wasm32 | itch.io / direct | Phase 3 |
| macOS | aarch64-apple | .app bundle | Phase 3 (if Huxley available) |

**Phase 1** = prove the pipeline end-to-end with one real game on Linux + Steam.
**Phase 2** = expand to all target stores.
**Phase 3** = web + macOS.

---

## 7. Development Phases

### Phase 0: Foundation (now → 2 weeks)
- [ ] Operator approves this architecture
- [ ] Repo scaffold (Cargo workspace, CI, AGENTS.md)
- [ ] `forge-core` + `forge-cli` skeleton
- [ ] `forge.toml` schema v1
- [ ] ADR-001: Rust + cargo workspace
- [ ] ADR-002: Bevy as Forge2D foundation (if approved)
- [ ] ADR-003: UE5 adapter via UAT subprocess

### Phase 1: First Build Pipeline (2-6 weeks)
- [ ] UE5 adapter: `forge build --engine ue5` invokes UAT, produces Linux build
- [ ] Forge2D adapter: `forge build --engine forge2d` produces Linux build
- [ ] `forge publish --target linux-native` → AppImage
- [ ] `forge publish --target steam` → SteamPipe upload (dry-run first)
- [ ] One real test game built and published (even a minimal one)
- [ ] Integration tests on Windows CI

### Phase 2: Mod System + More Stores (6-12 weeks)
- [ ] Mod SDK: `mod.toml` schema, hot-loading, Lua scripting
- [ ] Mod loader runtime: dependency resolution, sandboxing
- [ ] Universal mod loader: first game profile (pick one game you own)
- [ ] Humble publish plugin
- [ ] Xbox publish plugin (requires GDK on Windows)
- [ ] Mod conflict detection + load order

### Phase 3: Version System + Polish (12+ weeks)
- [ ] `forge lock` lockfile generation
- [ ] `forge upgrade` with compatibility checks
- [ ] Asset store with Blender pipeline integration
- [ ] Save-file migration system
- [ ] Web target (wasm32)
- [ ] More universal mod loader game profiles

---

## 8. Windows as Primary Machine

Windows is the primary development machine because:
- UE5 is installed (required for UE5 adapter development and testing)
- Good GPU (required for UE5 rendering and Forge2D wgpu testing)
- Blender installed (required for asset pipeline)
- Rust/cargo, Node, .NET, Python all confirmed installed
- Already has self-hosted GitHub Actions runner capability

**Linux** (this machine) serves as:
- Secondary development (Rust code compiles here, CLI work)
- Documentation, planning, agent coordination
- CI for non-GPU tests (forge-core, forge-cli, forge-version)

**Workflow:**
- Rust code: develop on Linux or Windows, cargo workspace is portable
- UE5 work: develop on Windows (UE5 not on Linux)
- Asset work: develop on Windows (Blender not on Linux)
- Testing: cargo test on both; UE5 integration tests on Windows only

---

## 9. Authority & Governance

Per the the games repository AGENTS.md invariants (which inform this repo's posture):

- **Operator (the operator)** retains: architectural authority, licensing decisions, publication authority, promotion authority
- **an agent (this agent)** may: draft architecture, write code, test, propose, open PRs
- **an agent may NOT**: choose the permanent architecture without approval, choose licenses, make public claims, import third-party game code without approval, merge own PRs

This proposal is a **draft for operator approval**. Nothing here is canonical until you say yes.

---

## 10. Open Questions for Operator

1. **Bevy vs from-scratch for Forge2D?** I recommend Bevy (faster start, MIT-licensed, mature). Your call.
2. **First universal mod loader target?** Which game do you want mod support for first? (Skyrim, Minecraft, Factorio, Stardew Valley, other?)
3. **First real game to build?** What's the simplest game you want to ship through this pipeline first? Even a minimal one proves the pipeline end-to-end.
4. **Steam developer account?** Do you have a Steamworks partner account, or is that a prerequisite to acquire?
5. **Xbox/Microsoft developer account?** Same question for ID@Xbox / Partner Center.
6. **Repo location?** New repo `hummbl-forge-engine` under `hummbl-io` org? Or a different name/location?
7. **License?** I will NOT choose this. What license do you want for the engine code?

---

## 11. What I Will Do When You Approve

1. Create the repo with the structure in §5
2. Write AGENTS.md with authority constraints
3. Scaffold the Cargo workspace
4. Implement `forge-core` + `forge-cli` skeleton
5. Write ADRs for the foundational decisions
6. Set up CI on self-hosted Windows runner
7. Begin Phase 1: UE5 adapter + first build pipeline

All of this is code I can write on Linux. UE5-specific testing happens on Windows when it's awake.

---

## Changelog

### v0.1 — 2026-09-01
- Initial architecture proposal
- Awaiting operator approval
