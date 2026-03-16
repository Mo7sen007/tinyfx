//! # config
//!
//! TinyFX configuration model and `tinyfx.toml` loading.
//!
//! ## Responsibilities
//! - Define the compiler configuration shape
//! - Provide defaults for missing config keys
//! - Parse `tinyfx.toml` into a `Config`
//!
//! ## What this module does NOT do
//! - It does not run build phases
//! - It does not scaffold projects
//! - It does not resolve pages or components

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

/// Compiler configuration loaded from `tinyfx.toml`.
///
/// Each field controls either an input location, an output location, or a
/// build-time option used by `tinyfx build` and `tinyfx serve`.
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    #[serde(default = "default_entry_root")]
    pub entry: String,
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

/// Load `tinyfx.toml` from `path`.
///
/// If the file does not exist, this returns [`Config::default`]. If the file
/// exists, this function reads it and deserializes it into a [`Config`].
///
/// # Errors
/// Returns an error when the file exists but cannot be read or when its TOML
/// contents do not match the current [`Config`] schema.
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
