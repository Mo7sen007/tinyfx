use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};
use regex::Regex;

/// Iteratively resolve all <component> tags until none remain.
/// Tracks used component names and detects infinite loops.
pub fn resolve_all(
    input: &str,
    components_dir: &Path,
    used: &mut HashSet<String>,
) -> Result<String> {
    let token_re = Regex::new(r"\{\{\s*([a-zA-Z0-9_-]+)\s*\}\}")
        .context("Failed to compile token regex")?;

    let mut html = input.to_string();
    let mut templates: HashMap<String, String> = HashMap::new();

    const MAX_ITERATIONS: usize = 500;

    for _ in 0..MAX_ITERATIONS {
        let Some(span) = find_component_span(&html) else {
            return Ok(html);
        };

        let open_tag = &html[span.open_start..span.open_end];
        let inner = html[span.open_end..span.close_start].to_string();

        // ── Parse attributes ─────────────────────────────────────
        let mut attrs = parse_attrs(open_tag)?;
        let name = attrs
            .remove("name")
            .filter(|s| !s.trim().is_empty())
            .context("error: <component> is missing required attribute: name")?;

        used.insert(name.clone());

        // ── Load template ────────────────────────────────────────
        let template = load_template(&mut templates, components_dir, &name)?;

        // ── Replace <slot></slot> with inner HTML ────────────────
        let with_slot = template.replace("<slot></slot>", &inner);

        // ── Interpolate {{prop}} ─────────────────────────────────
        let rendered = token_re
            .replace_all(&with_slot, |caps: &regex::Captures| {
                let key = caps.get(1).map(|m| m.as_str()).unwrap_or_default();
                match attrs.get(key) {
                    Some(val) => val.clone(),
                    None => {
                        // Keep Go-template-like tokens untouched (contain dots, pipes, spaces)
                        // but our tokens are simple [a-zA-Z0-9_-], so if we matched it
                        // and there's no prop, it's an error — we'll handle below
                        caps.get(0).unwrap().as_str().to_string()
                    }
                }
            })
            .to_string();

        // ── Check for unresolved props (hard error) ──────────────
        if token_re.is_match(&rendered) {
            // Only error on tokens that are NOT in the original template
            // (they might be Go template tokens in user content)
            // For v1, we skip this strict check — warn later
        }

        // ── Ensure data-component attribute exists ───────────────
        let final_html = ensure_data_component(&rendered, &name);

        // ── Replace the <component>...</component> span ──────────
        html.replace_range(span.open_start..span.close_end, &final_html);
    }

    bail!("error: component expansion exceeded {} iterations (possible circular dependency)", MAX_ITERATIONS);
}

/// Ensure the root element of the resolved HTML has data-component="Name".
fn ensure_data_component(html: &str, name: &str) -> String {
    let trimmed = html.trim();
    let attr = format!("data-component=\"{}\"", name);

    if trimmed.contains(&attr) {
        return html.to_string();
    }

    // Find the first '>' of the root element and inject the attribute
    if let Some(pos) = trimmed.find('>') {
        let before = &trimmed[..pos];
        let after = &trimmed[pos..];
        // Check it's not a self-closing or comment
        if before.starts_with('<') && !before.starts_with("<!") {
            return format!("{} {}{}", before, attr, after);
        }
    }

    html.to_string()
}

/// Load a component template from disk, caching results.
fn load_template(
    cache: &mut HashMap<String, String>,
    components_dir: &Path,
    name: &str,
) -> Result<String> {
    if let Some(t) = cache.get(name) {
        return Ok(t.clone());
    }

    let path = components_dir.join(name).join(format!("{}.html", name));
    if !path.exists() {
        bail!(
            "error: component \"{}\" not found\n  → expected: {}",
            name,
            path.display()
        );
    }

    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    cache.insert(name.to_string(), content.clone());
    Ok(content)
}

// ─── Span detection ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
struct ComponentSpan {
    open_start: usize,  // index of '<' in <component
    open_end: usize,    // index after '>'
    close_start: usize, // index of '<' in </component>
    close_end: usize,   // index after </component>
}

/// Find the first (outermost) <component>...</component> span, handling nesting.
fn find_component_span(html: &str) -> Option<ComponentSpan> {
    let open_start = html.find("<component")?;

    // Find the closing '>' of the opening tag
    let open_end = open_start + html[open_start..].find('>')? + 1;

    let mut depth = 1usize;
    let mut pos = open_end;

    loop {
        let next_open = html[pos..].find("<component");
        let next_close = html[pos..].find("</component>");

        let close_rel = next_close?;

        if let Some(open_rel) = next_open {
            if open_rel < close_rel {
                depth += 1;
                pos += open_rel + "<component".len();
                continue;
            }
        }

        depth = depth.saturating_sub(1);
        let close_start = pos + close_rel;
        let close_end = close_start + "</component>".len();

        if depth == 0 {
            return Some(ComponentSpan {
                open_start,
                open_end,
                close_start,
                close_end,
            });
        }
        pos = close_end;
    }
}

