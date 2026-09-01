# ADR-002: Bevy as Forge2D Foundation

**Status:** Approved (operator-approved 2026-09-01)
**Date:** 2026-09-01

## Context

Forge Engine supports multiple runtime engines. For 2D/indie/experimental
games, we need a lightweight engine ("Forge2D"). The decision is whether to
build this from scratch or build on top of an existing Rust game engine.

## Decision

Use **Bevy** as the foundation for Forge2D.

## Rationale

1. **Bevy is mature and actively maintained.** It is the most popular
   Rust-native game engine with a large community, regular releases, and
   extensive documentation.

2. **MIT-licensed.** No licensing restrictions on how we use or distribute it.
   Compatible with our Apache 2.0 license.

3. **We get a working 2D engine immediately.** Bevy provides:
   - Rendering via wgpu (Vulkan/Metal/DX12/WebGPU)
   - ECS (Entity Component System) for game objects
   - Input handling (keyboard, mouse, gamepad)
   - Audio playback
   - Asset loading
   - 2D and 3D rendering pipelines
   - Plugin system (we can add Forge-specific plugins)

4. **Our value is in the meta-layer, not in reinventing 2D rendering.**
   Building a 2D renderer from scratch would take months before we could
   make a single game. Bevy lets us start making games immediately.

5. **Same language as the meta-layer.** No FFI boundary between Forge Engine
   core and Forge2D — they're both Rust.

6. **Mod scripting via Lua.** Bevy can integrate with mlua for mod scripting,
   giving us hot-reloadable mod support.

## Consequences

- Forge2D is architecturally dependent on Bevy. If Bevy makes breaking
  changes between major versions, we need to adapt.
- We don't control the rendering internals. If we need a custom rendering
  feature Bevy doesn't support, we may need to fork or contribute upstream.
- Bevy's ECS paradigm (entities, components, systems) shapes how Forge2D
  games are structured. This is a reasonable constraint.

## Alternatives considered

- **From scratch (wgpu + rapier + mlua directly):** Maximum control but
  months of work before a single game could be made. Rebuilding what Bevy
  already provides. Not worth the opportunity cost.
- **Godot:** Excellent engine but not Rust. Would introduce a GDScript/CLR
  dependency and an FFI boundary. Also, Godot is a full engine — using it
  as "Forge2D" would mean wrapping a competing full engine, not building
  a lightweight component.
- **Fyrox:** Rust game engine, but less mature than Bevy and more focused
  on 3D.

## Future evolution

If we outgrow Bevy in specific areas, we can:
1. Replace individual Bevy plugins with custom implementations
2. Fork Bevy components we need to modify
3. Contribute improvements upstream

The architecture is not locked in — Bevy is a foundation, not a prison.
