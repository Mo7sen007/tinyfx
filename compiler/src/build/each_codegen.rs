//! # each_codegen
//!
//! TypeScript generation for parsed TinyFX `<each>` directive sites.
//!
//! ## Responsibilities
//! - Emit per-site create/update/reconcile functions
//! - Emit source wiring for signal/plain/numeric loop sources
//! - Keep generated symbol names deterministic and site-scoped
//!
//! ## What this module does NOT do
//! - It does not parse HTML
//! - It does not validate directive attributes
//! - It does not mount runtime components directly

use std::sync::OnceLock;

use regex::Regex;

use super::codegen::{DirectiveContext, LoopSource, TsBuffer, qualify_expr};
use super::each::EachSite;

#[derive(Debug, Clone, PartialEq, Eq)]
enum BindingKind {
    Attribute(String),
    TextContent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TemplateBinding {
    selector: String,
    expression: String,
    var_name: String,
    kind: BindingKind,
}

#[derive(Debug, Clone)]
struct NodeCtx {
    tag: String,
    first_class: Option<String>,
}

/// Generate all TypeScript code for one `<each>` site.
#[allow(dead_code)]
pub fn generate_each_code(site: &EachSite, context: &DirectiveContext, buf: &mut TsBuffer) {
    generate_each_helpers(site, context, buf);
    generate_each_wiring(site, context, buf);
}

/// Generate create/update/reconcile helpers for one `<each>` site.
pub fn generate_each_helpers(site: &EachSite, context: &DirectiveContext, buf: &mut TsBuffer) {
    let key_expr = key_expression(&site.item_var, &site.key_expr);
    let template = html_template_literal(&site.template_html);
    let bindings = extract_template_bindings(&site.template_html, &site.item_var);

    emit_create(site, &key_expr, &template, buf);
    emit_update(site, &bindings, buf);
    emit_reconcile(site, &key_expr, context, buf);
}

/// Generate runtime wiring calls for one `<each>` site.
pub fn generate_each_wiring(site: &EachSite, context: &DirectiveContext, buf: &mut TsBuffer) {
    match context {
        DirectiveContext::Page { module_alias } => emit_source_wiring(site, module_alias, buf),
        DirectiveContext::Component => {}
    }
}

fn emit_create(site: &EachSite, key_expr: &str, template: &str, buf: &mut TsBuffer) {
    buf.line(&format!("// <each> site {}", site.id));
    buf.line(&format!(
        "function __create_{}({}: any): HTMLElement {{",
        site.id, site.item_var
    ));
    buf.indent();
    buf.line("const __wrapper = document.createElement(\"div\");");
    buf.line(&format!("__wrapper.innerHTML = {};", template));
    buf.line("const el = __wrapper.firstElementChild as HTMLElement;");
    buf.line("if (!el) return document.createElement(\"div\");");
    buf.line(&format!("el.dataset.key = String({});", key_expr));
    buf.line("return el;");
    buf.dedent();
    buf.line("}");
    buf.blank();
}

fn emit_update(site: &EachSite, bindings: &[TemplateBinding], buf: &mut TsBuffer) {
    buf.line(&format!(
        "function __update_{}({}: any, el: HTMLElement): void {{",
        site.id, site.item_var
    ));
    buf.indent();
    for binding in bindings {
        match &binding.kind {
            BindingKind::Attribute(attr_name) => {
                buf.line(&format!(
                    "el.querySelector(\"{}\")?.setAttribute(\"{}\", String({}));",
                    binding.selector, attr_name, binding.expression
                ));
            }
            BindingKind::TextContent => {
                buf.line(&format!(
                    "const __el_{} = el.querySelector<HTMLElement>(\"{}\");",
                    binding.var_name, binding.selector
                ));
                buf.line(&format!(
                    "if (__el_{}) __el_{}.textContent = String({});",
                    binding.var_name, binding.var_name, binding.expression
                ));
            }
        }
    }
    buf.dedent();
    buf.line("}");
    buf.blank();
}

fn emit_reconcile(site: &EachSite, key_expr: &str, context: &DirectiveContext, buf: &mut TsBuffer) {
    let signature = match context {
        DirectiveContext::Page { .. } => {
            format!("function __reconcile_{}(container: HTMLElement, items: any[]): void {{", site.id)
        }
        DirectiveContext::Component => {
            format!(
                "function __reconcile_{}(container: HTMLElement, items: any[], inst: any): void {{",
                site.id
            )
        }
    };
    buf.line(&signature);
    buf.indent();
    buf.line("const existing = new Map<string, HTMLElement>();");
    buf.line("for (const child of Array.from(container.children)) {");
    buf.indent();
    buf.line("const key = (child as HTMLElement).dataset.key;");
    buf.line("if (key !== undefined) existing.set(key, child as HTMLElement);");
    buf.dedent();
    buf.line("}");
    buf.line("const seen = new Set<string>();");
    buf.line("for (let __idx = 0; __idx < items.length; __idx++) {");
    buf.indent();
    buf.line(&format!("const {} = items[__idx];", site.item_var));
    buf.line(&format!("const key = String({});", key_expr));
    buf.line("seen.add(key);");
    buf.line("const existingNode = existing.get(key);");
    buf.line("if (existingNode) {");
    buf.indent();
    buf.line(&format!(
        "__update_{}({}, existingNode);",
        site.id, site.item_var
    ));
    buf.line("if (container.children[__idx] !== existingNode) {");
    buf.indent();
    buf.line("container.insertBefore(existingNode, container.children[__idx] ?? null);");
    buf.dedent();
    buf.line("}");
    buf.dedent();
    buf.line("} else {");
    buf.indent();
    buf.line(&format!(
        "const newNode = __create_{}({});",
        site.id, site.item_var
    ));
    buf.line("container.insertBefore(newNode, container.children[__idx] ?? null);");
    buf.line("newNode.querySelectorAll(\"[data-component]:not([data-tinyfx-mounted])\").forEach((node) => {");
    buf.indent();
    buf.line("const name = node.getAttribute(\"data-component\");");
    match context {
        DirectiveContext::Page { .. } => {
            buf.line("if (name && registry[name]) registry[name](node as HTMLElement, ctx);");
        }
        DirectiveContext::Component => {
            buf.line(
                "if (name && registry[name]) registry[name](node as HTMLElement, inst?.ctx as TinyFxContext);",
            );
        }
    }
    buf.dedent();
    buf.line("});");
    buf.dedent();
    buf.line("}");
    buf.dedent();
    buf.line("}");
    buf.line("for (const [key, node] of existing) {");
    buf.indent();
    buf.line("if (!seen.has(key)) {");
    buf.indent();
    buf.line("__destroyNode(node);");
    buf.line("container.removeChild(node);");
    buf.dedent();
    buf.line("}");
    buf.dedent();
    buf.line("}");
    buf.dedent();
    buf.line("}");
    buf.blank();
}

fn emit_source_wiring(site: &EachSite, module_alias: &str, buf: &mut TsBuffer) {
    buf.line(&format!(
        "const __container_{} = document.querySelector(\"[data-each='{}']\") as HTMLElement | null;",
        site.id, site.id
    ));
    buf.line(&format!("if (__container_{}) {{", site.id));
    buf.indent();
    match &site.source {
        LoopSource::Signal(name) => {
            let qualified = qualify_expr(&format!("{}()", name), module_alias);
            buf.line(&format!(
                "effect(() => __reconcile_{}(__container_{}, {}));",
                site.id, site.id, qualified
            ));
        }
        LoopSource::Plain(expr) => {
            let qualified = qualify_expr(expr, module_alias);
            buf.line(&format!(
                "__reconcile_{}(__container_{}, {} as any[]);",
                site.id, site.id, qualified
            ));
        }
        LoopSource::Range(start, end) => {
            let len = end.saturating_sub(*start);
            buf.line(&format!(
                "const __items_{} = Array.from({{ length: {} }}, (_, __idx) => __idx + {});",
                site.id, len, start
            ));
            buf.line(&format!(
                "__reconcile_{}(__container_{}, __items_{} as any[]);",
                site.id, site.id, site.id
            ));
        }
    }
    buf.dedent();
    buf.line("}");
    buf.blank();
}

fn key_expression(item_var: &str, key_expr: &str) -> String {
    let trimmed = key_expr.trim();
    if trimmed == item_var || trimmed.starts_with(item_var) || trimmed.contains('.') {
        trimmed.to_string()
    } else {
        format!("{}.{}", item_var, trimmed)
    }
}

fn html_template_literal(template_html: &str) -> String {
    let mut escaped = template_html
        .replace('\\', "\\\\")
        .replace('`', "\\`")
        .replace("${", "\\${");
    for captures in binding_re().captures_iter(template_html) {
        let whole = captures.get(0).map(|m| m.as_str()).unwrap_or_default();
        let expr = captures
            .get(1)
            .map(|m| m.as_str())
            .unwrap_or_default()
            .trim();
        escaped = escaped.replace(whole, &format!("${{{}}}", expr));
    }
    format!("`{}`", escaped)
}

fn extract_template_bindings(template_html: &str, item_var: &str) -> Vec<TemplateBinding> {
    let mut bindings = Vec::new();
    let item_prefix = format!("{}.", item_var);
    let mut stack: Vec<NodeCtx> = Vec::new();
    let mut cursor = 0usize;

    while cursor < template_html.len() {
        let Some(next_tag_rel) = template_html[cursor..].find('<') else {
            let text = &template_html[cursor..];
            collect_text_bindings(text, &stack, &item_prefix, &mut bindings);
            break;
        };
        let next_tag = cursor + next_tag_rel;
        let text = &template_html[cursor..next_tag];
        collect_text_bindings(text, &stack, &item_prefix, &mut bindings);

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
            let _ = stack.pop();
            cursor = tag_end + 1;
            continue;
        }

        let self_closing = tag_src.ends_with('/');
        let open = if self_closing {
            tag_src[..tag_src.len().saturating_sub(1)].trim_end()
        } else {
            tag_src
        };

        let mut parts = open.splitn(2, char::is_whitespace);
        let tag = parts.next().unwrap_or_default().trim().to_string();
        if tag.is_empty() {
            cursor = tag_end + 1;
            continue;
        }
        let attrs = parts.next().unwrap_or_default();

        let node = NodeCtx {
            tag,
            first_class: first_class_from_attrs(attrs),
        };
        let selector = selector_for_node(&stack, &node);

        for cap in html_attr_re().captures_iter(attrs) {
            let attr_name = cap.get(1).map(|m| m.as_str()).unwrap_or_default();
            let attr_value = cap
                .get(2)
                .or_else(|| cap.get(3))
                .or_else(|| cap.get(4))
                .map(|m| m.as_str())
                .unwrap_or_default();

            for bind in binding_re().captures_iter(attr_value) {
                let expression = bind
                    .get(1)
                    .map(|m| m.as_str())
                    .unwrap_or_default()
                    .trim();
                if !expression.starts_with(&item_prefix) {
                    continue;
                }

                bindings.push(TemplateBinding {
                    selector: selector.clone(),
                    expression: expression.to_string(),
                    var_name: sanitize_var_name(&selector, bindings.len()),
                    kind: BindingKind::Attribute(attr_name.to_string()),
                });
            }
        }

        if !self_closing {
            stack.push(node);
        }

        cursor = tag_end + 1;
    }

