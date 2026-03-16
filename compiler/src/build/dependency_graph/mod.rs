pub mod data;

use std::{collections::HashMap, path::PathBuf};

pub use data::DependencyGraph;

pub type GraphResult<T> = anyhow::Result<T>;
pub fn create_dependency_graph(components_dir: &PathBuf, pages_dir: &PathBuf) -> GraphResult<DependencyGraph> {
    // Build the full dependency graph in four steps:
    // 1) register components (name -> path)
    // 2) register pages
    // 3) populate edges by parsing files for component refs
    // 4) validate order (detect cycles)
    let mut graph = DependencyGraph::new();

    // 1. register all components first — needed for name → path resolution
    for entry in scan_html_files(components_dir)? {
        let meta = parse_component_meta(&entry)?;
        graph.components.insert(entry.clone(), meta);
        // ensure node exists in dependency map even if it has no deps
        graph.dependencies.entry(entry).or_default();
    }

    // 2. register all pages
    for entry in scan_html_files(pages_dir)? {
        let meta = parse_page_meta(&entry)?;
        graph.pages.insert(entry.clone(), meta);
        graph.dependencies.entry(entry).or_default();
    }

    // 3. populate edges — now that all nodes are registered
    graph.populate(components_dir, pages_dir)?;

    // 4. validate — fail fast before any build work starts
    graph.resolve_compilation_order()?;

    Ok(graph)
}

// Helper: scan a directory recursively and return all `.html` files found.
fn scan_html_files(dir: &PathBuf) -> GraphResult<Vec<PathBuf>> {
    let mut files = Vec::new();
    if !dir.exists() {
        return Ok(files);
    }
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let mut nested = scan_html_files(&path)?;
            files.append(&mut nested);
            continue;
        }
        if path.extension().and_then(|s| s.to_str()) == Some("html") {
            files.push(path);
        }
    }
    Ok(files)
}

// Parse basic component metadata from a component file path.
fn parse_component_meta(path: &PathBuf) -> GraphResult<data::ComponentMeta> {
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow::anyhow!("invalid component filename"))?
        .to_string();
    Ok(data::ComponentMeta {
        name,
        path: path.clone(),
        is_dynamic: false,
    })
}

// Parse basic page metadata from a page file path.
fn parse_page_meta(path: &PathBuf) -> GraphResult<data::PageMeta> {
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow::anyhow!("invalid page filename"))?
        .to_string();

    let route = if name == "index" { "/".to_string() } else { format!("/{}", name) };
    Ok(data::PageMeta {
        name,
        path: path.clone(),
        route,
        layout: None,
    })
}