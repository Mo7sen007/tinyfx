//! # codegen
//!
//! Shared compile-time code generation utilities for TinyFX directives.
//!
//! ## Responsibilities
//! - Generate deterministic site IDs for directive locations
//! - Classify expressions as signal or plain values
//! - Parse `<each of="...">` sources into typed loop variants
//! - Provide a small TypeScript code buffer utility
//! - Rewrite `{{binding}}` placeholders for generated expressions
//!
//! ## What this module does NOT do
//! - It does not parse full HTML trees
//! - It does not perform directive-specific validation
//! - It does not emit full page glue by itself

use anyhow::{bail, Result};

/// Classifies an expression as signal-backed (`foo()`) or plain (`foo`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueKind {
    /// Expression ends in `()` and should be read inside `effect`.
    Signal(String),
    /// Expression should be read once without reactive subscription.
    Plain(String),
}

/// Parsed source kind for `<each of="...">` values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoopSource {
    /// Signal call source (`posts()`) as signal function name (`posts`).
    Signal(String),
    /// Plain expression source (`posts`).
    Plain(String),
    /// Numeric range source: `[start, end)`.
    Range(usize, usize),
}

/// Location where generated directive logic executes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DirectiveContext {
    /// Directive belongs to a page module and reads from that module alias.
    Page { module_alias: String, release: bool },
    /// Directive belongs to a component instance and reads from `inst`.
    Component { release: bool },
}

impl DirectiveContext {
    /// Whether current generation target is release mode.
    pub fn is_release(&self) -> bool {
        match self {
            Self::Page { release, .. } | Self::Component { release } => *release,
        }
    }
}

/// Generates a deterministic site identifier.
///
/// Takes the page name, directive name, and zero-based index and returns a
/// stable ID used by transformed HTML and generated function names.
pub fn site_id(page_name: &str, directive: &str, index: usize) -> String {
    format!("{}_{}_{}", page_name, directive, index)
}

/// Detects whether an expression is signal-like (`name()`) or plain.
///
/// Takes the raw expression text from template attributes and returns
/// [`ValueKind::Signal`] when it ends with `()`, otherwise
/// [`ValueKind::Plain`].
pub fn detect_value_kind(expr: &str) -> ValueKind {
    let trimmed = expr.trim();
    if trimmed.ends_with("()") {
        ValueKind::Signal(trimmed[..trimmed.len() - 2].trim().to_string())
    } else {
        ValueKind::Plain(trimmed.to_string())
    }
}

/// Parses `<each of="...">` into a typed [`LoopSource`].
///
/// Supports `signal()`, plain expressions, implicit ranges (`"10"` as
/// `0..10`), and explicit ranges (`"1..5"`).
///
/// # Errors
/// Returns an error when numeric values are zero/negative or when an explicit
/// range has `start >= end`.
pub fn parse_loop_source(of_attr: &str) -> Result<LoopSource> {
    let value = of_attr.trim();
    if let Some((left, right)) = value.split_once("..") {
        let start = parse_positive_int(left.trim())?;
        let end = parse_positive_int(right.trim())?;
        if start >= end {
            bail!(
                "numeric range in <each> has start >= end: \"{}\"\n  → use ascending ranges only, example: \"1..5\"",
                value
            );
        }
        return Ok(LoopSource::Range(start, end));
    }

    if value.chars().all(|c| c.is_ascii_digit()) {
        let end = parse_positive_int(value)?;
        return Ok(LoopSource::Range(0, end));
    }

    Ok(match detect_value_kind(value) {
        ValueKind::Signal(name) => LoopSource::Signal(name),
        ValueKind::Plain(expr) => LoopSource::Plain(expr),
    })
}

/// Qualify a template expression with its page module alias.
///
/// Page module exports are imported as `import * as PageModuleN from "..."`.
/// Any expression referencing a page export must be prefixed with that alias
/// so the generated TypeScript can resolve it at runtime.
///
/// Handles these cases:
/// - Signal call: "posts()" -> "PageModule0.posts()"
/// - Plain ident: "posts" -> "PageModule0.posts"
/// - Numeric lit: "10" -> "10" (no qualification)
/// - Range: "1..5" -> "1..5" (no qualification)
/// - Comparison: "user !== null" -> "PageModule0.user !== null"
///   (qualifies the first identifier, leaves operators intact)
pub fn qualify_expr(expr: &str, module_alias: &str) -> String {
    let trimmed = expr.trim();

    // When no page module exists for a route, keep expressions unchanged.
    if module_alias.trim().is_empty() {
        return trimmed.to_string();
    }

    // Numeric literal - no qualification.
    if trimmed.parse::<usize>().is_ok() {
        return trimmed.to_string();
    }

    // Range - no qualification.
    if trimmed.contains("..") && !trimmed.contains('(') {
        return trimmed.to_string();
    }

    // Signal call - "posts()" -> "PageModule0.posts()".
    if trimmed.ends_with("()") {
        let name = trimmed[..trimmed.len() - 2].trim();
        return format!("{}.{}()", module_alias, name);
    }

    // Plain identifier - "posts" -> "PageModule0.posts".
    if trimmed.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return format!("{}.{}", module_alias, trimmed);
    }

    // Complex expression - qualify the first identifier token only.
    let mut chars = trimmed.chars().peekable();
    let mut ident = String::new();
    while let Some(&c) = chars.peek() {
        if c.is_alphanumeric() || c == '_' {
            ident.push(c);
            chars.next();
        } else {
            break;
        }
    }
    let rest: String = chars.collect();
    if ident.is_empty() {
        trimmed.to_string()
    } else {
        format!("{}.{}{}", module_alias, ident, rest)
    }
}

