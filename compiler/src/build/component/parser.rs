// ─────────────────────────────────────────────────────────────────────────────
// parser.rs — HTML-level component tag detection and attribute extraction
// ─────────────────────────────────────────────────────────────────────────────
//
// This module is responsible for *finding* `<component>…</component>` tags in
// raw HTML and extracting their attributes. It knows nothing about templates,
// file I/O, or recursive expansion — that's the renderer's job.
//
// Two main capabilities:
//   1. **Span detection** (`find_component_span`)
//      Locates the byte offsets of the first outermost `<component>` tag,
//      correctly handling nested components (depth tracking).
//
//   2. **Attribute parsing** (`parse_attrs`)
//      Turns `<component name="X" variant="primary">` into a HashMap.
//
// Separation rationale:
//   • data.rs    — "what a component IS"
//   • parser.rs  — "how to find one in HTML"   ← you are here
//   • renderer.rs — "how to expand them all"
// ─────────────────────────────────────────────────────────────────────────────

use std::collections::HashMap;

use anyhow::{Result, bail};

// ─── Span detection ─────────────────────────────────────────────────────────

/// Byte-offset boundaries for a single `<component …>…</component>` occurrence.
///
/// ```text
/// <component name="X">inner HTML</component>
/// ^                  ^^          ^^           ^
/// open_start    open_end  close_start   close_end
/// ```
///
/// The caller can use these offsets to slice out the tag, its slot content,
/// and the surrounding HTML for replacement.
#[derive(Debug, Clone, Copy)]
pub struct ComponentSpan {
    /// Byte offset where `<component` starts.
    pub open_start: usize,
    /// Byte offset just *after* the closing `>` of the opening tag.
    pub open_end: usize,
    /// Byte offset where `</component>` starts.
    pub close_start: usize,
    /// Byte offset just *after* `</component>`.
    pub close_end: usize,
}

/// Find the first (outermost) `<component>…</component>` in the given HTML.
///
/// Returns `None` if no `<component` tag exists.
///
/// Handles nesting correctly by tracking depth: each nested `<component` tag
/// increments the depth counter, and each `</component>` decrements it. The
/// function only returns when depth drops back to zero (the outermost pair).
///
/// # Why outermost-first?
/// The renderer processes one tag per iteration, replacing it with its rendered
/// template. Inner components will be found in subsequent iterations, which
/// naturally supports deep/recursive nesting up to `MAX_ITERATIONS`.
pub fn find_component_span(html: &str) -> Option<ComponentSpan> {
    // Locate the opening `<component` tag.
    let open_start = html.find("<component")?;

    // Find the closing `>` of the opening tag.
    let open_end = open_start + html[open_start..].find('>')? + 1;

    // Walk forward, tracking nesting depth to find the matching `</component>`.
    let mut depth = 1usize;
    let mut pos = open_end;

    loop {
        // Search for the next opening and closing tags simultaneously.
        let next_open = html[pos..].find("<component");
        let next_close = html[pos..].find("</component>");

        // No closing tag found — the HTML is malformed; give up.
        let close_rel = next_close?;

        // If there's a nested opening tag that appears *before* the next close,
        // we need to increase depth and skip past it.
        if let Some(open_rel) = next_open {
            if open_rel < close_rel {
                depth += 1;
                pos += open_rel + "<component".len();
                continue;
            }
        }

        // This `</component>` closes one level of nesting.
        depth = depth.saturating_sub(1);
        let close_start = pos + close_rel;
        let close_end = close_start + "</component>".len();

        if depth == 0 {
            // We've found the matching close for the outermost open.
            return Some(ComponentSpan {
                open_start,
                open_end,
                close_start,
                close_end,
            });
        }

        // Not at depth 0 yet — keep looking past this inner close tag.
        pos = close_end;
    }
}

// ─── Attribute parsing ──────────────────────────────────────────────────────

/// Parse HTML attributes from an opening `<component …>` tag into a HashMap.
///
/// Supports:
///   - Quoted values:   `name="Button"`  or `name='Button'`
///   - Bare values:     `count=3`
///   - Boolean attrs:   `disabled`  → stored as `"true"`
///
/// # Errors
/// Returns an error if `open_tag` does not start with `<component`.
///
/// # Example
/// ```ignore
/// let attrs = parse_attrs(r#"<component name="Card" variant="primary">"#)?;
/// assert_eq!(attrs["name"], "Card");
/// assert_eq!(attrs["variant"], "primary");
/// ```
pub fn parse_attrs(open_tag: &str) -> Result<HashMap<String, String>> {
    let mut attrs = HashMap::new();

    let mut s = open_tag.trim();

    // Sanity check: we should only ever be called with a <component ...> tag.
    if !s.starts_with("<component") {
        bail!("Expected <component ...>, got: {}", open_tag);
    }

    // Strip the tag name and the trailing `>` to leave just the attributes.
    s = s.trim_start_matches("<component");
    s = s.trim_end_matches('>');

    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // ── Skip leading whitespace ──────────────────────────────
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }
        if i >= chars.len() {
            break;
        }

        // ── Read attribute key ───────────────────────────────────
        let key_start = i;
        while i < chars.len() && !chars[i].is_whitespace() && chars[i] != '=' && chars[i] != '>' {
            i += 1;
        }
        let key: String = chars[key_start..i].iter().collect();
        if key.is_empty() {
            break;
        }

        // ── Skip whitespace between key and potential `=` ────────
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }

        // ── No `=` means a boolean attribute (e.g. `disabled`) ──
        if i >= chars.len() || chars[i] != '=' {
            attrs.insert(key, "true".to_string());
            continue;
        }
        i += 1; // consume `=`

        // ── Skip whitespace between `=` and value ────────────────
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }
        if i >= chars.len() {
            // `=` at end of tag with no value — store as empty string.
            attrs.insert(key, String::new());
            break;
        }

        // ── Read value: either quoted or bare ────────────────────
        let value = if chars[i] == '"' || chars[i] == '\'' {
            // Quoted value — read until matching close quote.
            let quote = chars[i];
            i += 1; // skip opening quote
            let v_start = i;
            while i < chars.len() && chars[i] != quote {
                i += 1;
            }
            let v: String = chars[v_start..i].iter().collect();
            if i < chars.len() {
                i += 1; // skip closing quote
            }
            v
        } else {
            // Bare (unquoted) value — read until whitespace or `>`.
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

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Basic attribute parsing with two quoted values.
    #[test]
    fn parses_standard_attrs() {
        let attrs = parse_attrs(r#"<component name="Button" variant="primary">"#).unwrap();
        assert_eq!(attrs.get("name").unwrap(), "Button");
        assert_eq!(attrs.get("variant").unwrap(), "primary");
    }

    /// Locating a simple (non-nested) component span.
    #[test]
    fn finds_simple_span() {
        let html = r#"<p>before</p><component name="X">hello</component><p>after</p>"#;
        let span = find_component_span(html).unwrap();

        assert_eq!(
            &html[span.open_start..span.open_end],
            r#"<component name="X">"#,
            "Opening tag should be captured exactly",
        );
        assert_eq!(
            &html[span.open_end..span.close_start],
            "hello",
            "Slot content should be the text between open and close tags",
        );
    }

    /// Nested component tags: the outermost pair should be returned.
    #[test]
    fn finds_outermost_nested_span() {
        let html = r#"<component name="A"><component name="B">inner</component></component>"#;
        let span = find_component_span(html).unwrap();

        assert_eq!(
            &html[span.open_end..span.close_start],
            r#"<component name="B">inner</component>"#,
            "Slot content of the outer component should include the inner component",
        );
    }
}
