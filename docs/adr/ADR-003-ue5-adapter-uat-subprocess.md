# ADR-003: UE5 Adapter via UAT Subprocess

**Status:** Approved (operator-approved 2026-09-01)
**Date:** 2026-09-01

## Context

Forge Engine needs to build UE5 games as part of its pipeline. UE5 is a
massive C++ engine with its own build system (UnrealBuildTool), automation
framework (Unreal Automation Tool), and project format (.uproject). The
question is how Forge Engine communicates with UE5.

## Decision

The UE5 adapter communicates with UE5 via **subprocess invocation of
Unreal Automation Tool (UAT)** and **UnrealBuildTool (UBT)**.

## Rationale

1. **UAT is Epic's official automation pipeline.** It is the supported way
   to build, cook, package, and deploy UE5 projects programmatically.
   Using anything else would be fighting the engine.

2. **No FFI complexity.** UE5 is C++, Forge Engine is Rust. A direct FFI
   binding would be enormously complex, fragile across UE5 versions, and
   would require building UE5 engine modules as libraries. Subprocess
   invocation is simple and robust.

3. **UE5 version agnostic.** UAT's command-line interface is stable across
   UE5 versions. The adapter works with UE5 5.3, 5.4, 5.5+ without code
   changes (only the engine path changes).

4. **Full feature access.** UAT exposes everything: BuildCookRun, packaging,
   staging, archiving, plugin generation, project validation. We don't lose
   any UE5 capability by going through UAT.

5. **Error isolation.** If a UE5 build fails, the subprocess exits with a
   non-zero code and stderr output. The adapter can capture and report this
   without the failure affecting the Forge Engine process.

## Consequences

- UE5 builds require UE5 to be installed on the build machine (Anvil).
  This is already the case.
- Build output parsing depends on UAT's stdout/stderr format, which could
  change between versions. We'll need to handle this gracefully.
- Subprocess invocation has some overhead (process spawn), but this is
  negligible compared to UE5 build times (minutes to hours).

## Implementation

```
forge build --engine ue5 --platform linux-steam
    │
    ▼
forge-adapter-ue5::Ue5Adapter::build()
    │
    ├── resolves .uproject path from forge.toml
    ├── resolves UE5 engine path (auto-detect or from forge.toml)
    ├── constructs UAT command:
    │     RunUAT BuildCookRun
    │       -project=/path/to/Game.uproject
    │       -platform=Linux
    │       -clientconfig=Shipping
    │       -cook -build -stage -pak -archive
    │       -archivedirectory=/path/to/output
    │
    └── spawns subprocess, captures stdout/stderr, reports result
```

## Alternatives considered

- **Rust FFI to UE5 C++ modules:** Extremely complex, fragile, would require
  building UE5 as libraries. Not worth it.
- **UE5 plugin that calls Forge Engine:** Inverts the control flow. UE5
  would need to know about Forge Engine, creating a circular dependency.
- **Project-specific build scripts:** No abstraction, every game needs its
  own build script. Defeats the purpose of the meta-layer.