    bindings
}

fn collect_text_bindings(
    text: &str,
    stack: &[NodeCtx],
    item_prefix: &str,
    bindings: &mut Vec<TemplateBinding>,
) {
    let Some(node) = stack.last() else {
        return;
    };

    for cap in binding_re().captures_iter(text) {
        let expression = cap
            .get(1)
            .map(|m| m.as_str())
            .unwrap_or_default()
            .trim();
        if !expression.starts_with(item_prefix) {
            continue;
        }

        let selector = selector_for_node(&stack[..stack.len().saturating_sub(1)], node);
        bindings.push(TemplateBinding {
            selector: selector.clone(),
            expression: expression.to_string(),
            var_name: sanitize_var_name(&selector, bindings.len()),
            kind: BindingKind::TextContent,
        });
    }
}

fn selector_for_node(parent_stack: &[NodeCtx], node: &NodeCtx) -> String {
    let current = segment_for(node);
    if node.first_class.is_some() {
        return current;
    }

    let Some(parent) = parent_stack.last() else {
        return current;
    };

    if parent.first_class.is_some() {
        format!("{} > {}", segment_for(parent), current)
    } else {
        current
    }
}

fn segment_for(node: &NodeCtx) -> String {
    if let Some(class_name) = &node.first_class {
        format!("{}.{}", node.tag, class_name)
    } else {
        node.tag.clone()
    }
}

