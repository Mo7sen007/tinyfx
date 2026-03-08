# tinyfx

> A minimal compile-time HTML component compiler + reactive TypeScript runtime for building fast, SSR-friendly web apps — without virtual DOMs or heavy frameworks.

`tinyfx` is a **hybrid system**:
- **Rust CLI** — compile-time HTML component expansion, asset bundling orchestration
- **npm runtime** (`@tinyfx/runtime`) — signals, DOM helpers, typed HTTP client

---

## Why tinyfx exists

Modern frontend frameworks solve many problems — but they also:
- add runtime overhead
- hide the DOM
- complicate SSR
- require learning and maintaining large abstractions

`tinyfx` takes a different approach:

- **HTML is the source of truth**
- **Most work happens at compile time**
- **JavaScript enhances, not replaces, SSR**
- **Explicit code over magic**

If you like Go templates, direct DOM access, and TypeScript safety, this is for you.

---

## Documentation

Comprehensive documentation is available in the [docs/](docs) folder:

- [**Introduction**](docs/introduction.md) - Core concepts and architecture.
- [**Getting Started**](docs/getting-started.md) - Installation and basic workflow.
- [**Components**](docs/components.md) - Templates, slots, and behavior logic.
- [**Signals & Reactivity**](docs/signals.md) - Managing state with minimal overhead.
- [**DOM Bindings**](docs/dom-helpers.md) - Helpers for syncing state to the UI.
- [**HTTP Client**](docs/http-client.md) - Making type-safe requests.
- [**Context & Services**](docs/dependency-injection.md) - Passing route context and creating services explicitly.
- [**CLI Reference**](docs/cli.md) - Command reference and configuration.

---

## Requirements

- **Rust** (for the CLI)
- **Node.js + npm** (for esbuild and the runtime package)

---

## Getting Started

### 1. Install the CLI

```sh
cargo install --path compiler
```

This gives you the `tinyfx` command globally.

### 2. Scaffold a new project

```sh
tinyfx init my-app
cd my-app
```

This creates a standalone npm project:

```
my-app/
├─ src/
│  ├─ pages/
│  │  └─ index.html
│  ├─ main.ts
│  ├─ components/
│  │  └─ Button/
│  │     ├─ Button.html
│  │     └─ Button.ts
│  ├─ lib/
│  │  ├─ dto/
│  │  ├─ models/
│  │  ├─ services/
│  │  └─ state/
│  └─ styles/
│     └─ main.css
├─ package.json          # @tinyfx/runtime + esbuild
├─ AGENTS.md
├─ tsconfig.json
└─ tinyfx.toml
```

Dependencies are installed automatically (`npm install`).

### 3. Build for production

```sh
tinyfx build
```

Output:

```
dist/
├─ index.html    # root page output
├─ about/
│  └─ index.html # nested page output example
├─ js/main.js    # bundled + minified JS
└─ css/main.css  # copied styles
```

---

## Configuration

`tinyfx.toml` in your project root:

```toml
pages      = "src/pages"
entry_ts   = "src/main.ts"
out_dir    = "dist"
components = "src/components"
styles     = "src/styles"
```

---

## Architecture

### Build pipeline

```
tinyfx build
│
├─ Phase 1: Component compilation
│  └─ Discover component templates in src/components/
├─ Phase 2: Page processing
│  ├─ Scan src/pages/**/*.html
│  ├─ Expand component tags in each page
│  └─ Write routed files to dist/
├─ Phase 3: Asset pipeline
│  ├─ esbuild: bundle src/main.ts → dist/js/main.js
│  └─ Copy styles/assets
└─ Phase 4: Glue generation
   └─ Generate src/tinyfx.gen.ts from discovered pages/components
```

The Rust CLI **never reimplements** JS/TS compilation — it delegates to esbuild.

---

## Runtime overview

The runtime is a normal npm package: `@tinyfx/runtime`

### Signals (reactivity)

