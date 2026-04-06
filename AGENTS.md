# TinyFX Agent Guide

## What This Project Is

TinyFX is a hybrid framework with two parts:
- **Rust CLI compiler** (`compiler/`): scans files, builds dependency graphs, compiles components, processes pages and directives, generates `tinyfx.gen.ts`, and runs bundling and copy phases.
- **TypeScript runtime** (`@tinyfx/runtime`): provides explicit reactivity and browser helpers used by generated glue and app code.

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
│       ├── http/
│       ├── router/
│       ├── context.ts
│       ├── init.ts
│       ├── registry.ts
│       ├── page-registry.ts
│       └── mount-state.ts
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

## Configuration (`tinyfx.toml`)

```toml
entry        = "src"
entry_ts     = "src/main.ts"
out_dir      = "dist"
components   = "src/components"
styles       = "src/styles"
static_dirs  = ["public"]
assets_dir   = "src/assets"
pages        = "src/pages"
phase_one_only = false
release      = false
```

| Field | Type | Default | Description |
|---|---|---|---|
| `entry` | string | `"src"` | Source directory |
| `entry_ts` | string | `"src/main.ts"` | TypeScript entry point |
| `out_dir` | string | `"dist"` | Build output directory |
| `components` | string | `"src/components"` | Components directory |
| `styles` | string | `"src/styles"` | Styles directory |
| `static_dirs` | string[] | `[]` | Additional static directories to copy |
| `assets_dir` | string | `"src/assets"` | Assets directory to copy to output |
| `pages` | string | `"src/pages"` | Pages directory |
| `phase_one_only` | bool | `false` | Only run phase 1 (component compilation, for debugging) |
| `release` | bool | `false` | Enable production optimizations |

## CLI Commands

```bash
# Initialize a new project
tinyfx init <name>

# Build the project
tinyfx build [--config tinyfx.toml] [--release] [--phase-one-only] [--assets-dir <path>]

# Dev server with live reload
tinyfx serve [--config tinyfx.toml]

# Generate scaffolding (alias: g)
tinyfx generate component <Name> [--dynamic] [--force]   # alias: c
tinyfx generate page <name> [--force]                     # alias: p
tinyfx generate service <Name> [--force]                  # alias: s
tinyfx generate model <Name> [--force]                    # alias: m
tinyfx generate dto <Name> [--force]                      # alias: d
tinyfx generate interface <Name> [--force]                # alias: i
```

## Project File Structure

```text
my-app/
├── tinyfx.toml
├── tsconfig.json
├── package.json
└── src/
    ├── main.ts                    # Entry point (bootstraps via tinyfx.gen.ts)
    ├── tinyfx.gen.ts              # Generated — DO NOT EDIT
    ├── pages/
    │   ├── index.html             # Page HTML with <page> root
    │   └── index.ts               # Page init function
    ├── components/
    │   ├── FeatureCard/
    │   │   └── FeatureCard.html   # Static component (HTML-only)
    │   └── ThemeToggle/
    │       ├── ThemeToggle.html   # Dynamic component HTML
    │       └── ThemeToggle.ts     # Dynamic component logic
    ├── lib/
    │   ├── models/                # Frontend model classes
    │   ├── dto/                   # API DTO types
    │   ├── state/                 # Shared signals
    │   └── services/              # Service factory functions
    ├── styles/
    │   └── main.css
    └── assets/                    # Static assets copied to output
```

## Runtime Exports

All exports come from `@tinyfx/runtime`:

### Reactivity
| Export | Purpose |
|---|---|
| `signal<T>(v: T)` | Create a reactive signal; call to read, `.set()` to write |
| `effect(fn)` | Run `fn` and re-run when any signal read inside changes |

### DOM Bindings
| Export | Purpose |
|---|---|
| `bindText(el, get)` | Bind `textContent` to a reactive getter |
| `bindAttr(el, attr, get)` | Bind an attribute to a reactive getter |
| `bindClass(el, cls, get)` | Toggle a CSS class based on a boolean getter |

