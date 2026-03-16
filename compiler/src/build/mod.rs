mod component;
mod glue;
pub mod pages;
mod dependency_graph;

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::collections::HashMap;

use anyhow::{Context, Result, bail};
use regex::Regex;

use crate::build::component::normalize_component_tags;
use crate::build::dependency_graph::DependencyGraph;
use crate::config::Config;

pub fn run_build(cfg: &Config) -> Result<()> {
    println!("tinyfx build\n");

    let components_dir = PathBuf::from(&cfg.components);
    let pages_dir = PathBuf::from(&cfg.pages);
    let out_dir = PathBuf::from(&cfg.out_dir);
    let entry_ts = PathBuf::from(&cfg.entry_ts);
    let src_dir = entry_ts.parent().unwrap_or(Path::new("."));

    // ══════════════════════════════════════════════════════════════
    // Phase 0: Dependency graph
    // ══════════════════════════════════════════════════════════════
    println!("Phase 0: Dependency graph construction");

    let graph = dependency_graph::create_dependency_graph(&components_dir, &pages_dir)
        .with_context(|| "Failed to create dependency graph")?;

    // fail fast — catch cycles and missing refs before any file writing
    let build_order = graph.resolve_compilation_order()
        .with_context(|| "Failed to resolve build order")?;

    println!("  ✔ {} component(s)", graph.components.len());
    println!("  ✔ {} page(s)", graph.pages.len());

    // ══════════════════════════════════════════════════════════════
    // Phase 1: Component compilation
    // ══════════════════════════════════════════════════════════════
    println!("\nPhase 1: Component compilation");

    if !components_dir.exists() {
        bail!("error: components directory not found\n  → {}", components_dir.display());
    }
    if !pages_dir.exists() {
        bail!("error: pages directory not found\n  → {}", pages_dir.display());
    }

    fs::create_dir_all(&out_dir)?;
    fs::create_dir_all(out_dir.join("js"))?;
    fs::create_dir_all(out_dir.join("css"))?;

    // build registry in dependency order — replaces discover_components + resolve_all
    let registry = build_component_registry(&graph, &build_order)
        .with_context(|| "Component compilation failed")?;

    let dynamic_count = registry.values().filter(|c| c.is_dynamic).count();
    println!(
        "  ✔ {} static, {} dynamic",
        registry.len() - dynamic_count,
        dynamic_count
    );

    // ══════════════════════════════════════════════════════════════
    // Phase 2: Page processing
    // ══════════════════════════════════════════════════════════════
    println!("\nPhase 2: Page processing");

    let resolved_pages = pages::resolve_pages(
        &pages_dir,
        &out_dir,
        &registry,      // ← pass registry instead of components_dir + used_components
    ).with_context(|| format!("Failed to resolve pages in {}", pages_dir.display()))?;

    for page in &resolved_pages {
        pages::write_page(page)?;
        println!("  ✔ {} ({})", page.out_path.display(), page.route);
    }

    // ══════════════════════════════════════════════════════════════
    // Phase 3: Glue generation
    // ══════════════════════════════════════════════════════════════
    println!("\nPhase 3: Glue generation");

    let gen_ts = glue::generate_registry(&registry, &resolved_pages);
    let gen_ts_path = src_dir.join("tinyfx.gen.ts");
    fs::write(&gen_ts_path, &gen_ts)?;
    println!("  ✔ {}", gen_ts_path.display());

    if cfg.phase_one_only {
        println!("\nStopped after phase 1 (flag).");
        return Ok(());
    }

    // ══════════════════════════════════════════════════════════════
    // Phase 4: Asset pipeline — unchanged
    // ══════════════════════════════════════════════════════════════
    println!("\nPhase 4: Asset pipeline");

    if entry_ts.exists() {
        run_esbuild(&entry_ts, &gen_ts_path, &out_dir.join("js"))?;
        println!("  ✔ dist/js/main.js");
    }

    let styles_dir = PathBuf::from(&cfg.styles);
    if styles_dir.exists() {
        copy_dir_contents(&styles_dir, &out_dir.join("css"))?;
        println!("  ✔ dist/css/");
    }

    for static_dir in &cfg.static_dirs {
        let src = PathBuf::from(static_dir);
        if src.exists() {
            let dest = out_dir.join(src.file_name().unwrap_or_default());
            copy_dir_contents(&src, &dest)?;
        }
    }

    let assets_src = PathBuf::from(&cfg.assets_dir);
    if assets_src.exists() {
        let dest = out_dir.join(assets_src.file_name().unwrap_or_else(|| assets_src.as_os_str()));
        copy_dir_contents(&assets_src, &dest)?;
    }

    let _ = fs::remove_file(&gen_ts_path);

    println!("\nDone! Output in {}/\n", out_dir.display());
    Ok(())
}
pub struct CompiledComponent {
    pub name: String,
    pub path: PathBuf,
    pub html: String,
    pub is_dynamic: bool,
}

