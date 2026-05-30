//! # dependency_graph
//!
//! Constructs and queries the compile-time dependency graph for TinyFX builds.
//!
//! ## Responsibilities
//! - Scan component and page HTML files
//! - Register dependency graph nodes for components and pages
//! - Resolve component references into graph edges
//! - Detect cycles and provide topological build order
//!
//! ## What this module does NOT do
//! - It does not compile component HTML
//! - It does not write files to disk
//! - It does not generate runtime glue code

use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use anyhow::{Context, Result, anyhow, bail};
use regex::Regex;

/// Metadata for a discovered component node.
#[derive(Debug, Clone)]
pub struct ComponentMeta {
    pub name: String,
    pub source_html: String,
}

/// Metadata for a discovered page node.
#[derive(Debug, Clone)]
pub struct PageMeta {
    pub name: String,
    pub source_html: String,
}

/// Graph of component and page dependencies used during the build.
///
/// `dependencies` stores outgoing edges, while `dependents` stores the reverse
/// lookup used during topological ordering.
#[derive(Debug, Clone, Default)]
pub struct DependencyGraph {
    pub dependencies: HashMap<PathBuf, HashSet<PathBuf>>,
    pub dependents: HashMap<PathBuf, HashSet<PathBuf>>,
    pub components: HashMap<PathBuf, ComponentMeta>,
    pub pages: HashMap<PathBuf, PageMeta>,
}

/// Create the dependency graph for the current project.
///
/// This scans the components and pages directories, registers every HTML file
/// as a node, resolves component references into dependency edges, and runs
/// cycle detection before returning the completed graph.
///
/// # Errors
/// Returns an error when any source directory cannot be read, when a page or
/// component filename cannot be interpreted, when a source file cannot be read,
/// or when a circular dependency is detected.
pub fn create_dependency_graph(components_dir: &Path, pages_dir: &Path) -> Result<DependencyGraph> {
    let mut dependency_graph = DependencyGraph::default();

    for component_path in scan_html_files(components_dir)? {
        let source_html = fs::read_to_string(&component_path).with_context(|| {
            format!(
                "failed to read component '{}' while building the dependency graph\n  → confirm the file exists and is readable",
                component_path.display()
            )
        })?;
        let component_meta = parse_component_meta(&component_path, source_html)?;
        dependency_graph
            .dependencies
            .entry(component_path.clone())
            .or_default();
        dependency_graph
            .components
            .insert(component_path, component_meta);
    }

    for page_path in scan_html_files(pages_dir)? {
        let source_html = fs::read_to_string(&page_path).with_context(|| {
            format!(
                "failed to read page '{}' while building the dependency graph\n  → confirm the file exists and is readable",
                page_path.display()
            )
        })?;
        let page_meta = parse_page_meta(&page_path, source_html)?;
        dependency_graph
            .dependencies
            .entry(page_path.clone())
            .or_default();
        dependency_graph.pages.insert(page_path, page_meta);
    }

    dependency_graph.populate();

    Ok(dependency_graph)
}

impl DependencyGraph {
    /// Add a directed dependency edge `from -> to`.
    ///
    /// The source node depends on the target node, so the reverse lookup is
    /// also updated in `dependents`.
    pub fn add_dependency(&mut self, from: PathBuf, to: PathBuf) {
        self.dependencies
            .entry(from.clone())
            .or_default()
            .insert(to.clone());
        self.dependents.entry(to).or_default().insert(from);
    }

    /// Resolve a topological build order for all graph nodes.
    ///
    /// The returned vector places dependencies before the nodes that consume
    /// them so component compilation can proceed in one pass.
    ///
    /// # Errors
    /// Returns an error when the graph contains a cycle and therefore cannot be
    /// reduced to a valid compilation order.
    pub fn resolve_compilation_order(&self) -> Result<Vec<PathBuf>> {
        let mut build_order = Vec::new();
        let mut in_degree: HashMap<PathBuf, usize> = self
            .components
            .keys()
            .chain(self.pages.keys())
            .map(|path| {
                let degree = self.dependencies.get(path).map_or(0, |deps| deps.len());
                (path.clone(), degree)
            })
            .collect();

        let mut ready_queue: VecDeque<PathBuf> = self
            .components
            .keys()
            .chain(self.pages.keys())
            .filter(|path| self.dependencies.get(*path).is_none_or(HashSet::is_empty))
            .cloned()
            .collect();

        while let Some(path) = ready_queue.pop_front() {
            build_order.push(path.clone());

            if let Some(dependents) = self.dependents.get(&path) {
                for dependent_path in dependents {
                    if let Some(count) = in_degree.get_mut(dependent_path) {
                        *count = count.saturating_sub(1);
                        if *count == 0 {
                            ready_queue.push_back(dependent_path.clone());
                        }
                    }
                }
            }
        }

        let total_nodes = self.components.len() + self.pages.len();
        if build_order.len() != total_nodes {
            let emitted_nodes: HashSet<&PathBuf> = build_order.iter().collect();
            let cycle_nodes: Vec<&str> = self
                .components
                .keys()
                .chain(self.pages.keys())
                .filter(|path| !emitted_nodes.contains(path))
                .filter_map(|path| {
                    self.components
                        .get(path)
                        .map(|component| component.name.as_str())
                        .or_else(|| self.pages.get(path).map(|page| page.name.as_str()))
                })
                .collect();

            bail!(
                "circular dependency detected between: {}\n  → remove the recursive component references and run the build again",
                cycle_nodes.join(", ")
            );
        }

        Ok(build_order)
    }