### Component Lifecycle
| Export | Purpose |
|---|---|
| `registerInstance(el, instance)` | Register a mounted component instance |
| `destroyNode(el)` | Destroy a component and all nested components |
| `markMounted(el)` | Mark element as mounted (prevents double-mount) |
| `isMounted(el)` | Check if element has been marked as mounted |

### Component Registry
| Export | Purpose |
|---|---|
| `registerComponent(name, factory)` | Register a component factory by name |
| `getComponentFactory(name)` | Retrieve a registered component factory |
| `mountComponentByName(name, el, ctx)` | Mount a single component by name (escape hatch) |
| `mountComponentsIn(root, ctx, names?)` | Mount all components in a subtree (escape hatch) |
| `mountComponents(ctx, root?)` | **Deprecated** — use `mountComponentsIn()` instead |

### Page Registry
| Export | Purpose |
|---|---|
| `registerPage(path, module)` | Register a page module by route path |
| `getPageModule(path)` | Retrieve a registered page module |
| `runPageInit(path, ctx)` | Run the `init()` function for a page |

### Router
| Export | Purpose |
|---|---|
| `init(config)` | Bootstrap the application; returns `TinyFxContext \| null` (called by generated code) |
| `getParam(key)` | Get a single route param from the current URL |
| `getParams()` | Get all route params as an object |
| `navigate(path)` | Navigate to a path (full page load) |
| `goBack()` | Go back in browser history |
| `matchPath(def, segments)` | Match a route definition against path segments |
| `splitPath(pathname)` | Split a pathname into segments |

### Lifecycle Hooks
| Export | Purpose |
|---|---|
| `onMount(fn)` | Register callback for when page is mounted (DOM ready) |
| `onDestroy(fn)` | Register callback for when user navigates away |

### Types
| Export | Purpose |
|---|---|
| `TinyFxContext` | Context with `params` and `navigate` |
| `RouteMap` | Full route map type (`Record<string, RouteMeta>`) |
| `RouteMeta` | Single route metadata (`{ page, path }`) |
| `RouteDef` | Route definition for matching (`{ staticSegments, paramNames }`) |
| `InitConfig` | Init configuration (`{ routes, setupDirectives? }`) |

### HTTP
| Export | Purpose |
|---|---|
| `createHttp(config)` | Create a typed HTTP client around `fetch` |

## Reactivity System

### Signals

Signals are explicit callable state holders — no proxies, no magic.

```typescript
// Creating a signal
const count = signal(0);

// Reading a signal (call it like a function)
console.log(count()); // 0

// Updating a signal
count.set(1);
```

Key mechanics:
- `signal<T>(v: T)` returns a callable function with a `.set()` method
- `Object.is` check prevents unnecessary re-runs on identical values
- Signals auto-register the active effect as a subscriber when read

### Effects

```typescript
effect(() => {
  console.log(`count = ${count()}`); // Logs on creation and whenever count changes
});
```

- Every signal read while `fn` runs is tracked
- The effect re-runs when any of those signal values change
- Effects run immediately when created

### DOM Bindings

All three use `effect()` internally for reactivity:

```typescript
// bindText — updates textContent reactively
bindText(span, () => `Count: ${count()}`);

// bindAttr — updates an attribute reactively
bindAttr(document.body, "data-theme", () => isDark() ? "dark" : "light");

// bindClass — toggles a CSS class based on a boolean signal
bindClass(panel, "is-open", () => open());
```

## Pages

Pages are HTML files in `src/pages/` with a `<page>` root element:

```html
<!-- src/pages/index.html -->
<page route="/" layout="">
  <main class="home">
    <h1>Welcome</h1>
    <Chat />
  </main>
</page>
```

```html
<!-- src/pages/blog-post.html -->
<page route="/blog/:slug" layout="">
  <main class="blog-post">
    <h1>Blog Post</h1>
    <article>Content here</article>
  </main>
</page>
```