fn build_component_registry(
    graph: &DependencyGraph,
    build_order: &[PathBuf],
) -> Result<HashMap<String, CompiledComponent>> {
    let mut registry: HashMap<String, CompiledComponent> = HashMap::new();

    for path in build_order
        .iter()
        .filter(|p| graph.components.contains_key(*p))
    {
        let meta = &graph.components[path];

        let raw_html = fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;

        // all dependencies are already in registry because of build order
        let expanded_html = expand_component_tags(&raw_html, &registry)?;

        let is_dynamic = expanded_html.contains("tinyfx-dynamic");

        // enforce the contract both ways
        let ts_path = path.with_extension("ts");
        if is_dynamic && !ts_path.exists() {
            bail!(
                "component '{}' has tinyfx-dynamic but no .ts file\n  → create {}",
                meta.name,
                ts_path.display()
            );
        }
        if !is_dynamic && ts_path.exists() {
            eprintln!(
                "warning: '{}' has a .ts file but no tinyfx-dynamic\n  → add tinyfx-dynamic to {}.html or remove {}.ts",
                meta.name, meta.name, meta.name
            );
        }

        registry.insert(meta.name.clone(), CompiledComponent {
            name: meta.name.clone(),
            path: path.clone(),
            html: expanded_html,
            is_dynamic,
        });
    }

    Ok(registry)
}

fn expand_component_tags(
    html: &str,
    registry: &HashMap<String, CompiledComponent>,
) -> Result<String> {
    let mut result = normalize_component_tags(html)?;

    let open_tag_re = Regex::new(r#"<component\b([^>]*)>([\s\S]*?)</component>"#).unwrap();
    let self_closing_re = Regex::new(r#"<component\b([^>]*)/\s*>"#).unwrap();

    loop {
        let before = result.clone();

        // pass 1 — self closing tags: <component name="Button" />
        result = self_closing_re.replace_all(&result, |caps: &regex::Captures| {
            let attrs_str = caps.get(1).map(|m| m.as_str()).unwrap_or_default();
            let Some(name) = extract_component_name(attrs_str) else {
                return caps[0].to_string();
            };

            let Some(component) = registry.get(&name) else {
                return caps[0].to_string();
            };

            substitute(component, attrs_str, "", &name)
        }).to_string();

        // pass 2 — open/close tags: <component name="Button">...</component>
        result = open_tag_re.replace_all(&result, |caps: &regex::Captures| {
            let attrs_str = caps.get(1).map(|m| m.as_str()).unwrap_or_default();
            let inner = caps.get(2).map(|m| m.as_str()).unwrap_or_default();
            let Some(name) = extract_component_name(attrs_str) else {
                return caps[0].to_string();
            };

            let Some(component) = registry.get(&name) else {
                return caps[0].to_string();
            };

            substitute(component, attrs_str, inner, &name)
        }).to_string();

        if result == before {
            break;
        }
    }

    Ok(result)
}

fn extract_component_name(attrs_str: &str) -> Option<String> {
    let attr_re = Regex::new(r#"(\w+)=["']([^"']*)["']"#).unwrap();

    attr_re
        .captures_iter(attrs_str)
        .find_map(|caps| match caps.get(1).map(|m| m.as_str()) {
            Some("name") => caps.get(2).map(|m| m.as_str().to_string()),
            _ => None,
        })
}

fn substitute(
    component: &CompiledComponent,
    attrs_str: &str,
    inner: &str,
    name: &str,
) -> String {
    let mut html = component.html.clone();

    // substitute {{prop}} placeholders with attribute values
    let attr_re = Regex::new(r#"(\w+)=["']([^"']*)["']"#).unwrap();
    for attr_cap in attr_re.captures_iter(attrs_str) {
        let key = &attr_cap[1];
        let val = &attr_cap[2];
        if key == "name" {
            continue;
        }
        html = html.replace(&format!("{{{{{}}}}}", key), val);
    }

    // substitute slot content
    html = html.replace("<slot></slot>", inner);
    html = html.replace("<slot/>", inner);

    // add data-component marker for dynamic components only
    if component.is_dynamic {
        html = html.replacen(">", &format!(" data-component=\"{}\">", name), 1);
    }

    html
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

#[cfg(test)]
mod tests {
    use super::*;

    fn static_component(name: &str, html: &str) -> CompiledComponent {
        CompiledComponent {
            name: name.to_string(),
            path: PathBuf::from(format!("{}.html", name)),
            html: html.to_string(),
            is_dynamic: false,
        }
    }

    #[test]
    fn expands_component_name_syntax() {
        let mut registry = HashMap::new();
        registry.insert(
            "Button".to_string(),
            static_component("Button", r#"<button>{{label}}<slot></slot></button>"#),
        );

        let result = expand_component_tags(
            r#"<component name="Button" label="Click">Now</component>"#,
            &registry,
        )
        .unwrap();

        assert!(result.contains("<button>ClickNow</button>"));
        assert!(!result.contains("<component"));
    }

    #[test]
    fn expands_pascal_case_syntax_via_normalization() {
        let mut registry = HashMap::new();
        registry.insert(
            "Button".to_string(),
            static_component("Button", r#"<button>{{variant}}<slot></slot></button>"#),
        );

        let result = expand_component_tags(r#"<Button variant="primary">Go</Button>"#, &registry)
            .unwrap();

        assert!(result.contains("<button>primaryGo</button>"));
        assert!(!result.contains("<Button"));
    }

    #[test]
    fn expands_nested_components_from_registry() {
        let mut registry = HashMap::new();
        registry.insert(
            "Icon".to_string(),
            static_component("Icon", r#"<i class="icon-{{kind}}"></i>"#),
        );
        registry.insert(
            "Button".to_string(),
            static_component(
                "Button",
                r#"<button><component name="Icon" kind="check"></component><slot></slot></button>"#,
            ),
        );

        let result = expand_component_tags(r#"<component name="Button">Save</component>"#, &registry)
            .unwrap();

        assert!(result.contains(r#"<i class="icon-check"></i>"#));
        assert!(result.contains("Save"));
        assert!(!result.contains("<component"));
    }
}