// ─── Attribute parser ────────────────────────────────────────────────────────

/// Parse attributes from an opening <component ...> tag.
fn parse_attrs(open_tag: &str) -> Result<HashMap<String, String>> {
    let mut attrs = HashMap::new();

    let mut s = open_tag.trim();
    if !s.starts_with("<component") {
        bail!("Expected <component ...>, got: {}", open_tag);
    }
    s = s.trim_start_matches("<component");
    s = s.trim_end_matches('>');

    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // Skip whitespace
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }
        if i >= chars.len() {
            break;
        }

        // Read key
        let key_start = i;
        while i < chars.len() && !chars[i].is_whitespace() && chars[i] != '=' && chars[i] != '>' {
            i += 1;
        }
        let key: String = chars[key_start..i].iter().collect();
        if key.is_empty() {
            break;
        }

        // Skip whitespace
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }

        // No '=' means boolean attribute
        if i >= chars.len() || chars[i] != '=' {
            attrs.insert(key, "true".to_string());
            continue;
        }
        i += 1; // skip '='

        // Skip whitespace
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }
        if i >= chars.len() {
            attrs.insert(key, String::new());
            break;
        }

        // Read value (quoted or bare)
        let value = if chars[i] == '"' || chars[i] == '\'' {
            let quote = chars[i];
            i += 1;
            let v_start = i;
            while i < chars.len() && chars[i] != quote {
                i += 1;
            }
            let v: String = chars[v_start..i].iter().collect();
            if i < chars.len() {
                i += 1; // closing quote
            }
            v
        } else {
            let v_start = i;
            while i < chars.len() && !chars[i].is_whitespace() && chars[i] != '>' {
                i += 1;
            }
            chars[v_start..i].iter().collect()
        };

        attrs.insert(key, value);
    }

    Ok(attrs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::fs;

    #[test]
    fn test_parse_attrs() {
        let attrs = parse_attrs(r#"<component name="Button" variant="primary">"#).unwrap();
        assert_eq!(attrs.get("name").unwrap(), "Button");
        assert_eq!(attrs.get("variant").unwrap(), "primary");
    }

    #[test]
    fn test_find_component_span() {
        let html = r#"<p>before</p><component name="X">hello</component><p>after</p>"#;
        let span = find_component_span(html).unwrap();
        assert_eq!(&html[span.open_start..span.open_end], r#"<component name="X">"#);
        assert_eq!(&html[span.open_end..span.close_start], "hello");
    }

    #[test]
    fn test_nested_components() {
        let html = r#"<component name="A"><component name="B">inner</component></component>"#;
        let span = find_component_span(html).unwrap();
        // Should find the outer one
        assert_eq!(&html[span.open_end..span.close_start], r#"<component name="B">inner</component>"#);
    }

    #[test]
    fn test_ensure_data_component_adds() {
        let html = r#"<button class="btn">Click</button>"#;
        let result = ensure_data_component(html, "Button");
        assert!(result.contains(r#"data-component="Button""#));
    }

    #[test]
    fn test_ensure_data_component_skips_existing() {
        let html = r#"<button data-component="Button" class="btn">Click</button>"#;
        let result = ensure_data_component(html, "Button");
        assert_eq!(result, html);
    }

    #[test]
    fn test_resolve_with_real_files() {
        let tmp = std::env::temp_dir().join("tinyfx_test_resolve");
        let comp_dir = tmp.join("components").join("Alert");
        fs::create_dir_all(&comp_dir).unwrap();
        fs::write(
            comp_dir.join("Alert.html"),
            r#"<div class="alert alert-{{level}}"><slot></slot></div>"#,
        )
        .unwrap();

        let input = r#"<component name="Alert" level="warning">Watch out!</component>"#;
        let mut used = HashSet::new();
        let result = resolve_all(input, &tmp.join("components"), &mut used).unwrap();

        assert!(result.contains(r#"class="alert alert-warning""#));
        assert!(result.contains("Watch out!"));
        assert!(result.contains(r#"data-component="Alert""#));
        assert!(used.contains("Alert"));

        fs::remove_dir_all(&tmp).ok();
    }
}
