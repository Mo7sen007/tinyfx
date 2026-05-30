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

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use anyhow::{bail, Context, Result};
use regex::Regex;

use super::component::{expand_component_tags, CompiledComponent};
use super::conditional::{parse_if_blocks, IfSite};
use super::dependency_graph::collect_component_references;
use super::each::{parse_each_blocks, EachSite};
use super::pages_util::{derive_route, interpolate_placeholders, parse_attrs};

#[derive(Debug, Clone)]
pub struct PageDef {
    pub name: String,
    pub route: String,
    pub html: String,
    pub source_path: PathBuf,
    pub out_path: PathBuf,
}

#[derive(Debug, Clone, Default)]
pub struct PageDirectives {
    pub page_name: String,
    pub each_sites: Vec<EachSite>,
    pub if_sites: Vec<IfSite>,
}

#[derive(Debug, Clone, Default)]
pub struct ComponentDirectives {
    pub component_name: String,
    pub each_sites: Vec<EachSite>,
    pub if_sites: Vec<IfSite>,
}

/// Collect pre-parsed directive sites for dynamic components.
pub fn collect_component_directives(
    component_registry: &HashMap<String, CompiledComponent>,
) -> Vec<ComponentDirectives> {
    let mut dynamic_components: Vec<&CompiledComponent> = component_registry
        .values()
        .filter(|component| component.is_dynamic)
        .collect();
    dynamic_components.sort_by(|left, right| left.name.cmp(&right.name));

    dynamic_components
        .into_iter()
        .map(|component| ComponentDirectives {
            component_name: component.name.clone(),
            each_sites: component.each_sites.clone(),
            if_sites: component.if_sites.clone(),
        })
        .collect()
}

/// Compile one page and collect compile-time directives.
pub fn compile_page(
    source_path: PathBuf,
    source_html: String,
    pages_root: PathBuf,
    out_dir: PathBuf,
    component_registry: std::sync::Arc<HashMap<String, CompiledComponent>>,
) -> Result<(PageDef, PageDirectives)> {
    parse_and_expand_page(
        &source_path,
        &source_html,
        &pages_root,
        &out_dir,
        component_registry.as_ref(),
    )
}

/// Write one resolved page to disk.
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

/// Resolve a page output path under `dist/`.
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

