//! # conditional
//!
//! Compile-time parsing and code generation for TinyFX `<if>` directives.
//!
//! ## Responsibilities
//! - Parse `<if>`, `<else-if>`, and `<else>` branch chains
//! - Expand components inside each branch at compile time
//! - Replace `<if>` blocks with hidden branch wrappers
//! - Emit specialized TypeScript togglers per directive site
//!
//! ## What this module does NOT do
//! - It does not evaluate conditions at compile time
//! - It does not implement a runtime template parser
//! - It does not write output files directly

use std::collections::HashMap;
use std::sync::OnceLock;

use anyhow::{Context, Result, bail};
use regex::Regex;

use super::codegen::{detect_value_kind, site_id};
use super::component::{CompiledComponent, expand_component_tags};
use super::each::find_tag_span;
use super::pages_util::{filter_non_directive_attrs, serialize_attrs_to_html};

/// One branch in a parsed `<if>` site.
#[derive(Debug, Clone)]
pub struct IfBranch {
    pub condition: Option<String>,
    pub is_signal: bool,
    pub html: String,
    pub dynamic_component_names: Vec<String>,
}

/// Compile-time descriptor for one `<if>` block.
#[derive(Debug, Clone)]
pub struct IfSite {
    pub id: String,
    pub branches: Vec<IfBranch>,
}

/// Parse all `<if>` blocks from one page and transform HTML.
///
/// Returns transformed HTML where each `<if>` site becomes a wrapper with hidden
/// branch children, plus metadata used for TypeScript generation.
///
/// # Errors
/// Returns an error when the root `<if>` is missing `condition`, when an
/// `<else-if>` appears after `<else>`, or when malformed branch structure is
/// encountered.
pub fn parse_if_blocks(
    html: &str,
    page_name: &str,
    component_registry: &HashMap<String, CompiledComponent>,
) -> Result<(String, Vec<IfSite>)> {
    let page_path = format!("src/pages/{}.html", page_name);
    let mut out = String::new();
    let mut cursor = 0usize;
    let mut index = 0usize;
    let mut sites = Vec::new();

    while let Some(span) = find_tag_span(html, "if", cursor) {
        out.push_str(&html[cursor..span.open_start]);

        let attrs = parse_attrs(&span.open_tag)?;
        let first_condition = attrs.get("condition").cloned().ok_or_else(|| {
            anyhow::anyhow!(
                "<if> block missing `condition` attribute in {}\n  → add condition=\"<expression>\" to the <if> element",
                page_path
            )
        })?;
        let wrapper_attrs = filter_non_directive_attrs(&attrs, &["condition", "data-if"]);
        let wrapper_attrs_html = serialize_attrs_to_html(&wrapper_attrs);

        let inner = html[span.open_end..span.close_start].to_string();
        let mut branches = parse_branch_chain(&inner, &first_condition, &page_path)?;
        for branch in &mut branches {
            branch.html = expand_component_tags(&branch.html, component_registry).with_context(|| {
                format!(
                    "failed to expand components in <if> branch in {}\n  → inspect component tags in conditional branches",
                    page_path
                )
            })?;
            branch.dynamic_component_names = collect_dynamic_names(&branch.html);
        }

        let id = site_id(page_name, "if", index);
        out.push_str(&render_if_wrapper(&id, &branches, &wrapper_attrs_html));
        sites.push(IfSite { id, branches });

        index += 1;
        cursor = span.close_end;
    }

    out.push_str(&html[cursor..]);
    Ok((out, sites))
}

fn render_if_wrapper(id: &str, branches: &[IfBranch], wrapper_attrs_html: &str) -> String {
    let mut out = String::new();
    out.push_str(&format!("<div data-if=\"{}\"{}>", id, wrapper_attrs_html));
    for (i, branch) in branches.iter().enumerate() {
        out.push_str(&format!(
            "<div data-branch=\"{}\" hidden>{}</div>",
            i, branch.html
        ));
    }
    out.push_str("</div>");
    out
}

fn parse_branch_chain(
    inner: &str,
    first_condition: &str,
    page_path: &str,
) -> Result<Vec<IfBranch>> {
    let mut branches = Vec::new();
    let mut cursor = 0usize;
    let mut current_cond = Some(first_condition.to_string());
    let mut seen_else = false;

    loop {
        let next = next_branch_marker(inner, cursor);
        let end = next
            .as_ref()
            .map(|(start, _, _)| *start)
            .unwrap_or(inner.len());
        let html = inner[cursor..end].to_string();
        let (condition, is_signal) = match &current_cond {
            Some(expr) => (
                Some(expr.clone()),
                matches!(
                    detect_value_kind(expr),
                    super::codegen::ValueKind::Signal(_)
                ),
            ),
            None => (None, false),
        };
        branches.push(IfBranch {
            condition,
            is_signal,
            html,
            dynamic_component_names: Vec::new(),
        });

        let Some((start, open_end, marker)) = next else {
            break;
        };

        cursor = open_end;
        match marker.as_str() {
            "else" => {
                seen_else = true;
                current_cond = None;
            }
            "else-if" => {
                if seen_else {
                    bail!(
                        "<else-if> appears after <else> in {}\n  → <else> must be the final branch in an <if> block",
                        page_path
                    );
                }
                let open = &inner[start..open_end];
                let attrs = parse_attrs(open)?;
                let cond = attrs.get("condition").cloned().ok_or_else(|| {
                    anyhow::anyhow!(
                        "<if> branch missing `condition` in {}\n  → add condition=\"<expression>\" to <else-if>",
                        page_path
                    )
                })?;
                current_cond = Some(cond);
            }
            _ => break,
        }
    }

    Ok(branches)
}