Page TypeScript files export an `init` function:

```typescript
// src/pages/blog-post.ts
import { onMount, onDestroy, getParam } from "@tinyfx/runtime";
import type { TinyFxContext } from "@tinyfx/runtime";

export function init(_el: HTMLElement, ctx: TinyFxContext): void {
  onMount(() => {
    const slug = getParam("slug"); // "hello-world" on /blog/hello-world
    console.log("Loading post:", slug);
  });

  onDestroy(() => {
    console.log("Leaving blog post page");
  });
}
```

### Routing

The router is a **hard router** — navigation always triggers a full browser page load:

```typescript
import { navigate, goBack } from "@tinyfx/runtime";

navigate("/blog/hello-world"); // Full page load
goBack(); // Browser back
```

Active link highlighting is automatic — `<a>` elements matching the current path get `data-active="true"`.

## Components

### Static Components (Default)

Static components are expanded at build time into plain HTML. No `.ts` file needed.

```html
<!-- src/components/FeatureCard/FeatureCard.html -->
<div class="feature-card">
  <div class="feature-icon">
    <i class="{{icon}}"></i>
  </div>
  <h3 class="feature-title">{{title}}</h3>
  <p class="feature-description">{{description}}</p>
</div>
```

Usage in a page (compiler substitutes `{{...}}` with attribute values):

```html
<page route="/" layout="">
  <FeatureCard icon="fa-star" title="Fast" description="Blazing speed" />
</page>
```

### Dynamic Components

Dynamic components require **both** `tinyfx-dynamic` on the root element **and** a sibling same-name `.ts` file:

```html
<!-- src/components/ThemeToggle/ThemeToggle.html -->
<button class="theme-toggle" tinyfx-dynamic aria-label="Toggle theme">
  <i class="fa-solid fa-moon"></i>
</button>
```

```typescript
// src/components/ThemeToggle/ThemeToggle.ts
import { effect } from "@tinyfx/runtime";
import type { TinyFxContext } from "@tinyfx/runtime";
import { theme } from "@state/theme.state";

export class ThemeToggle {
  private icon: HTMLElement | null = null;

  constructor(public el: HTMLElement, public ctx: TinyFxContext) {}

  init(): void {
    this.icon = this.el.querySelector("i");

    effect(() => {
      const isDark = theme() === "dark";
      if (this.icon) {
        this.icon.className = isDark ? "fa-solid fa-sun" : "fa-solid fa-moon";
      }
    });

    this.el.addEventListener("click", () => {
      theme.set(theme() === "dark" ? "light" : "dark");
    });
  }

  destroy?(): void {
    // cleanup
  }
}

export function mount(el: HTMLElement, ctx: TinyFxContext) {
  const inst = new ThemeToggle(el, ctx);
  inst.init();
  return inst;
}
```

Dynamic component patterns:

**Class-based** (for components with state and lifecycle):
```typescript
export class ComponentName {
  // public signals for directive access
  public items = signal<Item[]>([]);
  // private signals for internal state
  private loading = signal(false);

  constructor(public el: HTMLElement, public ctx: TinyFxContext) {}

  init(): void {
    // setup event listeners, effects, etc.
  }

  destroy?(): void {
    // cleanup
  }
}

export function mount(el: HTMLElement, ctx: TinyFxContext) {
  const inst = new ComponentName(el, ctx);
  inst.init();
  return inst;
}
```

**Function-based** (for simple components without lifecycle):
```typescript
export function mount(el: HTMLElement) {
  const count = signal(0);

  el.querySelector("#increment")?.addEventListener("click", () => count.set(count() + 1));

  effect(() => {
    el.querySelector("#count")!.textContent = String(count());
  });
}
```

### Component Communication

1. **Shared signals** in `src/lib/state/`:
```typescript
// src/lib/state/app.state.ts
import { signal } from "@tinyfx/runtime";
export const currentUser = signal<string | null>(null);
```

