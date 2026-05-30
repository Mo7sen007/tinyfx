//! # wrapper
//!
//! Generates per-component wrapper functions for `tinyfx.gen.ts`.
//!
//! ## Responsibilities
//! - Emit `__mount_ComponentName` functions that instantiate components
//!   and wire their directive sites to live instance signals
//! - Handle components with no directives (minimal wrapper)
//! - Handle components with `<each>` directives
//! - Handle components with `<if>` directives
//! - Handle components with both
//!
//! ## What this module does NOT do
//! - It does not generate reconciler functions (each_codegen.rs owns that)
//! - It does not generate toggle functions (conditional_codegen.rs owns that)
//! - It does not write files (glue.rs owns that)
//! - It does not scan components or pages (build phases own that)

use super::codegen::{LoopSource, TsBuffer};
use super::conditional::IfSite;
use super::each::EachSite;

/// Everything needed to generate a wrapper for one dynamic component.
#[derive(Debug, Clone, Default)]
pub struct ComponentWrapperSpec {
    /// PascalCase component name — e.g. "Chat"
    pub name: String,
    /// `<each>` sites found in this component's HTML, in source order
    pub each_sites: Vec<EachSite>,
    /// `<if>` sites found in this component's HTML, in source order
    pub if_sites: Vec<IfSite>,
}

/// Generate the `__mount_ComponentName` function for one dynamic component.
pub fn generate_component_wrapper(spec: &ComponentWrapperSpec, buf: &mut TsBuffer) {
    let fn_name = format!("__mount_{}", spec.name);

    buf.line(&format!(
        "function {}(el: HTMLElement, ctx: TinyFxContext): any {{",
        fn_name
    ));
    buf.indent();
    buf.line(&format!(
        "const inst = mountComponent({}Module, el, ctx);",
        spec.name
    ));
    buf.line("if (!inst) return null;");

    if !spec.each_sites.is_empty() || !spec.if_sites.is_empty() {
        buf.blank();
        generate_each_wirings(&spec.each_sites, buf);
        generate_if_wirings(&spec.if_sites, buf);
    }

    buf.blank();
    buf.line("return inst;");
    buf.dedent();
    buf.line("}");
    buf.blank();
}

fn generate_each_wirings(sites: &[EachSite], buf: &mut TsBuffer) {
    for site in sites {
        generate_each_wiring(site, buf);
    }
}

fn generate_each_wiring(site: &EachSite, buf: &mut TsBuffer) {
    buf.line(&format!(
        "// <each> of=\"{}\" - wired to inst.{}",
        source_expr(site),
        source_to_property(site)
    ));
    // Per-instance node map so multiple component instances do not share state.
    buf.line(&format!(
        "const __nodes_{} = new Map<string, NodeRecord_{}>();",
        site.id, site.id
    ));
    // Use el.querySelector so every mounted component instance wires its own
    // directive container instead of capturing another instance on the page.
    buf.line(&format!(
        "const __each_{} = el.querySelector<HTMLElement>(\"[data-each='{}']\");",
        site.id, site.id
    ));
    buf.line(&format!("if (__each_{}) {{", site.id));
    buf.indent();

    match &site.source {
        LoopSource::Signal(name) => {
            buf.line(&format!(
                "effect(() => __reconcile_{}(__each_{}, inst.{}(), ctx, __nodes_{}));",
                site.id, site.id, name, site.id
            ));
        }
        LoopSource::Plain(expr) => {
            buf.line(&format!(
                "__mount_static_{}(__each_{}, inst.{}, ctx);",
                site.id,
                site.id,
                expr.trim()
            ));
        }
        LoopSource::Range(start, end) => {
            let len = end.saturating_sub(*start);
            buf.line(&format!(
                "const __items_{} = Array.from({{ length: {} }}, (_, __idx) => __idx + {});",
                site.id, len, start
            ));
            buf.line(&format!(
                "__mount_static_{}(__each_{}, __items_{}, ctx);",
                site.id, site.id, site.id
            ));
        }
    }

    buf.dedent();
    buf.line("}");
    buf.blank();
}

fn generate_if_wirings(sites: &[IfSite], buf: &mut TsBuffer) {
    for site in sites {
        generate_if_wiring(site, buf);
    }
}

fn generate_if_wiring(site: &IfSite, buf: &mut TsBuffer) {
    if let Some(first_condition) = site.branches.first().and_then(|b| b.condition.as_ref()) {
        buf.line(&format!(
            "// <if> condition=\"{}\" - wired to inst",
            first_condition
        ));
    }

    let has_signal = site.branches.iter().any(|branch| {
        branch
            .condition
            .as_ref()
            .is_some_and(|condition| condition.trim().ends_with("()"))
    });

    if has_signal {
        buf.line(&format!("effect(() => __toggle_{}(inst));", site.id));
    } else {
        buf.line(&format!("__toggle_{}(inst);", site.id));
    }
    buf.blank();
}

fn source_expr(site: &EachSite) -> String {
    match &site.source {
        LoopSource::Signal(name) => format!("{}()", name),
        LoopSource::Plain(expr) => expr.trim().to_string(),
        LoopSource::Range(start, end) => format!("{}..{}", start, end),
    }
}

