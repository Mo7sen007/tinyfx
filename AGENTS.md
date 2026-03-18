# TinyFX Agent Guide

## What This Project Is
TinyFX is a hybrid framework with two implemented parts:
- Rust CLI compiler: scans files, builds dependency graphs, compiles components, processes pages and directives, generates `tinyfx.gen.ts`, and runs bundling and copy phases.
- TypeScript runtime (`@tinyfx/runtime`): provides explicit reactivity and browser helpers used by generated glue and app code.

TinyFX targets HTML-first multi-page applications where compile-time transformation handles most work and runtime code mounts focused behavior.

## Repository Structure

```text
tinyfx/
├── AGENTS.md
├── README.md
├── tinyfx.toml
├── compiler/
│   ├── src/
│   │   ├── main.rs
│   │   ├── config.rs
│   │   ├── init.rs
│   │   ├── generate.rs
│   │   ├── build/
│   │   │   ├── mod.rs
│   │   │   ├── dependency_graph.rs
│   │   │   ├── component.rs
│   │   │   ├── pages.rs
│   │   │   ├── each.rs
│   │   │   ├── conditional.rs
│   │   │   ├── each_codegen.rs
│   │   │   ├── conditional_codegen.rs
│   │   │   ├── wrapper.rs
│   │   │   └── glue.rs
│   │   └── serve/
│   └── templates/
├── runtime/
│   └── src/
│       ├── index.ts
│       ├── signals.ts
│       ├── dom.ts
│       ├── each.ts
│       ├── http.ts
│       └── router/
├── docs/
└── examples/
```

## Build Pipeline

| Phase | Description | Owning file |
|---|---|---|
| 0 | Build dependency graph from component/page templates | `compiler/src/build/dependency_graph.rs` |
| 1 | Compile component templates, expand nested usage, detect dynamic, parse component directive sites | `compiler/src/build/component.rs` |
| 2 | Parse and resolve pages, process page directives, write output HTML | `compiler/src/build/pages.rs` |
| 3 | Generate `src/tinyfx.gen.ts` (imports, directive helpers, wrappers, registry, routes, bootstrap) | `compiler/src/build/glue.rs` |
| 4 | Bundle TypeScript and copy static/style/assets output | `compiler/src/build/mod.rs` |

## Runtime Exports
From `runtime/src/index.ts`:
- `signal`, `effect`
- `registerInstance`, `destroyNode`
- `bindText`, `bindAttr`, `bindClass`
- `createHttp`
- `initRouter`, `getParam`, `navigate`, `goBack`
- `onMount`, `onDestroy`
- `TinyFxContext`
- `RouteMap`, `RouteMeta`

## Static and Dynamic Components

### Static Components
- Static is the default behavior.
- Static components are expanded at build time.
- Static components have no runtime mount behavior.

### Dynamic Components
- Dynamic requires both:
  - `tinyfx-dynamic` in component HTML
  - sibling same-name `.ts` file exporting `mount(el, ctx)` or `init(el, ctx)` shape compatible with generated glue
- Compiler enforcement:
  - `tinyfx-dynamic` without `.ts`: build error
  - `.ts` without `tinyfx-dynamic`: build warning

### Required Agent Rules
- NEVER add `tinyfx-static` to component markup in new code.
- NEVER create a static component with a `.ts` file.
- ALWAYS include `tinyfx-dynamic` when runtime behavior is required.
- ALWAYS create matching `.ts` file for dynamic components.

## Component Wrapper System
The compiler generates `__mount_ComponentName` wrappers for dynamic components in `tinyfx.gen.ts`.

Wrapper behavior:
1. Calls `mountComponent(<Component>Module, el, ctx)`.
2. Checks `if (!inst) return null;`.
3. Wires component directive sites (`<each>`, `<if>`) against instance fields.
4. Returns the mounted instance.

Critical scoping rule:
- Wrappers query directive containers via `el.querySelector(...)` so each component instance wires its own subtree.

Agent rule:
- NEVER design component directive wiring around `document.querySelector` for component scope.

## Directive Rules

### `<each>`
- Supported `of` forms: signal call (`items()`), plain expression (`items`), integer (`10`), range (`1..5`).
- `as` is required.
- `key` is required for non-range collection cases.
- Compiler emits `data-each` wrapper container and site-specific `__create_`, `__update_`, `__reconcile_` functions.

Agent rules:
- ALWAYS set `key` to an existing field from real data.
- NEVER invent a synthetic key field that does not exist.
- For backend records, use backend identity fields (`id`, `uuid`, and similar real fields).

Layout note:
- `data-each` wrapper can affect flex/grid layout.
- Use `display: contents` on the generated wrapper selector when needed.

### `<if>`
- Supports `<if>`, `<else-if>`, `<else>` chains.
- Compiler pre-renders branches and runtime toggles visibility.

## Public Signal Requirement in Component Templates
Signals referenced by component template directives must be `public` on the component class.

Example:
- `<each of="messages()" ...>` in component HTML requires `public messages = signal(...)` in the class.

Agent rules:
- NEVER use `private` for component fields referenced from component template directives.
- ALWAYS use `public` for directive-bound instance fields.

## Lib Folder and Alias Rules

```text
src/lib/
├── models/
├── dto/
├── state/
└── services/
```

Alias requirements:
- `@models/*` -> `src/lib/models/*`
- `@dto/*` -> `src/lib/dto/*`
- `@state/*` -> `src/lib/state/*`
- `@services/*` -> `src/lib/services/*`

Agent rules:
- NEVER create barrel `index.ts` files in `src/lib/*` subfolders.
- ALWAYS import concrete files via alias paths.
- NEVER use relative imports to reach `src/lib/*` from components/pages.

## Service Pattern
Services live in `src/lib/services/` as factory functions.

Agent rules:
- ALWAYS implement services as factory functions.
- NEVER use runtime DI container patterns.
- NEVER use global singleton service objects as default architecture.

## Files Agents Must Not Edit
- `src/tinyfx.gen.ts` (generated, overwritten each build)
- `dist/` (build output)
- `target/` (Rust build output)
- `node_modules/` (dependency output)

## Common Mistakes
WRONG: `tinyfx-static` on a component element
RIGHT: static is default behavior with no static marker required

WRONG: `private messages = signal([])` when used in `<each of="messages()">`
RIGHT: `public messages = signal([])` for directive access

WRONG: `<each key="i">` when items do not have `i`
RIGHT: `<each key="id">` (or another real field from data)

WRONG: `document.querySelector(...)` inside component class logic
RIGHT: `this.el.querySelector(...)` scoped to component root

WRONG: Creating client-side IDs before backend write when backend owns identity
RIGHT: Send backend-required fields and consume backend-provided IDs

WRONG: Adding `.ts` file to a static component
RIGHT: Keep static component HTML-only; use dynamic marker and sibling `.ts` only when behavior is required

WRONG: Importing `TinyFxContext` from nonexistent DI paths
RIGHT: `import type { TinyFxContext } from "@tinyfx/runtime"`

## CLI Generate Naming Rules
- Component names normalize to PascalCase.
- Generated page filenames normalize to kebab-case.
- Service names normalize to kebab-case with `.service.ts` suffix.
- `generate` command alias `g` is available.

## Working Discipline for Agents
- Prefer implementation details from source files over secondary docs.
- Describe only implemented behavior as implemented.
- Mark planned features as planned.
- Keep code suggestions aligned with current runtime and compiler exports.
