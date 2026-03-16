# TinyFX

## What TinyFX Is

TinyFX is a hybrid web framework made of a Rust CLI compiler and a small TypeScript runtime. It is aimed at developers who like server-rendered HTML, direct DOM access, and explicit code paths, especially Go developers and developers who are tired of the amount of machinery that usually comes with frontend frameworks.

## Why It Exists

TinyFX exists for cases where the browser should enhance HTML instead of owning the whole UI. The compiler expands components and writes real HTML files at build time, and the runtime only handles the pieces that need browser behavior such as signals, DOM bindings, route params, and HTTP calls. The tradeoff is that TinyFX is not a fit for applications that need a fully client-side router, virtual DOM diffing, or a large ecosystem of framework-level abstractions. Today it is best suited to multi-page applications, server-rendered apps, and teams that want most complexity to stay visible in HTML, TypeScript, and the filesystem.

## How It Works

TinyFX has two parts. The Rust CLI scans your project, expands components, resolves pages into output files, generates a small glue file, and runs the asset pipeline. The TypeScript runtime in `@tinyfx/runtime` provides signals, effects, DOM helpers, route param helpers, page lifecycle hooks, navigation helpers, and a typed HTTP client.

When you run `tinyfx build`, Phase 0 scans `src/components/` and `src/pages/` to build a dependency graph so components can be compiled in a safe order and circular references fail early. Phase 1 reads component templates, expands nested component usage, and decides which components are static and which are dynamic. Phase 2 reads every page in `src/pages/`, parses the `<page>` root element, expands components in the page body, optionally wraps the page in a layout component, and writes real HTML files into `dist/`. Phase 3 generates `src/tinyfx.gen.ts`, which imports dynamic component modules and page modules and wires them to the runtime router. Phase 4 bundles `src/main.ts` and the generated glue file with esbuild, then copies styles and assets into `dist/`.

Static components are just HTML templates. They are expanded at build time and add no runtime behavior of their own. Dynamic components need two things: a `.ts` file and the `tinyfx-dynamic` marker in the component HTML. The compiler uses that combination to mount behavior on the generated DOM node. Pages live in `src/pages/` as `.html` files, and an optional sibling `.ts` file can export `init(el, ctx)` to run page-specific code.

## Getting Started

```sh
cargo install --path compiler
tinyfx init my-app
cd my-app
tinyfx serve
```

`tinyfx init` creates the project, writes the starter files, and runs `npm install` for you. `tinyfx serve` performs an initial build, starts an HTTP server on `http://127.0.0.1:3000`, starts a live-reload WebSocket server on port `3001`, watches the project source tree, and rebuilds when files change.

In the browser you will see a real HTML page rendered from `src/pages/index.html` with the heading “Welcome to TinyFX”, a short message telling you to edit the page, and the sample `Button` component expanded into plain HTML. In your editor you will see a small project with `src/pages/`, `src/components/`, `src/lib/`, `src/main.ts`, `tinyfx.toml`, and TypeScript config files. The compiler also generates `src/tinyfx.gen.ts` during builds; that file is owned by the compiler and should not be edited by hand.

## Project Structure

```text
my-app/
├── AGENTS.md                        # Local agent instructions scaffolded into each new app
├── package.json                     # npm scripts and runtime/build dependencies
├── tinyfx.toml                      # TinyFX compiler configuration
├── tsconfig.app.json                # App-level TypeScript config with src/* path aliases
├── tsconfig.json                    # Template TypeScript config used by generated code patterns
└── src/
    ├── main.ts                      # App-level entry file bundled into dist/js/main.js
    ├── pages/
    │   └── index.html               # Home page with a <page> root element
    ├── components/
    │   └── Button/
    │       ├── Button.html          # Sample component template used by the starter page
    │       └── Button.ts            # Sample behavior file; it will not mount until Button.html is marked tinyfx-dynamic
    ├── lib/
    │   ├── dto/                     # Raw API response types
    │   ├── models/                  # Frontend model classes built from DTOs
    │   ├── services/                # Factory-style business logic and HTTP wrappers
    │   └── state/                   # Shared signals and other client state
    └── styles/
        └── main.css                 # Styles copied to dist/css during builds
```

