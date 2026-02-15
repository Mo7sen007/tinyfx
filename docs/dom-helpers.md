# DOM Bindings

TinyFX provides helper functions to easily bind reactive signals to DOM elements. These helpers wrap `effect()` for common tasks.

## `bindText`

Updates the `textContent` of an element whenever a signal changes.

```typescript
import { signal, bindText } from "@tinyfx/runtime";

export class Counter {
  count = signal(0);
  
  init() {
    const output = this.el.querySelector("#count-val") as HTMLElement;
    bindText(output, () => this.count());
  }
}
```

## `bindAttr`

Updates an attribute of an element based on a signal.

```typescript
import { signal, bindAttr } from "@tinyfx/runtime";

export class ThemeToggle {
  isDark = signal(false);
  
  init() {
    bindAttr(this.el, "data-theme", () => this.isDark() ? "dark" : "light");
  }
}
```

## `bindClass`

Toggles a CSS class based on a boolean reactive value.

```typescript
import { signal, bindClass } from "@tinyfx/runtime";

export class Modal {
  isOpen = signal(false);
  
  init() {
    const backdrop = this.el.querySelector(".backdrop") as HTMLElement;
    bindClass(backdrop, "is-visible", () => this.isOpen());
  }
}
```

## Manual Bindings

You can always create your own custom bindings using `effect()` if the built-in helpers don't cover your use case.

```typescript
import { effect } from "@tinyfx/runtime";

effect(() => {
  this.el.style.backgroundColor = this.color();
});
```
