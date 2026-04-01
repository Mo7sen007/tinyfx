//! # each
//!
//! Compile-time parsing for TinyFX `<each>` directives.
//!
//! Responsibilities: parse and validate `<each>` blocks, expand inner
//! components, replace blocks with deterministic containers, and collect
//! per-site metadata for code generation.
//!
//! This module does not implement runtime reconciliation, write output files,
//! or orchestrate build phases.

use std::collections::HashMap;
use std::sync::OnceLock;

use anyhow::{Context, Result, bail};
use regex::Regex;

use super::codegen::{LoopSource, parse_loop_source, site_id};
use super::component::{CompiledComponent, expand_component_tags};
use super::dependency_graph::collect_component_references;
use super::pages_util::{
    filter_non_directive_attrs, parse_attrs as parse_attrs_payload,
    parse_attrs_ordered as parse_attrs_ordered_payload, serialize_attrs_to_html,
};

/// Determines how list items are identified for reconciliation.
///
/// `Keyed` uses a named field on each item as the stable identity.
/// `Index` falls back to position and is suitable for primitives and
/// anonymous objects that do not expose a stable identifier.
#[derive(Debug, Clone, PartialEq)]
pub enum KeyStrategy {
    /// A named field on the item object is used as the key.
    /// `field_name` is the raw attribute value, for example `"id"`.
    Keyed { field_name: String },
    /// No key field is available and items are matched by index.
    ///
    /// This strategy updates by position and is not stable under reordering.
    Index,
}

/// The type of DOM mutation to perform for a template binding.
#[derive(Debug, Clone, PartialEq)]
pub enum BindingKind {
    /// Writes to `textContent`.
    TextContent,
    /// Writes to an attribute by name.
    Attribute(String),
}

/// A single resolved binding extracted from a loop template.
///
/// `field` is the loop-item field name (for example `"title"`).
/// `selector` is a structural CSS selector captured at compile time.
/// `kind` describes how generated update code mutates the selected node.
#[derive(Debug, Clone)]
pub struct TemplateBinding {
    pub field: String,
    pub selector: String,
    pub kind: BindingKind,
    pub attr_template: Option<String>,
}

/// Everything known about one `<each>` site at compile time.
#[derive(Debug, Clone)]
pub struct EachSite {
    pub id: String,
    pub source: LoopSource,
    pub item_var: String,
    pub key_strategy: KeyStrategy,
    pub template_html: String,
    pub bindings: Vec<TemplateBinding>,
    #[allow(dead_code)]
    pub has_dynamic_components: bool,
    #[allow(dead_code)]
    pub dynamic_component_names: Vec<String>,
}