fn source_to_property(site: &EachSite) -> String {
    match &site.source {
        LoopSource::Signal(name) => name.to_string(),
        LoopSource::Plain(expr) => expr.trim().to_string(),
        LoopSource::Range(_, _) => "<range>".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build::conditional::IfBranch;
    use crate::build::each::KeyStrategy;

    fn emit(spec: ComponentWrapperSpec) -> String {
        let mut buf = TsBuffer::new();
        generate_component_wrapper(&spec, &mut buf);
        buf.build()
    }

    #[test]
    fn minimal_wrapper_no_directives() {
        let code = emit(ComponentWrapperSpec {
            name: "ThemeToggle".to_string(),
            each_sites: Vec::new(),
            if_sites: Vec::new(),
        });

        assert!(code.contains("function __mount_ThemeToggle"));
        assert!(code.contains("const inst = mountComponent(ThemeToggleModule, el, ctx);"));
        assert!(code.contains("if (!inst) return null;"));
        assert!(code.contains("return inst;"));
    }

    #[test]
    fn wrapper_with_single_each_signal() {
        let code = emit(ComponentWrapperSpec {
            name: "Chat".to_string(),
            each_sites: vec![EachSite {
                id: "chat_each_0".to_string(),
                source: LoopSource::Signal("messages".to_string()),
                item_var: "message".to_string(),
                key_strategy: KeyStrategy::Keyed {
                    field_name: "id".to_string(),
                },
                template_html: "<div></div>".to_string(),
                bindings: Vec::new(),

                dynamic_component_names: Vec::new(),
            }],
            if_sites: Vec::new(),
        });

        assert!(
            code.contains("const __nodes_chat_each_0 = new Map<string, NodeRecord_chat_each_0>();")
        );
        assert!(code.contains(
            "effect(() => __reconcile_chat_each_0(__each_chat_each_0, inst.messages(), ctx, __nodes_chat_each_0));"
        ));
        assert!(!code.contains("__reconcile_chat_each_0(__each_chat_each_0, messages()"));
    }

    #[test]
    fn wrapper_with_single_each_plain() {
        let code = emit(ComponentWrapperSpec {
            name: "Chat".to_string(),
            each_sites: vec![EachSite {
                id: "chat_each_0".to_string(),
                source: LoopSource::Plain("items".to_string()),
                item_var: "item".to_string(),
                key_strategy: KeyStrategy::Keyed {
                    field_name: "id".to_string(),
                },
                template_html: "<div></div>".to_string(),
                bindings: Vec::new(),

                dynamic_component_names: Vec::new(),
            }],
            if_sites: Vec::new(),
        });

        assert!(code.contains("__mount_static_chat_each_0(__each_chat_each_0, inst.items, ctx);"));
        assert!(!code.contains("effect(() => __mount_static_chat_each_0"));
    }

    #[test]
    fn wrapper_with_if_signal() {
        let code = emit(ComponentWrapperSpec {
            name: "Chat".to_string(),
            each_sites: Vec::new(),
            if_sites: vec![IfSite {
                id: "chat_if_0".to_string(),
                branches: vec![IfBranch {
                    condition: Some("isOnline()".to_string()),
                    is_signal: true,
                    html: "<div></div>".to_string(),
                    dynamic_component_names: Vec::new(),
                }],
            }],
        });

        assert!(code.contains("effect(() => __toggle_chat_if_0(inst));"));
    }

    #[test]
    fn wrapper_with_mixed_directives() {
        let code = emit(ComponentWrapperSpec {
            name: "Chat".to_string(),
            each_sites: vec![EachSite {
                id: "chat_each_0".to_string(),
                source: LoopSource::Signal("messages".to_string()),
                item_var: "message".to_string(),
                key_strategy: KeyStrategy::Keyed {
                    field_name: "id".to_string(),
                },
                template_html: "<div></div>".to_string(),
                bindings: Vec::new(),

                dynamic_component_names: Vec::new(),
            }],
            if_sites: vec![IfSite {
                id: "chat_if_0".to_string(),
                branches: vec![IfBranch {
                    condition: Some("isOnline()".to_string()),
                    is_signal: true,
                    html: "<div></div>".to_string(),
                    dynamic_component_names: Vec::new(),
                }],
            }],
        });

        let each_idx = code
            .find("__reconcile_chat_each_0")
            .expect("each wiring should exist");
        let if_idx = code
            .find("__toggle_chat_if_0")
            .expect("if wiring should exist");
        assert!(each_idx < if_idx);
    }

    #[test]
    fn container_lookup_scoped_to_el() {
        let code = emit(ComponentWrapperSpec {
            name: "Chat".to_string(),
            each_sites: vec![EachSite {
                id: "chat_each_0".to_string(),
                source: LoopSource::Signal("messages".to_string()),
                item_var: "message".to_string(),
                key_strategy: KeyStrategy::Keyed {
                    field_name: "id".to_string(),
                },
                template_html: "<div></div>".to_string(),
                bindings: Vec::new(),

                dynamic_component_names: Vec::new(),
            }],
            if_sites: Vec::new(),
        });

        assert!(code.contains("el.querySelector<HTMLElement>"));
        assert!(!code.contains("document.querySelector<HTMLElement>"));
    }

    #[test]
    fn instance_null_check_present() {
        let code = emit(ComponentWrapperSpec {
            name: "Chat".to_string(),
            each_sites: Vec::new(),
            if_sites: Vec::new(),
        });
        assert!(code.contains("if (!inst) return null;"));
    }
}
