# TinyFX Agent Guide

## What This Project Is

TinyFX is a hybrid framework with a Rust CLI compiler and a TypeScript runtime. The compiler expands HTML components, resolves pages into real output files, generates the runtime glue, and runs the asset pipeline. The runtime adds only the browser behavior that cannot be done ahead of time, such as signals, DOM bindings, route param helpers, lifecycle hooks, and a typed HTTP client.

TinyFX is for projects that prefer explicit filesystem structure, server-friendly HTML output, and direct DOM work over heavy client-side abstractions. It is a better fit for multi-page applications and server-rendered frontends than for SPA-style client routers or large framework ecosystems.

## Repository Structure

```text
tinyfx/
├── AGENTS.md                              # Root instructions for agents working in this repository
├── README.md                              # Front-door documentation for TinyFX users
├── tinyfx.toml                            # Workspace config file used during repo development
├── tsconfig.json                          # Workspace TypeScript config
├── cli/                                   # Reserved top-level CLI area; currently not the main compiler implementation
├── compiler/                              # Rust CLI crate that builds and serves TinyFX apps
│   ├── Cargo.toml                         # Compiler crate manifest
│   ├── src/
│   │   ├── REFACTOR_AUDIT.md              # Audit notes from the compiler refactor
│   │   ├── main.rs                        # CLI entry point and subcommand dispatch
│   │   ├── config.rs                      # tinyfx.toml parsing and config defaults
│   │   ├── init.rs                        # Project scaffolding for `tinyfx init`
│   │   ├── generate.rs                    # `tinyfx generate *` implementation and naming normalization
│   │   ├── build/
│   │   │   ├── mod.rs                     # Build pipeline orchestrator and Phase 4 helpers
│   │   │   ├── dependency_graph.rs        # Phase 0 graph construction and cycle detection
│   │   │   ├── component.rs               # Phase 1 component compilation and expansion
│   │   │   ├── pages.rs                   # Phase 2 page parsing, layout wrapping, and output writing
│   │   │   └── glue.rs                    # Phase 3 `tinyfx.gen.ts` generation
│   │   └── serve/
│   │       ├── mod.rs                     # Serve subsystem entry point
│   │       ├── coordinator.rs             # Initial build + async server/watch orchestration
│   │       ├── server.rs                  # HTTP server for built output
│   │       ├── watcher.rs                 # File watcher and rebuild trigger
│   │       └── websocket.rs               # Live-reload WebSocket broadcaster
│   ├── templates/                         # Source templates used by `tinyfx init` and `tinyfx generate`
│   │   ├── AGENTS.md                      # App-level agent instructions copied into new projects
│   │   ├── package.json                   # Scaffolded npm package manifest
│   │   ├── tinyfx.toml                    # Scaffolded TinyFX config template
│   │   ├── tsconfig.json                  # Template TS config used by generated code patterns
│   │   ├── tsconfig.app.json              # App-level TS config used in scaffolded apps
│   │   ├── main.ts                        # Scaffolded app entry file
│   │   ├── pages/
│   │   │   └── index.html                 # Scaffolded starter page
│   │   ├── components/
│   │   │   └── Button/
│   │   │       ├── Button.html            # Scaffolded starter component template
│   │   │       └── Button.ts              # Scaffolded example behavior file
│   │   ├── lib/
│   │   │   ├── dto/                       # Empty DTO folder scaffolded into new apps
│   │   │   ├── models/                    # Empty models folder scaffolded into new apps
│   │   │   ├── services/                  # Empty services folder scaffolded into new apps
│   │   │   └── state/                     # Empty state folder scaffolded into new apps
│   │   ├── styles/
│   │   │   └── main.css                   # Scaffolded stylesheet copied to dist/css
│   │   └── generate/
│   │       ├── component/                 # Templates for `tinyfx generate component`
│   │       ├── dto/                       # Templates for `tinyfx generate dto`
│   │       ├── interface/                 # Templates for `tinyfx generate interface`
│   │       ├── model/                     # Templates for `tinyfx generate model`
│   │       ├── page/                      # Templates for `tinyfx generate page`
│   │       └── service/                   # Templates for `tinyfx generate service`
│   └── target/                            # Cargo build artifacts; not source of truth
├── runtime/                               # `@tinyfx/runtime` TypeScript runtime package
│   ├── package.json                       # Runtime package manifest
│   ├── README.md                          # Runtime package readme
│   ├── tsconfig.json                      # Runtime TypeScript config
│   └── src/
│       ├── index.ts                       # Public barrel for runtime consumers
│       ├── context.ts                     # `TinyFxContext` type
│       ├── signals.ts                     # Signal and effect primitives
│       ├── dom.ts                         # DOM binding helpers
│       ├── http.ts                        # Public HTTP exports
│       ├── http/
│       │   ├── data.ts                    # HTTP error types and option interfaces
│       │   ├── helper.ts                  # URL building and retry-delay helpers
│       │   └── http.ts                    # `createHttp()` implementation
│       └── router/
│           ├── index.ts                   # Runtime router entry point
│           ├── types.ts                   # Route map types emitted by the compiler
│           ├── params.ts                  # Route param resolution and lookup
│           ├── navigate.ts                # Hard-navigation helpers
│           ├── lifecycle.ts               # `onMount` and `onDestroy` hooks
│           └── active-links.ts            # `data-active` highlighting for matching anchors
├── docs/                                  # Secondary markdown docs; verify against source before trusting them
├── examples/                              # Example projects and generator fixtures
│   ├── basic/                             # Basic example app skeleton
│   └── generators/                        # Snapshot-style generator output examples
└── templates/                             # Additional workspace-level templates and samples
```