/// Parses all `<each>` blocks in an HTML string.
///
/// Replaces each block with a deterministic `<div data-each="...">`
/// placeholder and returns the transformed HTML plus collected [`EachSite`]
/// metadata for later TypeScript generation.
///
/// `scope_name` is used to build stable directive site IDs and should be the
/// page or component scope token already used by the caller.
///
/// # Errors
///
/// Returns an error when required attributes are invalid, when referenced
/// components are missing, when template expansion fails, or when an expanded
/// `<each>` body has more than one root element.
pub fn parse_each_blocks(
    html: &str,
    scope_name: &str,
    source_file: &str,
    component_registry: &HashMap<String, CompiledComponent>,
) -> Result<(String, Vec<EachSite>)> {
    let mut out = String::new();
    let mut cursor = 0usize;
    let mut index = 0usize;
    let mut sites = Vec::new();

    while let Some(span) = find_tag_span(html, "each", cursor) {
        out.push_str(&html[cursor..span.open_start]);

        let attrs = parse_attrs(&span.open_tag).with_context(|| {
            format!(
                "failed to parse <each> attributes in {}\n  → verify attribute quoting and the opening tag",
                source_file
            )
        })?;
        let of_attr = attrs.get("of").cloned().unwrap_or_default();
        let source = parse_loop_source(&of_attr).with_context(|| {
            format!(
                "invalid <each> source in {}\n  → use signal(), an array identifier, a positive integer, or a range",
                source_file
            )
        })?;

        let item_var = attrs.get("as").cloned().ok_or_else(|| {
            anyhow::anyhow!(
                "<each> block missing `as` attribute in {}\n  → add as=\"<variable>\" to name the loop variable\n  → example: <each of=\"posts()\" as=\"post\" key=\"id\">",
                source_file
            )
        })?;

        let key_strategy = match &source {
            LoopSource::Range(_, _) => KeyStrategy::Index,
            _ => match attrs.get("key") {
                Some(v) if !v.trim().is_empty() => KeyStrategy::Keyed {
                    field_name: v.trim().to_string(),
                },
                _ => {
                    let line = line_number_at(html, span.open_start);
                    eprintln!(
                        "warning: <each> block missing `key` in {} line {}\n  → using index-based keys (position identity)\n  → limitation: item reordering is not stable with index keys",
                        source_file, line
                    );
                    KeyStrategy::Index
                }
            },
        };

        let wrapper_attrs = filter_non_directive_attrs(&attrs, &["of", "as", "key", "data-each"]);
        let wrapper_attrs_html = serialize_attrs_to_html(&wrapper_attrs);

        let template_raw = html[span.open_end..span.close_start].to_string();
        ensure_components_registered(&template_raw, component_registry, source_file)?;
        let template_html = expand_component_tags(&template_raw, component_registry).with_context(|| {
            format!(
                "failed to expand <each> template components in {}\n  → inspect component tags inside the loop body",
                source_file
            )
        })?;

        let root_count = count_root_elements(&template_html);
        if root_count > 1 {
            let key_hint = match &key_strategy {
                KeyStrategy::Keyed { field_name } => field_name.clone(),
                KeyStrategy::Index => "index".to_string(),
            };
            bail!(
                "error: <each> block in {} has {} root elements\n  \
                 → wrap them in a single container element\n  \
                 → example:\n    \
                 <each of=\"{}\" as=\"{}\" key=\"{}\">\n      \
                 <div class=\"...\">\n        \
                 <!-- your elements here -->\n      \
                 </div>\n    \
                 </each>",
                source_file,
                root_count,
                of_attr,
                item_var,
                key_hint
            );
        }

        let bindings = extract_template_bindings(&template_html, &item_var);

        let id = site_id(scope_name, "each", index);
        out.push_str(&format!(
            "<div data-each=\"{}\"{}></div>",
            id, wrapper_attrs_html
        ));

        let dynamic_component_names = collect_dynamic_names(&template_html);
        sites.push(EachSite {
            id,
            source,
            item_var,
            key_strategy,
            has_dynamic_components: !dynamic_component_names.is_empty(),
            dynamic_component_names,
            template_html,
            bindings,
        });

        index += 1;
        cursor = span.close_end;
    }

    out.push_str(&html[cursor..]);
    Ok((out, sites))
}

#[derive(Debug, Clone)]
pub(crate) struct TagSpan {
    pub(crate) open_start: usize,
    pub(crate) open_end: usize,
    pub(crate) close_start: usize,
    pub(crate) close_end: usize,
    pub(crate) open_tag: String,
}

pub(crate) fn find_tag_span(html: &str, tag_name: &str, from: usize) -> Option<TagSpan> {
    let open_pat = format!("<{}", tag_name);
    let close_pat = format!("</{}>", tag_name);
    let open_rel = html[from..].find(&open_pat)?;
    let open_start = from + open_rel;
    let open_end = open_start + html[open_start..].find('>')? + 1;
    let mut depth = 1usize;
    let mut pos = open_end;

    while pos < html.len() {
        let next_open = html[pos..].find(&open_pat).map(|v| pos + v);
        let next_close = html[pos..].find(&close_pat).map(|v| pos + v);
        let close_start = next_close?;

        if let Some(open_nested) = next_open
            && open_nested < close_start
        {
            depth += 1;
            pos = open_nested + open_pat.len();
            continue;
        }

        depth = depth.saturating_sub(1);
        let close_end = close_start + close_pat.len();
        if depth == 0 {
            return Some(TagSpan {
                open_start,
                open_end,
                close_start,
                close_end,
                open_tag: html[open_start..open_end].to_string(),
            });
        }
        pos = close_end;
    }

    None
}

