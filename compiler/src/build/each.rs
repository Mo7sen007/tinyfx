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
use super::pages_util::{filter_non_directive_attrs, serialize_attrs_to_html};

/// Everything known about one `<each>` site at compile time.
#[derive(Debug, Clone)]
pub struct EachSite {
    pub id: String,
    pub source: LoopSource,
    pub item_var: String,
    pub key_expr: String,
    pub template_html: String,
    #[allow(dead_code)]
    pub has_dynamic_components: bool,
    #[allow(dead_code)]
    pub dynamic_component_names: Vec<String>,
}

/// Parse all `<each>` blocks in page HTML and replace them with placeholders.
pub fn parse_each_blocks(
    html: &str,
    page_name: &str,
    component_registry: &HashMap<String, CompiledComponent>,
) -> Result<(String, Vec<EachSite>)> {
    let page_path = format!("src/pages/{}.html", page_name);
    let mut out = String::new();
    let mut cursor = 0usize;
    let mut index = 0usize;
    let mut sites = Vec::new();

    while let Some(span) = find_tag_span(html, "each", cursor) {
        out.push_str(&html[cursor..span.open_start]);

        let attrs = parse_attrs(&span.open_tag).with_context(|| {
            format!(
                "failed to parse <each> attributes in {}\n  → verify attribute quoting and the opening tag",
                page_path
            )
        })?;
        let of_attr = attrs.get("of").cloned().unwrap_or_default();
        let source = parse_loop_source(&of_attr).with_context(|| {
            format!(
                "invalid <each> source in {}\n  → use signal(), an array identifier, a positive integer, or a range",
                page_path
            )
        })?;

        let item_var = attrs.get("as").cloned().ok_or_else(|| {
            anyhow::anyhow!(
                "<each> block missing `as` attribute in {}\n  → add as=\"<variable>\" to name the loop variable\n  → example: <each of=\"posts()\" as=\"post\" key=\"id\">",
                page_path
            )
        })?;

        let key_expr = match attrs.get("key") {
            Some(v) => v.clone(),
            None if matches!(source, LoopSource::Range(_, _)) => item_var.clone(),
            None => {
                bail!(
                    "<each> block missing `key` attribute in {}\n  → add key=\"<property>\" where <property> uniquely identifies each item\n  → example: <each of=\"posts()\" as=\"post\" key=\"id\">",
                    page_path
                )
            }
        };

        let wrapper_attrs = filter_non_directive_attrs(&attrs, &["of", "as", "key", "data-each"]);
        let wrapper_attrs_html = serialize_attrs_to_html(&wrapper_attrs);

        let template_raw = html[span.open_end..span.close_start].to_string();
        ensure_components_registered(&template_raw, component_registry, &page_path)?;
        let template_html = expand_component_tags(&template_raw, component_registry).with_context(|| {
            format!(
                "failed to expand <each> template components in {}\n  → inspect component tags inside the loop body",
                page_path
            )
        })?;

        if let LoopSource::Plain(expr) = &source {
            let line = line_number_at(html, span.open_start);
            eprintln!(
                "warning: <each> block with plain array source — updates will not be reactive\n  in {} line {}\n  → use {}() instead of {} if {} is a signal",
                page_path, line, expr, expr, expr
            );
        }

        let id = site_id(page_name, "each", index);
        out.push_str(&format!(
            "<div data-each=\"{}\"{}></div>",
            id, wrapper_attrs_html
        ));

        let dynamic_component_names = collect_dynamic_names(&template_html);
        sites.push(EachSite {
            id,
            source,
            item_var,
            key_expr,
            has_dynamic_components: !dynamic_component_names.is_empty(),
            dynamic_component_names,
            template_html,
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

fn parse_attrs(open_tag: &str) -> Result<HashMap<String, String>> {
    let mut attrs = HashMap::new();
    let mut input = open_tag
        .trim()
        .trim_start_matches("<each")
        .trim_end_matches('>');
    if input.ends_with('/') {
        input = input[..input.len().saturating_sub(1)].trim_end();
    }
    for cap in attr_re().captures_iter(input) {
        let key = cap.get(1).map(|m| m.as_str()).unwrap_or_default();
        let val = cap
            .get(2)
            .or_else(|| cap.get(3))
            .or_else(|| cap.get(4))
            .map(|m| m.as_str())
            .unwrap_or_default();
        attrs.insert(key.to_string(), val.to_string());
    }
    Ok(attrs)
}

fn line_number_at(source: &str, byte_index: usize) -> usize {
    source[..byte_index].bytes().filter(|b| *b == b'\n').count() + 1
}

fn attr_re() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r#"([:@a-zA-Z0-9_-]+)\s*=\s*(?:\"([^\"]*)\"|'([^']*)'|([^\s>]+))"#)
            .expect("valid attr regex")
    })
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
            parse_each_blocks(html, "index", &registry(false)).expect("should parse each");
        assert!(out.contains("data-each=\"index_each_0\""));
        assert_eq!(sites.len(), 1);
    }

    #[test]
    fn parse_numeric_each_block() {
        let html = "<each of=\"10\" as=\"i\" key=\"i\"><span>{{i}}</span></each>";
        let (_, sites) =
            parse_each_blocks(html, "index", &registry(false)).expect("should parse numeric each");
        assert!(matches!(sites[0].source, LoopSource::Range(0, 10)));
    }

    #[test]
    fn parse_each_with_dynamic_component() {
        let html = "<each of=\"posts()\" as=\"post\" key=\"id\"><PostCard title=\"{{post.title}}\" /></each>";
        let (_, sites) = parse_each_blocks(html, "index", &registry(true))
            .expect("should parse dynamic component each");
        assert!(sites[0].has_dynamic_components);
    }

    #[test]
    fn parse_each_preserves_wrapper_attributes() {
        let html =
            "<each of=\"items\" as=\"item\" key=\"id\" class=\"cards\" id=\"loop\" data-role=\"feed\"><div></div></each>";
        let (out, _) = parse_each_blocks(html, "index", &registry(false)).expect("should parse");
        assert!(out.contains("data-each=\"index_each_0\""));
        assert!(out.contains("class=\"cards\""));
        assert!(out.contains("id=\"loop\""));
        assert!(out.contains("data-role=\"feed\""));
    }

    #[test]
    fn parse_each_strips_control_attributes() {
        let html = "<each of=\"items\" as=\"item\" key=\"id\" class=\"keep\"><div></div></each>";
        let (out, _) = parse_each_blocks(html, "index", &registry(false)).expect("should parse");
        assert!(out.contains("class=\"keep\""));
        assert!(!out.contains(" of=\"items\""));
        assert!(!out.contains(" as=\"item\""));
        assert!(!out.contains(" key=\"id\""));
    }

    #[test]
    fn parse_each_preserves_aria_and_empty_values() {
        let html =
            "<each of=\"items\" as=\"item\" key=\"id\" aria-live=\"polite\" data-empty=\"\"><div></div></each>";
        let (out, _) = parse_each_blocks(html, "index", &registry(false)).expect("should parse");
        assert!(out.contains("aria-live=\"polite\""));
        assert!(out.contains("data-empty=\"\""));
    }

    #[test]
    fn error_on_missing_key() {
        let html = "<each of=\"posts\" as=\"post\"><div></div></each>";
        let err = parse_each_blocks(html, "index", &registry(false))
            .expect_err("missing key should fail");
        assert!(err.to_string().contains("missing `key`"));
    }

    #[test]
    fn error_on_missing_as() {
        let html = "<each of=\"posts\" key=\"id\"><div></div></each>";
        let err =
            parse_each_blocks(html, "index", &registry(false)).expect_err("missing as should fail");
        assert!(err.to_string().contains("missing `as`"));
    }
}