fn next_branch_marker(input: &str, from: usize) -> Option<(usize, usize, String)> {
    let else_if = input[from..].find("<else-if").map(|v| from + v);
    let else_tag = input[from..].find("<else>").map(|v| from + v);
    let (start, marker) = match (else_if, else_tag) {
        (Some(a), Some(b)) if a < b => (a, "else-if".to_string()),
        (Some(a), Some(b)) if b < a => (b, "else".to_string()),
        (Some(a), Some(_)) => (a, "else-if".to_string()),
        (Some(a), None) => (a, "else-if".to_string()),
        (None, Some(b)) => (b, "else".to_string()),
        (None, None) => return None,
    };
    let open_end = start + input[start..].find('>')? + 1;
    Some((start, open_end, marker))
}

fn parse_attrs(open_tag: &str) -> Result<HashMap<String, String>> {
    let mut attrs = HashMap::new();
    for cap in attr_re().captures_iter(open_tag) {
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

fn collect_dynamic_names(html: &str) -> Vec<String> {
    let mut out = Vec::new();
    for cap in data_component_re().captures_iter(html) {
        let name = cap
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

    fn registry() -> HashMap<String, CompiledComponent> {
        HashMap::new()
    }

    #[test]
    fn parse_simple_if_block() {
        let html = "<if condition=\"isLoggedIn()\"><h1>ok</h1></if>";
        let (out, sites) = parse_if_blocks(html, "index", &registry()).expect("if should parse");
        assert!(out.contains("data-if=\"index_if_0\""));
        assert_eq!(sites.len(), 1);
        assert_eq!(sites[0].branches.len(), 1);
    }

    #[test]
    fn parse_if_else_block() {
        let html = "<if condition=\"a\"><h1>a</h1><else><h1>b</h1></if>";
        let (_, sites) = parse_if_blocks(html, "index", &registry()).expect("if else should parse");
        assert_eq!(sites[0].branches.len(), 2);
        assert!(sites[0].branches[1].condition.is_none());
    }

    #[test]
    fn parse_if_elseif_else_chain() {
        let html = "<if condition=\"a\"><h1>a</h1><else-if condition=\"b\"><h1>b</h1><else><h1>c</h1></if>";
        let (_, sites) =
            parse_if_blocks(html, "index", &registry()).expect("if chain should parse");
        assert_eq!(sites[0].branches.len(), 3);
    }

    #[test]
    fn error_on_missing_condition() {
        let html = "<if><h1>a</h1></if>";
        let err =
            parse_if_blocks(html, "index", &registry()).expect_err("missing condition should fail");
        assert!(err.to_string().contains("missing `condition`"));
    }

    #[test]
    fn error_on_else_if_after_else() {
        let html = "<if condition=\"a\"><h1>a</h1><else><h1>b</h1><else-if condition=\"c\"><h1>c</h1></if>";
        let err = parse_if_blocks(html, "index", &registry())
            .expect_err("else-if after else should fail");
        assert!(err.to_string().contains("appears after <else>"));
    }

    #[test]
    fn all_branches_hidden_in_html_output() {
        let html = "<if condition=\"a\"><h1>a</h1><else><h1>b</h1></if>";
        let (out, _) =
            parse_if_blocks(html, "index", &registry()).expect("if parse should succeed");
        assert!(out.contains("data-branch=\"0\" hidden"));
        assert!(out.contains("data-branch=\"1\" hidden"));
    }

    #[test]
    fn parse_if_preserves_wrapper_attributes() {
        let html =
            "<if condition=\"ready\" class=\"auth\" id=\"gate\" aria-live=\"polite\"><h1>a</h1></if>";
        let (out, _) = parse_if_blocks(html, "index", &registry()).expect("if parse should succeed");
        assert!(out.contains("data-if=\"index_if_0\""));
        assert!(out.contains("class=\"auth\""));
        assert!(out.contains("id=\"gate\""));
        assert!(out.contains("aria-live=\"polite\""));
    }

    #[test]
    fn parse_if_strips_control_condition_attribute() {
        let html = "<if condition=\"ready\" class=\"auth\"><h1>a</h1></if>";
        let (out, _) = parse_if_blocks(html, "index", &registry()).expect("if parse should succeed");
        assert!(out.contains("class=\"auth\""));
        assert!(!out.contains("condition=\"ready\""));
    }
}
