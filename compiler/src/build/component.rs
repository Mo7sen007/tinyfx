//! # component
//!
//! Component registry construction and HTML expansion for TinyFX builds.
//!
//! ## Responsibilities
//! - Compile component templates into a dependency-ordered registry
//! - Normalize supported component syntaxes into a canonical form
//! - Expand component tags using compiled component HTML
//! - Stamp dynamic components with `data-component` markers
//!
//! ## What this module does NOT do
//! - It does not scan the filesystem for pages
//! - It does not generate glue code
//! - It does not orchestrate build phases

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

use anyhow::{bail, Context, Result};
use regex::Regex;

use super::dependency_graph::DependencyGraph;

const MAX_EXPANSIONS: usize = 500;

/// A compiled component ready for page expansion and glue generation.
///
/// The `html` field contains the fully expanded component template after any
/// nested component usage has already been resolved.
#[derive(Debug, Clone)]
pub struct CompiledComponent {
    pub name: String,
    pub path: PathBuf,
    pub html: String,
    pub is_dynamic: bool,
}

/// Build a component registry in dependency order.
///
/// Each component template is read from disk, expanded against its already
/// compiled dependencies, and stored for later page processing and glue
/// generation.
///
/// # Errors
/// Returns an error when a component template cannot be read, when expansion
/// fails, or when a component is marked `tinyfx-dynamic` but its matching
/// TypeScript file is missing.
pub fn build_component_registry(
    dependency_graph: &DependencyGraph,
    build_order: &[PathBuf],
) -> Result<HashMap<String, CompiledComponent>> {
    let mut component_registry = HashMap::new();

    for component_path in build_order
        .iter()
        .filter(|path| dependency_graph.components.contains_key(*path))
    {
        let component_meta = &dependency_graph.components[component_path];
        let raw_html = fs::read_to_string(component_path).with_context(|| {
            format!(
                "failed to read component '{}'\n  → confirm the file exists and is readable",
                component_path.display()
            )
        })?;
        let expanded_html = expand_component_tags(&raw_html, &component_registry)?;
        let is_dynamic = raw_html.contains("tinyfx-dynamic");
        let script_path = component_path.with_extension("ts");

        if is_dynamic && !script_path.exists() {
            bail!(
                "component '{}' is marked tinyfx-dynamic but '{}' is missing\n  → create the TypeScript file or remove the tinyfx-dynamic marker",
                component_meta.name,
                script_path.display()
            );
        }

        if !is_dynamic && script_path.exists() {
            eprintln!(
                "warning: component '{}' has '{}' but is not marked tinyfx-dynamic\n  → add tinyfx-dynamic to the template root or remove the TypeScript file",
                component_meta.name,
                script_path.display()
            );
        }

        component_registry.insert(
            component_meta.name.clone(),
            CompiledComponent {
                name: component_meta.name.clone(),
                path: component_path.clone(),
                html: expanded_html,
                is_dynamic,
            },
        );
    }

    Ok(component_registry)
}

/// Expand canonical and PascalCase component tags using the compiled registry.
///
/// Supports both `<component name="Button">` and `<Button>` syntaxes and
/// substitutes slots and `{{token}}` values using the resolved component HTML.
/// Unknown components are left in place so callers can warn without aborting
/// the entire build.
///
/// # Errors
/// Returns an error when a canonical component tag is malformed, when token
/// substitution hits invalid component syntax, or when nested expansion exceeds
/// the hard safety limit used to catch circular references.
pub fn expand_component_tags(
    html: &str,
    component_registry: &HashMap<String, CompiledComponent>,
) -> Result<String> {
    let mut expanded_html = normalize_component_tags(html);

    for iteration in 0..MAX_EXPANSIONS {
        let Some(component_span) = find_component_span(&expanded_html) else {
            return Ok(expanded_html);
        };

        let open_tag = &expanded_html[component_span.open_start..component_span.open_end];
        let slot_content = expanded_html[component_span.open_end..component_span.close_start]
            .to_string();
        let attrs = parse_attrs(open_tag)?;
        let Some(component_name) = attrs.get("name") else {
            bail!(
                "component tag is missing a 'name' attribute\n  → use <component name=\"Button\">...</component>"
            );
        };

        let Some(component) = component_registry.get(component_name) else {
            break;
        };

        let rendered_component = substitute(component, &attrs, &slot_content)?;
        expanded_html.replace_range(
            component_span.open_start..component_span.close_end,
            &rendered_component,
        );

        if iteration + 1 == MAX_EXPANSIONS {
            bail!(
                "component expansion exceeded {} iterations\n  → check for circular component references in your templates",
                MAX_EXPANSIONS
            );
        }
    }

    Ok(expanded_html)
}

