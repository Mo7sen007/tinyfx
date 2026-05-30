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

mod codegen;
mod component;
mod conditional;
mod conditional_codegen;
mod dependency_graph;
mod each;
mod each_codegen;
mod glue;
pub mod pages;
mod pages_util;
mod pool;
mod wrapper;

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use anyhow::{Context, Result, bail};

use self::component::{CompiledComponent, build_component_registry};
use self::dependency_graph::{DependencyGraph, create_dependency_graph};
use self::pages::{ComponentDirectives, PageDef, PageDirectives};
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
    println!(
        "tinyfx build{}\n",
        if config.release { " --release" } else { "" }
    );

    let (dependency_graph, build_order) = phase_0_build_dependency_graph(config)?;
    let component_registry = Arc::new(phase_1_compile_components(
        config,
        &dependency_graph,
        &build_order,
    )?);
    let (resolved_pages, directives) =
        phase_2_process_pages(config, &dependency_graph, Arc::clone(&component_registry))?;
    let component_directives = pages::collect_component_directives(component_registry.as_ref());
    let generated_glue_path = phase_3_generate_glue(
        config,
        component_registry.as_ref(),
        &resolved_pages,
        &directives,
        &component_directives,
    )?;

    if config.phase_one_only {
        println!(
            "\nStopped after phase 3 (flag). Generated glue left at '{}' for inspection.\n",
            generated_glue_path.display()
        );
        return Ok(());
    }

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
    dependency_graph: &DependencyGraph,
    component_registry: Arc<ComponentRegistry>,
) -> Result<(Vec<PageDef>, Vec<PageDirectives>)> {
    println!("\nPhase 2: Page processing");
    let num_threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);

    let pages_dir = PathBuf::from(&config.pages);
    let out_dir = PathBuf::from(&config.out_dir);

    let mut page_sources: Vec<(PathBuf, String)> = dependency_graph
        .pages
        .iter()
        .map(|(path, meta)| (path.clone(), meta.source_html.clone()))
        .collect();
    page_sources.sort_by(|(left, _), (right, _)| left.cmp(right));

    let jobs: Vec<_> = page_sources
        .into_iter()
        .map(|(path, source_html)| {
            let registry = Arc::clone(&component_registry);
            let pages_root = pages_dir.clone();
            let out_dir = out_dir.clone();
            move || -> Result<(PageDef, PageDirectives)> {
                pages::compile_page(path, source_html, pages_root, out_dir, registry)
            }
        })
        .collect();

    let compiled_pages = pool::run_parallel(jobs, num_threads)?;
    let mut resolved_pages = Vec::with_capacity(compiled_pages.len());
    let mut directives = Vec::with_capacity(compiled_pages.len());
    for (page, page_directives) in compiled_pages {
        resolved_pages.push(page);
        directives.push(page_directives);
    }

    // Write out pages and report
    for page in &resolved_pages {
        pages::write_page(page)?;
        println!("  ✔ {} ({})", page.out_path.display(), page.route);
    }

    Ok((resolved_pages, directives))
}

fn phase_3_generate_glue(
    config: &Config,
    component_registry: &ComponentRegistry,
    resolved_pages: &[PageDef],
    directives: &[PageDirectives],
    component_directives: &[ComponentDirectives],
) -> Result<PathBuf> {
    println!("\nPhase 3: Glue generation");

    let entry_ts = PathBuf::from(&config.entry_ts);
    let src_dir = entry_ts.parent().unwrap_or(Path::new("."));
    let generated_glue_path = src_dir.join("tinyfx.gen.ts");
    let generated_glue = glue::generate_registry(
        component_registry,
        resolved_pages,
        directives,
        component_directives,
        config.release,
    );

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
        run_esbuild(
            &entry_ts,
            generated_glue_path,
            &out_dir.join("js"),
            config.release,
        )?;
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
fn run_esbuild(entry: &Path, generated_glue_path: &Path, out_js: &Path, release: bool) -> Result<()> {
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
    let command = if esbuild_command == "npx" {
        let mut command = Command::new("npx");
        command.arg("esbuild");
        command
    } else {
        Command::new(&esbuild_command)
    };

    let mut command = command;
    command
        .arg(&temporary_entry)
        .arg("--bundle")
        .arg(format!("--outfile={}", out_file.display()))
        .arg("--platform=browser")
        .arg("--format=esm")
        .arg("--target=es2020")
        .arg("--charset=utf8")
        .arg("--alias:@models=./src/lib/models")
        .arg("--alias:@dto=./src/lib/dto")
        .arg("--alias:@state=./src/lib/state")
        .arg("--alias:@services=./src/lib/services")
        .arg("--alias:@utils=./src/lib/utils");

    if release {
        command.arg("--minify");
        command.arg("--minify-whitespace");
        command.arg("--minify-syntax");
        command.arg("--minify-identifiers");
        command.arg("--tree-shaking=ignore-annotations");
        command.arg("--legal-comments=none");
        command.arg("--drop:debugger");
        command.arg("--define:__DEV__=false");
    } else {
        command.arg("--sourcemap");
        command.arg("--define:__DEV__=true");
    }

    let status = command
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
    for local in [
        "node_modules/.bin/esbuild",
        "node_modules/.bin/esbuild.cmd",
        "node_modules/.bin/esbuild.exe",
    ] {
        let local_esbuild = Path::new(local);
        if local_esbuild.is_file() {
            return Ok(local_esbuild.to_string_lossy().into_owned());
        }
    }

    if command_exists("esbuild") {
        return Ok("esbuild".to_string());
    }

    if command_exists("npx") {
        return Ok("npx".to_string());
    }

    bail!(
        "esbuild was not found\n  → install it with 'npm install --save-dev esbuild' or 'npm install -g esbuild'"
    )
}

fn command_exists(command: &str) -> bool {
    let Some(path_var) = std::env::var_os("PATH") else {
        return false;
    };

    #[cfg(windows)]
    let exts: Vec<String> = std::env::var_os("PATHEXT")
        .map(|value| {
            value
                .to_string_lossy()
                .split(';')
                .map(|part| part.to_string())
                .collect()
        })
        .unwrap_or_else(|| {
            vec![
                ".COM".to_string(),
                ".EXE".to_string(),
                ".BAT".to_string(),
                ".CMD".to_string(),
            ]
        });

    #[cfg(not(windows))]
    let exts: Vec<String> = vec![String::new()];

    for dir in std::env::split_paths(&path_var) {
        for ext in &exts {
            let candidate = dir.join(format!("{}{}", command, ext));
            if candidate.is_file() {
                return true;
            }
        }
    }

    false
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
