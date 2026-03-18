# DOM Helpers

Runtime helper exports from `@tinyfx/runtime`:
- `bindText`
- `bindAttr`
- `bindClass`

These helpers are thin wrappers around `effect()` and are used for direct DOM synchronization.

## `bindText`
```ts
// src/components/Status/Status.ts
import { bindText, signal } from "@tinyfx/runtime";

const value = signal("ready");
const el = document.querySelector("#status") as HTMLElement;
bindText(el, () => value());
```

## `bindAttr`
```ts
// src/components/ThemeBadge/ThemeBadge.ts
import { bindAttr, signal } from "@tinyfx/runtime";

const theme = signal("light");
const el = document.querySelector("#theme-badge") as HTMLElement;
bindAttr(el, "data-theme", () => theme());
```

## `bindClass`
```ts
// src/components/Modal/Modal.ts
import { bindClass, signal } from "@tinyfx/runtime";

const open = signal(false);
const el = document.querySelector("#modal") as HTMLElement;
bindClass(el, "is-open", () => open());
```

Use `effect()` directly when helper signatures do not match required logic.