## Core Concepts

### Pages (`src/pages/`)

Each page is an HTML file with a `<page>` root element. The `route` attribute is optional because TinyFX can derive a route from the filename, and the `layout` attribute is optional because the page body can be wrapped in a component. If a sibling `.ts` file exists, the generated glue imports it and calls its `init()` function for the matching route.

```html
<!-- src/pages/blog-post.html -->
<page route="/blog/:slug" layout="MainLayout">
  <main class="blog-post-page" data-page="BlogPost">
    <h1>Blog post</h1>
    <p class="slug"></p>
  </main>
</page>
```

```ts
// src/pages/blog-post.ts
import { getParam, onMount } from "@tinyfx/runtime";
import type { TinyFxContext } from "@tinyfx/runtime";

export function init(el: HTMLElement, ctx: TinyFxContext): void {
  onMount(() => {
    const slugEl = el.querySelector(".slug");
    if (slugEl) {
      slugEl.textContent = `slug = ${getParam("slug") ?? "missing"}`;
    }
    void ctx;
  });
}
```

### Static Components

Static components are HTML-only templates. TinyFX expands them at build time, replaces `{{token}}` placeholders from attributes, and injects children into `<slot></slot>`. There is no runtime mount step unless you intentionally make the component dynamic.

```html
<!-- src/components/Badge/Badge.html -->
<span class="badge badge-{{tone}}">
  <slot></slot>
</span>
```

```html
<!-- src/pages/index.html -->
<page route="/">
  <main>
    <Badge tone="success">Saved</Badge>
  </main>
</page>
```

### Dynamic Components

Dynamic components are the components TinyFX mounts in the browser. The convention is to put `tinyfx-dynamic` on the root element of the component HTML and to create a same-name `.ts` file that exports `mount(el, ctx)`. The current compiler requires both. A `.ts` file by itself is not enough.

```html
<!-- src/components/Counter/Counter.html -->
<section class="counter" tinyfx-dynamic>
  <button class="decrement">-</button>
  <span class="value">0</span>
  <button class="increment">+</button>
</section>
```

```ts
// src/components/Counter/Counter.ts
import { bindText, signal } from "@tinyfx/runtime";
import type { TinyFxContext } from "@tinyfx/runtime";

export class Counter {
  public count = signal(0);

  constructor(public el: HTMLElement, public ctx: TinyFxContext) {}

  init(): void {
    const value = this.el.querySelector(".value") as HTMLElement;
    const inc = this.el.querySelector(".increment") as HTMLButtonElement;
    const dec = this.el.querySelector(".decrement") as HTMLButtonElement;

    bindText(value, () => this.count());
    inc.addEventListener("click", () => this.count.set(this.count() + 1));
    dec.addEventListener("click", () => this.count.set(this.count() - 1));
    void this.ctx;
  }

  destroy?(): void {}
}

export function mount(el: HTMLElement, ctx: TinyFxContext) {
  const inst = new Counter(el, ctx);
  inst.init();
  return inst;
}
```

### Signals and Reactivity

Signals are TinyFX’s smallest reactive unit. A signal is a function you call to read the current value, and it exposes `.set()` to update the value. `effect()` tracks signal reads while it runs and re-runs when those values change. The DOM helpers are thin wrappers around `effect()`.

```ts
// src/components/SearchStatus/SearchStatus.ts
import { bindText, effect, signal } from "@tinyfx/runtime";
import type { TinyFxContext } from "@tinyfx/runtime";

export class SearchStatus {
  public query = signal("");
  public resultCount = signal(0);

  constructor(public el: HTMLElement, public ctx: TinyFxContext) {}

  init(): void {
    const output = this.el.querySelector(".summary") as HTMLElement;
    bindText(output, () => `${this.resultCount()} results for ${this.query()}`);

    effect(() => {
      console.log("search state changed", this.query(), this.resultCount());
    });

    void this.ctx;
  }
}
```

