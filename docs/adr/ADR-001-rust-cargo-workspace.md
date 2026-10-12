# ADR-001: Rust + Cargo Workspace

**Status:** Approved (operator-approved 2026-09-01)
**Date:** 2026-09-01

## Context

Forge Engine is a meta-engine layer that orchestrates multiple runtime engines
(UE5, Forge2D/Bevy), build pipelines, publishing to multiple stores, a mod
system, and versioning. It needs to be fast, cross-platform, safe, and
maintainable over years.

## Decision

Use **Rust** as the core language with a **Cargo workspace** for project structure.

## Rationale

1. **Performance:** Rust compiles to native code with zero runtime overhead.
   The CLI (`forge`) starts instantly — no JVM, no Python interpreter, no
   Node.js startup. This matters for a tool that will be invoked frequently
   during game development.

2. **Safety:** Rust's ownership system eliminates entire classes of bugs
   (null pointers, use-after-free, data races). A tool that orchestrates
   builds and manages assets needs this.

3. **Cross-platform:** Rust compiles to Linux, Windows, macOS, and WebAssembly
   natively. Forge Engine needs to run on Windows (Windows) and Linux (Linux)
   and eventually produce builds for all those platforms.

4. **Toolchain already present:** Rust/cargo is confirmed installed on both
   Windows and Linux.

5. **Ecosystem:** Bevy (our Forge2D foundation) is Rust. wgpu (rendering),
   rapier (physics), mlua (Lua scripting) are all mature Rust crates. Using
   Rust for the meta-layer means no FFI boundary to the 2D engine.

6. **Single binary:** The `forge` CLI is a single binary that can be
   distributed easily — no runtime dependencies to install.

7. **Workspace structure:** Cargo workspaces allow multiple crates to share
   a lockfile, dependencies, and build cache while maintaining clean
   separation of concerns.

## Consequences

- All Forge Engine code is Rust. Lua is used for mod scripting (embedded
  via mlua), but the engine itself is Rust.
- UE5 adapter communicates with UE5 via subprocess (UAT/UBT) — no Rust-UE5
  FFI needed.
- Contributors need Rust toolchain knowledge.
- Compilation is slower than interpreted languages, but `cargo check` is
  fast enough for development iteration.

## Alternatives considered

- **Python:** Slower startup, runtime dependency, weaker type safety. Good
  for scripting but not for a tool that needs to be fast and distributable.
- **C++:** Same performance as Rust but worse safety, worse tooling, more
  development friction. UE5 is C++ but the meta-layer doesn't need to be.
- **Go:** Good for CLIs but weaker ecosystem for game development (no Bevy
  equivalent, no wgpu).
- **Node.js/TypeScript:** Runtime dependency, slower, weaker for systems
  programming tasks like asset hashing and file management.
