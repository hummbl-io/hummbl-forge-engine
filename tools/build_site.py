#!/usr/bin/env python3
"""
Build script for hummbl-forge-engine edge surface (forge.hummbl.dev).
Compiles game meta-engine architecture, 8 core Rust crates, target matrix,
and interactive build/mod pipeline simulator into a zero-dependency static SPA with JSON CDN endpoints.
"""

import json
import os
import shutil
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
PUBLIC_DIR = REPO_ROOT / "public"
API_DIR = PUBLIC_DIR / "api"

CRATES = [
    {
        "name": "forge-core",
        "role": "Core Primitives & Config",
        "description": "Engine abstraction traits, project metadata parsing (forge.toml), and pipeline lifecycle hooks."
    },
    {
        "name": "forge-adapter",
        "role": "Runtime Engine Adapters",
        "description": "Unified adapter interfaces bridging Unreal Engine 5 (UE5) C++ workflows and Bevy/Forge2D ECS environments."
    },
    {
        "name": "forge-build",
        "role": "Multi-Target Compiler",
        "description": "Cross-compilation orchestrator targeting Linux native, Windows x86_64, macOS Apple Silicon, and WebAssembly."
    },
    {
        "name": "forge-publish",
        "role": "Store Distribution",
        "description": "Automated artifact packaging, digital signature hashing, and direct publishing to Steam, itch.io, and Xbox."
    },
    {
        "name": "forge-asset",
        "role": "Content-Addressable Asset Store",
        "description": "Blender to glTF export pipeline, texture compression (KTX2/Basis), and deterministic asset lockfiles."
    },
    {
        "name": "forge-mod",
        "role": "Universal Modding & Sandboxing",
        "description": "Isolated mod loader supporting custom game extensions and universal injection for Factorio, Minecraft, and Bevy."
    },
    {
        "name": "forge-version",
        "role": "Lockfile & Coherence Engine",
        "description": "Coordinated version resolution guaranteeing lockstep compatibility across engine, game, mod, and assets."
    },
    {
        "name": "forge-cli",
        "role": "Unified Operator Interface",
        "description": "Command-line tool providing `forge init`, `forge check`, `forge build`, `forge mod`, and `forge publish`."
    }
]

BUILD_TARGETS = [
    {"platform": "Linux Native", "engine": "Bevy / Forge2D", "format": "ELF Binary + Assets", "status": "Proven End-to-End"},
    {"platform": "Windows x86_64", "engine": "UE5 / Bevy", "format": "PE32+ Executable", "status": "Supported"},
    {"platform": "WebAssembly (Wasm)", "engine": "Bevy / WebGL2", "format": "Wasm + Canvas", "status": "Active Target"},
    {"platform": "Steam / itch.io", "engine": "Multi-Engine", "format": "Store Deployment Bundle", "status": "Automated"}
]

