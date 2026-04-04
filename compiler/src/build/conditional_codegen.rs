//! # conditional_codegen
//!
//! TypeScript generation for parsed TinyFX `<if>` sites.
//!
//! ## Responsibilities
//! - Emit per-site toggle functions
//! - Emit reactive/non-reactive wiring based on condition kinds
//! - Keep generated code site-scoped and deterministic
//!
//! ## What this module does NOT do
//! - It does not parse HTML
//! - It does not evaluate conditions during compilation
//! - It does not write generated files

use super::codegen::{detect_value_kind, qualify_expr, DirectiveContext, TsBuffer, ValueKind};
use super::conditional::IfSite;

/// Generate TypeScript for one `<if>` site.
#[allow(dead_code)]
pub fn generate_if_code(site: &IfSite, context: &DirectiveContext, buf: &mut TsBuffer) {
    generate_if_helpers(site, context, buf);
    generate_if_wiring(site, context, buf);
}

/// Generate the `__toggle_*` helper for one `<if>` site.
pub fn generate_if_helpers(site: &IfSite, context: &DirectiveContext, buf: &mut TsBuffer) {
    let else_branch_index = site
        .branches
        .iter()
        .position(|b| b.condition.is_none())
        .map(|v| v as isize)
        .unwrap_or(-1);

    buf.line(&format!("// <if> site {}", site.id));
    match context {
        DirectiveContext::Page { .. } => {
            buf.line(&format!("function __toggle_{}(): void {{", site.id));
        }
        DirectiveContext::Component { .. } => {
            buf.line(&format!(
                "function __toggle_{}(inst: any): void {{",
                site.id
            ));
        }
    }
    buf.indent();
    match context {
        DirectiveContext::Page { .. } => {
            buf.line(&format!(
                "const wrapper = document.querySelector(\"[data-if='{}']\");",
                site.id
            ));
        }
        DirectiveContext::Component { .. } => {
            // Scope to the mounted component root so multiple instances do not share wrappers.
            buf.line("const host = inst?.el as HTMLElement | undefined;");
            buf.line(&format!(
                "const wrapper = host?.querySelector(\"[data-if='{}']\") as HTMLElement | null;",
                site.id
            ));
        }
    }
    buf.line("if (!wrapper) return;");
    buf.line("const branches = Array.from(wrapper.querySelectorAll(\":scope > [data-branch]\"));");
    buf.line("const conditions = [");
    buf.indent();
    for branch in &site.branches {
        if let Some(condition) = &branch.condition {
            let qualified = qualify_if_expr(condition, context);
            buf.line(&format!("{},", qualified));
        }
    }
    buf.dedent();
    buf.line("];\nlet shown = -1;");
    buf.line("for (let i = 0; i < conditions.length; i++) {");
    buf.indent();
    buf.line("if (conditions[i]) { shown = i; break; }");
    buf.dedent();
    buf.line("}");
    buf.line(&format!("const elseBranchIndex = {};", else_branch_index));
    buf.line("branches.forEach((branch, i) => {");
    buf.indent();
    buf.line("const shouldShow = i === shown || (shown === -1 && i === elseBranchIndex);");
    buf.line("(branch as HTMLElement).hidden = !shouldShow;");
    buf.dedent();
    buf.line("});");
    buf.line("if (shown >= 0 || elseBranchIndex >= 0) {");
    buf.indent();
    buf.line("const activeIndex = shown >= 0 ? shown : elseBranchIndex;");
    buf.line("const activeBranch = branches[activeIndex] as HTMLElement;");
    buf.line("activeBranch.querySelectorAll(\"[data-component]:not([data-tinyfx-mounted])\").forEach((node) => {");
    buf.indent();
    buf.line("const name = node.getAttribute(\"data-component\");");
    match context {
        DirectiveContext::Page { .. } => {
            buf.line("if (name && registry[name]) registry[name](node as HTMLElement, ctx);");
        }
        DirectiveContext::Component { .. } => {
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
    buf.blank();
}

/// Generate runtime execution wiring for one `<if>` site.
pub fn generate_if_wiring(site: &IfSite, context: &DirectiveContext, buf: &mut TsBuffer) {
    let has_signal = site.branches.iter().any(|branch| {
        branch.is_signal
            || branch
                .condition
                .as_ref()
                .is_some_and(|expr| matches!(detect_value_kind(expr), ValueKind::Signal(_)))
    });

    match context {
        DirectiveContext::Page { .. } => {
            if has_signal {
                buf.line(&format!("effect(() => __toggle_{}());", site.id));
            } else {
                buf.line(&format!("__toggle_{}();", site.id));
            }
        }
        DirectiveContext::Component { .. } => {
            if has_signal {
                buf.line(&format!("effect(() => __toggle_{}(inst));", site.id));
            } else {
                buf.line(&format!("__toggle_{}(inst);", site.id));
            }
        }
    }
    buf.blank();
}

fn qualify_if_expr(expr: &str, context: &DirectiveContext) -> String {
    match context {
        DirectiveContext::Page { module_alias, .. } => qualify_expr(expr, module_alias),
        DirectiveContext::Component { .. } => qualify_component_expr(expr),
    }
}

fn qualify_component_expr(expr: &str) -> String {
    let trimmed = expr.trim();
    if trimmed.parse::<usize>().is_ok() {
        return trimmed.to_string();
    }

    if trimmed.ends_with("()") {
        let name = trimmed[..trimmed.len() - 2].trim();
        return format!("inst.{}()", name);
    }

    if trimmed.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return format!("inst.{}", trimmed);
    }

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
    if ident.is_empty() {
        return trimmed.to_string();
    }

    let rest: String = chars.collect();
    format!("inst.{}{}", ident, rest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build::conditional::{IfBranch, IfSite};

    #[test]
    fn mixed_signal_and_plain_conditions_uses_effect() {
        let site = IfSite {
            id: "index_if_0".to_string(),
            branches: vec![
                IfBranch {
                    condition: Some("isAdmin()".to_string()),
                    is_signal: true,
                    html: "<h1>a</h1>".to_string(),
                    dynamic_component_names: Vec::new(),
                },
                IfBranch {
                    condition: Some("user !== null".to_string()),
                    is_signal: false,
                    html: "<h1>b</h1>".to_string(),
                    dynamic_component_names: Vec::new(),
                },
            ],
        };
        let mut buf = TsBuffer::new();
        generate_if_code(
            &site,
            &DirectiveContext::Page {
                module_alias: "PageModule0".to_string(),
                release: false,
            },
            &mut buf,
        );
        assert!(buf.build().contains("effect(() => __toggle_index_if_0())"));
    }

    #[test]
    fn generated_toggle_has_null_check() {
        let site = IfSite {
            id: "index_if_0".to_string(),
            branches: vec![IfBranch {
                condition: Some("ready".to_string()),
                is_signal: false,
                html: "<h1>a</h1>".to_string(),
                dynamic_component_names: Vec::new(),
            }],
        };
        let mut buf = TsBuffer::new();
        generate_if_code(
            &site,
            &DirectiveContext::Page {
                module_alias: "PageModule0".to_string(),
                release: false,
            },
            &mut buf,
        );
        assert!(buf.build().contains("if (!wrapper) return;"));
    }
}
