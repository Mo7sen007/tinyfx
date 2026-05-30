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

/// Generate the `__toggle_*` helper for one `<if>` site.
pub fn generate_if_helpers(site: &IfSite, context: &DirectiveContext, buf: &mut TsBuffer) {
    let else_branch_index = site
        .branches
        .iter()
        .position(|b| b.condition.is_none())
        .map(|v| v as isize)
        .unwrap_or(-1);

    let all_dynamic_names = collect_all_dynamic_names(site);

    buf.line(&format!("// <if> site {}", site.id));
    match context {
        DirectiveContext::Page { .. } => {
            buf.line(&format!(
                "function __toggle_{}(ctx: TinyFxContext): void {{",
                site.id
            ));
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
    emit_if_branch_mounts(&all_dynamic_names, context, buf);
    buf.dedent();
    buf.line("}");
    buf.dedent();
    buf.line("}");
    buf.blank();
}

fn collect_all_dynamic_names(site: &IfSite) -> Vec<String> {
    let mut names = Vec::new();
    for branch in &site.branches {
        for name in &branch.dynamic_component_names {
            if !names.contains(name) {
                names.push(name.clone());
            }
        }
    }
    names
}

fn emit_if_branch_mounts(names: &[String], context: &DirectiveContext, buf: &mut TsBuffer) {
    if names.is_empty() {
        return;
    }

    for name in names {
        let mount_fn = format!("__mount_{}", name);
        match context {
            DirectiveContext::Page { .. } => {
                buf.line(&format!(
                    "activeBranch.querySelectorAll<HTMLElement>(\"[data-component='{}']\").forEach((node) => {{",
                    name
                ));
                buf.indent();
                buf.line(&format!("{}(node, ctx);", mount_fn));
                buf.dedent();
                buf.line("});");
            }
            DirectiveContext::Component { .. } => {
                buf.line(&format!(
                    "activeBranch.querySelectorAll<HTMLElement>(\"[data-component='{}']\").forEach((node) => {{",
                    name
                ));
                buf.indent();
                buf.line(&format!("{}(node, inst?.ctx as TinyFxContext);", mount_fn));
                buf.dedent();
                buf.line("});");
            }
        }
    }
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
                buf.line(&format!("effect(() => __toggle_{}(ctx));", site.id));
            } else {
                buf.line(&format!("__toggle_{}(ctx);", site.id));
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
        DirectiveContext::Component { .. } => qualify_expr(expr, "inst"),
    }
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
        let ctx = DirectiveContext::Page {
            module_alias: "PageModule0".to_string(),
            release: false,
        };
        let mut buf = TsBuffer::new();
        generate_if_helpers(&site, &ctx, &mut buf);
        generate_if_wiring(&site, &ctx, &mut buf);
        assert!(buf
            .build()
            .contains("effect(() => __toggle_index_if_0(ctx))"));
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
        let ctx = DirectiveContext::Page {
            module_alias: "PageModule0".to_string(),
            release: false,
        };
        let mut buf = TsBuffer::new();
        generate_if_helpers(&site, &ctx, &mut buf);
        generate_if_wiring(&site, &ctx, &mut buf);
        assert!(buf.build().contains("if (!wrapper) return;"));
    }
}