2. **HTML attributes** for parent-to-child data (static components):
```html
<UserCard name="Alice" role="admin" />
```

3. **Custom DOM events** for child-to-parent notifications:
```typescript
// Child dispatches event
this.el.dispatchEvent(
  new CustomEvent("message:sent", {
    detail: { content: "hello" },
    bubbles: true,
  }),
);

// Parent listens
this.el.addEventListener("message:sent", (event: Event) => {
  const custom = event as CustomEvent<{ content: string }>;
  console.log(custom.detail.content);
});
```

### Component Wrapper System

For each dynamic component, the compiler generates a `__mount_ComponentName` wrapper in `tinyfx.gen.ts`:

```typescript
function __mount_Chat(el: HTMLElement, ctx: TinyFxContext): any {
  const inst = mountComponent(ChatModule, el, ctx);
  if (!inst) return null;

  const __each_chat_each_0 = el.querySelector<HTMLElement>("[data-each='chat_each_0']");
  if (__each_chat_each_0) {
    effect(() => __reconcile_chat_each_0(__each_chat_each_0, inst.messages(), ctx));
  }

  return inst;
}
```

Critical scoping rule:
- Wrappers use `el.querySelector(...)`, **not** `document.querySelector(...)`, so each component instance wires its own subtree.

## Directives

### `<each>`

Supported `of` forms:
- Signal call: `items()`
- Plain expression: `items`
- Integer count: `10`
- Range: `1..5`

```html
<each of="messages()" as="message" key="id">
  <ChatMessage username="{{message.username}}" content="{{message.content}}" />
</each>
```

Compiler behavior:
- Replaces directive site with: `<div data-each="SITE_ID" ...></div>`
- Generates `__create_*`, `__update_*`, and `__reconcile_*` functions
- Signal sources use `effect()` for reconciliation; plain and range sources do not

Layout note:
- `data-each` wrapper can affect flex/grid layout
- Use `display: contents` on the generated wrapper selector when needed:
```css
[data-each="chat_each_0"] {
  display: contents;
}
```

### `<if>`

```html
<if condition="isLoggedIn()">
  <Dashboard />
<else>
  <Login />
</if>
```

Chains are supported:

```html
<if condition="isAdmin()">
  <AdminPanel />
<else-if condition="isLoggedIn()">
  <Dashboard />
<else>
  <Login />
</if>
```

Compiler behavior:
- Pre-renders all branches into HTML
- Wraps each branch with `data-branch` and initial `hidden` state
- Generates a `__toggle_*` function
- Signal-based conditions use `effect()`; plain conditions do not

## HTTP Client

```typescript
import { createHttp } from "@tinyfx/runtime";

const http = createHttp({
  baseUrl: "/api",
  timeout: 5000,
  retries: 1,
  retryDelay: 1000,
  headers: { "X-Custom-Header": "value" },
  requestInterceptors: [
    (url, options) => ({
      url,
      options: { ...options, headers: { ...options.headers, Authorization: "Bearer token" } },
    }),
  ],
  responseInterceptors: [
    (response) => {
      console.log(response.status);
      return response;
    },
  ],
});

// Usage
const users = await http.get<UserDto[]>("/users");
const created = await http.post<UserDto>("/users", body);
const updated = await http.put<UserDto>(`/users/${id}`, body);
const patched = await http.patch<UserDto>(`/users/${id}`, body);
await http.delete(`/users/${id}`);
// or
await http.del(`/users/${id}`);

// With per-request options
const result = await http.get<UserDto>(`/users/${id}`, {
  params: { include: "profile" },
  timeout: 10000,
  signal: abortController.signal,
});

// Raw body (no JSON serialization)
const formData = new FormData();
formData.append("file", file);
await http.post("/upload", formData, { json: false });
```

HTTP error types (discriminated union):
```typescript
type HttpError =
  | { kind: "http"; status: number; statusText: string; url: string; method: string; response?: Response }
  | { kind: "timeout"; url: string; timeout: number }
  | { kind: "parse"; message: string; url: string };

// Type guards
isHttpError(err);     // HTTP error (4xx, 5xx)
isTimeoutError(err);  // Request timeout
isParseError(err);    // JSON parse failure
```