fn ensure_components_registered(
    html: &str,
    component_registry: &HashMap<String, CompiledComponent>,
    page_path: &str,
) -> Result<()> {
    for component_name in collect_component_references(html) {
        if !component_registry.contains_key(&component_name) {
            bail!(
                "<each> block references component '{}' which is not registered\n  in {}\n  → ensure src/components/{0}/{0}.html exists",
                component_name,
                page_path
            );
        }
    }
    Ok(())
}

fn collect_dynamic_names(html: &str) -> Vec<String> {
    let mut out = Vec::new();
    for captures in data_component_re().captures_iter(html) {
        let name = captures
            .get(1)
            .map(|m| m.as_str())
            .unwrap_or_default()
            .to_string();
        if !name.is_empty() && !out.contains(&name) {
            out.push(name);
        }
    }
    out
}

#[derive(Debug, Clone)]
struct ElementCtx {
    tag: String,
    id: Option<String>,
    first_class: Option<String>,
    nth_child: usize,
}

// Count top-level root elements in an HTML fragment.
fn count_root_elements(html: &str) -> usize {
    let mut index = 0usize;
    let mut depth = 0i32;
    let mut roots = 0usize;

    while index < html.len() {
        let Some(tag_rel) = html[index..].find('<') else {
            break;
        };
        let tag_start = index + tag_rel;
        let Some(tag_end_rel) = html[tag_start..].find('>') else {
            break;
        };
        let tag_end = tag_start + tag_end_rel;
        let raw_tag = html[tag_start + 1..tag_end].trim();

        if raw_tag.starts_with("!--") {
            index = tag_end + 1;
            continue;
        }

        if raw_tag.starts_with('!') || raw_tag.starts_with('?') {
            index = tag_end + 1;
            continue;
        }

        if raw_tag.starts_with('/') {
            if depth > 0 {
                depth -= 1;
            }
            index = tag_end + 1;
            continue;
        }

        let tag_name = raw_tag
            .split_whitespace()
            .next()
            .unwrap_or_default()
            .trim_end_matches('/');
        let lower = tag_name.to_ascii_lowercase();
        let self_closing = raw_tag.ends_with('/') || is_void_element(&lower);

        if depth == 0 {
            roots += 1;
        }

        if !self_closing {
            depth += 1;
        }

        index = tag_end + 1;
    }

    roots
}

// Extract first-seen template bindings from expanded loop HTML.
fn extract_template_bindings(template_html: &str, item_var: &str) -> Vec<TemplateBinding> {
    let mut out = Vec::new();
    let mut seen_fields = Vec::<String>::new();
    let mut stack: Vec<ElementCtx> = Vec::new();
    let mut child_counts: Vec<usize> = vec![0];
    let mut cursor = 0usize;

    while cursor < template_html.len() {
        let Some(next_tag_rel) = template_html[cursor..].find('<') else {
            let text = &template_html[cursor..];
            if !stack.is_empty() {
                collect_text_bindings(
                    text,
                    item_var,
                    selector_from_stack(&stack),
                    &mut seen_fields,
                    &mut out,
                );
            }
            break;
        };

        let next_tag = cursor + next_tag_rel;
        let text = &template_html[cursor..next_tag];
        if !stack.is_empty() {
            collect_text_bindings(
                text,
                item_var,
                selector_from_stack(&stack),
                &mut seen_fields,
                &mut out,
            );
        }

        let Some(tag_end_rel) = template_html[next_tag..].find('>') else {
            break;
        };
        let tag_end = next_tag + tag_end_rel;
        let tag_src = template_html[next_tag + 1..tag_end].trim();

        if tag_src.starts_with("!--") || tag_src.starts_with('!') || tag_src.starts_with('?') {
            cursor = tag_end + 1;
            continue;
        }

        if tag_src.starts_with('/') {
            if !stack.is_empty() {
                stack.pop();
            }
            if child_counts.len() > 1 {
                child_counts.pop();
            }
            cursor = tag_end + 1;
            continue;
        }

        let self_closing = tag_src.ends_with('/');
        let open_src = if self_closing {
            tag_src[..tag_src.len().saturating_sub(1)].trim_end()
        } else {
            tag_src
        };

        let mut parts = open_src.splitn(2, char::is_whitespace);
        let tag = parts.next().unwrap_or_default().trim().to_ascii_lowercase();
        if tag.is_empty() {
            cursor = tag_end + 1;
            continue;
        }

        let attrs_src = parts.next().unwrap_or_default();
        let parent_depth = child_counts.len().saturating_sub(1);
        child_counts[parent_depth] += 1;
        let nth_child = child_counts[parent_depth];

        let ctx = ElementCtx {
            tag: tag.clone(),
            id: attr_value(attrs_src, "id"),
            first_class: first_class_from_attrs(attrs_src),
            nth_child,
        };
        let selector = current_selector(&stack, &ctx);

        for (attr_name, attr_value) in parse_attrs_ordered_payload(attrs_src) {
            let attr_name = attr_name.as_str();
            let attr_value = attr_value.as_str();

            for field in extract_binding_fields(attr_value, item_var) {
                if seen_fields.iter().any(|existing| existing == &field) {
                    continue;
                }
                seen_fields.push(field.clone());
                out.push(TemplateBinding {
                    field,
                    selector: selector.clone(),
                    kind: BindingKind::Attribute(attr_name.to_string()),
                    attr_template: Some(attr_value.to_string()),
                });
            }
        }

        let is_void = is_void_element(&tag);
        if !self_closing && !is_void {
            stack.push(ctx);
            child_counts.push(0);
        }

        cursor = tag_end + 1;
    }

    out
}