fn parse_positive_int(input: &str) -> Result<usize> {
    let parsed: usize = input.parse().map_err(|_| {
        anyhow::anyhow!(
            "numeric range value '{}' is invalid in <each>\n  → use a positive integer like \"5\" or a range like \"1..5\"",
            input
        )
    })?;
    if parsed == 0 {
        bail!(
            "numeric range value must be > 0 in <each>: '{}'\n  → use a positive integer like \"5\"",
            input
        );
    }
    Ok(parsed)
}

/// Buffered TypeScript emitter with indentation support.
#[derive(Debug, Default)]
pub struct TsBuffer {
    lines: Vec<String>,
    indent: usize,
}

impl TsBuffer {
    /// Creates an empty TypeScript buffer.
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            indent: 0,
        }
    }

    /// Increases indentation by one level.
    pub fn indent(&mut self) {
        self.indent += 1;
    }

    /// Decreases indentation by one level.
    pub fn dedent(&mut self) {
        self.indent = self.indent.saturating_sub(1);
    }

    /// Appends a line with current indentation.
    pub fn line(&mut self, code: &str) {
        let pad = "  ".repeat(self.indent);
        self.lines.push(format!("{}{}", pad, code));
    }

    /// Appends a blank line.
    pub fn blank(&mut self) {
        self.lines.push(String::new());
    }

    /// Builds the final TypeScript source text.
    pub fn build(self) -> String {
        self.lines.join("\n")
    }
}

/// Rewrites binding placeholders against a loop item variable.
///
/// Replaces all `{{item.foo}}` occurrences in `html` with `item_var.foo` so
/// generated code can emit expression-based updates.
#[allow(dead_code)]
pub fn substitute_bindings(html: &str, item_var: &str) -> String {
    let needle = format!("{{{{{}.", item_var);
    let mut out = String::with_capacity(html.len());
    let mut i = 0usize;
    while let Some(found) = html[i..].find(&needle) {
        let start = i + found;
        out.push_str(&html[i..start]);
        let after = start + needle.len();
        if let Some(end_rel) = html[after..].find("}}") {
            let end = after + end_rel;
            out.push_str(item_var);
            out.push('.');
            out.push_str(html[after..end].trim());
            i = end + 2;
        } else {
            out.push_str(&html[start..]);
            return out;
        }
    }
    out.push_str(&html[i..]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_signal_expression() {
        assert_eq!(
            detect_value_kind("posts()"),
            ValueKind::Signal("posts".to_string())
        );
    }

    #[test]
    fn detects_plain_expression() {
        assert_eq!(
            detect_value_kind("posts"),
            ValueKind::Plain("posts".to_string())
        );
    }

    #[test]
    fn parses_numeric_range_implicit() {
        assert_eq!(
            parse_loop_source("10").expect("numeric implicit range should parse"),
            LoopSource::Range(0, 10)
        );
    }

    #[test]
    fn parses_numeric_range_explicit() {
        assert_eq!(
            parse_loop_source("1..5").expect("numeric explicit range should parse"),
            LoopSource::Range(1, 5)
        );
    }

    #[test]
    fn parses_signal_source() {
        assert_eq!(
            parse_loop_source("posts()").expect("signal source should parse"),
            LoopSource::Signal("posts".to_string())
        );
    }

    #[test]
    fn parses_plain_source() {
        assert_eq!(
            parse_loop_source("posts").expect("plain source should parse"),
            LoopSource::Plain("posts".to_string())
        );
    }

    #[test]
    fn site_id_is_deterministic() {
        assert_eq!(site_id("index", "each", 1), "index_each_1");
        assert_eq!(site_id("index", "each", 1), "index_each_1");
    }

    #[test]
    fn qualifies_signal_call() {
        assert_eq!(
            qualify_expr("posts()", "PageModule0"),
            "PageModule0.posts()"
        );
    }

    #[test]
    fn qualifies_plain_identifier() {
        assert_eq!(qualify_expr("posts", "PageModule0"), "PageModule0.posts");
    }

    #[test]
    fn does_not_qualify_numeric_literal() {
        assert_eq!(qualify_expr("10", "PageModule0"), "10");
    }

    #[test]
    fn does_not_qualify_range() {
        assert_eq!(qualify_expr("1..5", "PageModule0"), "1..5");
    }

    #[test]
    fn qualifies_complex_expression() {
        assert_eq!(
            qualify_expr("user !== null", "PageModule0"),
            "PageModule0.user !== null"
        );
    }

    #[test]
    fn qualifies_comparison() {
        assert_eq!(
            qualify_expr("count > 0", "PageModule0"),
            "PageModule0.count > 0"
        );
    }

    #[test]
    fn trims_whitespace() {
        assert_eq!(
            qualify_expr("  posts()  ", "PageModule0"),
            "PageModule0.posts()"
        );
    }
}
