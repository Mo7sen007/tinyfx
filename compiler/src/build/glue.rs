use std::collections::HashMap;
use std::path::{Path, PathBuf};
#[path = "generate_registry.rs"]
mod generate_registry;
pub use generate_registry::generate_registry;

use super::CompiledComponent;
use super::pages::PageDef;

/// Generate `tinyfx.gen.ts` — the component mount registry.
///
/// For each dynamic component that has a `.ts` file exporting a `mount()` factory,
/// emit an import and wire it into `setupComponents(ctx)`.
/// Returns class instances for optional teardown (e.g. calling `destroy()`).


fn infer_src_dir(registry: &HashMap<String, CompiledComponent>, pages: &[PageDef]) -> PathBuf {
    for component in registry.values() {
        if let Some(src_dir) = find_src_ancestor(&component.path) {
            return src_dir;
        }
    }

    for page in pages {
        if let Some(src_dir) = find_src_ancestor(&page.source_path) {
            return src_dir;
        }
    }

    PathBuf::from(".")
}

fn find_src_ancestor(path: &Path) -> Option<PathBuf> {
    for ancestor in path.ancestors() {
        if ancestor.file_name().and_then(|n| n.to_str()) == Some("src") {
            return Some(ancestor.to_path_buf());
        }
    }
    None
}

/// Compute a relative import path from `from_dir` to `target` (without extension).
/// Both should be relative to the project root.
fn relative_import(from_dir: &Path, target: &Path) -> String {
    let from_parts: Vec<_> = from_dir.components().collect();
    let to_parts: Vec<_> = target.components().collect();

    // Find common prefix length
    let mut common = 0;
    while common < from_parts.len()
        && common < to_parts.len()
        && from_parts[common] == to_parts[common]
    {
        common += 1;
    }

    let mut result = String::new();
    let ups = from_parts.len() - common;
    if ups == 0 {
        result.push_str("./");
    } else {
        for _ in 0..ups {
            result.push_str("../");
        }
    }

    for (i, part) in to_parts[common..].iter().enumerate() {
        if i > 0 {
            result.push('/');
        }
        result.push_str(&part.as_os_str().to_string_lossy());
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn empty_registry() {
        let result = super::generate_registry(&HashMap::new(), &[]);
        assert!(result.contains("const registry"));
        assert!(result.contains("setupComponents"));
        assert!(result.contains("bootstrapTinyFx"));
        assert!(result.contains("createContext"));
        assert!(result.contains("import type { TinyFxContext }"));
        assert!(!result.contains("import * as"));
        assert!(result.contains("initRouter"));
        // Legacy initComponents was removed — should NOT be present
        assert!(!result.contains("initComponents"));
    }

    #[test]
    fn relative_import_same_parent() {
        let result = relative_import(Path::new("src"), Path::new("src/components/Button/Button"));
        assert_eq!(result, "./components/Button/Button");
    }

    #[test]
    fn relative_import_sibling() {
        let result = relative_import(
            Path::new("src/lib"),
            Path::new("src/components/Button/Button"),
        );
        assert_eq!(result, "../components/Button/Button");
    }
}
