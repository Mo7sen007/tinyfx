//! # pages
//!
//! Page scanning, compilation, and writing for TinyFX builds.
//!
//! ## Responsibilities
//! - Scan `src/pages/` for HTML page files
//! - Parse the root `<page>` element and its attributes
//! - Expand page component usage against the compiled registry
//! - Wrap pages in optional layout components
//! - Write resolved HTML files into `dist/`
//!
//! ## What this module does NOT do
//! - It does not compile component templates
//! - It does not build the dependency graph
//! - It does not generate runtime glue code

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use anyhow::{bail, Context, Result};
use regex::Regex;

use super::component::{expand_component_tags, CompiledComponent};
use super::dependency_graph::collect_component_references;

/// A resolved page ready to be written to the build output.
///
/// This is the Phase 2 output consumed later by glue generation.
#[derive(Debug, Clone)]
pub struct PageDef {
    pub name: String,
    pub route: String,
    pub html: String,
    pub source_path: PathBuf,
    pub out_path: PathBuf,
}

/// Resolve every page under `pages_dir` into a compiled HTML document.
///
/// This parses each `<page>` root, expands component usage against
/// `component_registry`, wraps layouts when requested, and computes the final
/// output path under `out_dir`.
///
/// # Errors
/// Returns an error when a page cannot be read, when the page root is invalid,
/// when component expansion fails, or when a requested layout cannot be
/// resolved through the compiled component registry.
// pub fn resolve_pages(
//     pages_dir: &Path,
//     out_dir: &Path,
//     component_registry: &HashMap<String, CompiledComponent>,
// ) -> Result<Vec<PageDef>> {
//     if !pages_dir.exists() {
//         return Ok(Vec::new());
//     }

//     let mut resolved_pages = Vec::new();
//     collect_pages(pages_dir, pages_dir, out_dir, component_registry, &mut resolved_pages)?;
//     Ok(resolved_pages)
// }

/// Collect all page file paths under `pages_dir` (recursively).
pub fn collect_page_paths(pages_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    if !pages_dir.exists() {
        return Ok(paths);
    }
    collect_page_paths_inner(pages_dir, &mut paths)?;
    Ok(paths)
}

fn collect_page_paths_inner(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir).with_context(|| {
        format!(
            "failed to read pages directory '{}'\n  → confirm the path exists and is readable",
            dir.display()
        )
    })? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_page_paths_inner(&path, out)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("html") {
            out.push(path);
        }
    }
    Ok(())
}

/// Compile a single page into a `PageDef`.
/// Accepts owned `PathBuf`s and an `Arc` registry so it can be called from threads.
pub fn compile_page(
    source_path: PathBuf,
    pages_root: PathBuf,
    out_dir: PathBuf,
    component_registry: std::sync::Arc<HashMap<String, CompiledComponent>>,
) -> Result<PageDef> {
    parse_and_expand_page(&source_path, &pages_root, &out_dir, component_registry.as_ref())
}

/// Write a resolved page to disk, creating parent directories as needed.
///
/// The page HTML is written exactly as stored in `page.html`.
///
/// # Errors
/// Returns an error when the output directory cannot be created or when the
/// destination file cannot be written.
pub fn write_page(page: &PageDef) -> Result<()> {
    if let Some(parent_dir) = page.out_path.parent() {
        fs::create_dir_all(parent_dir).with_context(|| {
            format!(
                "failed to create page output directory '{}'\n  → confirm the destination path is writable",
                parent_dir.display()
            )
        })?;
    }

    fs::write(&page.out_path, &page.html).with_context(|| {
        format!(
            "failed to write page '{}'\n  → confirm the destination path is writable",
            page.out_path.display()
        )
    })?;
    Ok(())
}

/// Resolve the final output path for a page under `dist/`.
///
/// `index.html` pages map to the directory root while nested page paths map to
/// `dist/<segments>/index.html`.
pub fn resolve_page_out_path(source_path: &Path, pages_root: &Path, out_dir: &Path) -> PathBuf {
    let relative_path = source_path.strip_prefix(pages_root).unwrap_or(source_path);
    let path_without_extension = relative_path.with_extension("");
    let parts: Vec<_> = path_without_extension
        .components()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect();

    if parts.is_empty() || (parts.len() == 1 && parts[0] == "index") {
        return out_dir.join("index.html");
    }

    let mut out_path = out_dir.to_path_buf();
    for segment in parts.into_iter().filter(|segment| segment != "index") {
        out_path = out_path.join(segment);
    }

    out_path.join("index.html")
}

// fn collect_pages(
//     dir: &Path,
//     pages_root: &Path,
//     out_dir: &Path,
//     component_registry: &HashMap<String, CompiledComponent>,
//     resolved_pages: &mut Vec<PageDef>,
// ) -> Result<()> {
//     for entry in fs::read_dir(dir).with_context(|| {
//         format!(
//             "failed to read pages directory '{}'\n  → confirm the path exists and is readable",
//             dir.display()
//         )
//     })? {
//         let entry = entry?;
//         let path = entry.path();
//         if path.is_dir() {
//             collect_pages(&path, pages_root, out_dir, component_registry, resolved_pages)?;
//         } else if path.extension().and_then(|ext| ext.to_str()) == Some("html") {
//             resolved_pages.push(parse_and_expand_page(
//                 &path,
//                 pages_root,
//                 out_dir,
//                 component_registry,
//             )?);
//         }
//     }

//     Ok(())
// }