## Lib Folder and Alias Rules

```text
src/lib/
├── models/     # Frontend model classes (built from DTOs)
├── dto/        # Raw API response types
├── state/      # Shared signals and global state
├── services/   # HTTP client wrappers and business logic
└── utils/      # Shared utility functions (formatBytes, truncateName, etc.)
```

Alias requirements:
- `@models/*` -> `src/lib/models/*`
- `@dto/*` -> `src/lib/dto/*`
- `@state/*` -> `src/lib/state/*`
- `@services/*` -> `src/lib/services/*`

```typescript
import { User } from "@models/user.model";
import type { UserDto } from "@dto/user.dto";
import { currentUser } from "@state/app.state";
import { createUserService } from "@services/user.service";
import { formatBytes } from "@utils/formatBytes";
```

### Shared Utilities Rule

Utility functions that are used in more than one component **must** live in `src/lib/utils/` and be imported via the `@utils/*` alias. Do not copy-paste helper functions into multiple component `.ts` files — the bundler includes each file independently, so duplicated functions appear in the output bundle multiple times.

```typescript
// GOOD — shared, imported once, bundled once
// src/lib/utils/formatBytes.ts
export function formatBytes(bytes: number): string { /* ... */ }

// src/components/Chat/Chat.ts
import { formatBytes } from "@utils/formatBytes";

// src/components/FileList/FileList.ts
import { formatBytes } from "@utils/formatBytes";

// BAD — duplicated in bundle
// src/components/Chat/Chat.ts
function formatBytes(bytes: number): string { /* ... */ }

// src/components/FileList/FileList.ts
function formatBytes(bytes: number): string { /* ... */ }
```

## Service Pattern

Services are factory functions in `src/lib/services/`:

```typescript
// src/lib/services/user.service.ts
import { createHttp } from "@tinyfx/runtime";
import type { UserDto } from "@dto/user.dto";
import { User } from "@models/user.model";

const http = createHttp({ baseUrl: "/api" });

export function createUserService() {
  return {
    async getUser(id: number): Promise<User> {
      const dto = await http.get<UserDto>(`/users/${id}`);
      return User.from(dto);
    },
    async createUser(data: Partial<UserDto>): Promise<User> {
      const dto = await http.post<UserDto>("/users", data);
      return User.from(dto);
    },
  };
}

export type UserService = ReturnType<typeof createUserService>;
```

Usage in components:
```typescript
import { createUserService } from "@services/user.service";

export class UserList {
  private userService = createUserService();
  public users = signal<User[]>([]);

  constructor(public el: HTMLElement, public ctx: TinyFxContext) {}

  async init(): void {
    this.users.set(await this.userService.getUser(1));
  }
}
```

## TinyFxContext

Lightweight context passed to page and component behavior:

```typescript
interface TinyFxContext {
  params: Record<string, string>;    // Route params
  navigate: (path: string) => void;   // Navigation helper
}
```

```typescript
// In a component
export class UserProfile {
  constructor(public el: HTMLElement, public ctx: TinyFxContext) {}

  init(): void {
    const userId = this.ctx.params["id"];
    this.ctx.navigate("/dashboard");
  }
}
```

## Initialization

The `init()` function is the single bootstrap entry point, called automatically by generated code in `tinyfx.gen.ts`:

```typescript
const ctx = init({ routes, setupDirectives });
if (!ctx) return;
// Generated mount calls follow...
```

What `init()` does:
1. Matches current URL path against registered routes
2. Initializes lifecycle hooks
3. Highlights active navigation links
4. Sets route params
5. Runs `setupDirectives(ctx)` if provided
6. Runs page-level `init()`
7. Returns `TinyFxContext | null` (null if no route matched)

