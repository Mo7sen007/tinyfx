use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};

const PAGES_INDEX_HTML: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/pages/index.html"));
const BUTTON_HTML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/components/Button/Button.html"
));
const BUTTON_TS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/components/Button/Button.ts"
));
const USER_DTO_TS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/lib/dto/user.dto.ts"
));
const USER_MODEL_TS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/lib/models/user.model.ts"
));
const APP_STATE_TS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/lib/state/app.state.ts"
));
const USER_SERVICE_TS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/lib/services/user.service.ts"
));
const MAIN_CSS: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/styles/main.css"));
const MAIN_TS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/main.ts"));
const TINYFX_TOML: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/tinyfx.toml"));
const TSCONFIG_JSON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/tsconfig.app.json"
));
const PACKAGE_JSON: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/package.json"));
const AGENTS_MD: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/AGENTS.md"));

fn write(path: &Path, contents: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory {}", parent.display()))?;
    }
    fs::write(path, contents).with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(())
}

pub fn run(name: &str) -> Result<()> {
    let base = Path::new(name);

    if base.exists() {
        bail!(
            "error: directory \"{}\" already exists\n  → choose a different name or remove the existing directory",
            name
        );
    }

    println!();
    println!("tinyfx init → {}/", name);
    println!();

    let dirs = [
        "src/pages",
        "src/components/Button",
        "src/lib/dto",
        "src/lib/models",
        "src/lib/state",
        "src/lib/services",
        "src/styles",
    ];

    for d in &dirs {
        let dir_path = base.join(d);
        fs::create_dir_all(&dir_path)
            .with_context(|| format!("Failed to create {}", dir_path.display()))?;
    }

    let package_json = PACKAGE_JSON.replace("__TINYFX_NAME__", name);
    let files = [
        ("src/pages/index.html", PAGES_INDEX_HTML),
        ("src/components/Button/Button.html", BUTTON_HTML),
        ("src/components/Button/Button.ts", BUTTON_TS),
        ("src/lib/dto/user.dto.ts", USER_DTO_TS),
        ("src/lib/models/user.model.ts", USER_MODEL_TS),
        ("src/lib/state/app.state.ts", APP_STATE_TS),
        ("src/lib/services/user.service.ts", USER_SERVICE_TS),
        ("src/styles/main.css", MAIN_CSS),
        ("src/main.ts", MAIN_TS),
        ("tinyfx.toml", TINYFX_TOML),
        ("tsconfig.json", TSCONFIG_JSON),
        ("package.json", &package_json),
        ("AGENTS.md", AGENTS_MD),
    ];

    for (relative_path, contents) in files {
        write(&base.join(relative_path), contents)?;
    }

    println!();
    let npm_status = Command::new("npm")
        .arg("install")
        .current_dir(base)
        .status();

    println!("✓ created {}", name);

    match npm_status {
        Ok(status) if status.success() => {
            println!("✓ installed dependencies");
        }
        Ok(status) => {
            println!("⚠ npm install failed with exit code {:?}", status.code());
            println!("  You can run it manually after changing into the project directory.");
        }
        Err(_) => {
            println!("⚠ npm not found — install dependencies manually after entering the project directory.");
        }
    }

    println!();
    println!("  cd {}", name);
    println!("  tinyfx serve");
    println!();
    println!("  → http://localhost:3000");
    println!();

    Ok(())
}

pub fn run_init(name: &str, _force: bool) -> Result<()> {
    run(name)
}