### Component Communication

TinyFX does not have a framework event bus. Components communicate the normal web-platform ways: shared signals in `src/lib/state/`, attributes passed through HTML, and custom DOM events.

```ts
// src/lib/state/cart.state.ts
import { signal } from "@tinyfx/runtime";

export const cartCount = signal(0);
```

```html
<!-- src/components/AddToCart/AddToCart.html -->
<button class="add-to-cart" tinyfx-dynamic data-sku="{{sku}}">
  <slot></slot>
</button>
```

```ts
// src/components/AddToCart/AddToCart.ts
import { cartCount } from "@state/cart.state";
import type { TinyFxContext } from "@tinyfx/runtime";

export function mount(el: HTMLElement, ctx: TinyFxContext) {
  el.addEventListener("click", () => {
    cartCount.set(cartCount() + 1);
    el.dispatchEvent(new CustomEvent("cart:added", {
      bubbles: true,
      detail: { sku: el.getAttribute("data-sku") },
    }));
    void ctx;
  });
}
```

### The HTTP Client

`createHttp()` is meant to be used inside service factories, not as a global dependency-injection container. The client supports typed responses, request options, retry and timeout settings, and request or response interceptors.

```ts
// src/lib/services/post.service.ts
import { createHttp } from "@tinyfx/runtime";
import type { PostDto } from "@dto/post.dto";

const http = createHttp({
  baseUrl: "/api",
  timeout: 5000,
  retries: 1,
});

export function createPostService() {
  return {
    async getPost(slug: string): Promise<PostDto> {
      return http.get<PostDto>(`/posts/${slug}`);
    },
  };
}
```

### The Router

TinyFX ships a hard router today. That means navigation is a normal browser page load and each route is backed by a real HTML file in `dist/`. Soft routing is planned, but it is not implemented in the current runtime even though the scaffolded `tinyfx.toml` still contains commented router placeholders.

```ts
// src/pages/blog-post.ts
import { getParam, navigate, onDestroy, onMount } from "@tinyfx/runtime";
import type { TinyFxContext } from "@tinyfx/runtime";

export function init(el: HTMLElement, ctx: TinyFxContext): void {
  const slug = getParam("slug");

  onMount(() => {
    console.log("page mounted", slug, ctx.params);
    el.querySelector(".back-home")?.addEventListener("click", () => navigate("/"));
  });

  onDestroy(() => {
    console.log("leaving", slug);
  });
}
```

### The Lib Folder

`src/lib/` is where application code that should not live inside a single component goes. TinyFX uses path aliases so that generated code and handwritten code can import models, DTOs, state, and services from stable paths. There are no barrel files; each import points directly at a file so the dependency path stays explicit.

```ts
// src/lib/models/post.model.ts
import type { PostDto } from "@dto/post.dto";

export class Post {
  constructor(
    public id: number,
    public title: string,
  ) {}

  static from(dto: PostDto): Post {
    return new Post(dto.id, dto.title);
  }
}
```

```ts
// src/components/PostCard/PostCard.ts
import { Post } from "@models/post.model";
import { cartCount } from "@state/cart.state";
import { createPostService } from "@services/post.service";

void Post;
void cartCount;
void createPostService;
```

| Alias | Folder | Use it for |
|---|---|---|
| `@models/*` | `src/lib/models/*` | Frontend model classes |
| `@dto/*` | `src/lib/dto/*` | Raw API response types |
| `@state/*` | `src/lib/state/*` | Shared signals and state |
| `@services/*` | `src/lib/services/*` | Service factories and business logic |

## CLI Reference

### `tinyfx init <name>`

```sh
tinyfx init <name> [--force]
```