def generate_index_html() -> str:
    crates_json = json.dumps(CRATES)
    targets_json = json.dumps(BUILD_TARGETS)

    return f"""<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>HUMMBL Forge Engine | forge.hummbl.dev</title>
  <meta name="description" content="Meta-engine layer for building, publishing, and modding games across multiple runtime engines at forge.hummbl.dev">
  <link rel="icon" href="data:image/svg+xml,<svg xmlns=%22http://www.w3.org/2000/svg%22 viewBox=%220 0 100 100%22><text y=%22.9em%22 font-size=%2290%22>⚔️</text></svg>">
  <style>
    :root {{
      --bg: #0b0f19;
      --bg-panel: #131b2e;
      --bg-hover: #1c2742;
      --border: #1e2c4a;
      --border-accent: #2e4776;
      --text: #f0f6fc;
      --text-muted: #8c9eb5;
      --orange: #f97316;
      --orange-glow: rgba(249, 115, 22, 0.2);
      --amber: #f59e0b;
      --emerald: #10b981;
      --cyan: #06b6d4;
      --indigo: #6366f1;
      --font-sans: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
      --font-mono: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    }}

    * {{
      box-sizing: border-box;
      margin: 0;
      padding: 0;
    }}

    body {{
      background: var(--bg);
      color: var(--text);
      font-family: var(--font-sans);
      line-height: 1.5;
      min-height: 100vh;
      display: flex;
      flex-direction: column;
    }}

    header {{
      background: rgba(19, 27, 46, 0.85);
      backdrop-filter: blur(12px);
      border-bottom: 1px solid var(--border);
      position: sticky;
      top: 0;
      z-index: 50;
      padding: 1rem 1.5rem;
    }}

    .header-content {{
      max-width: 1200px;
      margin: 0 auto;
      display: flex;
      justify-content: space-between;
      align-items: center;
      flex-wrap: wrap;
      gap: 1rem;
    }}

    .brand {{
      display: flex;
      align-items: center;
      gap: 0.75rem;
    }}

    .logo-badge {{
      width: 40px;
      height: 40px;
      border-radius: 8px;
      background: linear-gradient(135deg, #f97316, #f59e0b);
      display: flex;
      align-items: center;
      justify-content: center;
      font-size: 1.25rem;
      box-shadow: 0 0 15px var(--orange-glow);
    }}

    .brand-title {{
      font-size: 1.15rem;
      font-weight: 700;
      letter-spacing: -0.02em;
    }}

    .brand-sub {{
      font-size: 0.8rem;
      color: var(--text-muted);
      font-family: var(--font-mono);
    }}

    .status-badge {{
      display: inline-flex;
      align-items: center;
      gap: 0.5rem;
      background: rgba(249, 115, 22, 0.1);
      border: 1px solid var(--orange);
      color: #fdba74;
      padding: 0.25rem 0.75rem;
      border-radius: 9999px;
      font-size: 0.75rem;
      font-weight: 600;
      font-family: var(--font-mono);
    }}

    .pulse-dot {{
      width: 8px;
      height: 8px;
      border-radius: 50%;
      background-color: var(--orange);
      box-shadow: 0 0 8px var(--orange);
      animation: pulse 2s infinite;
    }}

    @keyframes pulse {{
      0%, 100% {{ opacity: 1; transform: scale(1); }}
      50% {{ opacity: 0.4; transform: scale(0.85); }}
    }}

    main {{
      max-width: 1200px;
      margin: 0 auto;
      padding: 2rem 1.5rem;
      flex: 1;
      width: 100%;
    }}

    .hero {{
      margin-bottom: 2.5rem;
    }}

    .hero-badge {{
      display: inline-block;
      font-size: 0.75rem;
      font-family: var(--font-mono);
      text-transform: uppercase;
      letter-spacing: 0.05em;
      color: var(--orange);
      background: rgba(249, 115, 22, 0.1);
      border: 1px solid rgba(249, 115, 22, 0.3);
      padding: 0.2rem 0.6rem;
      border-radius: 4px;
      margin-bottom: 0.75rem;
    }}

    .hero h1 {{
      font-size: 2.25rem;
      font-weight: 800;
      letter-spacing: -0.03em;
      margin-bottom: 0.75rem;
      background: linear-gradient(135deg, #ffffff 60%, #fdba74);
      -webkit-background-clip: text;
      -webkit-text-fill-color: transparent;
    }}

    .hero p {{
      color: var(--text-muted);
      font-size: 1.05rem;
      max-width: 820px;
    }}

    .section-title {{
      font-size: 1.35rem;
      font-weight: 700;
      margin-bottom: 1.25rem;
      display: flex;
      align-items: center;
      gap: 0.5rem;
    }}

    /* Crates Grid */
    .crates-grid {{
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
      gap: 1.25rem;
      margin-bottom: 2.5rem;
    }}

    .crate-card {{
      background: var(--bg-panel);
      border: 1px solid var(--border);
      border-radius: 10px;
      padding: 1.25rem;
      display: flex;
      flex-direction: column;
      justify-content: space-between;
      transition: all 0.2s ease;
    }}

    .crate-card:hover {{
      border-color: var(--orange);
      transform: translateY(-2px);
    }}

    .crate-name {{
      font-family: var(--font-mono);
      font-size: 1.05rem;
      font-weight: 700;
      color: var(--orange);
      margin-bottom: 0.25rem;
    }}

    .crate-role {{
      font-size: 0.75rem;
      color: var(--cyan);
      font-family: var(--font-mono);
      margin-bottom: 0.5rem;
    }}

    .crate-desc {{
      font-size: 0.8rem;
      color: var(--text-muted);
      line-height: 1.4;
    }}

    /* Interactive Simulator */
    .simulator-grid {{
      display: grid;
      grid-template-columns: 1fr 1fr;
      gap: 1.5rem;
      margin-bottom: 2.5rem;
    }}

    @media (max-width: 900px) {{
      .simulator-grid {{
        grid-template-columns: 1fr;
      }}
    }}

    .panel {{
      background: var(--bg-panel);
      border: 1px solid var(--border);
      border-radius: 10px;
      padding: 1.5rem;
    }}

    .panel-head {{
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 1.25rem;
    }}

    .panel-head h3 {{
      font-size: 1.1rem;
      font-weight: 700;
    }}

    .form-group {{
      margin-bottom: 1rem;
    }}

    .form-label {{
      display: block;
      font-size: 0.8rem;
      color: var(--text-muted);
      margin-bottom: 0.35rem;
      font-family: var(--font-mono);
      text-transform: uppercase;
    }}

    .form-control {{
      width: 100%;
      background: var(--bg);
      border: 1px solid var(--border);
      color: var(--text);
      font-family: var(--font-mono);
      font-size: 0.85rem;
      padding: 0.6rem 0.75rem;
      border-radius: 6px;
      outline: none;
    }}

    .form-control:focus {{
      border-color: var(--orange);
    }}

    .btn {{
      background: linear-gradient(135deg, #f97316, #ea580c);
      color: #fff;
      font-weight: 700;
      border: none;
      padding: 0.65rem 1.25rem;
      border-radius: 6px;
      cursor: pointer;
      font-size: 0.85rem;
      font-family: var(--font-mono);
      transition: opacity 0.2s;
      width: 100%;
    }}

    .btn:hover {{
      opacity: 0.9;
    }}

    .sim-receipt {{
      background: var(--bg);
      border: 1px solid var(--border);
      border-radius: 8px;
      padding: 1rem;
      font-family: var(--font-mono);
      font-size: 0.85rem;
    }}

    .sim-row {{
      display: flex;
      justify-content: space-between;
      margin-bottom: 0.5rem;
    }}

    /* Targets Grid */
    .targets-grid {{
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
      gap: 1.25rem;
      margin-bottom: 2.5rem;
    }}

    .target-card {{
      background: var(--bg-panel);
      border: 1px solid var(--border);
      border-radius: 8px;
      padding: 1.25rem;
    }}

    .target-title {{
      font-weight: 700;
      font-size: 1rem;
      color: var(--text);
      margin-bottom: 0.25rem;
    }}

    .target-engine {{
      font-family: var(--font-mono);
      font-size: 0.75rem;
      color: var(--cyan);
      margin-bottom: 0.5rem;
    }}

    .target-desc {{
      font-size: 0.8rem;
      color: var(--text-muted);
    }}

    .cdn-box {{
      background: #060911;
      border: 1px solid var(--border);
      border-radius: 8px;
      padding: 1.25rem;
      font-family: var(--font-mono);
      font-size: 0.85rem;
      margin-bottom: 2rem;
    }}

    .cdn-links {{
      display: flex;
      gap: 1rem;
      flex-wrap: wrap;
      margin-top: 0.75rem;
    }}

    .cdn-link {{
      color: var(--orange);
      text-decoration: none;
      background: rgba(249, 115, 22, 0.1);
      padding: 0.3rem 0.6rem;
      border-radius: 4px;
      border: 1px solid rgba(249, 115, 22, 0.3);
      font-size: 0.8rem;
    }}

    .cdn-link:hover {{
      background: rgba(249, 115, 22, 0.2);
    }}

    footer {{
      background: #060911;
      border-top: 1px solid var(--border);
      padding: 1.5rem;
      font-size: 0.8rem;
      color: var(--text-muted);
      text-align: center;
    }}
  </style>
</head>
<body>
  <header>
    <div class="header-content">
      <div class="brand">
        <div class="logo-badge">⚔️</div>
        <div>
          <div class="brand-title">HUMMBL Forge Engine</div>
          <div class="brand-sub">forge.hummbl.dev • Game Meta-Engine &amp; Universal Modding Pipeline</div>
        </div>
      </div>
      <div class="status-badge">
        <div class="pulse-dot"></div>
        <span>CORE ENGINE ONLINE</span>
      </div>
    </div>
  </header>

  <main id="app">
    <section class="hero">
      <div class="hero-badge">Multi-Runtime Meta-Engine</div>
      <h1>Unified Game Pipeline &amp; Mod Framework</h1>
      <p>A declarative Rust meta-engine layer sitting above UE5 and Bevy: unified multi-target builds, store publishing, content-addressable asset hashing, and universal mod sandboxing.</p>
    </section>

    <!-- Crates Grid -->
    <h2 class="section-title">The 8 Modular Crates</h2>
    <div class="crates-grid" id="crates-container">
      <!-- Injected by JS -->
    </div>

    <!-- Interactive Pipeline Simulator -->
    <h2 class="section-title">Simulate Multi-Target Build &amp; Modding Pipeline</h2>
    <div class="simulator-grid">
      <div class="panel">
        <div class="panel-head">
          <h3>Configure Build Pipeline</h3>
          <span style="font-family: var(--font-mono); font-size: 0.8rem; color: var(--orange);">forge.toml</span>
        </div>
        <form id="sim-form" onsubmit="simulateBuild(event)">
          <div class="form-group">
            <label class="form-label">Game Engine Adapter</label>
            <select class="form-control" id="sim-engine">
              <option value="forge2d">Forge2D / Bevy (Indie 2D / ECS)</option>
              <option value="ue5">Unreal Engine 5 (AAA 3D / Nanite / Lumen)</option>
            </select>
          </div>
          <div class="form-group">
            <label class="form-label">Target Distribution Platform</label>
            <select class="form-control" id="sim-target">
              <option value="linux-native">Linux Native (ELF x86_64 Direct)</option>
              <option value="steam">Steamworks Package (Steam Pipe / Depot)</option>
              <option value="wasm">WebAssembly (Canvas / Audio / Zero Install)</option>
            </select>
          </div>
          <div class="form-group">
            <label class="form-label">Universal Mod Sandboxing</label>
            <select class="form-control" id="sim-mod">
              <option value="enabled">Enabled (Wasm isolated mod runtime)</option>
              <option value="disabled">Disabled (Vanilla game core only)</option>
            </select>
          </div>
          <button type="submit" class="btn">⚔️ Build &amp; Package Game Artifact</button>
        </form>
      </div>

      <div class="panel">
        <div class="panel-head">
          <h3>Build Manifest Receipt</h3>
          <span class="status-badge" id="build-status">READY</span>
        </div>
        <div class="sim-receipt">
          <div class="sim-row">
            <span>Target Package:</span>
            <strong id="disp-pkg" style="color: var(--orange);">Linux Native (Direct)</strong>
          </div>
          <div class="sim-row">
            <span>Runtime Engine:</span>
            <strong id="disp-eng">Forge2D (Bevy ECS)</strong>
          </div>
          <div class="sim-row">
            <span>Asset Bundle Hash:</span>
            <strong style="color: var(--cyan); font-size: 0.75rem;" id="disp-sha">8a3bc91f...</strong>
          </div>
          <div class="sim-row">
            <span>Modding Sandbox:</span>
            <strong id="disp-mod" style="color: var(--emerald);">ISOLATED (Wasm)</strong>
          </div>
          <div style="margin-top: 1rem; padding-top: 0.75rem; border-top: 1px solid var(--border); font-size: 0.8rem; color: var(--text-muted);" id="disp-desc">
            Artifact compiled and verified. All glTF assets bundled into content-addressable cache.
          </div>
        </div>
      </div>
    </div>

    <!-- Supported Targets -->
    <h2 class="section-title">Distribution Targets &amp; Engine Matrix</h2>
    <div class="targets-grid" id="targets-container">
      <!-- Injected by JS -->
    </div>

    <!-- CDN Endpoints -->
    <div class="cdn-box">
      <div style="font-weight: 700; color: var(--text); margin-bottom: 0.5rem;">⚡ Machine-Readable JSON CDN Endpoints</div>
      <p style="color: var(--text-muted); font-size: 0.8rem; margin-bottom: 0.75rem;">
        Edge schemas for automated game compilation manifests, mod loader interfaces, and engine crate catalogs.
      </p>
      <div class="cdn-links">
        <a class="cdn-link" href="/api/data.json" target="_blank">GET /api/data.json</a>
        <a class="cdn-link" href="/api/crates.json" target="_blank">GET /api/crates.json</a>
        <a class="cdn-link" href="/api/targets.json" target="_blank">GET /api/targets.json</a>
      </div>
    </div>
  </main>

  <footer>
    <p>HUMMBL Forge Engine • Zero Marginal Cost Edge Architecture • Published at forge.hummbl.dev</p>
    <p style="margin-top: 0.35rem; font-family: var(--font-mono); font-size: 0.75rem; color: var(--text-muted);">
      Node: Anvil • Core: Rust Crates • Runtimes: UE5 / Bevy / WebAssembly
    </p>
  </footer>

  <script>
    const crates = {crates_json};
    const targets = {targets_json};

    function initCrates() {{
      const container = document.getElementById('crates-container');
      container.innerHTML = crates.map(c => `
        <div class="crate-card">
          <div>
            <div class="crate-name">${{c.name}}</div>
            <div class="crate-role">${{c.role}}</div>
            <div class="crate-desc">${{c.description}}</div>
          </div>
        </div>
      `).join('');
    }}

    function initTargets() {{
      const container = document.getElementById('targets-container');
      container.innerHTML = targets.map(t => `
        <div class="target-card">
          <div class="target-title">${{t.platform}}</div>
          <div class="target-engine">Engine: ${{t.engine}} &bull; Format: ${{t.format}}</div>
          <div class="target-desc">Status: <span style="color: var(--emerald);">${{t.status}}</span></div>
        </div>
      `).join('');
    }}

    window.simulateBuild = function(e) {{
      e.preventDefault();
      const engine = document.getElementById('sim-engine').value;
      const target = document.getElementById('sim-target').value;
      const mod = document.getElementById('sim-mod').value;

      const dispPkg = document.getElementById('disp-pkg');
      const dispEng = document.getElementById('disp-eng');
      const dispMod = document.getElementById('disp-mod');
      const dispDesc = document.getElementById('disp-desc');
      const statusBadge = document.getElementById('build-status');

      dispEng.textContent = engine === 'forge2d' ? 'Forge2D (Bevy ECS)' : 'Unreal Engine 5 (UE5)';
      dispPkg.textContent = target.toUpperCase();
      dispMod.textContent = mod === 'enabled' ? 'ACTIVE (SANDBOXED)' : 'DISABLED';

      statusBadge.textContent = 'BUILD_PASS';
      statusBadge.style.borderColor = 'var(--emerald)';
      statusBadge.style.color = 'var(--emerald)';

      dispDesc.textContent = `Pipeline successfully built for ${{target}} using ${{engine}} adapter. Zero runtime missing dependencies.`;

      const randHex = Array.from({{length: 64}}, () => Math.floor(Math.random() * 16).toString(16)).join('');
      document.getElementById('disp-sha').textContent = randHex;
    }};

    document.addEventListener('DOMContentLoaded', () => {{
      initCrates();
      initTargets();
    }});
  </script>
</body>
</html>
"""

