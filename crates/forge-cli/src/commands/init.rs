//! `forge init` — initialize a new game project.

pub async fn run(name: &str, engine: &str) -> anyhow::Result<()> {
    tracing::info!("Initializing new Forge Engine project: {name} (engine: {engine})");

    let project_id = format!("com.hummbl.{}", name.to_lowercase().replace(' ', "-"));

    let forge_toml = match engine {
        "forge2d" => format!(
            r#"version = "0.1.0"

[project]
name = "{name}"
id = "{project_id}"
description = ""
authors = []

[engine]
kind = "forge2d"
cargo_toml = "Cargo.toml"

[build]
targets = ["linux-native"]
profile = "release"

[publish]
targets = []
dry_run = true

[mods]
enabled = false
sandbox = true

[assets]
assets_dir = "assets"

[assets.blender]
source_dir = "blender"
export_format = "glTF"
"#,
        ),
        "ue5" => format!(
            r#"version = "0.1.0"

[project]
name = "{name}"
id = "{project_id}"
description = ""
authors = []

[engine]
kind = "ue5"
uproject = "{name}.uproject"
version = "5.4"

[build]
targets = ["linux-steam"]
profile = "shipping"

[publish]
targets = []
dry_run = true

[mods]
enabled = false
sandbox = true

[assets]
assets_dir = "assets"

[assets.blender]
source_dir = "blender"
export_format = "glTF"
"#,
        ),
        other => anyhow::bail!("unknown engine: {other} (expected: ue5, forge2d)"),
    };

    let path = std::path::Path::new(name);
    if path.exists() {
        anyhow::bail!("directory already exists: {name}");
    }

    std::fs::create_dir_all(path)?;
    std::fs::write(path.join("forge.toml"), forge_toml)?;
    std::fs::create_dir_all(path.join("assets"))?;
    std::fs::create_dir_all(path.join("mods"))?;

    tracing::info!("Created {name}/forge.toml");
    tracing::info!("Created {name}/assets/");
    tracing::info!("Created {name}/mods/");
    tracing::info!(
        "Project initialized. Edit forge.toml to configure build targets and publishing."
    );

    Ok(())
}