    fn populate(&mut self) {
        let component_name_to_path: HashMap<String, PathBuf> = self
            .components
            .iter()
            .map(|(path, component_meta)| (component_meta.name.clone(), path.clone()))
            .collect();

        let component_sources: Vec<(PathBuf, String)> = self
            .components
            .iter()
            .map(|(path, meta)| (path.clone(), meta.source_html.clone()))
            .collect();
        for (component_path, component_html) in component_sources {
            self.register_component_refs(
                &component_path,
                &component_html,
                &component_name_to_path,
                false,
            );
        }

        let page_sources: Vec<(PathBuf, String)> = self
            .pages
            .iter()
            .map(|(path, meta)| (path.clone(), meta.source_html.clone()))
            .collect();
        for (page_path, page_html) in page_sources {
            self.register_component_refs(&page_path, &page_html, &component_name_to_path, true);
        }
    }

    fn register_component_refs(
        &mut self,
        source_path: &Path,
        source_html: &str,
        component_name_to_path: &HashMap<String, PathBuf>,
        is_page: bool,
    ) {
        for component_name in collect_component_references(source_html) {
            if let Some(dependency_path) = component_name_to_path.get(&component_name) {
                self.add_dependency(source_path.to_path_buf(), dependency_path.clone());
            } else if is_page {
                eprintln!(
                    "warning: page '{}' references missing component '{}'\n  → create src/components/{name}/{name}.html or fix the tag name",
                    source_path.display(),
                    component_name,
                    name = component_name,
                );
            } else {
                eprintln!(
                    "warning: component '{}' references missing component '{}'\n  → create src/components/{name}/{name}.html or fix the tag name",
                    source_path.display(),
                    component_name,
                    name = component_name,
                );
            }
        }
    }
}

pub(crate) fn collect_component_references(source_html: &str) -> Vec<String> {
    let mut refs: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    for captures in pascal_component_ref_re()
        .captures_iter(source_html)
        .chain(canonical_component_ref_re().captures_iter(source_html))
    {
        if let Some(name) = captures.get(1).map(|m| m.as_str().to_string()) {
            if seen.insert(name.clone()) {
                refs.push(name);
            }
        }
    }

    refs
}

fn scan_html_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut html_files = Vec::new();
    if !dir.exists() {
        return Ok(html_files);
    }

    for entry in fs::read_dir(dir).with_context(|| {
        format!(
            "failed to read directory '{}'\n  → confirm the path exists and is readable",
            dir.display()
        )
    })? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            html_files.extend(scan_html_files(&path)?);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("html") {
            html_files.push(path);
        }
    }

    Ok(html_files)
}

fn parse_component_meta(path: &Path, source_html: String) -> Result<ComponentMeta> {
    let name = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or_else(|| {
            anyhow!(
                "invalid component filename '{}'\n  → use a valid PascalCase .html file name",
                path.display()
            )
        })?
        .to_string();

    Ok(ComponentMeta { name, source_html })
}

fn parse_page_meta(path: &Path, source_html: String) -> Result<PageMeta> {
    let name = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or_else(|| {
            anyhow!(
                "invalid page filename '{}'\n  → use a valid .html file name under src/pages",
                path.display()
            )
        })?
        .to_string();

    Ok(PageMeta { name, source_html })
}


fn pascal_component_ref_re() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r#"<([A-Z][a-zA-Z0-9]*)\b"#).unwrap())
}

fn canonical_component_ref_re() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r#"<component\b[^>]*\bname=["']([A-Z][a-zA-Z0-9]*)["'][^>]*>"#).unwrap()
    })
}

#[cfg(test)]
mod tests {
    use super::collect_component_references;

    #[test]
    fn parses_pascal_and_canonical_component_refs() {
        let refs = collect_component_references(
            r#"
            <div>
              <Button variant="primary">Go</Button>
              <component name="Card"></component>
            </div>
            "#,
        );

        assert!(refs.contains(&"Button".to_string()));
        assert!(refs.contains(&"Card".to_string()));
    }
}