fn parse_and_expand_page(
    source_path: &Path,
    pages_root: &Path,
    out_dir: &Path,
    component_registry: &HashMap<String, CompiledComponent>,
) -> Result<PageDef> {
    let source_html = fs::read_to_string(source_path).with_context(|| {
        format!(
            "failed to read page '{}'\n  → confirm the file exists and is readable",
            source_path.display()
        )
    })?;

    let (page_attrs, page_inner_html) = extract_page_element(&source_html, source_path)?;
    let route = page_attrs
        .get("route")
        .cloned()
        .filter(|route| !route.trim().is_empty())
        .unwrap_or_else(|| derive_route(source_path, pages_root));
    let layout = page_attrs
        .get("layout")
        .cloned()
        .filter(|layout| !layout.trim().is_empty());
    let body_without_script = strip_tinyfx_script(&page_inner_html, source_path);

    warn_missing_components(&body_without_script, source_path, component_registry);

    let expanded_body = expand_component_tags(&body_without_script, component_registry).with_context(|| {
        format!(
            "failed to expand page components in '{}'\n  → inspect the referenced component tags for invalid names",
            source_path.display()
        )
    })?;

    let page_body = if let Some(layout_name) = layout {
        wrap_in_layout(&expanded_body, &layout_name, component_registry)?
    } else {
        expanded_body
    };

    let page_name = source_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("page")
        .to_string();

    Ok(PageDef {
        name: page_name,
        route,
        html: build_html_shell(&page_body),
        source_path: source_path.to_path_buf(),
        out_path: resolve_page_out_path(source_path, pages_root, out_dir),
    })
}

fn extract_page_element(source_html: &str, source_path: &Path) -> Result<(HashMap<String, String>, String)> {
    let captures = page_open_re().captures(source_html).with_context(|| {
        format!(
            "page '{}' is missing a root <page> element\n  → wrap the file contents in <page>...</page>",
            source_path.display()
        )
    })?;
    let attrs_str = captures.get(1).map(|m| m.as_str()).unwrap_or_default();
    let open_end = captures.get(0).map(|m| m.end()).unwrap_or(0);
    let close_start = source_html.rfind("</page>").with_context(|| {
        format!(
            "page '{}' is missing a closing </page> tag\n  → close the root page element",
            source_path.display()
        )
    })?;

    if close_start < open_end {
        bail!(
            "page '{}' has a malformed <page> element\n  → check the root tag ordering and close the page element correctly",
            source_path.display()
        );
    }

    Ok((parse_attrs(attrs_str), source_html[open_end..close_start].to_string()))
}

fn parse_attrs(attrs: &str) -> HashMap<String, String> {
    let mut parsed = HashMap::new();
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
            parsed.insert(key, "true".to_string());
            continue;
        }
        index += 1;

        while index < chars.len() && chars[index].is_whitespace() {
            index += 1;
        }

        if index >= chars.len() {
            parsed.insert(key, String::new());
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

        parsed.insert(key, value);
    }

    parsed
}

fn derive_route(path: &Path, pages_root: &Path) -> String {
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

fn strip_tinyfx_script(inner_html: &str, source_path: &Path) -> String {
    if let Some(captures) = tinyfx_script_re().captures(inner_html) {
        let script_body = captures.get(1).map(|m| m.as_str()).unwrap_or_default().trim();
        if tinyfx_script_function_re().is_match(script_body) {
            eprintln!(
                "warning: page '{}' defines functions inside <script type=\"tinyfx\">\n  → move reusable functions into src/lib/ and keep the page script to calls only",
                source_path.display()
            );
        }
    }

    tinyfx_script_re().replace(inner_html, "").to_string()
}

fn wrap_in_layout(
    body: &str,
    layout_name: &str,
    component_registry: &HashMap<String, CompiledComponent>,
) -> Result<String> {
    let wrapped_html = format!("<{}>{}</{}>", layout_name, body, layout_name);
    expand_component_tags(&wrapped_html, component_registry).with_context(|| {
        format!(
            "failed to expand layout '{}'\n  → confirm the layout component exists in src/components/{}",
            layout_name,
            layout_name
        )
    })
}

fn warn_missing_components(
    html: &str,
    source_path: &Path,
    component_registry: &HashMap<String, CompiledComponent>,
) {
    for component_name in collect_component_references(html) {
        if !component_registry.contains_key(&component_name) {
            eprintln!(
                "warning: page '{}' references missing component '{}'\n  → create src/components/{name}/{name}.html or fix the tag name",
                source_path.display(),
                component_name,
                name = component_name,
            );
        }
    }
}

fn build_html_shell(body: &str) -> String {
    if body.trim_start().starts_with("<!") || body.contains("<html") {
        return body.to_string();
    }

    format!(
        "<!DOCTYPE html>\n<html lang=\"en\">\n<head><meta charset=\"UTF-8\">\
         <meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">\
         <link rel=\"stylesheet\" href=\"/css/main.css\">\n</head>\n\
         <body>\n{}\n<script type=\"module\" src=\"/js/main.js\"></script>\n</body>\n</html>",
        body
    )
}

fn page_open_re() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r#"(?s)<page([^>]*)>"#).unwrap())
}

fn tinyfx_script_re() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r#"(?si)<script\s+type=["']tinyfx["'][^>]*>(.*?)</script>"#).unwrap()
    })
}

fn tinyfx_script_function_re() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(
            r#"(?m)^\s*(async\s+)?function\s+\w+|^\s*(const|let|var)\s+\w+\s*=\s*(async\s+)?\("#,
        )
        .unwrap()
    })
}
