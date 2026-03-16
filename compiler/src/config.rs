use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    #[serde(default = "default_entry_root")]
    pub entry : String,
    /// Path to the HTML entry (relative to project root)
    #[serde(default = "default_entry_html")]
    pub entry_html: String,
    /// Path to the TS entry (relative to project root)
    #[serde(default = "default_entry_ts")]
    pub entry_ts: String,
    /// Output directory
    #[serde(default = "default_out_dir")]
    pub out_dir: String,
    /// Components directory
    #[serde(default = "default_components_dir")]
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
    /// Pages directory for the file-based router
    #[serde(default = "default_pages_dir")]
    pub pages: String,
    ///For Debugging
    #[serde(default)]
    pub phase_one_only: bool,
}

fn default_entry_root() -> String {
    "src".to_string()
}

fn default_entry_html() -> String {
    "src/index.html".to_string()
}

fn default_entry_ts() -> String {
    "src/main.ts".to_string()
}

fn default_out_dir() -> String {
    "dist".to_string()
}

fn default_components_dir() -> String {
    "src/components".to_string()
}

fn default_styles() -> String {
    "src/styles".to_string()
}

fn default_assets_dir() -> String {
    "src/assets".to_string()
}

fn default_pages_dir() -> String {
    "src/pages".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            entry: default_entry_root(),
            entry_html: default_entry_html(),
            entry_ts: default_entry_ts(),
            out_dir: default_out_dir(),
            components: default_components_dir(),
            styles: "src/styles".to_string(),
            static_dirs: vec![],
            assets_dir: default_assets_dir(),
            pages: default_pages_dir(),
            phase_one_only: false,
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
