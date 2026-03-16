// ─────────────────────────────────────────────────────────────────────────────
// pages.rs — File-based hard router: page discovery, parsing, and expansion
// ─────────────────────────────────────────────────────────────────────────────
//
// Responsibilities:
//   1. Scan `src/pages/` recursively for `.html` files
//   2. Parse the `<page>` root element → extract `route` / `layout` attributes
//   3. Extract the optional `<script type="tinyfx">` block (and warn on fn defs)
//   4. Extract the page body (everything inside `<page>` except the script)
//   5. Expand `<component>` tags in the body using the existing renderer
//   6. Optionally wrap the body in a layout component
//   7. Write dist/{name}/index.html   (or dist/index.html for route="/")
//   8. Return Vec<PageDef> for the glue generator (route map)
// ─────────────────────────────────────────────────────────────────────────────

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use regex::Regex;

use super::{CompiledComponent, expand_component_tags};

// ─── Data ────────────────────────────────────────────────────────────────────

/// Represents a single resolved page ready to be written to dist/.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PageDef {
    /// Filename stem, e.g. "blog"
    pub name: String,
    /// Declared URL pattern, e.g. "/blog/:slug"
    pub route: String,
    /// Optional layout component name, e.g. "MainLayout"
    pub layout: Option<String>,
    /// Optional <script type="tinyfx"> body (cleaned, no tag)
    pub script: Option<String>,
    /// Fully-expanded page body HTML
    pub body: String,
    /// Source .html file path
    pub source_path: PathBuf,
    /// Destination path under dist/, e.g. dist/blog/index.html
    pub out_path: PathBuf,
}

// ─── Public API ──────────────────────────────────────────────────────────────

/// Scan `pages_dir`, parse and expand every `.html` file found, then write
/// the resulting HTML to `out_dir`.  Returns a `Vec<PageDef>` describing
/// every route, which the glue generator uses to emit the route map.
pub fn resolve_pages(
    pages_dir: &Path,
    out_dir: &Path,
    registry: &HashMap<String, CompiledComponent>,
) -> Result<Vec<PageDef>> {
    if !pages_dir.exists() {
        // No pages directory — silently return empty; the router simply won't
        // emit any routes.
        return Ok(vec![]);
    }

    let mut pages = Vec::new();
    collect_pages(
        pages_dir,
        pages_dir,
        out_dir,
        registry,
        &mut pages,
    )?;
    Ok(pages)
}

// ─── Internal helpers ─────────────────────────────────────────────────────────

