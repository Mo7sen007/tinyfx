// ─────────────────────────────────────────────────────────────────────────────
// renderer.rs — Iterative component expansion (the "build" orchestrator)
// ─────────────────────────────────────────────────────────────────────────────
//
// This module ties together `data::Component` and `parser::*` to implement
// the full component resolution pipeline:
//
//   1. Find the first `<component>` tag in the HTML  (parser)
//   2. Extract its attributes and slot content         (parser)
//   3. Load and render the matching template           (data)
//   4. Splice the rendered output back into the HTML
//   5. Repeat until no `<component>` tags remain
//
// The loop has a hard cap (`MAX_ITERATIONS`) to catch circular dependencies
// where component A includes B which includes A.
//
// Separation rationale:
//   • data.rs     — "what a component IS"
//   • parser.rs   — "how to find one in HTML"
//   • renderer.rs — "how to expand them all"   ← you are here
// ─────────────────────────────────────────────────────────────────────────────

use std::collections::HashSet;
use std::path::Path;

use anyhow::{Context, Result, bail};
use regex::Regex;

use super::data::Component;
use super::parser::{find_component_span, parse_attrs};

// ─── Configuration ──────────────────────────────────────────────────────────

/// Safety limit to prevent infinite expansion in case of circular component
/// references (e.g. A embeds B, B embeds A).
const MAX_ITERATIONS: usize = 500;

// ─── Public API ─────────────────────────────────────────────────────────────

/// Iteratively resolve **all** `<component>` tags in `input` until none remain.
///
/// Each iteration:
///   1. Finds the first (outermost) `<component …>…</component>` span.
///   2. Parses the opening tag to get the `name` attribute and any props.
///   3. Creates a `Component` handle, loads its template, and renders it
///      (slot injection + token interpolation + data-component stamping).
///   4. Splices the rendered result back into the HTML string.
///
/// The set `used` accumulates every component name that was encountered, so
/// the caller (the build pipeline) knows which `.ts` files to wire into the
/// generated glue registry.
///
/// # Errors
/// - Returns an error if a `<component>` is missing the required `name` attr.
/// - Returns an error if a component's template file does not exist.
/// - Returns an error if expansion exceeds `MAX_ITERATIONS` (circular dep).
///
/// # Example (conceptual)
/// ```ignore
/// let html = r#"<component name="Alert" level="info">Hello</component>"#;
/// let mut used = HashSet::new();
/// let result = resolve_all(&html, Path::new("src/components"), &mut used)?;
/// // result ≈ `<div class="alert alert-info" data-component="Alert">Hello</div>`
/// // used   ≈ {"Alert"}
/// ```
pub fn resolve_all(
    input: &str,
    components_dir: &Path,
    used: &mut HashSet<String>,
) -> Result<String> {
    // Pre-compile the token regex once — it's reused for every component render
    // within this build invocation. Pattern: `{{ propName }}` with optional
    // whitespace inside the braces.
    let token_re =
        Regex::new(r"\{\{\s*([a-zA-Z0-9_-]+)\s*\}\}").context("Failed to compile token regex")?;

    let mut html = input.to_string();

    for iteration in 0..MAX_ITERATIONS {
        // ── Step 1: Find the next component tag ──────────────────
        let Some(span) = find_component_span(&html) else {
            // No more `<component>` tags — we're done!
            return Ok(html);
        };

        // ── Step 2: Extract the opening tag and slot content ─────
        let open_tag = &html[span.open_start..span.open_end];
        let slot_content = html[span.open_end..span.close_start].to_string();

        // ── Step 3: Parse attributes and pull out the name ───────
        let mut attrs = parse_attrs(open_tag)?;
        let name = attrs
            .remove("name")
            .filter(|s| !s.trim().is_empty())
            .with_context(|| {
                format!(
                    "error: <component> is missing required attribute: name (iteration {})",
                    iteration
                )
            })?;

        // Record this component as "used" for glue generation.
        used.insert(name.clone());

        // ── Step 4: Load the template, render, and splice ────────
        let comp = Component::new(&name, components_dir);
        let rendered = comp.render(&attrs, &slot_content, &token_re)?;

        // Replace the original `<component …>…</component>` span with the
        // fully rendered template output.
        html.replace_range(span.open_start..span.close_end, &rendered);
    }

    // If we get here, we've exhausted the iteration budget — almost certainly
    // a circular dependency.
    bail!(
        "error: component expansion exceeded {} iterations (possible circular dependency)",
        MAX_ITERATIONS
    );
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// End-to-end test: create a real component on disk and resolve it.
    ///
    /// Verifies:
    ///  - Token interpolation (`{{level}}` → `"warning"`)
    ///  - Slot injection (`<slot></slot>` → `"Watch out!"`)
    ///  - `data-component` attribute is stamped on the root element
    ///  - The component name is recorded in the `used` set
    #[test]
    fn resolves_component_with_real_files() {
        // Set up a temporary component directory.
        let tmp = std::env::temp_dir().join("tinyfx_test_resolve");
        let comp_dir = tmp.join("components").join("Alert");
        fs::create_dir_all(&comp_dir).unwrap();
        fs::write(
            comp_dir.join("Alert.html"),
            r#"<div class="alert alert-{{level}}"><slot></slot></div>"#,
        )
        .unwrap();

        // Input HTML using the component.
        let input = r#"<component name="Alert" level="warning">Watch out!</component>"#;
        let mut used = HashSet::new();
        let result = resolve_all(input, &tmp.join("components"), &mut used).unwrap();

        // Assertions.
        assert!(
            result.contains(r#"class="alert alert-warning""#),
            "Token {{level}} should be replaced with 'warning'",
        );
        assert!(
            result.contains("Watch out!"),
            "Slot content should be injected",
        );
        assert!(
            result.contains(r#"data-component="Alert""#),
            "Root element should be stamped with data-component",
        );
        assert!(
            used.contains("Alert"),
            "Alert should be recorded in the used set",
        );

        // Cleanup.
        fs::remove_dir_all(&tmp).ok();
    }
}
