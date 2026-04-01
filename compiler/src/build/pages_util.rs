//! # pages_util
//!
//! Focused helpers used by page compilation.
//!
//! ## Responsibilities
//! - Parse lightweight HTML-style attribute strings
//! - Derive route paths from page source paths
//! - Apply final placeholder interpolation pass
//!
//! ## What this module does NOT do
//! - It does not read files from disk
//! - It does not expand components
//! - It does not write compiled pages

use std::collections::HashMap;
use std::path::Path;
use std::sync::OnceLock;

use regex::Regex;

/// Keep only attributes that are not directive control attributes.
pub fn filter_non_directive_attrs(
    attrs: &HashMap<String, String>,
    directive_attrs: &[&str],
) -> HashMap<String, String> {
    attrs
        .iter()
        .filter(|(key, _)| !directive_attrs.contains(&key.as_str()))
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect()
}

/// Serialize attributes into deterministic HTML syntax.
pub fn serialize_attrs_to_html(attrs: &HashMap<String, String>) -> String {
    if attrs.is_empty() {
        return String::new();
    }

    let mut keys: Vec<&String> = attrs.keys().collect();
    keys.sort();

    let mut pairs = Vec::with_capacity(keys.len());
    for key in keys {
        let value = attrs
            .get(key)
            .map(|v| escape_attr_value(v))
            .unwrap_or_default();
        pairs.push(format!("{}=\"{}\"", key, value));
    }

    format!(" {}", pairs.join(" "))
}

/// Parse `key="value"` style attributes from a tag payload.
pub fn parse_attrs(attrs: &str) -> HashMap<String, String> {
    let mut parsed = HashMap::new();
    for (key, value) in parse_attrs_ordered(attrs) {
        parsed.insert(key, value);
    }
    parsed
}

/// Parse attributes while preserving source order.
pub fn parse_attrs_ordered(attrs: &str) -> Vec<(String, String)> {
    let mut parsed = Vec::new();
    let chars: Vec<char> = attrs.chars().collect();
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
            parsed.push((key, "true".to_string()));
            continue;
        }
        index += 1;

        while index < chars.len() && chars[index].is_whitespace() {
            index += 1;
        }

        if index >= chars.len() {
            parsed.push((key, String::new()));
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

        parsed.push((key, value));
    }

    parsed
}

/// Derive a route path from a page source path.
pub fn derive_route(path: &Path, pages_root: &Path) -> String {
    let relative_path = path.strip_prefix(pages_root).unwrap_or(path);
    let path_without_extension = relative_path.with_extension("");
    let parts: Vec<_> = path_without_extension
        .components()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect();

    if parts.is_empty() || (parts.len() == 1 && parts[0] == "index") {
        return "/".to_string();
    }

    format!(
        "/{}",
        parts
            .into_iter()
            .filter(|segment| segment != "index")
            .collect::<Vec<_>>()
            .join("/")
    )
}

/// Final placeholder interpolation pass for unresolved moustache tokens.
pub fn interpolate_placeholders(html: &str) -> String {
    moustache_re().replace_all(html, "").to_string()
}

/// Asset path rewrite hook.
pub fn rewrite_asset_paths(html: &str) -> String {
    html.to_string()
}

fn moustache_re() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r#"\{\{\s*[^}]+\s*\}\}"#).unwrap())
}

fn escape_attr_value(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filters_out_directive_attributes() {
        let mut attrs = HashMap::new();
        attrs.insert("of".to_string(), "items".to_string());
        attrs.insert("as".to_string(), "item".to_string());
        attrs.insert("key".to_string(), "id".to_string());
        attrs.insert("class".to_string(), "cards".to_string());

        let filtered = filter_non_directive_attrs(&attrs, &["of", "as", "key"]);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered.get("class").map(String::as_str), Some("cards"));
    }

    #[test]
    fn serializes_attributes_deterministically() {
        let mut attrs = HashMap::new();
        attrs.insert("id".to_string(), "hero".to_string());
        attrs.insert("class".to_string(), "card".to_string());

        let html = serialize_attrs_to_html(&attrs);
        assert_eq!(html, " class=\"card\" id=\"hero\"");
    }

    #[test]
    fn escapes_attribute_values() {
        let mut attrs = HashMap::new();
        attrs.insert("data-msg".to_string(), "a & \"b\" <c>".to_string());

        let html = serialize_attrs_to_html(&attrs);
        assert_eq!(html, " data-msg=\"a &amp; &quot;b&quot; &lt;c&gt;\"");
    }
}
