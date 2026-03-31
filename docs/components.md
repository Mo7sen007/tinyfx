# Components

## Static Components (Default)
A component is static unless runtime behavior is explicitly enabled.

```html
<!-- src/components/FeatureCard/FeatureCard.html -->
<div class="feature-card">
  <h3>{{title}}</h3>
</div>
```

Static components are HTML-only. The compiler expands them at build time and they do not mount any runtime behavior. Do not create a `.ts` file for static components.

## Dynamic Components
Dynamic behavior requires both:
1. `tinyfx-dynamic` on the root element in component HTML.
2. A sibling same-name `.ts` file.

```html
<!-- src/components/ThemeToggle/ThemeToggle.html -->
<button class="theme-toggle" tinyfx-dynamic>
  <i class="fa-solid fa-moon"></i>
</button>
```

```ts
// src/components/ThemeToggle/ThemeToggle.ts
import type { TinyFxContext } from "@tinyfx/runtime";

export class ThemeToggle {
  constructor(public el: HTMLElement, public ctx: TinyFxContext) {}

  init(): void {
    this.el.addEventListener("click", () => {
      console.log("toggle");
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

Compiler enforcement:
- `tinyfx-dynamic` and no `.ts` → error.
- `.ts` and no `tinyfx-dynamic` → warning.

## Generated Wrapper System
For each dynamic component, the compiler generates `__mount_ComponentName` in `tinyfx.gen.ts`.

Wrapper responsibilities:
- Calls `mountComponent`.
- Guards with `if (!inst) return null;`.
- Wires component directive sites to instance fields.
- Returns the instance.

Important scope rule:
- Wrapper uses `el.querySelector(...)` for directive site lookup.
- This avoids cross-instance interference when the same component appears multiple times.

## `<each>` Directive
```html
<!-- src/components/Chat/Chat.html -->
<section class="chat" data-component="Chat" tinyfx-dynamic>
  <each of="messages()" as="message" key="id" class="chat-list">
    <ChatMessage content="{{message.content}}" />
  </each>
</section>
```

Supported `of` forms:
- signal call: `messages()`
- plain expression: `messages`
- integer: `10`
- range: `1..5`

Rules:
- `as` is required.
- `key` is required for non-range collection forms.
- `key` must match a real field on each item.

Compiler output:
- Replaces site with `data-each` container.
- Emits `__create_*`, `__update_*`, and `__reconcile_*`.
- Emits signal wiring via `effect()` for signal sources. Plain and range sources do not use `effect()`.

Layout note:
```css
/* src/styles/main.css */
[data-each="chat_each_0"] {
  display: contents;
}
```

Use this when the generated wrapper affects flex/grid behavior.

## `<if>` Directive
```html
<!-- src/components/AuthGate/AuthGate.html -->
<section class="auth" data-component="AuthGate" tinyfx-dynamic>
  <if condition="isLoggedIn()">
    <Dashboard />
  <else>
    <Login />
  </if>
</section>
```

Also supports `<else-if>` chains. Branches are pre-rendered in HTML and runtime toggle helpers control visibility. Signal-based conditions use `effect()` for reactivity; plain conditions do not.

## Public Field Requirement for Template Signals
When a component template references `messages()` or another instance-backed expression in directives, the generated wiring reads through `inst.<field>()`.

Directive-bound fields must be declared `public` in the component class.

```ts
// src/components/Chat/Chat.ts
import { signal } from "@tinyfx/runtime";

export class Chat {
  public messages = signal<string[]>([]);
}
```