## Build Pipeline

| Phase | What it reads | What it produces | Owning file |
|---|---|---|---|
| 0. Dependency graph | `src/components/**/*.html`, `src/pages/**/*.html` | A `DependencyGraph` with component nodes, page nodes, and dependency edges | `compiler/src/build/dependency_graph.rs` |
| 1. Component compilation | The graph plus component templates in dependency order | A `HashMap<String, CompiledComponent>` with expanded component HTML and dynamic/static flags | `compiler/src/build/component.rs` |
| 2. Page processing | Page HTML files plus the compiled component registry | Resolved page HTML and output paths under `dist/` | `compiler/src/build/pages.rs` |
| 3. Glue generation | The component registry and resolved pages | `src/tinyfx.gen.ts` with route wiring and dynamic component mounts | `compiler/src/build/glue.rs` |
| 4. Asset pipeline | `src/main.ts`, generated glue, styles directory, assets/static dirs | `dist/js/main.js`, copied CSS, and copied assets/static content | `compiler/src/build/mod.rs` |

If you need to change how components are discovered or expanded, edit `compiler/src/build/component.rs`. If you need to change route parsing, page layout wrapping, or output paths, edit `compiler/src/build/pages.rs`. If you need to change route map or component mounting code generation, edit `compiler/src/build/glue.rs`. If you need to change esbuild invocation, output directory preparation, or copying behavior, edit `compiler/src/build/mod.rs`.

## Key Data Structures

### `Config`

`Config` is the compiler’s view of `tinyfx.toml`. It lives in `compiler/src/config.rs` and controls which directories and entry files the build and serve commands use.

```rust
pub struct Config {
		pub entry: String,
		pub entry_ts: String,
		pub out_dir: String,
		pub components: String,
		pub styles: String,
		pub static_dirs: Vec<String>,
		pub assets_dir: String,
		pub pages: String,
		pub phase_one_only: bool,
}
```

### `DependencyGraph`

`DependencyGraph` lives in `compiler/src/build/dependency_graph.rs`. It stores graph edges in both directions and keeps separate metadata maps for components and pages. Build code queries it for cycle detection and topological order.

```rust
pub struct DependencyGraph {
		pub dependencies: HashMap<PathBuf, HashSet<PathBuf>>,
		pub dependents: HashMap<PathBuf, HashSet<PathBuf>>,
		pub components: HashMap<PathBuf, ComponentMeta>,
		pub pages: HashMap<PathBuf, PageMeta>,
}
```

### `CompiledComponent`

`CompiledComponent` lives in `compiler/src/build/component.rs`. It represents a component after its nested dependencies have already been expanded. Pages and glue generation both consume this struct.

```rust
pub struct CompiledComponent {
		pub name: String,
		pub path: PathBuf,
		pub html: String,
		pub is_dynamic: bool,
}
```

