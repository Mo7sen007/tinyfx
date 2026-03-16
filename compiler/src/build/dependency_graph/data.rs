use std::{
    collections::{HashMap, HashSet, VecDeque},
    path::PathBuf,
};

use anyhow::{bail, Result};

/// Metadata for a discovered component.
///
/// - `name`: human-friendly component name (e.g. "Button").
/// - `path`: filesystem path to the component source file.
/// - `is_dynamic`: whether the component is loaded dynamically at runtime.
pub struct ComponentMeta {
    pub name: String,
    pub path: PathBuf,
    pub is_dynamic: bool,
}

/// Metadata for a discovered page.
///
/// - `name`: page identifier (file stem).
/// - `path`: filesystem path to the page source file.
/// - `route`: the route that the page will be served at.
/// - `layout`: optional layout name used by the page.
pub struct PageMeta {
    pub name: String,
    pub path: PathBuf,
    pub route: String,
    pub layout: Option<String>,
}

/// Dependency graph representing relationships between pages and components.
///
/// - `dependencies`: adjacency map from a node -> set of nodes it depends on.
/// - `dependents`: reverse adjacency map (node -> set of nodes that depend on it).
/// - `components`: map of component path -> component metadata.
/// - `pages`: map of page path -> page metadata.
///
/// The graph is used to perform validation (missing/cycle detection), compute
/// compilation order (topological sort), and to efficiently determine which
/// pages are affected by a changed component.
pub struct DependencyGraph {
    pub dependencies: HashMap<PathBuf, HashSet<PathBuf>>,
    pub dependents: HashMap<PathBuf, HashSet<PathBuf>>,
    pub components: HashMap<PathBuf, ComponentMeta>,
    pub pages: HashMap<PathBuf, PageMeta>,
}

impl DependencyGraph {
    /// Create an empty dependency graph.
    pub fn new() -> Self {
        Self {
            dependencies: HashMap::new(),
            dependents: HashMap::new(),
            components: HashMap::new(),
            pages: HashMap::new(),
        }
    }

    /// Record a directed dependency edge `from -> to`.
    ///
    /// This updates both the `dependencies` adjacency map and the `dependents`
    /// reverse map so lookups in either direction are O(1) on average.
    pub fn add_dependency(&mut self, from: PathBuf, to: PathBuf) {
        self.dependencies
            .entry(from.clone())
            .or_default()
            .insert(to.clone());
        self.dependents
            .entry(to)
            .or_default()
            .insert(from);
    }

    /// Get the set of nodes that `node` depends on (its outgoing edges).
    ///
    /// Returns `None` when the node has no recorded dependencies.
    pub fn get_dependencies(&self, node: &PathBuf) -> Option<&HashSet<PathBuf>> {
        self.dependencies.get(node)
    }

    /// Get the set of nodes that depend on `node` (incoming edges in reverse map).
    ///
    /// Returns `None` when no dependents are recorded for the node.
    pub fn get_dependents(&self, node: &PathBuf) -> Option<&HashSet<PathBuf>> {
        self.dependents.get(node)
    }

    /// Return all nodes (components and pages) that have no outgoing dependencies.
    ///
    /// These are "leaf" nodes in the dependency sense — they can be compiled
    /// first because they don't require any other components/pages to be built
    /// before them.
    pub fn get_leaf_nodes(&self) -> Vec<PathBuf> {
        self.components
            .keys()
            .chain(self.pages.keys())
            .filter(|node| {
                self.dependencies
                    .get(*node)
                    .map_or(true, |deps| deps.is_empty())
            })
            .cloned()
            .collect()
    }

    /// Compute a compilation order for all components and pages.
    ///
    /// Returns a topologically-sorted `Vec<PathBuf>` where dependencies appear
    /// before dependents. If a cycle is detected the function returns an
    /// error describing the involved nodes.
    pub fn resolve_compilation_order(&self) -> Result<Vec<PathBuf>> {
        let mut result = Vec::new();
        // in-degree map counts how many dependencies each node has
        let mut in_degree: HashMap<PathBuf, usize> = self
            .components
            .keys()
            .chain(self.pages.keys())
            .map(|node| {
                let degree = self.dependencies
                    .get(node)
                    .map_or(0, |deps| deps.len());
                (node.clone(), degree)
            })
            .collect();

        // Start with nodes that have zero dependencies (leaf nodes)
        let mut ready_queue: VecDeque<PathBuf> = self
            .get_leaf_nodes()
            .into_iter()
            .collect();

        // Kahn's algorithm: repeatedly remove nodes with in-degree 0
        while let Some(node) = ready_queue.pop_front() {
            result.push(node.clone());

            if let Some(dependents) = self.dependents.get(&node) {
                for dep in dependents {
                    if let Some(count) = in_degree.get_mut(dep) {
                        *count -= 1;
                        if *count == 0 {
                            ready_queue.push_back(dep.clone());
                        }
                    }
                }
            }
        }

        // If not all nodes were emitted, there is a cycle
        let total = self.components.len() + self.pages.len();
        if result.len() != total {
            let compiled: HashSet<&PathBuf> = result.iter().collect();
            let cycle_nodes: Vec<&str> = self
                .components
                .keys()
                .chain(self.pages.keys())
                .filter(|node| !compiled.contains(node))
                .filter_map(|node| {
                    self.components
                        .get(node)
                        .map(|c| c.name.as_str())
                        .or_else(|| self.pages.get(node).map(|p| p.name.as_str()))
                })
                .collect();

            bail!(
                "Circular dependency detected. Involved nodes: {}",
                cycle_nodes.join(", ")
            );
        }

        Ok(result)
    }

