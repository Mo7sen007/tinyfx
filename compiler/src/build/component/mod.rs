// ─────────────────────────────────────────────────────────────────────────────
// component/mod.rs — Module root (declarations + re-exports only)
// ─────────────────────────────────────────────────────────────────────────────
//
// The component module is split into three focused sub-modules following the
// "data vs logic" separation principle:
//
//   ┌──────────────┐   ┌──────────────┐   ┌──────────────┐
//   │   data.rs    │   │  parser.rs   │   │ renderer.rs  │
//   │              │   │              │   │              │
//   │  Component   │   │  Span detect │   │  resolve_all │
//   │  struct,     │◄──│  Attr parse  │◄──│  (orchestr.) │
//   │  template    │   │              │   │              │
//   │  load/render │   │              │   │              │
//   └──────────────┘   └──────────────┘   └──────────────┘
//
//   data.rs     — "What a component IS": struct, template I/O, rendering.
//   parser.rs   — "How to find one in HTML": span detection, attr parsing.
//   renderer.rs — "How to expand them all": iterative resolution loop.
//
// This file (mod.rs) declares those sub-modules and re-exports the public API
// so that the rest of the crate can simply use `component::resolve_all(…)`.
// ─────────────────────────────────────────────────────────────────────────────

// ── Sub-module declarations ──────────────────────────────────────────────────

/// Component data model — struct definition, template loading, and rendering.
pub mod data;

/// HTML parser — locates `<component>` tags and extracts their attributes.
pub mod parser;

/// Renderer / orchestrator — iterative component expansion loop.
mod renderer;

// ── Public re-exports ────────────────────────────────────────────────────────
//
// The only symbol the rest of the crate needs from this module is `resolve_all`.
// We re-export it at the `component::` level so callers can write:
//
//   component::resolve_all(&html, &components_dir, &mut used)?;
//
// without reaching into `component::renderer::resolve_all`.

pub(crate) use renderer::normalize_component_tags;
pub use renderer::resolve_all;
