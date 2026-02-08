# tinyfx

> A minimal frontend helper library and compile-time HTML composition tool, designed for **Go SSR**, **TypeScript**, and **direct DOM manipulation** — without virtual DOMs or heavy frameworks.

`tinyfx` exists because sometimes you want:
- typed API calls
- simple reactivity
- reusable HTML pieces
- fast DOM updates
- SSR-friendly behavior

…without adopting a full frontend framework.

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

## Getting Started

### 1. Installation
The compiler is written in Rust. You can build it from source:

```sh
cd compiler
cargo build --release
```

### 2. Initialize a new project
Run the init command to create the recommended folder structure:

```sh
./compiler/target/release/compiler init
```

This creates a `frontend/` folder with everything you need:
- `frontend/index.html`: Entry points with components
- `frontend/main.ts`: Runtime initialization
- `frontend/lib/`: HTTP, DTOs, Models and State
- `frontend/components/`: Example component
- `tinyfx.toml`: Project configuration

### 3. Build for production
Compile your components and bundle your assets:

```sh
./compiler/target/release/compiler build
```

---

## Configuration

`tinyfx` uses a `tinyfx.toml` file in your root:

```toml
src = "frontend"
out = "dist"
components = "frontend/components"
entry = "frontend/main.ts"
```

---

## What tinyfx is (and is not)

### It **is**
- A small **runtime library** for:
  - signals (reactivity)
  - DOM bindings
  - typed HTTP requests
  - DTO / model mapping
- A **compile-time tool** to:
  - split HTML into reusable components
  - inline components into final HTML
  - generate minimal JS glue code

### It is **not**
- A virtual DOM
- A JSX-based framework
- A router
- A full SSR engine
- A build tool replacement

## Project structure

```
tinyfx/
├─ compiler/ # Build-time HTML component compiler
├─ runtime/ # Browser-side TypeScript runtime
├─ examples/ # Real usage examples
├─ docs/ # Design notes & decisions
├─ package.json
├─ tsconfig.json
└─ README.md
```

The **compiler** and **runtime** are independent by design.

---

## Runtime overview

### Signals (reactivity)

Minimal, explicit reactivity — no proxies, no magic.

```ts
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
bindText(span, count);
```

HTML stays HTML.
JS stays JS.

### HTTP client (typed, explicit)

A thin, typed wrapper over fetch.

```ts
const api = createHttp({ baseUrl: "/api" });

const user = await api.get("/users/1", User);
```

#### Features

- typed responses
- centralized error handling
- abort / timeout support
- DTO binding

No interceptors. No global state.

### DTO / Model mapping

Runtime-safe mapping between backend responses and frontend models.

```ts
class User {
  id!: number;
  createdAt!: Date;

  static from(api: any): User {
    return Object.assign(new User(), {
      id: api.id,
      createdAt: new Date(api.created_at),
    });
  }
}
```


This keeps backend contracts explicit and debuggable.

### Compile-time components

Components are a build-time concept, not a runtime one.

Component definition

```
components/
  Button/
    Button.html
    Button.ts      # optional
    Button.css     # optional
```

Button.html

```html
<button class="btn btn-{{variant}}">
  <slot></slot>
</button>
```

Usage

```html
<component name="Button" variant="primary">
  Save
</component>
```

Compiler output

```html
<button class="btn btn-primary" data-component="Button">
  Save
</button>
```


No runtime rendering.
No virtual DOM.
Just plain HTML.

### Component behavior (optional)

If a component needs behavior, it exports an init function:

```ts
export function init(el: HTMLElement) {
  el.addEventListener("click", () => {
    console.log("clicked");
  });
}
```


The compiler generates a small registry and initializes components on page load.

### Go SSR integration

tinyfx is designed to work with Go templates, not against them.

Typical flow:

1. Go renders HTML (templates)
2. tinyfx compiler processes components
3. Browser loads final HTML
4. Runtime enhances the DOM

Optional state hydration:

```html
<script id="__STATE__" type="application/json">
{{ .State | toJSON }}
</script>
```


No hydration framework required.

## Design principles

1. Prefer compile-time over runtime
2. Keep abstractions shallow
3. Optimize for readability and debuggability
4. Avoid cleverness
5. Keep the total codebase small enough to understand

If a feature makes the project feel like a framework — it probably doesn’t belong.

## Status

This project is experimental and intentionally small.

The goal is not to compete with frameworks, but to:

- reduce frontend complexity
- support fast, SSR-friendly apps
- stay maintainable over time

## License
MIT
