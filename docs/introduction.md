# Introduction

TinyFX is a hybrid web framework with two implemented parts:
- Rust compiler CLI: handles component and page processing at build time.
- TypeScript runtime (`@tinyfx/runtime`): provides browser-side primitives for reactivity, DOM updates, navigation helpers, and HTTP.

## Build-Time First Model
The compiler performs most of the work before the browser runs application code:
1. Build dependency graph from `src/components/` and `src/pages/`.
2. Compile components in dependency order.
3. Parse and write resolved pages.
4. Generate `src/tinyfx.gen.ts` glue.
5. Run esbuild and copy styles/assets.

## Runtime Scope
The runtime exports:
- `signal`, `effect`
- `registerInstance`, `destroyNode`
- `bindText`, `bindAttr`, `bindClass`
- `createHttp`
- `getParam`, `getParams`, `navigate`, `goBack`
- `onMount`, `onDestroy`
- `TinyFxContext`
- `RouteMap`, `RouteMeta`
- `registerComponent`, `getComponentFactory`, `mountComponents`
- `registerPage`, `getPageModule`, `runPageInit`
- `markMounted`, `isMounted`
- `matchPath`, `splitPath`, `RouteDef`
- `init`

## What TinyFX Is Optimized For
- HTML-first multi-page applications.
- Compile-time expansion of component templates.
- Explicit state and DOM wiring.
- Opt-in runtime behavior through dynamic components.

Soft routing is planned and not currently implemented.