fn collect_text_bindings(
    text: &str,
    item_var: &str,
    selector: String,
    seen_fields: &mut Vec<String>,
    out: &mut Vec<TemplateBinding>,
) {
    for field in extract_binding_fields(text, item_var) {
        if seen_fields.iter().any(|existing| existing == &field) {
            continue;
        }
        seen_fields.push(field.clone());
        out.push(TemplateBinding {
            field,
            selector: selector.clone(),
            kind: BindingKind::TextContent,
            attr_template: None,
        });
    }
}

fn extract_binding_fields(input: &str, item_var: &str) -> Vec<String> {
    let mut out = Vec::new();
    for cap in binding_re().captures_iter(input) {
        let raw = cap.get(1).map(|m| m.as_str()).unwrap_or_default().trim();
        if raw.is_empty() {
            continue;
        }

        let field = if let Some(rest) = raw.strip_prefix(&format!("{}.", item_var)) {
            if rest.contains('.') || rest.is_empty() {
                continue;
            }
            rest.to_string()
        } else {
            if raw.contains('.') || !is_identifier(raw) {
                continue;
            }
            raw.to_string()
        };

        if !out.iter().any(|existing| existing == &field) {
            out.push(field);
        }
    }
    out
}

fn current_selector(stack: &[ElementCtx], current: &ElementCtx) -> String {
    let mut parts = Vec::new();
    for ctx in stack {
        parts.push(selector_segment(ctx));
    }
    parts.push(selector_segment(current));
    parts.join(" > ")
}

fn selector_from_stack(stack: &[ElementCtx]) -> String {
    let mut parts = Vec::new();
    for ctx in stack {
        parts.push(selector_segment(ctx));
    }
    parts.join(" > ")
}

fn selector_segment(ctx: &ElementCtx) -> String {
    if let Some(id) = &ctx.id {
        return format!("{}#{}", ctx.tag, id);
    }
    if let Some(class_name) = &ctx.first_class {
        return format!("{}.{}", ctx.tag, class_name);
    }
    format!("{}:nth-child({})", ctx.tag, ctx.nth_child)
}

fn attr_value(attrs: &str, name: &str) -> Option<String> {
    parse_attrs_ordered_payload(attrs)
        .into_iter()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value)
        .map(|val| val.trim().to_string())
        .filter(|val| !val.is_empty())
}

fn first_class_from_attrs(attrs: &str) -> Option<String> {
    attr_value(attrs, "class").and_then(|classes| {
        classes
            .split_whitespace()
            .find(|class_name| {
                !class_name.is_empty()
                    && class_name
                        .chars()
                        .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
            })
            .map(|class_name| class_name.to_string())
    })
}

fn binding_re() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r#"\{\{\s*([^}]+)\s*\}\}"#).expect("valid binding regex"))
}