This creates a new TinyFX project directory, writes the scaffold from `compiler/templates/`, and runs `npm install` inside the new project. It creates `package.json`, `tinyfx.toml`, TypeScript config files, `src/main.ts`, the starter page, the starter `Button` component, and the `src/lib/` and `src/styles/` folders. The `--force` flag exists in the CLI, but the current implementation still errors if the target directory already exists, so you should treat it as a placeholder rather than a working overwrite mode.

### `tinyfx build`

```sh
tinyfx build [--config <path>] [--phase-one-only] [--assets-dir <path>]
```

This reads the TinyFX config, compiles components, resolves pages, writes `dist/`, generates `src/tinyfx.gen.ts`, bundles JavaScript with esbuild, and copies styles and assets. `--config` points to a different `tinyfx.toml`. `--phase-one-only` stops after component compilation, which is mainly useful for debugging the build pipeline. `--assets-dir` overrides the configured assets directory for that build only. The command modifies `dist/`, writes `src/tinyfx.gen.ts`, and removes its temporary esbuild entry file when bundling finishes.

The command errors if the configured components directory does not exist, if a dynamic component is missing its `.ts` file, if component references form a cycle, if a page cannot be parsed, or if esbuild cannot be found on `PATH`, in local `node_modules/.bin`, or through `npx`.

### `tinyfx serve`

```sh
tinyfx serve [--config <path>]
```

This loads the config, runs an initial build, starts the HTTP server, starts the WebSocket reload server, watches the project tree, rebuilds on change, and injects the reload client into served HTML responses. It serves the configured output directory on `127.0.0.1:3000` and pushes reload messages on `127.0.0.1:3001`. It modifies the same build output and generated files as `tinyfx build`. It errors if the build fails, if the runtime cannot create its Tokio event loop, or if either network port cannot be bound.

### `tinyfx generate component <Name>`

```sh
tinyfx generate component <Name> [--dynamic] [--force]
tinyfx g component <Name> [--dynamic] [--force]
```

This normalizes the component name to PascalCase, creates `src/components/<Name>/<Name>.html`, and, if `--dynamic` is set, also creates `src/components/<Name>/<Name>.ts`. Without `--dynamic`, the generator creates HTML only. With `--dynamic`, the generated HTML includes the `tinyfx-dynamic` marker and the generated TypeScript exports a `mount(el, ctx)` function. The command errors if you are not inside a TinyFX project, or if the target component directory already exists and `--force` is not provided.

### `tinyfx generate page <name>`

```sh
tinyfx generate page <name> [--force]
tinyfx g page <name> [--force]
```

This normalizes the page name to kebab-case, creates `src/pages/<name>.html`, and creates the sibling `src/pages/<name>.ts`. The route defaults to `/<name>` except for `index`, which becomes `/`. The command errors if you are not inside a TinyFX project or if either target file already exists and `--force` is not provided.

### `tinyfx generate service <Name>`

```sh
tinyfx generate service <Name> [--force]
tinyfx g service <Name> [--force]
```

This creates `src/lib/services/<name>.service.ts` using the service factory template. If you pass `AuthService`, the generated file is still `auth.service.ts`; the trailing `Service` is stripped from the filename stem but kept in the exported type and factory name. The command errors if you are not inside a TinyFX project or if the target file already exists and `--force` is not provided.

### `tinyfx generate model <Name>`

```sh
tinyfx generate model <Name> [--force]
tinyfx g model <Name> [--force]
```

This normalizes the name and creates `src/lib/models/<name>.model.ts`. The generated model imports its DTO peer through the `@dto/*` alias and includes a static `from(dto)` helper. The command errors if you are not inside a TinyFX project or if the target file already exists and `--force` is not provided.

### `tinyfx generate dto <Name>`

```sh
tinyfx generate dto <Name> [--force]
tinyfx g dto <Name> [--force]
```

This normalizes the name and creates `src/lib/dto/<name>.dto.ts`. The command errors if you are not inside a TinyFX project or if the target file already exists and `--force` is not provided.

### `tinyfx generate interface <Name>`

```sh
tinyfx generate interface <Name> [--force]
tinyfx g interface <Name> [--force]
```