def main():
    print("[*] Building hummbl-forge-engine static SPA and JSON CDN endpoints...")
    PUBLIC_DIR.mkdir(parents=True, exist_ok=True)
    API_DIR.mkdir(parents=True, exist_ok=True)

    # 1. Generate index.html
    html_content = generate_index_html()
    (PUBLIC_DIR / "index.html").write_text(html_content.strip() + "\n", encoding="utf-8")
    print(f"    - public/index.html ({len(html_content)} bytes)")

    # 2. JSON CDN Endpoints
    master_data = {
        "name": "hummbl-forge-engine",
        "title": "HUMMBL Forge Engine",
        "subdomain": "forge.hummbl.dev",
        "version": "0.1.0",
        "crates_count": len(CRATES),
        "crates": CRATES,
        "targets": BUILD_TARGETS
    }
    (API_DIR / "data.json").write_text(json.dumps(master_data, indent=2) + "\n", encoding="utf-8")
    print("    - public/api/data.json")

    (API_DIR / "crates.json").write_text(json.dumps(CRATES, indent=2) + "\n", encoding="utf-8")
    print("    - public/api/crates.json")

    (API_DIR / "targets.json").write_text(json.dumps(BUILD_TARGETS, indent=2) + "\n", encoding="utf-8")
    print("    - public/api/targets.json")

    # 3. Security headers & manifest
    headers_content = """/*
  X-Frame-Options: DENY
  X-Content-Type-Options: nosniff
  Referrer-Policy: strict-origin-when-cross-origin
  Permissions-Policy: document-domain=(), geolocation=(), microphone=(), camera=()
  Content-Security-Policy: default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self';
  Cache-Control: public, max-age=3600

/api/*
  Access-Control-Allow-Origin: *
  Access-Control-Allow-Methods: GET, HEAD, OPTIONS
  Access-Control-Allow-Headers: Content-Type
"""
    (PUBLIC_DIR / "_headers").write_text(headers_content.strip() + "\n", encoding="utf-8")

    manifest = {
        "name": "HUMMBL Forge Engine",
        "short_name": "ForgeEngine",
        "start_url": "/",
        "display": "standalone",
        "background_color": "#0b0f19",
        "theme_color": "#f97316",
        "description": "Game meta-engine and universal modding pipeline at forge.hummbl.dev"
    }
    (PUBLIC_DIR / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    print("    - public/_headers and public/manifest.json")

    print("[+] hummbl-forge-engine build complete.")

if __name__ == "__main__":
    main()