fn is_identifier(input: &str) -> bool {
    let mut chars = input.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }
    chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

fn is_void_element(tag_name: &str) -> bool {
    matches!(
        tag_name,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

fn parse_attrs(open_tag: &str) -> Result<HashMap<String, String>> {
    let mut input = open_tag
        .trim()
        .trim_start_matches("<each")
        .trim_end_matches('>');
    if input.ends_with('/') {
        input = input[..input.len().saturating_sub(1)].trim_end();
    }
    Ok(parse_attrs_payload(input))
}

fn line_number_at(source: &str, byte_index: usize) -> usize {
    source[..byte_index].bytes().filter(|b| *b == b'\n').count() + 1
}

fn data_component_re() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r#"data-component\s*=\s*["']([A-Za-z0-9_:-]+)["']"#)
            .expect("valid data-component regex")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn registry(dynamic: bool) -> HashMap<String, CompiledComponent> {
        let mut map = HashMap::new();
        map.insert(
            "PostCard".to_string(),
            CompiledComponent {
                name: "PostCard".to_string(),
                path: "src/components/PostCard/PostCard.html".into(),
                html: if dynamic {
                    "<article tinyfx-dynamic data-component=\"PostCard\" data-title=\"{{title}}\"></article>"
                        .to_string()
                } else {
                    "<article data-title=\"{{title}}\"></article>".to_string()
                },
                is_dynamic: dynamic,
                each_sites: Vec::new(),
                if_sites: Vec::new(),
            },
        );
        map
    }

    #[test]
    fn parse_simple_each_block() {
        let html = "<each of=\"posts()\" as=\"post\" key=\"id\"><div>{{post.title}}</div></each>";
        let (out, sites) =
            parse_each_blocks(html, "index", "src/pages/index.html", &registry(false))
                .expect("should parse each");
        assert!(out.contains("data-each=\"index_each_0\""));
        assert_eq!(sites.len(), 1);
    }

    #[test]
    fn parse_each_keyless_plain_array() {
        let html = "<each of=\"items\" as=\"item\"><div>{{item}}</div></each>";
        let (_, sites) = parse_each_blocks(html, "index", "src/pages/index.html", &registry(false))
            .expect("should parse keyless plain array");
        assert!(matches!(sites[0].source, LoopSource::Plain(_)));
        assert!(matches!(sites[0].key_strategy, KeyStrategy::Index));
    }

    #[test]
    fn parse_each_keyless_signal() {
        let html = "<each of=\"items()\" as=\"item\"><div>{{item}}</div></each>";
        let (_, sites) = parse_each_blocks(html, "index", "src/pages/index.html", &registry(false))
            .expect("should parse keyless signal");
        assert!(matches!(sites[0].source, LoopSource::Signal(_)));
        assert!(matches!(sites[0].key_strategy, KeyStrategy::Index));
    }

    #[test]
    fn parse_each_keyed_field() {
        let html = "<each of=\"posts\" as=\"post\" key=\"id\"><div>{{post.title}}</div></each>";
        let (_, sites) = parse_each_blocks(html, "index", "src/pages/index.html", &registry(false))
            .expect("should parse keyed field");
        assert_eq!(
            sites[0].key_strategy,
            KeyStrategy::Keyed {
                field_name: "id".to_string()
            }
        );
    }

    #[test]
    fn parse_each_extracts_text_bindings() {
        let html = "<each of=\"posts\" as=\"post\" key=\"id\"><p>{{post.title}}</p></each>";
        let (_, sites) = parse_each_blocks(html, "index", "src/pages/index.html", &registry(false))
            .expect("should parse text binding");
        assert_eq!(sites[0].bindings.len(), 1);
        assert_eq!(sites[0].bindings[0].field, "title");
        assert_eq!(sites[0].bindings[0].kind, BindingKind::TextContent);
        assert_eq!(sites[0].bindings[0].selector, "p:nth-child(1)");
    }

    #[test]
    fn parse_each_extracts_attr_bindings() {
        let html = "<each of=\"posts\" as=\"post\" key=\"id\"><img src=\"{{post.url}}\"></each>";
        let (_, sites) = parse_each_blocks(html, "index", "src/pages/index.html", &registry(false))
            .expect("should parse attr binding");
        assert_eq!(sites[0].bindings.len(), 1);
        assert_eq!(sites[0].bindings[0].field, "url");
        assert_eq!(
            sites[0].bindings[0].kind,
            BindingKind::Attribute("src".to_string())
        );
        assert_eq!(
            sites[0].bindings[0].attr_template.as_deref(),
            Some("{{post.url}}")
        );
    }

    #[test]
    fn parse_each_extracts_attr_template_with_static_and_dynamic_parts() {
        let html = "<each of=\"posts\" as=\"post\" key=\"id\"><div class=\"message {{post.own}}\"></div></each>";
        let (_, sites) = parse_each_blocks(
            html,
            "chat",
            "src/components/Chat/Chat.html",
            &registry(false),
        )
        .expect("should parse mixed class binding");

        assert_eq!(sites[0].bindings.len(), 1);
        assert_eq!(sites[0].bindings[0].field, "own");
        assert_eq!(
            sites[0].bindings[0].attr_template.as_deref(),
            Some("message {{post.own}}")
        );
    }

    #[test]
    fn parse_each_range_is_always_index_keyed() {
        let numeric = "<each of=\"10\" as=\"i\" key=\"id\"><span>{{i}}</span></each>";
        let explicit = "<each of=\"1..5\" as=\"i\" key=\"id\"><span>{{i}}</span></each>";

        let (_, numeric_sites) =
            parse_each_blocks(numeric, "index", "src/pages/index.html", &registry(false))
                .expect("numeric range should parse");
        let (_, explicit_sites) =
            parse_each_blocks(explicit, "index", "src/pages/index.html", &registry(false))
                .expect("explicit range should parse");

        assert!(matches!(numeric_sites[0].source, LoopSource::Range(0, 10)));
        assert!(matches!(explicit_sites[0].source, LoopSource::Range(1, 5)));
        assert!(matches!(numeric_sites[0].key_strategy, KeyStrategy::Index));
        assert!(matches!(explicit_sites[0].key_strategy, KeyStrategy::Index));
    }

    #[test]
    fn parse_each_single_root_passes() {
        let html = "<each of=\"items\" as=\"item\"><div><span>{{item}}</span></div></each>";
        parse_each_blocks(html, "index", "src/pages/index.html", &registry(false))
            .expect("single-root each should pass");
    }

    #[test]
    fn parse_each_multiple_roots_errors() {
        let html = "<each of=\"items\" as=\"item\"><div></div><p></p></each>";
        let err = parse_each_blocks(html, "index", "src/pages/index.html", &registry(false))
            .expect_err("multiple roots should error");
        assert!(err.to_string().contains("has 2 root elements"));
    }

    #[test]
    fn parse_each_multiple_roots_error_contains_source_file() {
        let html = "<each of=\"items\" as=\"item\"><div></div><p></p></each>";
        let err = parse_each_blocks(
            html,
            "chat",
            "src/components/Chat/Chat.html",
            &registry(false),
        )
        .expect_err("multiple roots should error");
        assert!(err.to_string().contains("src/components/Chat/Chat.html"));
    }

    #[test]
    fn parse_each_multiple_roots_error_contains_of_attr() {
        let html = "<each of=\"messages()\" as=\"message\"><div></div><p></p></each>";
        let err = parse_each_blocks(
            html,
            "chat",
            "src/components/Chat/Chat.html",
            &registry(false),
        )
        .expect_err("multiple roots should error");
        assert!(
            err.to_string()
                .contains("<each of=\"messages()\" as=\"message\" key=\"index\">")
        );
    }

    #[test]
    fn parse_each_text_binding_selector_not_duplicated() {
        let html = "<each of=\"messages()\" as=\"message\" key=\"id\"><div class=\"message\"><span class=\"user\">{{message.username}}</span></div></each>";
        let (_, sites) = parse_each_blocks(
            html,
            "chat",
            "src/components/Chat/Chat.html",
            &registry(false),
        )
        .expect("should parse nested text binding selector");

        assert_eq!(sites[0].bindings.len(), 1);
        assert_eq!(sites[0].bindings[0].field, "username");
        assert_eq!(sites[0].bindings[0].selector, "div.message > span.user");
    }
}
