//! # build
//!
//! Build pipeline orchestrator for `tinyfx build`.
//!
//! ## Responsibilities
//! - Run build phases in a predictable order
//! - Coordinate graph construction, component compilation, page processing,
//!   glue generation, and the asset pipeline
//! - Own Phase 4 helper functions for esbuild and file copying
//!
//! ## What this module does NOT do
//! - It does not parse component or page HTML
//! - It does not contain regex-based expansion logic
//! - It does not generate glue code inline

mod component;
mod dependency_graph;
mod glue;
pub mod pages;

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};

use self::component::{build_component_registry, CompiledComponent};
use self::dependency_graph::{create_dependency_graph, DependencyGraph};
use self::pages::PageDef;
use crate::config::Config;

type ComponentRegistry = HashMap<String, CompiledComponent>;

/// Run the TinyFX build pipeline.
///
/// The pipeline executes dependency graph construction, component compilation,
/// page processing, glue generation, and the asset pipeline in that order.
/// The build behavior is controlled entirely by `config`.
///
/// # Errors
/// Returns an error when any build phase fails, including invalid component or
/// page input, missing dynamic component scripts, route/layout resolution
/// failures, or esbuild and file-copy errors during the asset pipeline.
pub fn run_build(config: &Config) -> Result<()> {
    println!("tinyfx build\n");

    let (dependency_graph, build_order) = phase_0_build_dependency_graph(config)?;
    let component_registry = phase_1_compile_components(config, &dependency_graph, &build_order)?;

    if config.phase_one_only {
        println!("\nStopped after phase 1 (flag).\n");
        return Ok(());
    }

    let resolved_pages = phase_2_process_pages(config, &component_registry)?;
    let generated_glue_path = phase_3_generate_glue(config, &component_registry, &resolved_pages)?;
    phase_4_run_asset_pipeline(config, &generated_glue_path)?;

    let _ = fs::remove_file(&generated_glue_path);

    println!("\nDone! Output in {}/\n", config.out_dir);
    Ok(())
}

fn phase_0_build_dependency_graph(config: &Config) -> Result<(DependencyGraph, Vec<PathBuf>)> {
    println!("Phase 0: Dependency graph construction");

    let components_dir = PathBuf::from(&config.components);
    let pages_dir = PathBuf::from(&config.pages);
    let dependency_graph = create_dependency_graph(&components_dir, &pages_dir)?;
    let build_order = dependency_graph.resolve_compilation_order()?;

    println!("  ✔ {} component(s)", dependency_graph.components.len());
    println!("  ✔ {} page(s)", dependency_graph.pages.len());

    Ok((dependency_graph, build_order))
}

fn phase_1_compile_components(
    config: &Config,
    dependency_graph: &DependencyGraph,
    build_order: &[PathBuf],
) -> Result<ComponentRegistry> {
    println!("\nPhase 1: Component compilation");

    let components_dir = PathBuf::from(&config.components);
    let out_dir = PathBuf::from(&config.out_dir);

    if !components_dir.exists() {
        bail!(
            "components directory '{}' was not found\n  → create it or update [components] in tinyfx.toml",
            components_dir.display()
        );
    }

    ensure_output_dirs(&out_dir)?;

    let component_registry = build_component_registry(dependency_graph, build_order)?;
    let dynamic_count = component_registry
        .values()
        .filter(|component| component.is_dynamic)
        .count();

    println!(
        "  ✔ {} static, {} dynamic",
        component_registry.len().saturating_sub(dynamic_count),
        dynamic_count
    );

    Ok(component_registry)
}

fn phase_2_process_pages(
    config: &Config,
    component_registry: &ComponentRegistry,
) -> Result<Vec<PageDef>> {
    println!("\nPhase 2: Page processing");

    let pages_dir = PathBuf::from(&config.pages);
    let out_dir = PathBuf::from(&config.out_dir);
    let resolved_pages = pages::resolve_pages(&pages_dir, &out_dir, component_registry)?;

    for page in &resolved_pages {
        pages::write_page(page)?;
        println!("  ✔ {} ({})", page.out_path.display(), page.route);
    }

    Ok(resolved_pages)
}

fn phase_3_generate_glue(
    config: &Config,
    component_registry: &ComponentRegistry,
    resolved_pages: &[PageDef],
) -> Result<PathBuf> {
    println!("\nPhase 3: Glue generation");

    let entry_ts = PathBuf::from(&config.entry_ts);
    let src_dir = entry_ts.parent().unwrap_or(Path::new("."));
    let generated_glue_path = src_dir.join("tinyfx.gen.ts");
    let generated_glue = glue::generate_registry(component_registry, resolved_pages);

    fs::write(&generated_glue_path, generated_glue).with_context(|| {
        format!(
            "failed to write generated glue file '{}'\n  → confirm the source directory is writable",
            generated_glue_path.display()
        )
    })?;

    println!("  ✔ {}", generated_glue_path.display());

    Ok(generated_glue_path)
}