### `ResolvedPage`

The current code names this struct `PageDef` in `compiler/src/build/pages.rs`. It is the resolved page artifact produced by Phase 2 and consumed by Phase 3.

```rust
pub struct PageDef {
		pub name: String,
		pub route: String,
		pub html: String,
		pub source_path: PathBuf,
		pub out_path: PathBuf,
}
```

## Conventions — Things an Agent Must Never Violate

NEVER create barrel `index.ts` files in `src/lib/` subfolders.

NEVER use `@tinyfx/runtime/di` or any other nonexistent runtime DI path; import `TinyFxContext` from `@tinyfx/runtime` directly.

NEVER add `.unwrap()` calls in Rust outside `OnceLock::get_or_init` regex initialization blocks.

NEVER use `moduleResolution: "node"` in TinyFX TypeScript configs; use `"bundler"`.

NEVER edit `src/tinyfx.gen.ts`; it is auto-generated and overwritten on every build.

NEVER use backreference syntax such as `\1` or `\2` in Rust regexes; the `regex` crate does not support it.

NEVER treat a `.ts` file alone as enough to make a component dynamic.

NEVER assume the scaffolded `[server]` or `[router]` sections in `tinyfx.toml` are active config; the current compiler ignores them.

NEVER use relative imports for files under `src/lib/`; use the configured aliases.

ALWAYS use path aliases (`@models/*`, `@dto/*`, `@state/*`, `@services/*`) for lib imports.

ALWAYS use PascalCase for component names, component folders, and component filenames.

ALWAYS use kebab-case for page filenames generated into `src/pages/`.

ALWAYS add `tinyfx-dynamic` to the component HTML and create a matching `.ts` file when behavior should mount in the browser.

ALWAYS keep service code in `src/lib/services/` as factory functions, not framework-managed classes.

ALWAYS treat `dist/`, `target/`, and `node_modules/` as generated output, not source of truth.

## Import Aliases

| Alias | Folder | What belongs there |
|---|---|---|
| `@models/*` | `src/lib/models/*` | Frontend model classes built from DTOs |
| `@dto/*` | `src/lib/dto/*` | Raw API response types |
| `@state/*` | `src/lib/state/*` | Shared signals and app state |
| `@services/*` | `src/lib/services/*` | Service factories and business logic |

Correct imports:

```ts
import { Post } from "@models/post.model";
import type { PostDto } from "@dto/post.dto";
import { cartCount } from "@state/cart.state";
import { createPostService } from "@services/post.service";
```

Incorrect imports:

```ts
import { Post } from "../lib/models/post.model";
import { createPostService } from "./lib/services/post.service";
import * as Models from "@models";
```

## Component Conventions

Static components are HTML-only and expand entirely at build time. The current generator creates only the `.html` file unless `--dynamic` is passed.

```html
<!-- src/components/Badge/Badge.html -->
<section class="badge" data-component="Badge" tinyfx-static>
	<h2>Badge</h2>
	<slot></slot>
</section>
```

Dynamic components need the HTML marker and a same-name `.ts` file. The compiler expands the HTML at build time and the generated glue mounts the TypeScript behavior at runtime.

```html
<!-- src/components/Counter/Counter.html -->
<section class="counter" data-component="Counter" tinyfx-dynamic>
	<h2>Counter</h2>
	<slot></slot>
</section>
```

```ts
// src/components/Counter/Counter.ts
import { signal } from "@tinyfx/runtime";
import type { TinyFxContext } from "@tinyfx/runtime";

export class Counter {
	constructor(public el: HTMLElement, public ctx: TinyFxContext) {}

	init(): void {
		void signal;
	}

	destroy?(): void {}
}

export function mount(el: HTMLElement, ctx: TinyFxContext) {
	const inst = new Counter(el, ctx);
	inst.init();
	return inst;
}
```

## Page Conventions

Pages live in `src/pages/` and must use a `<page>` root element. Supported attributes today are `route` and `layout`. A sibling `.ts` file is optional and should export `init(el, ctx)`.

```html
<!-- src/pages/about.html -->
<page route="/about" layout="">
	<main class="about-page" data-page="About">
		<h1>About</h1>
		<p>Edit <code>src/pages/about.html</code> to customize this page.</p>
	</main>
</page>
```