fn normalize_component_tags(input: &str) -> String {
    let normalized = pascal_self_closing_tag_re()
        .replace_all(input, |captures: &regex::Captures| {
            let component_name = captures.get(1).map(|m| m.as_str()).unwrap_or_default();
            let attrs = normalized_attrs(captures.get(2).map(|m| m.as_str()).unwrap_or_default());
            format!("<component name=\"{}\"{}></component>", component_name, attrs)
        })
        .to_string();

    let normalized = pascal_open_tag_re()
        .replace_all(&normalized, |captures: &regex::Captures| {
            let component_name = captures.get(1).map(|m| m.as_str()).unwrap_or_default();
            let attrs = normalized_attrs(captures.get(2).map(|m| m.as_str()).unwrap_or_default());
            format!("<component name=\"{}\"{}>", component_name, attrs)
        })
        .to_string();

    let normalized = pascal_close_tag_re()
        .replace_all(&normalized, "</component>")
        .to_string();

    component_self_closing_tag_re()
        .replace_all(&normalized, |captures: &regex::Captures| {
            let attrs = captures.get(1).map(|m| m.as_str()).unwrap_or_default().trim();
            if attrs.is_empty() {
                "<component></component>".to_string()
            } else {
                format!("<component {}></component>", attrs)
            }
        })
        .to_string()
}

fn substitute(
    component: &CompiledComponent,
    attrs: &HashMap<String, String>,
    slot_content: &str,
) -> Result<String> {
    let rendered_html = token_re()
        .replace_all(&component.html, |captures: &regex::Captures| {
            let token_name = captures.get(1).map(|m| m.as_str()).unwrap_or_default().trim();
            attrs.get(token_name)
                .cloned()
                .unwrap_or_else(|| captures.get(0).map(|m| m.as_str()).unwrap_or_default().to_string())
        })
        .to_string()
        .replace("<slot></slot>", slot_content)
        .replace("<slot/>", slot_content)
        .replace("<slot />", slot_content);

    if component.is_dynamic {
        return Ok(ensure_data_component(&rendered_html, &component.name));
    }

    Ok(rendered_html)
}

#[derive(Debug, Clone, Copy)]
struct ComponentSpan {
    open_start: usize,
    open_end: usize,
    close_start: usize,
    close_end: usize,
}

fn find_component_span(html: &str) -> Option<ComponentSpan> {
    let open_start = html.find("<component")?;
    let open_end = open_start + html[open_start..].find('>')? + 1;

    let mut depth = 1usize;
    let mut position = open_end;

    loop {
        let next_open = html[position..].find("<component");
        let next_close = html[position..].find("</component>");
        let close_relative = next_close?;

        if let Some(open_relative) = next_open {
            if open_relative < close_relative {
                depth += 1;
                position += open_relative + "<component".len();
                continue;
            }
        }

        depth = depth.saturating_sub(1);
        let close_start = position + close_relative;
        let close_end = close_start + "</component>".len();

        if depth == 0 {
            return Some(ComponentSpan {
                open_start,
                open_end,
                close_start,
                close_end,
            });
        }

        position = close_end;
    }
}

