# Signals and Effects

TinyFX reactivity is explicit and function-based.

## `signal()`
```ts
// src/lib/state/counter.state.ts
import { signal } from "@tinyfx/runtime";

export const count = signal(0);
```

Read current value:
```ts
const current = count();
```

Write value:
```ts
count.set(count() + 1);
```

## `effect()`
`effect()` tracks signal reads performed in its callback and re-runs when those values change.

```ts
// src/components/Counter/Counter.ts
import { effect } from "@tinyfx/runtime";
import { count } from "@state/counter.state";

const dispose = effect(() => {
  console.log("count changed", count());
});

// later, when no longer needed:
dispose();
```

`effect()` returns an unsubscribe function (`EffectHandle`).

### Cleanup Pattern in Components

You are responsible for disposing effects created in component code.

```ts
import { effect } from "@tinyfx/runtime";
import type { TinyFxContext } from "@tinyfx/runtime";

export class Counter {
  private disposeCount?: () => void;

  constructor(public el: HTMLElement, public ctx: TinyFxContext) {}

  init(): void {
    this.disposeCount = effect(() => {
      // reactive logic
    });
  }

  destroy(): void {
    this.disposeCount?.();
  }
}
```

## Directive Integration
Compiler-generated glue uses `effect()` to wire signal-based directive sources:
- `<each of="items()">` -> reconciler call inside effect.
- signal-based `<if>` conditions -> toggle call inside effect.

This behavior is generated into `tinyfx.gen.ts`.