Component mounting is handled by the generated code — `init()` returns a context that the generated bootstrap uses to call per-component mount functions directly. See [docs/component-mounting.md](../docs/component-mounting.md) for the full mounting architecture.

## CLI Generate Naming Rules

- Component names normalize to **PascalCase**: `tinyfx generate component userCard` -> `UserCard/`
- Generated page filenames normalize to **kebab-case**: `tinyfx generate page UserProfile` -> `user-profile.html`
- Service names normalize to **kebab-case** with `.service.ts` suffix: `tinyfx generate service AuthService` -> `auth.service.ts`

Generated file locations:
- `tinyfx generate component <Name>` -> `src/components/<Name>/<Name>.html`
- `tinyfx generate component <Name> --dynamic` -> `src/components/<Name>/<Name>.html` + `<Name>.ts`
- `tinyfx generate page <name>` -> `src/pages/<name>.html` + `<name>.ts`
- `tinyfx generate service <Name>` -> `src/lib/services/<name>.service.ts`
- `tinyfx generate model <Name>` -> `src/lib/models/<name>.model.ts`
- `tinyfx generate dto <Name>` -> `src/lib/dto/<name>.dto.ts`
- `tinyfx generate interface <Name>` -> `src/lib/<name>.interface.ts`

## Files Agents Must Not Edit

- `src/tinyfx.gen.ts` (generated, overwritten each build)
- `dist/` (build output)
- `target/` (Rust build output)
- `node_modules/` (dependency output)

## Agent Rules Summary

### Components
- NEVER add `tinyfx-static` to component markup — static is the default
- NEVER create a static component with a `.ts` file
- ALWAYS include `tinyfx-dynamic` when runtime behavior is required
- ALWAYS create matching `.ts` file for dynamic components
- `.ts` file must export `mount(el, ctx)` function
- Use `constructor(public el: HTMLElement, public ctx: TinyFxContext)` for class-based components

### Signals and Directives
- NEVER use `private` for component fields referenced from template directives
- ALWAYS use `public` for directive-bound instance fields (e.g., `<each of="messages()">` requires `public messages = signal(...)`)
- ALWAYS set `<each key="...">` to an existing field from real data
- NEVER invent a synthetic key field that does not exist
- For backend records, use backend identity fields (`id`, `uuid`, etc.)

### DOM Scoping
- NEVER use `document.querySelector(...)` inside component class logic
- ALWAYS use `this.el.querySelector(...)` scoped to component root

### Imports
- NEVER create barrel `index.ts` files in `src/lib/*` subfolders
- ALWAYS import concrete files via alias paths
- NEVER use relative imports to reach `src/lib/*` from components/pages
- Import `TinyFxContext` from `@tinyfx/runtime`, not from nonexistent DI paths

### Services
- ALWAYS implement services as factory functions
- NEVER use runtime DI container patterns
- NEVER use global singleton service objects as default architecture

### Identity
- NEVER create client-side IDs before backend write when backend owns identity
- Send backend-required fields and consume backend-provided IDs

## Common Mistakes

| WRONG | RIGHT |
|---|---|
| `tinyfx-static` on a component element | Static is default — no marker needed |
| `private messages = signal([])` when used in `<each of="messages()">` | `public messages = signal([])` |
| `<each key="i">` when items do not have `i` | `<each key="id">` (real field from data) |
| `document.querySelector(...)` in component logic | `this.el.querySelector(...)` scoped to root |
| Creating client-side IDs before backend write | Consume backend-provided IDs |
| Adding `.ts` file to a static component | Keep static HTML-only; use dynamic marker when behavior needed |
| Importing `TinyFxContext` from nonexistent paths | `import type { TinyFxContext } from "@tinyfx/runtime"` |
| Using relative imports to reach `src/lib/*` | Use `@models/*`, `@services/*`, etc. |

## Working Discipline for Agents

- Prefer implementation details from source files over secondary docs
- Describe only implemented behavior as implemented
- Mark planned features as planned
- Keep code suggestions aligned with current runtime and compiler exports
