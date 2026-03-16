# Components

Components are the building blocks of TinyFX. Each component consists of an HTML template and an optional TypeScript class for behavior.

## Creating Components

Use the CLI to scaffold components. The canonical command is `tinyfx generate component <Name>`;
see the CLI Reference for information about available aliases.

Scaffolding creates `src/components/<PascalName>/` with:
- `<PascalName>.html` — the component template
- `<PascalName>.ts` — optional behavior file (created only when `--dynamic` is passed or when requested)

## HTML Templates

Component templates are standard HTML. They can include two special features: **Slots** and **Tokens**.

### Slots
The `<slot></slot>` element is where the children of the component tag will be placed.

```html
<!-- Alert.html -->
<div class="alert">
  <div class="icon">⚠️</div>
  <div class="content">
    <slot></slot>
  </div>
</div>
```

Usage:
```html
<component name="Alert">
  Something went wrong!
</component>
```

### Tokens (Props)
Tokens allow you to inject values passed as attributes on the `<component>` tag. Tokens use the `{{ name }}` syntax.

```html
<!-- Alert.html -->
<div class="alert alert-{{ type }}">
  <slot></slot>
</div>
```

Usage:
```html
<component name="Alert" type="danger">
  Critical error!
</component>
```

At build time, `{{ type }}` will be replaced with `danger`.

## TypeScript Behavior

To add behavior to a component, export a class and a `mount` function.

### The Component Class
The class receives the component's root `HTMLElement` and the `TinyFxContext` in its constructor.

```typescript
import type { TinyFxContext } from "@tinyfx/runtime";
import { bindText } from "@tinyfx/runtime";

export class Alert {
  constructor(public el: HTMLElement, public ctx: TinyFxContext) {}

  init() {
    console.log("Alert mounted");
    // Use this.el to find children or bind state
  }

  destroy() {
    // Clean up event listeners or subscriptions
  }
}
```

### The Mount Helper
The compiler looks for a `mount` function in your `.ts` file to initialize the component.

```typescript
export function mount(el: HTMLElement, ctx: TinyFxContext) {
  const inst = new Alert(el, ctx);
  inst.init();
  return inst;
}
```

### Dynamic components and `tinyfx-dynamic`

If a component should mount runtime behavior, follow these rules:

- Add the `tinyfx-dynamic` attribute to the component root in the `.html` file.
- Create a matching `.ts` alongside the `.html` that exports a `mount(el, ctx)` helper.
- The compiler will warn if a `.ts` exists without `tinyfx-dynamic`, or error if the `.html` has `tinyfx-dynamic` but the `.ts` is missing.

Example dynamic component root:

```html
<section class="counter" data-component="Counter" tinyfx-dynamic>
  <h2>Counter</h2>
  <slot></slot>
</section>
```

## Styling Components

TinyFX doesn't enforce a specific styling solution. You can use global CSS, CSS modules (if configured in your bundler), or utility-first frameworks like Tailwind.

Since the compiler just replaces tags, your component's classes will remain in the output HTML.
