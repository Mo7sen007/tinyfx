# Components

Components are the building blocks of TinyFX. Each component consists of an HTML template and an optional TypeScript class for behavior.

## Creating Components

The easiest way to create a component is using the CLI:

```bash
tinyfx g component Alert
```

This creates a folder in `src/components/` with:
- `Alert.html`: The HTML template.
- `Alert.ts`: The TypeScript logic.

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
import { TinyFxContext } from "@tinyfx/runtime/di";
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

## Styling Components

TinyFX doesn't enforce a specific styling solution. You can use global CSS, CSS modules (if configured in your bundler), or utility-first frameworks like Tailwind.

Since the compiler just replaces tags, your component's classes will remain in the output HTML.
