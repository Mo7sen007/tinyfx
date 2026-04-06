# @tinyfx/runtime

The TinyFX browser runtime: signals, DOM helpers, typed HTTP, router lifecycle utilities, and dynamic registration APIs.

## Features

- Signals (`signal`, `effect`)
- DOM binding helpers (`bindText`, `bindAttr`, `bindClass`)
- Instance management (`registerInstance`, `destroyNode`)
- Typed HTTP client (`createHttp`) with discriminated error types
- Router helpers (`getParam`, `getParams`, `navigate`, `goBack`)
- Lifecycle hooks (`onMount`, `onDestroy`)
- Lightweight `TinyFxContext` (`params`, `navigate`)
- Dynamic component registration (`registerComponent`, `getComponentFactory`, `mountComponentByName`, `mountComponentsIn`, `mountComponents`)
- Dynamic page registration (`registerPage`, `getPageModule`, `runPageInit`)
- Mount state tracking (`markMounted`, `isMounted`)
- Path matching (`matchPath`, `splitPath`, `RouteDef`)
- Unified initialization (`init`)

## Installation

```sh
npm install @tinyfx/runtime
```

## Usage

### Signals

```ts
import { signal, effect } from "@tinyfx/runtime";

const count = signal(0);

effect(() => {
  console.log("count:", count());
});

count.set(count() + 1);
```

### DOM Bindings

```ts
import { signal, bindText, bindClass, bindAttr } from "@tinyfx/runtime";

const count = signal(0);
const isActive = signal(false);

const el = document.getElementById("counter");
const btn = document.querySelector("button");

if (el && btn) {
  bindText(el, () => `Count: ${count()}`);
  bindClass(btn, "active", () => isActive());
  bindAttr(btn, "disabled", () => count() > 10);
}
```

### HTTP Client

```ts
import { createHttp } from "@tinyfx/runtime";

const http = createHttp({ baseUrl: "/api" });
const users = await http.get<{ id: number; name: string }[]>("/users");
```

### TinyFxContext

```ts
import type { TinyFxContext } from "@tinyfx/runtime";

export function init(_el: HTMLElement, ctx: TinyFxContext) {
  console.log(ctx.params);
  ctx.navigate("/about");
}
```

### Router + Lifecycle Hooks

```ts
import { onMount, onDestroy } from "@tinyfx/runtime";

onMount(() => {
  console.log("mounted");
});

onDestroy(() => {
  console.log("destroyed");
});
```

### Component Registration

```ts
import { registerComponent, getComponentFactory, mountComponentByName } from "@tinyfx/runtime";

registerComponent("MyButton", (el, ctx) => {
  el.addEventListener("click", () => console.log("clicked"));
  return { el };
});

// Later, mount by name
const factory = getComponentFactory("MyButton");
if (factory) {
  mountComponentByName("MyButton", someElement, ctx);
}
```

### Page Registration

```ts
import { registerPage, getPageModule, runPageInit } from "@tinyfx/runtime";

registerPage("/", {
  init(_el, _ctx) {
    console.log("home page loaded");
  },
});

// Later, retrieve and run
const page = getPageModule("/");
if (page) {
  runPageInit("/", ctx);
}
```

### Initialization

The compiler-generated `tinyfx.gen.ts` calls `init()` automatically. You typically do not need to call it yourself.

```ts
import { init } from "@tinyfx/runtime";

init({ routes, setupDirectives });
```

## API Overview

### Reactivity
- `signal<T>(value: T)` — creates a reactive signal; call to read, `.set()` to write
- `effect(fn)` — registers a reactive effect; returns `EffectHandle` for cleanup
- `registerInstance(el, instance)` — registers a mounted component instance
- `destroyNode(el)` — destroys a component and all nested components

### DOM Bindings
- `bindText(el, source)` — binds text content reactively
- `bindAttr(el, attr, source)` — binds an attribute reactively
- `bindClass(el, className, source)` — binds a class toggle reactively

### HTTP
- `createHttp(config?)` — creates a typed HTTP client

### Router
- `getParam(name)` — gets a single route parameter
- `getParams()` — gets all route parameters
- `navigate(path)` — triggers full browser navigation
- `goBack()` — goes back in browser history
- `matchPath(def, segments)` — matches a RouteDef against path segments
- `splitPath(pathname)` — splits a pathname into segments

### Lifecycle
- `onMount(fn)` — registers a mount callback
- `onDestroy(fn)` — registers a destroy callback

### Component Registry
- `registerComponent(name, factory)` — registers a dynamic component factory
- `getComponentFactory(name)` — retrieves a registered component factory
- `mountComponentByName(name, el, ctx)` — mounts a single component by name
- `mountComponentsIn(root, ctx, names?)` — mounts components in a subtree
- `mountComponents(ctx, root?)` — mounts all components in a root element

### Page Registry
- `registerPage(route, module)` — registers a page module
- `getPageModule(route)` — retrieves a registered page module
- `runPageInit(route, ctx)` — runs page init if registered

### Mount State
- `markMounted(el)` — marks an element as mounted (returns true on first call)
- `isMounted(el)` — checks if an element has been mounted

### Bootstrap
- `init(config)` — unified bootstrap entry point

### Types
- `TinyFxContext` — type for page/component context
- `RouteMap`, `RouteMeta` — routing collection types
- `RouteDef` — route definition for matching
- `InitConfig` — init configuration type

## License

MIT
