# @tinyfx/runtime

> The minimal, browser-side runtime for tinyfx.

A lightweight collection of helpers for reactivity, DOM manipulation, and typed HTTP communication. Designed to be used standalone or as part of the `tinyfx` build pipeline.

## Features

- **Signals** — Explicit reactivity without Proxies.
- **DOM Helpers** — One-way bindings for text, attributes, and classes.
- **Typed HTTP** — Minimal wrapper over `fetch` with response typing.
- **DI Container** — Simple `provide()`/`inject()` registry for app services.
- **Tiny** — No dependencies, tree-shakeable, and extremely fast.

## Installation

```sh
npm install @tinyfx/runtime
```

## Usage

### Signals (Reactivity)

Signals provide a simple way to manage state that triggers effects when changed.

```ts
import { signal, effect } from "@tinyfx/runtime";

const count = signal(0);

// Runs immediately, and re-runs whenever count() changes
effect(() => {
  console.log("The count is:", count());
});

count.set(1); // logs: "The count is: 1"
count.set(count() + 1); // logs: "The count is: 2"
```

### DOM Bindings

Keep your JS decoupled from the DOM while maintaining reactive updates.

```ts
import { signal, bindText, bindClass, bindAttr } from "@tinyfx/runtime";

const count = signal(0);
const isActive = signal(false);

const el = document.getElementById("counter");
const btn = document.querySelector("button");

bindText(el, () => `Count: ${count()}`);
bindClass(btn, "active", isActive);
bindAttr(btn, "disabled", () => count() > 10);
```

### Typed HTTP Client

A thin wrapper over `fetch` that returns typed responses.

```ts
import { createHttp } from "@tinyfx/runtime";

interface User {
  id: number;
  name: string;
}

const api = createHttp({ baseUrl: "/api" });

// Fully typed response
const users = await api.get<User[]>("/users");

// POST with data
await api.post("/users", { name: "Alice" });
```

### DI Container (provide / inject)

TinyFX includes a small dependency injection container to avoid importing singleton instances everywhere.

Import it from `@tinyfx/runtime/di`:

```ts
import { Container } from "@tinyfx/runtime/di";

class Config {
  constructor(public baseUrl: string) {}
}

const container = new Container();
container.provide(Config, new Config("/api"));

const cfg = container.inject(Config);
console.log(cfg.baseUrl);
```

#### Using DI with tinyfx components

When you use the tinyfx compiler, the generated glue calls your component behavior as:

```ts
init(el, { container })
```

So your component can inject services without importing global singletons:

```ts
import type { TinyFxContext } from "@tinyfx/runtime/di";
import { HttpService } from "../lib/http.service";

export function init(el: HTMLElement, ctx: TinyFxContext) {
  const http = ctx.container.inject(HttpService);
  http.getWeather().then((text) => {
    el.textContent = text;
  });
}
```

#### Example: service that uses the HTTP helper

```ts
// src/lib/http.service.ts
import { createHttp } from "@tinyfx/runtime";

export class HttpService {
  private readonly http;

  constructor(baseUrl: string) {
    this.http = createHttp({ baseUrl });
  }

  getWeather() {
    // wttr.in can return plain text; your wrapper returns text when not JSON.
    return this.http.get<string>("");
  }
}
```

```ts
// src/main.ts
import { Container } from "@tinyfx/runtime/di";
import { setupComponents } from "./tinyfx.gen";
import { HttpService } from "./lib/http.service";

const container = new Container();
container.provide(HttpService, new HttpService("https://wttr.in/tunisa"));

document.addEventListener("DOMContentLoaded", () => {
  setupComponents(container);
});
```

## API Overview

### Signals
- `signal<T>(value: T): Signal<T>`
- `effect(fn: () => void): void`

### DOM
- `bindText(el: HTMLElement, source: Signal<any> | (() => any)): void`
- `bindAttr(el: HTMLElement, attr: string, source: Signal<any> | (() => any)): void`
- `bindClass(el: HTMLElement, className: string, source: Signal<boolean> | (() => boolean)): void`

### HTTP
- `createHttp(config: HttpConfig): HttpClient`
- `client.get<T>(url: string): Promise<T>`
- `client.post<T>(url: string, data?: any): Promise<T>`
- `client.put<T>(url: string, data?: any): Promise<T>`
- `client.del<T>(url: string): Promise<T>`

### DI
- `new Container()`
- `container.provide(token, instance)`
- `container.inject(token)`
- `createToken<T>(description: string): symbol`
- `TinyFxContext` (passed to component init via tinyfx glue)

## License

MIT