fn phase_4_run_asset_pipeline(config: &Config, generated_glue_path: &Path) -> Result<()> {
    println!("\nPhase 4: Asset pipeline");

    let entry_ts = PathBuf::from(&config.entry_ts);
    let out_dir = PathBuf::from(&config.out_dir);

    if entry_ts.exists() {
        run_esbuild(&entry_ts, generated_glue_path, &out_dir.join("js"))?;
        println!("  ✔ dist/js/main.js");
    }

    let styles_dir = PathBuf::from(&config.styles);
    if styles_dir.exists() {
        copy_dir_contents(&styles_dir, &out_dir.join("css"))?;
        println!("  ✔ dist/css/");
    }

    for static_dir in &config.static_dirs {
        let source_dir = PathBuf::from(static_dir);
        if source_dir.exists() {
            let destination_dir = out_dir.join(source_dir.file_name().unwrap_or_default());
            copy_dir_contents(&source_dir, &destination_dir)?;
        }
    }

    let assets_source_dir = PathBuf::from(&config.assets_dir);
    if assets_source_dir.exists() {
        let destination_dir = out_dir.join(
            assets_source_dir
                .file_name()
                .unwrap_or_else(|| assets_source_dir.as_os_str()),
        );
        copy_dir_contents(&assets_source_dir, &destination_dir)?;
    }

    Ok(())
}

fn ensure_output_dirs(out_dir: &Path) -> Result<()> {
    fs::create_dir_all(out_dir).with_context(|| {
        format!(
            "failed to create output directory '{}'\n  → confirm the destination path is writable",
            out_dir.display()
        )
    })?;
    fs::create_dir_all(out_dir.join("js")).with_context(|| {
        format!(
            "failed to create JavaScript output directory '{}'\n  → confirm the destination path is writable",
            out_dir.join("js").display()
        )
    })?;
    fs::create_dir_all(out_dir.join("css")).with_context(|| {
        format!(
            "failed to create CSS output directory '{}'\n  → confirm the destination path is writable",
            out_dir.join("css").display()
        )
    })?;
    Ok(())
}

/// Run esbuild to bundle the entry and generated glue into `dist/js/main.js`.
fn run_esbuild(entry: &Path, generated_glue_path: &Path, out_js: &Path) -> Result<()> {
    let out_file = out_js.join("main.js");
    let temporary_entry = out_js.parent().unwrap_or(Path::new(".")).join("_entry.ts");
    let entry_content = format!(
        r#"import "{}";
    import "{}";
    "#,
        generated_glue_path.canonicalize()?.display(),
        entry.canonicalize()?.display()
    );

    fs::write(&temporary_entry, entry_content).with_context(|| {
        format!(
            "failed to write temporary esbuild entry '{}'\n  → confirm the output directory is writable",
            temporary_entry.display()
        )
    })?;

    let esbuild_command = find_esbuild()?;
    let mut command = if esbuild_command == "npx" {
        let mut command = Command::new("npx");
        command.arg("esbuild");
        command
    } else {
        Command::new(&esbuild_command)
    };

    let status = command
        .arg(&temporary_entry)
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
        .with_context(|| {
            format!(
                "failed to run esbuild using '{}'\n  → install esbuild locally with 'npm install --save-dev esbuild' or ensure it is on PATH",
                esbuild_command
            )
        })?;

    let _ = fs::remove_file(&temporary_entry);

    if !status.success() {
        bail!(
            "esbuild exited with status {:?}\n  → inspect the TypeScript entry '{}' and generated glue for syntax errors",
            status.code(),
            entry.display()
        );
    }

    Ok(())
}

/// Find the esbuild executable, preferring local installs before global ones.
fn find_esbuild() -> Result<String> {
    let local_esbuild = Path::new("node_modules/.bin/esbuild");
    if local_esbuild.exists() {
        return Ok(local_esbuild.to_string_lossy().into_owned());
    }

    if let Ok(output) = Command::new("which").arg("esbuild").output() {
        if output.status.success() {
            return Ok("esbuild".to_string());
        }
    }

    if let Ok(output) = Command::new("which").arg("npx").output() {
        if output.status.success() {
            return Ok("npx".to_string());
        }
    }

    bail!(
        "esbuild was not found\n  → install it with 'npm install --save-dev esbuild' or 'npm install -g esbuild'"
    )
}

/// Recursively copy all files from `src` into `dest`.
fn copy_dir_contents(src: &Path, dest: &Path) -> Result<()> {
    fs::create_dir_all(dest).with_context(|| {
        format!(
            "failed to create directory '{}'\n  → confirm the destination path is writable",
            dest.display()
        )
    })?;

    for entry in fs::read_dir(src).with_context(|| {
        format!(
            "failed to read directory '{}'\n  → confirm the source path exists and is readable",
            src.display()
        )
    })? {
        let entry = entry?;
        let source_path = entry.path();
        let destination_path = dest.join(entry.file_name());

        if source_path.is_dir() {
            copy_dir_contents(&source_path, &destination_path)?;
        } else {
            fs::copy(&source_path, &destination_path).with_context(|| {
                format!(
                    "failed to copy '{}' to '{}'\n  → confirm both paths are accessible",
                    source_path.display(),
                    destination_path.display()
                )
            })?;
        }
    }

    Ok(())
}
