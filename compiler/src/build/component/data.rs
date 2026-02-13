// ─────────────────────────────────────────────────────────────────────────────
// data.rs — Component struct (pure data + template I/O)
// ─────────────────────────────────────────────────────────────────────────────
//
// This module owns the *data* side of a component: its identity (name),
// where its template lives on disk, and how to load and render that template.
//
// It deliberately does NOT know how to find `<component>` tags in HTML or how
// to recursively resolve an entire page. That responsibility belongs to the
// parser and renderer modules respectively.
//
// Separation rationale:
//   • data.rs  — "what a component IS"   (struct, template loading, rendering)
//   • parser.rs — "how to find one in HTML" (span detection, attribute parsing)
//   • renderer.rs — "how to expand them all" (orchestration loop)
// ─────────────────────────────────────────────────────────────────────────────

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use regex::Regex;

// ─── Component struct ────────────────────────────────────────────────────────

/// A resolved component handle.
///
/// Holds the component's name (e.g. "Button") and the expected path to its
/// `.html` template file. This struct is cheap to construct — it does no I/O
/// until you call `load_template()` or `render()`.
pub struct Component {
    /// The PascalCase component name, e.g. "Button" or "NavBar".
    pub name: String,

    /// Absolute (or project-relative) path to the component's template file,
    /// e.g. `src/components/Button/Button.html`.
    pub template_path: PathBuf,
}

impl Component {
    /// Create a new `Component` handle from a name and the project's
    /// components directory.
    ///
    /// The template is expected at `<components_dir>/<name>/<name>.html`.
    /// No I/O is performed here — use `exists()` to check the file first.
    ///
    /// # Example
    /// ```ignore
    /// let btn = Component::new("Button", Path::new("src/components"));
    /// // btn.template_path == "src/components/Button/Button.html"
    /// ```
    pub fn new(name: &str, components_dir: &Path) -> Self {
        Self {
            name: name.to_string(),
            template_path: components_dir.join(name).join(format!("{}.html", name)),
        }
    }

    /// Returns `true` if the template file exists on disk.
    pub fn exists(&self) -> bool {
        self.template_path.exists()
    }

    /// Read the raw template HTML from disk.
    ///
    /// Fails with a user-friendly message if the file is missing.
    pub fn load_template(&self) -> Result<String> {
        if !self.exists() {
            bail!(
                "error: component \"{}\" not found\n  → expected: {}",
                self.name,
                self.template_path.display()
            );
        }
        fs::read_to_string(&self.template_path)
            .with_context(|| format!("Failed to read {}", self.template_path.display()))
    }

    /// Render the component by applying three transformations in order:
    ///
    /// 1. **Slot injection** — replace `<slot></slot>` in the template with
    ///    the inner HTML from the call-site (`slot_content`).
    ///
    /// 2. **Token interpolation** — replace `{{prop}}` placeholders with values
    ///    from the attribute map (`attrs`). Unmatched tokens are left as-is so
    ///    downstream code (or the developer) can spot them.
    ///
    /// 3. **`data-component` stamp** — ensure the root element carries a
    ///    `data-component="Name"` attribute so the runtime can bind behavior.
    ///
    /// # Arguments
    /// - `attrs`        — key/value pairs parsed from `<component ... >`.
    /// - `slot_content` — the inner HTML between `<component>` and `</component>`.
    /// - `token_re`     — precompiled regex for `{{...}}` tokens (shared across
    ///                    iterations so we don't recompile it every call).
    pub fn render(
        &self,
        attrs: &HashMap<String, String>,
        slot_content: &str,
        token_re: &Regex,
    ) -> Result<String> {
        let template = self.load_template()?;

        // Step 1 — inject slot content into the template
        let with_slot = template.replace("<slot></slot>", slot_content);

        // Step 2 — interpolate {{prop}} tokens from attributes
        let rendered = token_re
            .replace_all(&with_slot, |caps: &regex::Captures| {
                let key = caps.get(1).map(|m| m.as_str()).unwrap_or_default();
                attrs
                    .get(key)
                    .cloned()
                    .unwrap_or_else(|| caps.get(0).unwrap().as_str().to_string())
            })
            .to_string();

        // Step 3 — stamp the root element with data-component="Name"
        Ok(ensure_data_component(&rendered, &self.name))
    }
}

// ─── Internal helper ─────────────────────────────────────────────────────────

/// Ensure the root element of `html` carries a `data-component="name"` attribute.
///
/// If the attribute already exists the string is returned unchanged.
/// Otherwise it is injected just before the first `>` of the root tag.
///
/// This attribute is what the runtime queries (`[data-component]`) to bind
/// the component's TypeScript mount function to the correct DOM node.
fn ensure_data_component(html: &str, name: &str) -> String {
    let trimmed = html.trim();
    let attr = format!("data-component=\"{}\"", name);

    // Already present — nothing to do.
    if trimmed.contains(&attr) {
        return html.to_string();
    }

    // Find the first `>` that closes the root opening tag and inject before it.
    if let Some(pos) = trimmed.find('>') {
        let before = &trimmed[..pos];
        let after = &trimmed[pos..];

        // Guard: only inject into real elements, not comments `<!-- ... -->`.
        if before.starts_with('<') && !before.starts_with("<!") {
            return format!("{} {}{}", before, attr, after);
        }
    }

    // Fallback: return as-is (e.g. the template is just a text node).
    html.to_string()
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify that `ensure_data_component` adds the attribute when absent.
    #[test]
    fn adds_data_component_attr() {
        let html = r#"<button class="btn">Click</button>"#;
        let result = ensure_data_component(html, "Button");
        assert!(
            result.contains(r#"data-component="Button""#),
            "Expected data-component attribute in: {}",
            result,
        );
    }

    /// Verify that `ensure_data_component` does not duplicate the attribute.
    #[test]
    fn skips_existing_data_component_attr() {
        let html = r#"<button data-component="Button" class="btn">Click</button>"#;
        let result = ensure_data_component(html, "Button");
        assert_eq!(
            result, html,
            "Should not modify HTML that already has the attribute"
        );
    }
}
