mod component;
mod glue;
pub mod pages;

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use regex::Regex;

use crate::config::Config;

pub fn run_build(cfg: &Config) -> Result<()> {
    println!("tinyfx build");
    println!();

    // ══════════════════════════════════════════════════════════════
    // Phase 1: Component compilation
    // ══════════════════════════════════════════════════════════════
    println!("Phase 1: Component compilation");

    let out_dir = PathBuf::from(&cfg.out_dir);
    let components_dir = PathBuf::from(&cfg.components);

    // ── Validate inputs ──────────────────────────────────────────
    let input_html = PathBuf::from(&cfg.entry_html);
    let pages_dir = PathBuf::from(&cfg.pages);
    if !components_dir.exists() {
        bail!(
            "error: components directory not found\n  → {}",
            components_dir.display()
        );
    }
    if !input_html.exists() && !pages_dir.exists() {
        bail!(
            "error: no source entry found\n  → expected {} or {}",
            input_html.display(),
            pages_dir.display()
        );
    }

    // ── Create output directories ────────────────────────────────
    let out_js = out_dir.join("js");
    let out_css = out_dir.join("css");
    fs::create_dir_all(&out_dir)
        .with_context(|| format!("Failed to create {}", out_dir.display()))?;
    fs::create_dir_all(&out_js)
        .with_context(|| format!("Failed to create {}", out_js.display()))?;
    fs::create_dir_all(&out_css)
        .with_context(|| format!("Failed to create {}", out_css.display()))?;

    let mut used_components = discover_components(&components_dir)?;
    println!("  ✔ discovered {} component(s)", used_components.len());

    // ══════════════════════════════════════════════════════════════
    // Phase 2: Page processing
    // ══════════════════════════════════════════════════════════════
    println!("Phase 2: Page processing");

    let boilerplate = fs::read_to_string(&input_html).unwrap_or_default();

    if input_html.exists() {
        let html = fs::read_to_string(&input_html)
            .with_context(|| format!("Failed to read {}", input_html.display()))?;
        let compiled_html = component::resolve_all(&html, &components_dir, &mut used_components)?;
        let final_html = rewrite_html_paths(&compiled_html);

        let out_html_path = out_dir.join("index.html");
        fs::write(&out_html_path, &final_html)
            .with_context(|| format!("Failed to write {}", out_html_path.display()))?;
        println!("  ✔ {}", out_html_path.display());
    }

    // Resolve pages so the route map can be baked into generated glue.
    let resolved_pages =
        pages::resolve_pages(&pages_dir, &out_dir, &components_dir, &mut used_components).with_context(|| {
            format!("Failed to resolve pages in {}", pages_dir.display())
        })?;

    if !resolved_pages.is_empty() {
        for page in &resolved_pages {
            pages::write_page(page, &boilerplate).with_context(|| {
                format!("Failed to write page '{}'", page.name)
            })?;
            println!("  ✔ {} ({})", page.out_path.display(), page.route);
        }
    } else {
        println!("  · no pages found in {}", pages_dir.display());
    }

    let entry_ts = PathBuf::from(&cfg.entry_ts);
    let src_dir = entry_ts.parent().unwrap_or(Path::new("."));
    let gen_ts = glue::generate_registry(&used_components, &components_dir, src_dir, &resolved_pages);
    let gen_ts_path = src_dir.join("tinyfx.gen.ts");
    fs::write(&gen_ts_path, &gen_ts)
        .with_context(|| format!("Failed to write {}", gen_ts_path.display()))?;
    println!("  ✔ {} (generated glue)", gen_ts_path.display());

    if !used_components.is_empty() {
        let mut names: Vec<&String> = used_components.iter().collect();
        names.sort();
        println!(
            "  components: {}",
            names
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    println!();
    if cfg.pahse_one_only {
        println!("Stopped after phase 1(flag).");
        //let _ = fs::remove_file(&gen_ts_path);
        return Ok(());
    }

    // ══════════════════════════════════════════════════════════════
    // Phase 3: Asset pipeline (delegated to external tools)
    // ══════════════════════════════════════════════════════════════
    println!("Phase 3: Asset pipeline");

    // ── 2a. Bundle JS/TS with esbuild ────────────────────────────
    let entry_path = PathBuf::from(&cfg.entry_ts);
    if entry_path.exists() {
        run_esbuild(&entry_path, &gen_ts_path, &out_js)?;
        println!("  ✔ {}/main.js (bundled)", out_js.display());
    } else {
        println!(
            "  · skipped JS bundling (entry not found: {})",
            cfg.entry_ts
        );
    }

    // ── 2b. Copy CSS ─────────────────────────────────────────────
    let styles_dir = PathBuf::from(&cfg.styles);
    if styles_dir.exists() {
        copy_dir_contents(&styles_dir, &out_css)?;
        println!("  ✔ {}/  (copied styles)", out_css.display());
    } else {
        println!("  · skipped CSS (styles dir not found: {})", cfg.styles);
    }

    // ── 2c. Copy static assets ───────────────────────────────────
    for static_dir in &cfg.static_dirs {
        let src_static = PathBuf::from(static_dir);
        if src_static.exists() {
            let dest = out_dir.join(src_static.file_name().unwrap_or_default());
            copy_dir_contents(&src_static, &dest)?;
            println!("  ✔ {}/  (copied static)", dest.display());
        }
    }

    // ── 2d. Copy assets directory ─────────────────────────────────
    let assets_src = PathBuf::from(&cfg.assets_dir);
    if assets_src.exists() {
        let dest = out_dir.join(
            assets_src
                .file_name()
                .unwrap_or_else(|| assets_src.as_os_str()),
        );
        copy_dir_contents(&assets_src, &dest)?;
        println!("  ✔ {}/  (copied assets)", dest.display());
    } else {
        println!("  · skipped assets (not found: {})", cfg.assets_dir);
    }

    // ══════════════════════════════════════════════════════════════
    // Phase 4: Glue generation
    // ══════════════════════════════════════════════════════════════
    println!("Phase 4: Glue generation");
    println!("  ✔ {}", gen_ts_path.display());

    // ── Cleanup: remove intermediate .ts file ────────────────────
    let _ = fs::remove_file(&gen_ts_path);

    println!();
    println!("Done! Output in {}/", out_dir.display());
    println!();

    Ok(())
}

fn discover_components(components_dir: &Path) -> Result<HashSet<String>> {
    let mut discovered = HashSet::new();
    collect_components(components_dir, &mut discovered)?;
    Ok(discovered)
}

fn collect_components(dir: &Path, discovered: &mut HashSet<String>) -> Result<()> {
    for entry in fs::read_dir(dir).with_context(|| format!("Failed to read {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            collect_components(&path, discovered)?;
            continue;
        }

        if path.extension().and_then(|ext| ext.to_str()) != Some("html") {
            continue;
        }

        if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
            if name
                .chars()
                .next()
                .map(|ch| ch.is_ascii_uppercase())
                .unwrap_or(false)
            {
                discovered.insert(name.to_string());
            }
        }
    }

    Ok(())
}

/// Rewrite HTML paths for production output structure:
/// - ./styles/main.css → ./css/main.css
/// - ./main.ts → ./js/main.js
fn rewrite_html_paths(html: &str) -> String {
    let mut result = html.to_string();

    // Rewrite CSS: ./styles/*.css → ./css/*.css
    let css_re = Regex::new(r#"href="\.?/?styles/([^"]+\.css)""#).unwrap();
    result = css_re
        .replace_all(&result, r#"href="./css/$1""#)
        .to_string();

    // Rewrite JS: ./main.ts or src="main.ts" → ./js/main.js
    let js_re = Regex::new(r#"src="\.?/?[^"]*\.ts""#).unwrap();
    result = js_re
        .replace_all(&result, r#"src="./js/main.js""#)
        .to_string();

    result
}

/// Run esbuild to bundle the entry + generated glue into dist/js/main.js
fn run_esbuild(entry: &Path, gen_ts: &Path, out_js: &Path) -> Result<()> {
    let out_file = out_js.join("main.js");

    // Create a temporary entry that imports both the glue and the user entry
    let tmp_entry = out_js.parent().unwrap_or(Path::new(".")).join("_entry.ts");
    let entry_content = format!(
        r#"import "{}";
import "{}";
"#,
        gen_ts.canonicalize()?.display(),
        entry.canonicalize()?.display()
    );
    fs::write(&tmp_entry, &entry_content)
        .with_context(|| "Failed to write temporary entry file")?;

    // Try to find esbuild: local node_modules first, then global, then npx
    let esbuild_cmd = find_esbuild()?;

    // If using npx, add 'esbuild' as the next argument
    let mut cmd = if esbuild_cmd == "npx" {
        let mut c = Command::new("npx");
        c.arg("esbuild");
        c
    } else {
        Command::new(&esbuild_cmd)
    };

    let status = cmd
        .arg(&tmp_entry)
        .arg("--bundle")
        .arg(format!("--outfile={}", out_file.display()))
        .arg("--platform=browser")
        .arg("--format=esm")
        .arg("--alias:@models=./src/lib/models")
        .arg("--alias:@dto=./src/lib/dto")
        .arg("--alias:@state=./src/lib/state")
        .arg("--alias:@services=./src/lib/services")
        .arg("--minify")
        .status()
        .with_context(|| format!("Failed to run esbuild ({})", esbuild_cmd))?;

    // Cleanup temp entry
    let _ = fs::remove_file(&tmp_entry);

    if !status.success() {
        bail!("esbuild failed with exit code: {:?}", status.code());
    }

    Ok(())
}

/// Find esbuild binary: check local node_modules, then PATH, then use npx
fn find_esbuild() -> Result<String> {
    // Check local node_modules/.bin/esbuild (relative to cwd)
    let local = Path::new("node_modules/.bin/esbuild");
    if local.exists() {
        return Ok(local.to_string_lossy().to_string());
    }

    // Check if esbuild is in PATH (globally installed)
    if let Ok(output) = Command::new("which").arg("esbuild").output() {
        if output.status.success() {
            return Ok("esbuild".to_string());
        }
    }

    // Fall back to npx (will search parent dirs or auto-install)
    if let Ok(output) = Command::new("which").arg("npx").output() {
        if output.status.success() {
            return Ok("npx".to_string());
        }
    }

    bail!(
        "esbuild not found. Install it with:\n  npm install --save-dev esbuild\nor globally:\n  npm install -g esbuild"
    )
}

/// Recursively copy directory contents
fn copy_dir_contents(src: &Path, dest: &Path) -> Result<()> {
    fs::create_dir_all(dest).with_context(|| format!("Failed to create {}", dest.display()))?;

    for entry in fs::read_dir(src).with_context(|| format!("Failed to read {}", src.display()))? {
        let entry = entry?;
        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_contents(&src_path, &dest_path)?;
        } else {
            fs::copy(&src_path, &dest_path).with_context(|| {
                format!(
                    "Failed to copy {} → {}",
                    src_path.display(),
                    dest_path.display()
                )
            })?;
        }
    }

    Ok(())
}