fn parse_attrs(open_tag: &str) -> Result<HashMap<String, String>> {
    let mut attrs = HashMap::new();
    let mut input = open_tag.trim();

    if !input.starts_with("<component") {
        bail!(
            "expected a canonical component tag but found '{}'\n  → normalize tags before parsing attributes",
            open_tag
        );
    }

    input = input.trim_start_matches("<component");
    input = input.trim_end_matches('>');

    let chars: Vec<char> = input.chars().collect();
    let mut index = 0;

    while index < chars.len() {
        while index < chars.len() && chars[index].is_whitespace() {
            index += 1;
        }
        if index >= chars.len() {
            break;
        }

        let key_start = index;
        while index < chars.len()
            && !chars[index].is_whitespace()
            && chars[index] != '='
            && chars[index] != '>'
        {
            index += 1;
        }
        let key: String = chars[key_start..index].iter().collect();
        if key.is_empty() {
            break;
        }

        while index < chars.len() && chars[index].is_whitespace() {
            index += 1;
        }

        if index >= chars.len() || chars[index] != '=' {
            attrs.insert(key, "true".to_string());
            continue;
        }
        index += 1;

        while index < chars.len() && chars[index].is_whitespace() {
            index += 1;
        }

        if index >= chars.len() {
            attrs.insert(key, String::new());
            break;
        }

        let value = if chars[index] == '"' || chars[index] == '\'' {
            let quote = chars[index];
            index += 1;
            let value_start = index;
            while index < chars.len() && chars[index] != quote {
                index += 1;
            }
            let value: String = chars[value_start..index].iter().collect();
            if index < chars.len() {
                index += 1;
            }
            value
        } else {
            let value_start = index;
            while index < chars.len() && !chars[index].is_whitespace() && chars[index] != '>' {
                index += 1;
            }
            chars[value_start..index].iter().collect()
        };

        attrs.insert(key, value);
    }

    Ok(attrs)
}

fn ensure_data_component(html: &str, component_name: &str) -> String {
    let data_attr = format!("data-component=\"{}\"", component_name);
    if html.contains(&data_attr) {
        return html.to_string();
    }

    if let Some(tag_end) = html.find('>') {
        let open_tag = &html[..tag_end];
        let rest = &html[tag_end..];
        if open_tag.starts_with('<') && !open_tag.starts_with("<!") {
            return format!("{} {}{}", open_tag, data_attr, rest);
        }
    }

    html.to_string()
}

fn normalized_attrs(raw_attrs: &str) -> String {
    let trimmed = raw_attrs.trim();
    if trimmed.is_empty() {
        String::new()
    } else {
        format!(" {}", trimmed)
    }
}

fn pascal_self_closing_tag_re() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r#"<([A-Z][A-Za-z0-9]*)\b([^>]*)/\s*>"#).unwrap())
}

fn pascal_open_tag_re() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r#"<([A-Z][A-Za-z0-9]*)\b([^>]*)>"#).unwrap())
}

fn pascal_close_tag_re() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r#"</([A-Z][A-Za-z0-9]*)\s*>"#).unwrap())
}

fn component_self_closing_tag_re() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r#"<component\b([^>]*)/\s*>"#).unwrap())
}

fn token_re() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r#"\{\{\s*([a-zA-Z0-9_-]+)\s*\}\}"#).unwrap())
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
        let mut component_registry = HashMap::new();
        component_registry.insert(
            "Button".to_string(),
            static_component("Button", r#"<button>{{label}}<slot></slot></button>"#),
        );

        let result = expand_component_tags(
            r#"<component name="Button" label="Click">Now</component>"#,
            &component_registry,
        )
        .expect("component name syntax should expand");

        assert!(result.contains("<button>ClickNow</button>"));
        assert!(!result.contains("<component"));
    }

    #[test]
    fn expands_pascal_case_syntax_via_normalization() {
        let mut component_registry = HashMap::new();
        component_registry.insert(
            "Button".to_string(),
            static_component("Button", r#"<button>{{variant}}<slot></slot></button>"#),
        );

        let result = expand_component_tags(r#"<Button variant="primary">Go</Button>"#, &component_registry)
            .expect("PascalCase syntax should normalize and expand");

        assert!(result.contains("<button>primaryGo</button>"));
        assert!(!result.contains("<Button"));
    }

    #[test]
    fn expands_nested_components_from_registry() {
        let mut component_registry = HashMap::new();
        component_registry.insert(
            "Icon".to_string(),
            static_component("Icon", r#"<i class="icon-{{kind}}"></i>"#),
        );
        component_registry.insert(
            "Button".to_string(),
            static_component(
                "Button",
                r#"<button><component name="Icon" kind="check"></component><slot></slot></button>"#,
            ),
        );

        let result = expand_component_tags(
            r#"<component name="Button">Save</component>"#,
            &component_registry,
        )
        .expect("nested components should expand");

        assert!(result.contains(r#"<i class="icon-check"></i>"#));
        assert!(result.contains("Save"));
        assert!(!result.contains("<component"));
    }
}