/// Recurse through `dir`, processing every `.html` file.
fn collect_pages(
    dir: &Path,
    pages_root: &Path,
    out_dir: &Path,
    registry: &HashMap<String, CompiledComponent>,
    pages: &mut Vec<PageDef>,
) -> Result<()> {
    for entry in
        fs::read_dir(dir).with_context(|| format!("Failed to read pages dir {}", dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_pages(
                &path,
                pages_root,
                out_dir,
                registry,
                pages,
            )?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("html") {
            let page = parse_and_expand_page(
                &path,
                pages_root,
                out_dir,
                registry,
            )?;
            pages.push(page);
        }
    }
    Ok(())
}

/// Parse a single page file and perform component expansion.
fn parse_and_expand_page(
    source_path: &Path,
    pages_root: &Path,
    out_dir: &Path,
    registry: &HashMap<String, CompiledComponent>,
) -> Result<PageDef> {
    let src = fs::read_to_string(source_path)
        .with_context(|| format!("Failed to read page {}", source_path.display()))?;

    // ── 1. Extract the <page …> opening tag ─────────────────────────
    let (attrs_str, inner) = extract_page_element(&src, source_path)?;

    // ── 2. Parse attributes ──────────────────────────────────────────
    let route = parse_attr(&attrs_str, "route").unwrap_or_else(|| {
        // Fall back to deriving route from the file path relative to pages root
        derive_route(source_path, pages_root)
    });
    let layout = parse_attr(&attrs_str, "layout").filter(|value| !value.trim().is_empty());

    // ── 3. Extract <script type="tinyfx"> ────────────────────────────
    let (script, body_without_script) = extract_tinyfx_script(&inner, source_path);

    warn_missing_components(&body_without_script, source_path, registry);

    // ── 4. Expand <component> tags in body ───────────────────────────
    let expanded_body = expand_component_tags(&body_without_script, registry)
        .with_context(|| format!("Component expansion failed for {}", source_path.display()))?;

    // ── 5. Optionally wrap in layout ─────────────────────────────────
    let final_body = if let Some(ref layout_name) = layout {
        wrap_in_layout(&expanded_body, layout_name, registry)?
    } else {
        expanded_body
    };

    // ── 6. Derive name and output path ────────────────────────────────
    let name = source_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("page")
        .to_string();

    let out_path = derive_out_path(source_path, pages_root, out_dir);

    Ok(PageDef {
        name,
        route,
        layout,
        script,
        body: final_body,
        source_path: source_path.to_path_buf(),
        out_path,
    })
}

/// Find the `<page …>` element and return (attributes_string, inner_html).
fn extract_page_element(src: &str, path: &Path) -> Result<(String, String)> {
    // Match the opening <page …> tag (may span attributes across whitespace)
    let open_re = Regex::new(r"(?s)<page([^>]*)>").unwrap();
    let close = "</page>";

    let cap = open_re
        .captures(src)
        .with_context(|| format!("{}: missing root <page> element", path.display()))?;

    let attrs_str = cap.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
    let open_end = cap.get(0).unwrap().end();

    let close_start = src.rfind(close).with_context(|| {
        format!("{}: <page> element is not closed with </page>", path.display())
    })?;

    if close_start < open_end {
        bail!("{}: malformed <page> element", path.display());
    }

    let inner = src[open_end..close_start].to_string();
    Ok((attrs_str, inner))
}

/// Parse a single HTML attribute value from an attribute string.
fn parse_attr(attrs: &str, name: &str) -> Option<String> {
    // Matches both name="value" and name='value'
    let pattern = format!(r#"(?i){}=["']([^"']*)["']"#, regex::escape(name));
    let re = Regex::new(&pattern).ok()?;
    re.captures(attrs)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
}

/// Derive a route from the file path relative to `pages_root`.
/// `pages/index.html` → `/`
/// `pages/blog.html`  → `/blog`
/// `pages/blog/post.html` → `/blog/post`
fn derive_route(path: &Path, pages_root: &Path) -> String {
    let rel = path.strip_prefix(pages_root).unwrap_or(path);
    let without_ext = rel.with_extension("");
    let parts: Vec<_> = without_ext
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();

    if parts.is_empty() || (parts.len() == 1 && parts[0] == "index") {
        return "/".to_string();
    }

    let route = parts
        .iter()
        .filter(|p| *p != "index")
        .cloned()
        .collect::<Vec<_>>()
        .join("/");

    format!("/{}", route)
}

/// Extract `<script type="tinyfx">` content from the inner HTML.
/// Returns (Option<script_body>, html_without_script).
/// Emits a warning if a function definition is detected inside the script block.
fn extract_tinyfx_script(inner: &str, path: &Path) -> (Option<String>, String) {
    let script_re =
        Regex::new(r#"(?si)<script\s+type=["']tinyfx["'][^>]*>(.*?)</script>"#).unwrap();

    if let Some(cap) = script_re.captures(inner) {
        let script_body = cap.get(1).unwrap().as_str().trim().to_string();

        // Warn if a function definition is found (rule 4)
        let fn_def_re = Regex::new(
            r"(?m)^\s*(async\s+)?function\s+\w+|^\s*(const|let|var)\s+\w+\s*=\s*(async\s+)?\("
        ).unwrap();
        if fn_def_re.is_match(&script_body) {
            eprintln!(
                "  warning: {}: <script type=\"tinyfx\"> should only call functions, not define them. Move function definitions to src/lib/.",
                path.display()
            );
        }

        let html_without = script_re.replace(inner, "").to_string();
        (Some(script_body), html_without)
    } else {
        (None, inner.to_string())
    }
}

/// Wrap a page body inside a layout component by constructing a synthetic
/// `<component>` tag and running the expansion pipeline on it.
fn wrap_in_layout(
    body: &str,
    layout_name: &str,
    registry: &HashMap<String, CompiledComponent>,
) -> Result<String> {
    let wrapped = format!(
        "<{}>{}</{}>",
        layout_name, body, layout_name
    );
    expand_component_tags(&wrapped, registry)
        .with_context(|| format!("Failed to expand layout component '{}'", layout_name))
}

/// Compute the output HTML path in dist/ for a given route and page name.
/// `/`         → dist/index.html
/// `/blog`     → dist/blog/index.html
/// `/blog/:slug` → dist/blog/index.html  (params resolved at runtime)
fn derive_out_path(source_path: &Path, pages_root: &Path, out_dir: &Path) -> PathBuf {
    let relative = source_path.strip_prefix(pages_root).unwrap_or(source_path);
    let without_ext = relative.with_extension("");

    let parts: Vec<_> = without_ext
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();

    if parts.is_empty() || (parts.len() == 1 && parts[0] == "index") {
        return out_dir.join("index.html");
    }

    let mut path = out_dir.to_path_buf();
    for seg in &parts {
        if seg != "index" {
            path = path.join(seg);
        }
    }

    path.join("index.html")
}

fn warn_missing_components(
    html: &str,
    source_path: &Path,
    registry: &HashMap<String, CompiledComponent>,
) {
    let component_tag_re = Regex::new(r#"<([A-Z][A-Za-z0-9]*)\b"#).unwrap();

    for cap in component_tag_re.captures_iter(html) {
        let Some(name_match) = cap.get(1) else {
            continue;
        };

        let component_name = name_match.as_str();
        if !registry.contains_key(component_name) {
            eprintln!(
                "  warning: {} references missing component <{}>",
                source_path.display(),
                component_name
            );
        }
    }
}

/// Write a resolved page's HTML to disk, creating directories as needed.
pub fn write_page(page: &PageDef) -> Result<()> {
    // Inject the page body into a standard HTML shell if the body doesn't
    // already have a <!DOCTYPE> / <html> wrapper.
    let html = if page.body.trim_start().starts_with("<!") || page.body.contains("<html") {
        page.body.clone()
    } else {
        build_html_shell(&page.body)
    };

    fs::create_dir_all(page.out_path.parent().unwrap_or(Path::new(".")))
        .with_context(|| format!("Failed to create dir for {}", page.out_path.display()))?;

    fs::write(&page.out_path, &html)
        .with_context(|| format!("Failed to write page {}", page.out_path.display()))?;

    Ok(())
}

/// Build a minimal HTML shell wrapping the page body.
fn build_html_shell(body: &str) -> String {
    // Fallback: minimal wrapper
    format!(
        "<!DOCTYPE html>\n<html lang=\"en\">\n<head><meta charset=\"UTF-8\">\
         <meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">\
         <link rel=\"stylesheet\" href=\"/css/main.css\">\n</head>\n\
         <body>\n{}\n<script type=\"module\" src=\"/js/main.js\"></script>\n</body>\n</html>",
        body
    )
}
