# DOM Helpers

Runtime helper exports from `@tinyfx/runtime`:
- `bindText`
- `bindAttr`
- `bindClass`

These helpers are thin wrappers around `effect()` and are used for direct DOM synchronization.
Each helper returns an unsubscribe function (`EffectHandle`) so you can tear down
bindings when a component is destroyed.

## `bindText`
```ts
// src/components/Status/Status.ts
import { bindText, signal } from "@tinyfx/runtime";

const value = signal("ready");
const el = document.querySelector("#status") as HTMLElement;
const dispose = bindText(el, () => value());
// later: dispose()
```

## `bindAttr`
```ts
// src/components/ThemeBadge/ThemeBadge.ts
import { bindAttr, signal } from "@tinyfx/runtime";

const theme = signal("light");
const el = document.querySelector("#theme-badge") as HTMLElement;
const dispose = bindAttr(el, "data-theme", () => theme());
// later: dispose()
```

## `bindClass`
```ts
// src/components/Modal/Modal.ts
import { bindClass, signal } from "@tinyfx/runtime";

const open = signal(false);
const el = document.querySelector("#modal") as HTMLElement;
const dispose = bindClass(el, "is-open", () => open());
// later: dispose()
```

Use `effect()` directly when helper signatures do not match required logic.
