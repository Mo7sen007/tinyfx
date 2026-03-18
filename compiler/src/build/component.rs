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

use anyhow::{Context, Result, bail};
use regex::Regex;

use super::codegen::{LoopSource, detect_value_kind};
use super::conditional::{IfSite, parse_if_blocks};
use super::dependency_graph::DependencyGraph;
use super::each::{EachSite, parse_each_blocks};

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
    pub each_sites: Vec<EachSite>,
    pub if_sites: Vec<IfSite>,
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
        let mut compiled_html = expanded_html;
        let mut each_sites = Vec::new();
        let mut if_sites = Vec::new();

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

        if !is_dynamic && has_directive_markup(&raw_html) {
            eprintln!(
                "warning: component '{}' has directive sites but no dynamic components\n  were found in its HTML\n  → ensure {} has tinyfx-dynamic on its root element",
                component_meta.name,
                component_path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("component template")
            );
        }

        if is_dynamic {
            let scope_name = component_meta.name.to_lowercase();
            let (with_each, sites_each) = parse_each_blocks(&compiled_html, &scope_name, &component_registry)
                .with_context(|| {
                    format!(
                        "failed to parse <each> directives in component '{}\n  → fix loop markup in '{}' and rebuild",
                        component_meta.name,
                        component_path.display()
                    )
                })?;
            let (with_conditionals, sites_if) = parse_if_blocks(&with_each, &scope_name, &component_registry)
                .with_context(|| {
                    format!(
                        "failed to parse <if> directives in component '{}\n  → fix conditional markup in '{}' and rebuild",
                        component_meta.name,
                        component_path.display()
                    )
                })?;

            validate_component_directive_visibility(
                &component_meta.name,
                &script_path,
                &sites_each,
                &sites_if,
                component_path,
            )?;

            compiled_html = with_conditionals;
            each_sites = sites_each;
            if_sites = sites_if;
        }

        component_registry.insert(
            component_meta.name.clone(),
            CompiledComponent {
                name: component_meta.name.clone(),
                path: component_path.clone(),
                html: compiled_html,
                is_dynamic,
                each_sites,
                if_sites,
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
        let slot_content =
            expanded_html[component_span.open_end..component_span.close_start].to_string();
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
            format!(
                "<component name=\"{}\"{}></component>",
                component_name, attrs
            )
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
            let attrs = captures
                .get(1)
                .map(|m| m.as_str())
                .unwrap_or_default()
                .trim();
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
            let token_name = captures
                .get(1)
                .map(|m| m.as_str())
                .unwrap_or_default()
                .trim();
            attrs.get(token_name).cloned().unwrap_or_else(|| {
                captures
                    .get(0)
                    .map(|m| m.as_str())
                    .unwrap_or_default()
                    .to_string()
            })
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

fn has_directive_markup(raw_html: &str) -> bool {
    raw_html.contains("<each") || raw_html.contains("<if")
}

fn validate_component_directive_visibility(
    component_name: &str,
    script_path: &PathBuf,
    each_sites: &[EachSite],
    if_sites: &[IfSite],
    template_path: &PathBuf,
) -> Result<()> {
    let script_source = fs::read_to_string(script_path).with_context(|| {
        format!(
            "failed to read component script '{}'\n  → confirm the file exists and is readable",
            script_path.display()
        )
    })?;

    for property in collect_referenced_properties(each_sites, if_sites) {
        ensure_property_is_public(
            component_name,
            &property,
            &script_source,
            template_path,
            each_sites,
            if_sites,
        )?;
    }

    Ok(())
}

fn collect_referenced_properties(each_sites: &[EachSite], if_sites: &[IfSite]) -> Vec<String> {
    let mut properties = Vec::new();

    for site in each_sites {
        match &site.source {
            LoopSource::Signal(name) => push_unique(&mut properties, name),
            LoopSource::Plain(expr) => {
                if let Some(name) = first_identifier(expr) {
                    push_unique(&mut properties, &name);
                }
            }
            LoopSource::Range(_, _) => {}
        }
    }

    for site in if_sites {
        for branch in &site.branches {
            if let Some(condition) = &branch.condition {
                let property = match detect_value_kind(condition) {
                    super::codegen::ValueKind::Signal(name) => Some(name),
                    super::codegen::ValueKind::Plain(expr) => first_identifier(&expr),
                };
                if let Some(name) = property {
                    push_unique(&mut properties, &name);
                }
            }
        }
    }

    properties
}

fn ensure_property_is_public(
    component_name: &str,
    property: &str,
    script_source: &str,
    template_path: &PathBuf,
    each_sites: &[EachSite],
    if_sites: &[IfSite],
) -> Result<()> {
    let escaped = regex::escape(property);
    let private_re = Regex::new(&format!(r"(?m)\b(private|protected)\s+{}\b", escaped))
        .with_context(|| {
            format!(
                "failed to compile visibility check regex for component '{}': {}",
                component_name, property
            )
        })?;
    let public_re = Regex::new(&format!(r"(?m)\bpublic\s+{}\b", escaped)).with_context(|| {
        format!(
            "failed to compile public member regex for component '{}': {}",
            component_name, property
        )
    })?;

    if private_re.is_match(script_source) || !public_re.is_match(script_source) {
        let each_source = each_sites
            .iter()
            .find_map(|site| match &site.source {
                LoopSource::Signal(name) if name == property => Some(format!("{}()", name)),
                LoopSource::Plain(expr) if first_identifier(expr).as_deref() == Some(property) => {
                    Some(expr.trim().to_string())
                }
                _ => None,
            })
            .unwrap_or_else(|| {
                if_sites
                    .iter()
                    .flat_map(|site| &site.branches)
                    .filter_map(|branch| branch.condition.as_ref())
                    .find(|condition| {
                        first_identifier(condition).as_deref() == Some(property)
                            || detect_value_kind(condition)
                                == super::codegen::ValueKind::Signal(property.to_string())
                    })
                    .cloned()
                    .unwrap_or_else(|| property.to_string())
            });

        bail!(
            "component '{}' uses directive source '{}' in {}\n  but '{}' is not declared public on the {} class\n  → change: private {} = signal<...>(...)\n  → to:     public {} = signal<...>(...)",
            component_name,
            each_source,
            template_path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("component template"),
            property,
            component_name,
            property,
            property
        );
    }

    Ok(())
}

fn push_unique(values: &mut Vec<String>, value: &str) {
    if !value.is_empty() && !values.iter().any(|existing| existing == value) {
        values.push(value.to_string());
    }
}

fn first_identifier(expr: &str) -> Option<String> {
    let trimmed = expr.trim();
    let mut out = String::new();
    for ch in trimmed.chars() {
        if ch.is_alphanumeric() || ch == '_' {
            out.push(ch);
        } else {
            break;
        }
    }

    if out.is_empty() { None } else { Some(out) }
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
            each_sites: Vec::new(),
            if_sites: Vec::new(),
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

        let result = expand_component_tags(
            r#"<Button variant="primary">Go</Button>"#,
            &component_registry,
        )
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
