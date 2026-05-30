//! # init
//!
//! Project scaffolding for `tinyfx init`.
//!
//! ## Responsibilities
//! - Create the starter directory structure
//! - Write template files for a new TinyFX project
//! - Optionally run `npm install`
//!
//! ## What this module does NOT do
//! - It does not build the project
//! - It does not generate per-resource code like `tinyfx generate`
//! - It does not serve the project

use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{bail, Context, Result};

const PAGES_INDEX_HTML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/init/pages/index.html"
));
const PAGES_INDEX_TS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/init/pages/index.ts"
));
const FEATURE_HTML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/init/components/FeatureCard/FeatureCard.html"
));
const THEME_HTML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/init/components/ThemeToggle/ThemeToggle.html"
));
const THEME_TS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/init/components/ThemeToggle/ThemeToggle.ts"
));
const THEME_STATE_TS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/init/lib/state/theme.state.ts"
));
const MODELS_GITKEEP: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/init/lib/models/.gitkeep"
));
const DTO_GITKEEP: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/init/lib/dto/.gitkeep"
));
const SERVICES_GITKEEP: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/init/lib/services/.gitkeep"
));
const MAIN_CSS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/init/styles/main.css"
));
const MAIN_TS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/init/main.ts"
));
const TINYFX_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/init/tinyfx.toml"
));
const TSCONFIG_JSON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/init/tsconfig.json"
));
const PACKAGE_JSON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/init/package.json"
));
const AGENTS_MD: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/init/AGENTS.md"
));

fn write(path: &Path, contents: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory {}", parent.display()))?;
    }
    fs::write(path, contents).with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(())
}

/// Create a new TinyFX project in a directory named `name`.
///
/// This writes the scaffold templates, creates the starter source tree, and
/// runs `npm install` inside the new project directory.
///
/// # Errors
/// Returns an error when the target directory already exists, when any
/// scaffold file cannot be written, or when `npm install` fails.
pub fn run_init(name: &str, force: bool) -> Result<()> {
    let base = Path::new(name);

    if base.exists() && !force {
        bail!(
            "error: directory \"{}\" already exists\n  → choose a different name, remove the existing directory, or pass --force to overwrite",
            name
        );
    }

    println!();
    println!("tinyfx init → {}/", name);
    println!();

    let dirs = [
        "src/pages",
        "src/components/FeatureCard",
        "src/components/ThemeToggle",
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

    // package.json uses a `{{name}}` placeholder in the template
    let package_json = PACKAGE_JSON.replace("{{name}}", name);

    let files = [
        ("src/pages/index.html", PAGES_INDEX_HTML),
        ("src/pages/index.ts", PAGES_INDEX_TS),
        ("src/components/FeatureCard/FeatureCard.html", FEATURE_HTML),
        ("src/components/ThemeToggle/ThemeToggle.html", THEME_HTML),
        ("src/components/ThemeToggle/ThemeToggle.ts", THEME_TS),
        ("src/lib/state/theme.state.ts", THEME_STATE_TS),
        ("src/lib/models/.gitkeep", MODELS_GITKEEP),
        ("src/lib/dto/.gitkeep", DTO_GITKEEP),
        ("src/lib/services/.gitkeep", SERVICES_GITKEEP),
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
            println!(
                "⚠ npm not found — install dependencies manually after entering the project directory."
            );
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