    /// Given a changed node path, return all pages that depend (directly or
    /// transitively) on it. This is used by the dev-server to perform
    /// incremental rebuilds for only the affected pages.
    pub fn affected_pages(&self, changed: &PathBuf) -> Vec<&PageMeta> {
        let mut affected = Vec::new();
        let mut to_check = vec![changed];
        let mut checked = HashSet::new();

        // Walk the reverse graph (dependents) until all reachable nodes are
        // visited, collecting PageMeta entries encountered along the way.
        while let Some(path) = to_check.pop() {
            if !checked.insert(path) {
                continue;
            }

            if let Some(page) = self.pages.get(path) {
                affected.push(page);
            }

            if let Some(dependents) = self.dependents.get(path) {
                for dep in dependents {
                    to_check.push(dep);
                }
            }
        }

        affected
    }

    pub fn populate(&mut self, _components_dir: &PathBuf, _pages_dir: &PathBuf) -> Result<()> {
        // Build an owned name -> path map to avoid holding immutable borrows of
        // `self` while later mutating it. We clone the PathBufs and names so
        // lookups do not borrow from `self`.
        let name_to_path: HashMap<String, PathBuf> = self
            .components
            .iter()
            .map(|(path, meta)| (meta.name.clone(), path.clone()))
            .collect();

        // second pass — parse component HTML files for nested component references
        let component_paths: Vec<PathBuf> = self.components.keys().cloned().collect();
        for path in component_paths {
            let refs = parse_component_refs(&path)?;
            for ref_name in refs {
                if let Some(dep_path) = name_to_path.get(&ref_name) {
                    self.add_dependency(path.clone(), dep_path.clone());
                } else {
                    // component referenced but not registered — warn here
                    eprintln!(
                        "warning: component '{}' referenced in '{}' but not found",
                        ref_name,
                        path.display()
                    );
                }
            }
        }

        // third pass — parse page HTML files for component references
        let page_paths: Vec<PathBuf> = self.pages.keys().cloned().collect();
        for path in page_paths {
            let refs = parse_component_refs(&path)?;
            for ref_name in refs {
                if let Some(dep_path) = name_to_path.get(&ref_name) {
                    self.add_dependency(path.clone(), dep_path.clone());
                } else {
                    eprintln!(
                        "warning: component '{}' referenced in page '{}' but not found",
                        ref_name,
                        path.display()
                    );
                }
            }
        }

        Ok(())
    }
}
fn parse_component_refs(path: &PathBuf) -> Result<Vec<String>> {
    let content = std::fs::read_to_string(path)?;
    let mut refs = Vec::new();

    // components are PascalCase tags: <Button>, <NavBar>, <PostCard />
    // a simple approach: find all tags that start with an uppercase letter
    let tag_re = regex::Regex::new(r"<([A-Z][a-zA-Z0-9]*)")?;
    for cap in tag_re.captures_iter(&content) {
        let name = cap[1].to_string();
        if !refs.contains(&name) {
            refs.push(name);
        }
    }

    let canonical_re = regex::Regex::new(r#"<component\b[^>]*\bname=["']([A-Z][a-zA-Z0-9]*)["'][^>]*>"#)?;
    for cap in canonical_re.captures_iter(&content) {
        let name = cap[1].to_string();
        if !refs.contains(&name) {
            refs.push(name);
        }
    }

    Ok(refs)
}
impl Default for DependencyGraph {
    /// Provide a default value using `DependencyGraph::new()` which creates an
    /// empty graph.
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::parse_component_refs;
    use std::fs;

    #[test]
    fn parses_pascal_and_canonical_component_refs() {
        let tmp = std::env::temp_dir().join("tinyfx_parse_component_refs_test.html");
        fs::write(
            &tmp,
            r#"
            <div>
              <Button variant="primary">Go</Button>
              <component name="Card"></component>
            </div>
            "#,
        )
        .unwrap();

        let refs = parse_component_refs(&tmp).unwrap();

        assert!(refs.contains(&"Button".to_string()));
        assert!(refs.contains(&"Card".to_string()));

        let _ = fs::remove_file(&tmp);
    }
}

