use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub src: String,
    pub out: String,
    pub components: String,
    pub entry: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            src: "frontend".to_string(),
            out: "dist".to_string(),
            components: "frontend/components".to_string(),
            entry: "frontend/main.ts".to_string(),
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

pub fn ensure_out_dirs(out_dir: &str) -> Result<()> {
    let base = Path::new(out_dir);
    for sub in &["html", "css", "js"] {
        fs::create_dir_all(base.join(sub))
            .with_context(|| format!("Failed to create {}/{}", out_dir, sub))?;
    }
    Ok(())
}