```ts
// src/pages/about.ts
import { onMount, onDestroy } from "@tinyfx/runtime";
import type { TinyFxContext } from "@tinyfx/runtime";

export function init(el: HTMLElement, ctx: TinyFxContext): void {
	onMount(() => {
		console.log("About page mounted", el);
		void ctx;
	});

	onDestroy(() => {
		console.log("About page destroyed");
	});
}
```

## Service Conventions

Services are explicit factory functions in `src/lib/services/`. TinyFX does not use a runtime DI container. The service owns its `createHttp()` client and returns a plain object of methods.

```ts
// src/lib/services/post.service.ts
import { createHttp } from "@tinyfx/runtime";

const http = createHttp({ baseUrl: "/api" });

export function createPostService() {
	return {
		async getPost(slug: string) {
			return http.get(`/posts/${slug}`);
		},
	};
}

export type PostService = ReturnType<typeof createPostService>;
```

```ts
// src/components/PostCard/PostCard.ts
import { createPostService } from "@services/post.service";

const postService = createPostService();
void postService;
```

## Files an Agent Must Never Edit

`src/tinyfx.gen.ts` — auto-generated and overwritten on every build.

`dist/` — build output.

Any file in `node_modules/` — dependency code, not project source.

`compiler/templates/` — source templates; changes here affect every newly scaffolded project.

## CLI Generate Command Reference

`tinyfx generate component <Name> [--dynamic] [--force]`

Creates `src/components/<PascalName>/<PascalName>.html`. With `--dynamic`, also creates `src/components/<PascalName>/<PascalName>.ts`. Names are normalized to PascalCase.

`tinyfx generate page <name> [--force]`

Creates `src/pages/<kebab-name>.html` and `src/pages/<kebab-name>.ts`. Route defaults to `/<kebab-name>` except `index`, which becomes `/`.

`tinyfx generate service <Name> [--force]`

Creates `src/lib/services/<kebab-name>.service.ts`. A trailing `Service` in the input name is removed from the filename stem but preserved in the exported factory type naming.

`tinyfx generate model <Name> [--force]`

Creates `src/lib/models/<kebab-name>.model.ts`.

`tinyfx generate dto <Name> [--force]`

Creates `src/lib/dto/<kebab-name>.dto.ts`.

`tinyfx generate interface <Name> [--force]`

Creates `src/lib/<kebab-name>.interface.ts`.

All generate commands walk upward from the current directory until they find `tinyfx.toml`. If no TinyFX project root is found, the command fails.

## Common Mistakes

WRONG: Creating an `index.ts` barrel in `src/lib/models/`.
RIGHT: Import the file directly with `@models/post.model`.

WRONG: `import type { TinyFxContext } from "@tinyfx/runtime/di"`.
RIGHT: `import type { TinyFxContext } from "@tinyfx/runtime"`.

WRONG: Adding a `.ts` file to a component but forgetting `tinyfx-dynamic` in the HTML.
RIGHT: Add `tinyfx-dynamic` to the component HTML and keep the matching `.ts` file.

WRONG: Using relative imports like `../lib/services/post.service`.
RIGHT: Use `@services/post.service`.

WRONG: Treating `tinyfx.toml [router]` as a live runtime config section.
RIGHT: Assume only the fields in `compiler/src/config.rs` are active today.

WRONG: Editing `src/tinyfx.gen.ts` to add custom runtime code.
RIGHT: Put custom code in `src/main.ts`, page `.ts` files, or component `.ts` files.

WRONG: Defining reusable business logic directly inside a page `init()` function.
RIGHT: Put reusable logic in `src/lib/services/`, `src/lib/models/`, or `src/lib/state/` and call it from the page.

WRONG: Querying global DOM with `document.getElementById()` inside a component.
RIGHT: Scope DOM work to `this.el.querySelector()` so the component only touches its own subtree.

WRONG: Assuming `tinyfx serve` is a separate app server from the build pipeline.
RIGHT: Remember that `serve` performs a build first and rebuilds on file changes.

WRONG: Using snake_case or kebab-case for component folder names.
RIGHT: Use PascalCase for component folders and filenames.
