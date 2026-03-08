# @tinyfx/runtime

The TinyFX browser runtime: signals, DOM helpers, typed HTTP, and router lifecycle utilities.

## Features

- Signals (`signal`, `effect`)
- DOM binding helpers (`bindText`, `bindAttr`, `bindClass`)
- Typed HTTP client (`createHttp`)
- Router helpers (`initRouter`, `navigate`, `goBack`, `getParam`)
- Lifecycle hooks (`onMount`, `onDestroy`)
- Lightweight `TinyFxContext` (`params`, `navigate`)

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

### DOM bindings

```ts
import { signal, bindText, bindClass, bindAttr } from "@tinyfx/runtime";

const count = signal(0);
const isActive = signal(false);

const el = document.getElementById("counter");
const btn = document.querySelector("button");

if (el && btn) {
  bindText(el, () => `Count: ${count()}`);
  bindClass(btn, "active", isActive);
  bindAttr(btn, "disabled", () => count() > 10);
}
```

### HTTP client

```ts
import { createHttp } from "@tinyfx/runtime";

const http = createHttp({ baseUrl: "/api" });
const users = await http.get<{ id: number; name: string }[]>("/users");
```

### TinyFxContext

```ts
import type { TinyFxContext } from "@tinyfx/runtime";

export function init(el: HTMLElement, ctx: TinyFxContext) {
  console.log(ctx.params);
  ctx.navigate("/about");
}
```

### Router + lifecycle hooks

```ts
import { initRouter, onMount, onDestroy } from "@tinyfx/runtime";

initRouter({
  "/": { page: "index", path: "/" },
});

onMount(() => {
  console.log("mounted");
});

onDestroy(() => {
  console.log("destroyed");
});
```

## API overview

- `signal<T>(value: T)`
- `effect(fn)`
- `bindText(el, source)`
- `bindAttr(el, attr, source)`
- `bindClass(el, className, source)`
- `createHttp(config?)`
- `initRouter(routes)`
- `getParam(name)`
- `navigate(path)`
- `goBack()`
- `onMount(fn)`
- `onDestroy(fn)`
- `TinyFxContext`

## License

MIT