fn first_class_from_attrs(attrs: &str) -> Option<String> {
    for cap in html_attr_re().captures_iter(attrs) {
        let name = cap.get(1).map(|m| m.as_str()).unwrap_or_default();
        if name != "class" {
            continue;
        }

        let value = cap
            .get(2)
            .or_else(|| cap.get(3))
            .or_else(|| cap.get(4))
            .map(|m| m.as_str())
            .unwrap_or_default();

        if let Some(class_name) = value.split_whitespace().next()
            && !class_name.is_empty()
            && class_name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
        {
            return Some(class_name.to_string());
        }
    }

    None
}

fn sanitize_var_name(selector: &str, idx: usize) -> String {
    let mut out = String::new();
    let mut prev_underscore = false;

    for c in selector.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c);
            prev_underscore = false;
        } else if !prev_underscore {
            out.push('_');
            prev_underscore = true;
        }
    }

    let trimmed = out.trim_matches('_');
    if trimmed.is_empty() {
        format!("node_{}", idx)
    } else {
        format!("{}_{}", trimmed, idx)
    }
}

fn binding_re() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r#"\{\{\s*([^}]+)\s*\}\}"#).expect("valid binding regex"))
}

fn html_attr_re() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r#"([:@a-zA-Z0-9_-]+)\s*=\s*(?:\"([^\"]*)\"|'([^']*)'|([^\s>]+))"#)
            .expect("valid html attr regex")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_create_function_sets_key() {
        let site = EachSite {
            id: "index_each_0".to_string(),
            source: LoopSource::Signal("posts".to_string()),
            item_var: "post".to_string(),
            key_expr: "id".to_string(),
            template_html: "<div>{{post.title}}</div>".to_string(),
            has_dynamic_components: false,
            dynamic_component_names: Vec::new(),
        };
        let mut buf = TsBuffer::new();
        generate_each_code(
            &site,
            &DirectiveContext::Page {
                module_alias: "PageModule0".to_string(),
            },
            &mut buf,
        );
        let code = buf.build();
        assert!(code.contains("const __wrapper = document.createElement(\"div\")"));
        assert!(code.contains("const el = __wrapper.firstElementChild as HTMLElement"));
        assert!(code.contains("el.dataset.key = String(post.id)"));
        assert!(code.contains("return el;"));
    }

    #[test]
    fn generated_update_function_omits_key() {
        let site = EachSite {
            id: "index_each_0".to_string(),
            source: LoopSource::Plain("posts".to_string()),
            item_var: "post".to_string(),
            key_expr: "id".to_string(),
            template_html: "<div>{{post.title}}</div>".to_string(),
            has_dynamic_components: false,
            dynamic_component_names: Vec::new(),
        };
        let mut buf = TsBuffer::new();
        generate_each_code(
            &site,
            &DirectiveContext::Page {
                module_alias: "PageModule0".to_string(),
            },
            &mut buf,
        );
        let code = buf.build();
        let update_start = code
            .find("function __update_index_each_0")
            .expect("update fn should exist");
        let reconcile_start = code
            .find("function __reconcile_index_each_0")
            .expect("reconcile fn should exist");
        assert!(!code[update_start..reconcile_start].contains("dataset.key"));
        assert!(!code[update_start..reconcile_start].contains("innerHTML"));
        assert!(!code[update_start..reconcile_start].contains("__destroyNode"));
    }

    #[test]
    fn generated_reconciler_has_null_check() {
        let site = EachSite {
            id: "index_each_0".to_string(),
            source: LoopSource::Plain("posts".to_string()),
            item_var: "post".to_string(),
            key_expr: "id".to_string(),
            template_html: "<div>{{post.title}}</div>".to_string(),
            has_dynamic_components: false,
            dynamic_component_names: Vec::new(),
        };
        let mut buf = TsBuffer::new();
        generate_each_code(
            &site,
            &DirectiveContext::Page {
                module_alias: "PageModule0".to_string(),
            },
            &mut buf,
        );
        assert!(buf.build().contains("if (__container_index_each_0)"));
    }

    #[test]
    fn signal_source_wraps_in_effect() {
        let site = EachSite {
            id: "index_each_0".to_string(),
            source: LoopSource::Signal("posts".to_string()),
            item_var: "post".to_string(),
            key_expr: "id".to_string(),
            template_html: "<div></div>".to_string(),
            has_dynamic_components: false,
            dynamic_component_names: Vec::new(),
        };
        let mut buf = TsBuffer::new();
        generate_each_code(
            &site,
            &DirectiveContext::Page {
                module_alias: "PageModule0".to_string(),
            },
            &mut buf,
        );
        assert!(
            buf.build()
                .contains("effect(() => __reconcile_index_each_0")
        );
    }

    #[test]
    fn plain_source_calls_directly() {
        let site = EachSite {
            id: "index_each_0".to_string(),
            source: LoopSource::Plain("posts".to_string()),
            item_var: "post".to_string(),
            key_expr: "id".to_string(),
            template_html: "<div></div>".to_string(),
            has_dynamic_components: false,
            dynamic_component_names: Vec::new(),
        };
        let mut buf = TsBuffer::new();
        generate_each_code(
            &site,
            &DirectiveContext::Page {
                module_alias: "PageModule0".to_string(),
            },
            &mut buf,
        );
        let code = buf.build();
        assert!(
            code.contains(
                "__reconcile_index_each_0(__container_index_each_0, PageModule0.posts as any[])"
            )
        );
    }

    #[test]
    fn numeric_source_generates_array_from() {
        let site = EachSite {
            id: "index_each_0".to_string(),
            source: LoopSource::Range(0, 10),
            item_var: "i".to_string(),
            key_expr: "i".to_string(),
            template_html: "<div></div>".to_string(),
            has_dynamic_components: false,
            dynamic_component_names: Vec::new(),
        };
        let mut buf = TsBuffer::new();
        generate_each_code(
            &site,
            &DirectiveContext::Page {
                module_alias: "PageModule0".to_string(),
            },
            &mut buf,
        );
        assert!(
            buf.build()
                .contains("Array.from({ length: 10 }, (_, __idx) => __idx + 0)")
        );
    }

    #[test]
    fn generated_reconciler_uses_non_shadowed_index() {
        let site = EachSite {
            id: "index_each_0".to_string(),
            source: LoopSource::Plain("items".to_string()),
            item_var: "item".to_string(),
            key_expr: "id".to_string(),
            template_html: "<div></div>".to_string(),
            has_dynamic_components: false,
            dynamic_component_names: Vec::new(),
        };
        let mut buf = TsBuffer::new();
        generate_each_code(
            &site,
            &DirectiveContext::Page {
                module_alias: "PageModule0".to_string(),
            },
            &mut buf,
        );
        let code = buf.build();
        assert!(code.contains("for (let __idx = 0; __idx < items.length; __idx++)"));
        assert!(!code.contains("const __idx = items[__idx]"));
    }

    #[test]
    fn generated_update_uses_targeted_bindings() {
        let site = EachSite {
            id: "index_each_0".to_string(),
            source: LoopSource::Plain("items".to_string()),
            item_var: "feature".to_string(),
            key_expr: "i".to_string(),
            template_html: "<div class=\"feature-card\"><div class=\"feature-icon\"><i class=\"{{feature.icon}}\"></i></div><h3 class=\"feature-title\">{{feature.title}}</h3><p class=\"feature-description\">{{feature.description}}</p></div>".to_string(),
            has_dynamic_components: false,
            dynamic_component_names: Vec::new(),
        };
        let mut buf = TsBuffer::new();
        generate_each_code(
            &site,
            &DirectiveContext::Page {
                module_alias: "PageModule0".to_string(),
            },
            &mut buf,
        );
        let code = buf.build();

        assert!(code.contains("function __update_index_each_0(feature: any, el: HTMLElement): void"));
        assert!(code.contains("el.querySelector(\"div.feature-icon > i\")?.setAttribute(\"class\", String(feature.icon));"));
        assert!(code.contains("el.querySelector<HTMLElement>(\"h3.feature-title\")"));
        assert!(code.contains("el.querySelector<HTMLElement>(\"p.feature-description\")"));
        assert!(!code.contains("innerHTML !=="));
        assert!(!code.contains("el.innerHTML ="));
    }

    #[test]
    fn reconciler_calls_update_with_item_first() {
        let site = EachSite {
            id: "index_each_0".to_string(),
            source: LoopSource::Plain("items".to_string()),
            item_var: "feature".to_string(),
            key_expr: "id".to_string(),
            template_html: "<div>{{feature.title}}</div>".to_string(),
            has_dynamic_components: false,
            dynamic_component_names: Vec::new(),
        };
        let mut buf = TsBuffer::new();
        generate_each_code(
            &site,
            &DirectiveContext::Page {
                module_alias: "PageModule0".to_string(),
            },
            &mut buf,
        );
        assert!(
            buf.build()
                .contains("__update_index_each_0(feature, existingNode);")
        );
    }
}
