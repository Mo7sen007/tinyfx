use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    /// Path to the HTML entry (relative to project root)
    pub entry_html: String,
    /// Path to the TS entry (relative to project root)
    pub entry_ts: String,
    /// Output directory
    pub out_dir: String,
    /// Components directory
    pub components: String,
    /// Styles directory
    #[serde(default = "default_styles")]
    pub styles: String,
    /// Static assets to copy verbatim
    #[serde(default)]
    pub static_dirs: Vec<String>,
    /// Directory for user-managed assets that should be copied to the output
    #[serde(default = "default_assets_dir")]
    pub assets_dir: String,
    ///For Debugging
    pub pahse_one_only: bool,
}

fn default_styles() -> String {
    "src/styles".to_string()
}

fn default_assets_dir() -> String {
    "src/assets".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            entry_html: "src/index.html".to_string(),
            entry_ts: "src/main.ts".to_string(),
            out_dir: "dist".to_string(),
            components: "src/components".to_string(),
            styles: "src/styles".to_string(),
            static_dirs: vec![],
            assets_dir: default_assets_dir(),
            pahse_one_only: false,
        }
    }
}

pub fn load_config(path: &Path) -> Result<Config> {
    if !path.exists() {
        return Ok(Config::default());
    }

    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read config at {}", path.display()))?;
    let cfg: Config = toml::from_str(&content)
        .with_context(|| format!("Failed to parse TOML at {}", path.display()))?;
    Ok(cfg)
}