fn parse_and_expand_page(
    source_path: &Path,
    source_html: &str,
    pages_root: &Path,
    out_dir: &Path,
    component_registry: &HashMap<String, CompiledComponent>,
) -> Result<(PageDef, PageDirectives)> {
    let (page_attrs, page_inner_html) = extract_page_element(source_html, source_path)?;
    let route = page_attrs
        .get("route")
        .cloned()
        .filter(|route| !route.trim().is_empty())
        .unwrap_or_else(|| derive_route(source_path, pages_root));
    let layout = page_attrs
        .get("layout")
        .cloned()
        .filter(|layout| !layout.trim().is_empty());
    let page_title = page_attrs
        .get("title")
        .cloned()
        .filter(|t| !t.trim().is_empty());
    let body_without_script = strip_tinyfx_script(&page_inner_html, source_path);

    // Extract <head> content early, before component expansion.
    let (user_head, body_without_head) = extract_head_content(&body_without_script);

    warn_missing_components(&body_without_head, source_path, component_registry);

    let expanded_body = expand_component_tags(&body_without_head, component_registry).with_context(|| {
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

    let source_file = source_path.display().to_string();
    let (with_each, each_sites) = parse_each_blocks(
        &page_body,
        &page_name,
        &source_file,
        component_registry,
    )
    .with_context(|| {
        format!(
            "failed to parse <each> directives in '{}'\n  → fix the invalid loop markup and rebuild",
            source_path.display()
        )
    })?;
    let (with_conditionals, if_sites) =
        parse_if_blocks(&with_each, &page_name, component_registry).with_context(|| {
            format!(
                "failed to parse <if> directives in '{}'\n  → fix the invalid conditional markup and rebuild",
                source_path.display()
            )
        })?;
    let interpolated = interpolate_placeholders(&with_conditionals);
    let (semantic_body, semantic_warnings) = ensure_semantic_html(&interpolated, &page_name);
    for warning in semantic_warnings {
        eprintln!("warning: {}", warning);
    }
    let template_sites =
        collect_template_sites_for_page(&semantic_body, &each_sites, component_registry);
    let html_with_templates = inject_each_templates(
        &build_html_shell(
            &semantic_body,
            &page_name,
            &user_head,
            page_title.as_deref(),
        ),
        &template_sites,
    );

    Ok((
        PageDef {
            name: page_name.clone(),
            route,
            html: html_with_templates,
            source_path: source_path.to_path_buf(),
            out_path: resolve_page_out_path(source_path, pages_root, out_dir),
        },
        PageDirectives {
            page_name,
            each_sites,
            if_sites,
        },
    ))
}

fn extract_page_element(
    source_html: &str,
    source_path: &Path,
) -> Result<(HashMap<String, String>, String)> {
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

    Ok((
        parse_attrs(attrs_str),
        source_html[open_end..close_start].to_string(),
    ))
}

/// Extract `<head>...</head>` content from the page body.
///
/// Returns `(head_content, body_without_head)`. If no `<head>` block is found,
/// `head_content` is empty and the body is returned unchanged.
fn extract_head_content(html: &str) -> (String, String) {
    let re = head_re();
    if let Some(caps) = re.captures(html) {
        let head_content = caps
            .get(1)
            .map(|m| m.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        let body_without_head = re.replace(html, "").trim().to_string();
        (head_content, body_without_head)
    } else {
        (String::new(), html.to_string())
    }
}

fn strip_tinyfx_script(inner_html: &str, source_path: &Path) -> String {
    if let Some(captures) = tinyfx_script_re().captures(inner_html) {
        let script_body = captures
            .get(1)
            .map(|m| m.as_str())
            .unwrap_or_default()
            .trim();
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
    let Some(layout) = component_registry.get(layout_name) else {
        bail!(
            "layout '{}' not found\n  → confirm the layout component exists in src/components/{}",
            layout_name,
            layout_name
        );
    };
    let html = layout
        .html
        .replace("<slot></slot>", body)
        .replace("<slot/>", body)
        .replace("<slot />", body);
    Ok(html)
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

fn build_html_shell(
    body: &str,
    page_name: &str,
    user_head: &str,
    page_title: Option<&str>,
) -> String {
    if body.trim_start().starts_with("<!") || body.contains("<html") {
        return body.to_string();
    }

    let title = page_title
        .map(|t| t.to_string())
        .unwrap_or_else(|| title_from_page_name(page_name));

    let head = merge_head(user_head, &title);

    format!(
        "<!DOCTYPE html>\n<html lang=\"en\">\n{}\n<body>\n{}\n<script type=\"module\" src=\"/js/main.js\"></script>\n</body>\n</html>",
        head, body
    )
}

/// Merge user-provided head content with required TinyFX essentials.
///
/// Rules:
/// - `<meta charset>` is always set to UTF-8 (user overrides ignored)
/// - `<meta name="viewport">` is always included (user overrides ignored)
/// - `<title>` uses the resolved title
/// - User `<link rel="stylesheet">` tags are appended; `/css/main.css` is always included
/// - All other user head content is passed through as-is
fn merge_head(user_head: &str, title: &str) -> String {
    let mut head = String::from("<head>");

    // Essentials that must be present
    head.push_str("<meta charset=\"UTF-8\">");
    head.push_str("<meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">");
    head.push_str(&format!("<title>{}</title>", escape_html(title)));

    // Collect user stylesheet links so we can deduplicate main.css
    let mut user_stylesheets = Vec::new();
    let mut has_main_css = false;

    // Parse user head content line by line for stylesheets, pass through everything else
    for line in user_head.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some(href) = extract_stylesheet_href(trimmed) {
            if href == "/css/main.css" {
                has_main_css = true;
            }
            user_stylesheets.push(trimmed.to_string());
        } else {
            head.push_str(trimmed);
        }
    }

    // Append user stylesheets
    for link in &user_stylesheets {
        head.push_str(link);
    }

    // Always include main.css
    if !has_main_css {
        head.push_str("<link rel=\"stylesheet\" href=\"/css/main.css\">");
    }

    head.push_str("\n</head>");
    head
}

/// If the line is a `<link rel="stylesheet" href="...">`, return the href.
fn extract_stylesheet_href(line: &str) -> Option<String> {
    let lower = line.to_ascii_lowercase();
    if !lower.contains("rel=\"stylesheet\"") && !lower.contains("rel='stylesheet'") {
        return None;
    }

    stylesheet_href_re()
        .captures(line)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
}

/// Escape HTML special characters in text content.
fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn ensure_semantic_html(body: &str, page_name: &str) -> (String, Vec<String>) {
    let mut warnings = Vec::new();
    let mut output = body.trim().to_string();

    if !contains_tag(&output, "main") {
        warnings.push(format!(
            "page '{}' does not include a <main> tag; TinyFX wrapped page content in <main>",
            page_name
        ));
        output = format!("<main>\n{}\n</main>", output);
    }

    (output, warnings)
}

fn inject_each_templates(html: &str, template_sites: &[(String, String)]) -> String {
    if template_sites.is_empty() {
        return html.to_string();
    }

    let mut block = String::new();
    for (site_id, template_html) in template_sites {
        block.push_str(&format!(
            "<template id=\"fx-tmpl-{}\">{}</template>\n",
            site_id, template_html
        ));
    }

    if let Some(pos) = html.rfind("</body>") {
        let mut out = String::new();
        out.push_str(&html[..pos]);
        out.push_str(&block);
        out.push_str(&html[pos..]);
        out
    } else {
        format!("{}\n{}", html, block)
    }
}

fn collect_template_sites_for_page(
    body_html: &str,
    page_each_sites: &[EachSite],
    component_registry: &HashMap<String, CompiledComponent>,
) -> Vec<(String, String)> {
    let mut by_id = HashMap::<String, String>::new();
    for site in page_each_sites {
        by_id.insert(site.id.clone(), site.template_html.clone());
    }
    for component in component_registry.values() {
        for site in &component.each_sites {
            by_id.insert(site.id.clone(), site.template_html.clone());
        }
    }

    let mut seen_ids = HashSet::new();
    let mut ordered = Vec::<(String, String)>::new();
    for cap in data_each_re().captures_iter(body_html) {
        let Some(id_match) = cap.get(1) else {
            continue;
        };
        let id = id_match.as_str().trim();
        if seen_ids.insert(id.to_string()) {
            if let Some(template_html) = by_id.get(id) {
                ordered.push((id.to_string(), template_html.clone()));
            }
        }
    }

    ordered
}

fn contains_tag(html: &str, tag: &str) -> bool {
    let open = format!("<{}", tag);
    let close = format!("</{}>", tag);
    html.contains(&open) || html.contains(&close)
}

fn title_from_page_name(page_name: &str) -> String {
    let words = page_name
        .split(['-', '_'])
        .filter(|part| !part.trim().is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => {
                    let mut w = String::new();
                    for ch in first.to_uppercase() {
                        w.push(ch);
                    }
                    w.push_str(chars.as_str());
                    w
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>();

    if words.is_empty() {
        "TinyFX".to_string()
    } else {
        words.join(" ")
    }
}

fn head_re() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(?s)<head[^>]*>(.*?)</head>").unwrap())
}

fn stylesheet_href_re() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r#"href\s*=\s*["']([^"']+)["']"#).unwrap())
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

fn data_each_re() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r#"data-each\s*=\s*["']([^"']+)["']"#).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semantic_guard_wraps_main_and_warns() {
        let (out, warnings) = ensure_semantic_html("<section>hello</section>", "index");
        assert!(out.contains("<main>"));
        assert!(warnings.iter().any(|w| w.contains("<main>")));
        assert_eq!(warnings.len(), 1);
    }

    #[test]
    fn extract_head_content_finds_head_block() {
        let html = "<head>\n  <meta name=\"description\" content=\"test\">\n</head>\n<main><h1>Hello</h1></main>";
        let (head, body) = extract_head_content(html);
        assert!(head.contains("description"));
        assert!(body.contains("<main>"));
        assert!(!body.contains("<head>"));
    }

    #[test]
    fn extract_head_content_returns_empty_when_no_head() {
        let html = "<main><h1>Hello</h1></main>";
        let (head, body) = extract_head_content(html);
        assert!(head.is_empty());
        assert_eq!(body, html);
    }

    #[test]
    fn merge_head_includes_essentials() {
        let head = merge_head("", "My Page");
        assert!(head.contains("<meta charset=\"UTF-8\">"));
        assert!(head.contains("name=\"viewport\""));
        assert!(head.contains("<title>My Page</title>"));
        assert!(head.contains("href=\"/css/main.css\""));
    }

    #[test]
    fn merge_head_passes_through_user_content() {
        let user_head = "<meta name=\"description\" content=\"A test page\">";
        let head = merge_head(user_head, "My Page");
        assert!(head.contains("description"));
        assert!(head.contains("A test page"));
    }

    #[test]
    fn merge_head_appends_user_stylesheets() {
        let user_head = "<link rel=\"stylesheet\" href=\"/css/blog.css\">";
        let head = merge_head(user_head, "Blog");
        assert!(head.contains("href=\"/css/blog.css\""));
        assert!(head.contains("href=\"/css/main.css\""));
    }

    #[test]
    fn merge_head_avoids_duplicate_main_css() {
        let user_head = "<link rel=\"stylesheet\" href=\"/css/main.css\">";
        let head = merge_head(user_head, "Page");
        let count = head.matches("href=\"/css/main.css\"").count();
        assert_eq!(count, 1);
    }

    #[test]
    fn build_html_shell_uses_page_title_attr() {
        let html = build_html_shell("<main>hi</main>", "blog-post", "", Some("Custom Title"));
        assert!(html.contains("<title>Custom Title</title>"));
        assert!(!html.contains("<title>Blog Post</title>"));
    }

    #[test]
    fn build_html_shell_falls_back_to_filename() {
        let html = build_html_shell("<main>hi</main>", "blog-post", "", None);
        assert!(html.contains("<title>Blog Post</title>"));
    }

    #[test]
    fn build_html_shell_escapes_title() {
        let html = build_html_shell("<main>hi</main>", "page", "", Some("Tom & Jerry"));
        assert!(html.contains("<title>Tom &amp; Jerry</title>"));
    }

    #[test]
    fn template_injection_occurs_before_body_close() {
        let html = "<html><body><div>ok</div></body></html>";
        let out = inject_each_templates(
            html,
            &[(
                "index_each_0".to_string(),
                "<div data-fx=\"0\"></div>".to_string(),
            )],
        );
        let body_close = out.rfind("</body>").expect("body close exists");
        let tmpl = out
            .find("<template id=\"fx-tmpl-index_each_0\">")
            .expect("template should be injected");
        assert!(tmpl < body_close);
    }

    #[test]
    fn collect_templates_includes_component_sites_when_used() {
        let body = "<article><div data-each=\"chat_each_0\"></div></article>";
        let mut registry = HashMap::new();
        registry.insert(
            "Chat".to_string(),
            CompiledComponent {
                name: "Chat".to_string(),
                path: "src/components/Chat/Chat.html".into(),
                html: "<section></section>".to_string(),
                is_dynamic: true,
                each_sites: vec![EachSite {
                    id: "chat_each_0".to_string(),
                    source: super::super::codegen::LoopSource::Signal("messages".to_string()),
                    item_var: "message".to_string(),
                    key_strategy: super::super::each::KeyStrategy::Keyed {
                        field_name: "id".to_string(),
                    },
                    template_html: "<div data-fx=\"0\"></div>".to_string(),
                    bindings: vec![],

                    dynamic_component_names: vec![],
                }],
                if_sites: vec![],
            },
        );

        let templates = collect_template_sites_for_page(body, &[], &registry);
        assert_eq!(templates.len(), 1);
        assert_eq!(templates[0].0, "chat_each_0");
        assert_eq!(templates[0].1, "<div data-fx=\"0\"></div>");
    }
}