Minimal, explicit reactivity — no proxies, no magic.

```ts
import { signal, effect } from "@tinyfx/runtime";

const count = signal(0);

effect(() => {
  console.log(count());
});

count.set(count() + 1);
```

Signals update only what you touch — no re-renders.

### DOM helpers

Helpers for binding state to the DOM without hiding it.

```ts
import { bindText } from "@tinyfx/runtime";

bindText(span, count);
```

HTML stays HTML.
JS stays JS.

### HTTP client (typed, explicit)

A thin, typed wrapper over fetch.

```ts
import { createHttp } from "@tinyfx/runtime";

const api = createHttp({ baseUrl: "/api" });

const user = await api.get<User>("/users/1");
```

#### Features

- typed responses
- centralized error handling
- configurable headers
- GET, POST, PUT, DELETE

Also supports timeouts, retries, query params, and request/response interceptors.

### Context & services

TinyFX passes a lightweight `TinyFxContext` into component/page behavior:

```ts
import type { TinyFxContext } from "@tinyfx/runtime";

export function init(el: HTMLElement, ctx: TinyFxContext) {
  console.log(ctx.params);
  ctx.navigate("/about");
}
```

Services are explicit factory functions (no runtime DI container):

```ts
import { createUserService } from "@services/user.service";

const userService = createUserService();
```

---

## Compile-time components

Components are a build-time concept, not a runtime one.

### Component definition

```
src/components/
  Button/
    Button.html
    Button.ts      # optional behavior
```

**Button.html**

```html
<button class="btn btn-{{variant}}">
  <slot></slot>
</button>
```

### Usage

```html
<component name="Button" variant="primary">
  Save
</component>
```

### Compiler output

```html
<button class="btn btn-primary" data-component="Button">
  Save
</button>
```

No runtime rendering. No virtual DOM. Just plain HTML.

### Component behavior (optional)

If a component needs behavior, it exports an `init` function:

```ts
export function init(el: HTMLElement) {
  el.addEventListener("click", () => {
    console.log("clicked");
  });
}
```

The compiler generates `tinyfx.gen.ts` — a small registry that initializes components on page load.

---

## Go SSR integration

tinyfx is designed to work with Go templates, not against them.

Typical flow:

1. Go renders HTML (templates)
2. `tinyfx build` processes components
3. Browser loads final HTML
4. Runtime enhances the DOM

Optional state hydration:

```html
<script id="__STATE__" type="application/json">
{{ .State | toJSON }}
</script>
```

No hydration framework required.

---

## Project structure (this repo)

```
tinyfx/
├─ compiler/          # Rust CLI (cargo install --path compiler)
│  └─ src/
│     ├─ main.rs      # CLI entry (init, build commands)
│     ├─ config.rs    # tinyfx.toml parser
│     ├─ init.rs      # Project scaffolder (embedded templates)
│     └─ build/
│        ├─ mod.rs        # Two-phase build orchestrator
│        ├─ component.rs  # HTML component resolver
│        └─ glue.rs       # tinyfx.gen.ts generator
├─ runtime/           # @tinyfx/runtime npm package
│  └─ src/
│     ├─ signals.ts   # Signal + effect
│     ├─ dom.ts       # bindText, bindAttr, bindClass
│     ├─ http.ts      # Typed HTTP client
│     └─ index.ts     # Barrel export
├─ examples/          # Template files (embedded into CLI at compile time)
└─ README.md
```

---

## Design principles

1. Prefer compile-time over runtime
2. Keep abstractions shallow
3. Optimize for readability and debuggability
4. Avoid cleverness
5. Keep the total codebase small enough to understand

If a feature makes the project feel like a framework — it probably doesn't belong.

---

## Status

This project is experimental and intentionally small.

The goal is not to compete with frameworks, but to:

- reduce frontend complexity
- support fast, SSR-friendly apps
- stay maintainable over time

## License

MIT