This normalizes the name and creates `src/lib/<name>.interface.ts`. The command errors if you are not inside a TinyFX project or if the target file already exists and `--force` is not provided.

## Configuration Reference

The current compiler reads these fields from `tinyfx.toml`.

| Field | Type | Default | What it controls |
|---|---|---|---|
| `entry` | `string` | `"src"` | Present in `Config`, but currently unused by the build pipeline |
| `entry_ts` | `string` | `"src/main.ts"` | The TypeScript entry file bundled in Phase 4 |
| `out_dir` | `string` | `"dist"` | The build output directory |
| `components` | `string` | `"src/components"` | The component source directory |
| `styles` | `string` | `"src/styles"` | The directory copied into `dist/css/` |
| `static_dirs` | `string[]` | `[]` | Extra directories copied verbatim into `dist/` |
| `assets_dir` | `string` | `"src/assets"` | A directory copied into `dist/` if it exists |
| `pages` | `string` | `"src/pages"` | The page source directory |
| `phase_one_only` | `boolean` | `false` | Internal debug flag that stops the build after component compilation |

```toml
# tinyfx.toml
entry = "src"                  # currently present in Config but not used by the build pipeline
entry_ts = "src/main.ts"       # bundled by esbuild with the generated glue file
out_dir = "dist"               # final HTML, JS, CSS, and copied assets land here
components = "src/components"  # component templates and optional behavior files
styles = "src/styles"          # copied to dist/css
static_dirs = ["public"]       # copied into dist/public
assets_dir = "src/assets"      # copied into dist/assets if the directory exists
pages = "src/pages"            # pages with <page> roots
phase_one_only = false          # stop after component compilation
```

The scaffold currently writes additional `[server]` and `[router]` sections into `tinyfx.toml`. Those sections are placeholders today. The compiler does not read them in the current codebase.

## Design Principles

### HTML Is the Source of Truth

TinyFX starts from HTML files, not JSX, template DSLs, or client-side render functions. That means pages are real `.html` files, components are HTML templates, and the compiler expands them into the final output. In practice, this means a TinyFX codebase should still make sense if you open it with JavaScript turned off in your head. It also means TinyFX will not turn HTML into a thin shell around a client-side render tree.

### Compile Time Over Runtime

The build pipeline does as much work as it can before the browser sees the page. Component expansion, layout wrapping, route file generation, and glue generation happen ahead of time. In practice, this means browser code is mostly for enhancement, not rendering. It also means TinyFX will favor build steps over large runtime abstractions when the choice is available.

### Explicit Over Magic

TinyFX uses filenames, directories, route strings, and plain function exports instead of hidden conventions or injection containers. Services are created with factory functions. Route params are read with `getParam()`. Dynamic components are dynamic because you marked them `tinyfx-dynamic` and gave them a `.ts` file. In practice, if a feature would need lots of hidden framework rules, it probably does not belong here.

### JavaScript Enhances, It Does Not Replace

The runtime is intentionally small. It mounts behavior on existing DOM, updates text or attributes through signals, and exposes a few helpers for routing and HTTP. It does not own rendering. In practice, this means a TinyFX app should still have meaningful HTML output before browser behavior runs. It also means TinyFX is not trying to become a client-rendered application shell.

### Small Enough to Understand

TinyFX is deliberately narrow. The compiler, runtime, and scaffold are small enough that a developer can trace how a feature works by reading a few files. In practice, this means the framework should stay understandable by inspection, not by memorizing ecosystem lore. It also means TinyFX will say no to features that would make the codebase harder to follow than the problem is worth.

## Status and Roadmap

| Feature | Status |
|---|---|
| Component expansion | ✅ Implemented |
| File-based hard router | ✅ Implemented |
| Signals and effects | ✅ Implemented |
| Dev server with live reload | ✅ Implemented |
| `<if>` conditional rendering | 📋 Planned |
| `<each>` list rendering | 📋 Planned |
| Soft router | 📋 Planned |
