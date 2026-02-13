// {{Name}} component (class-based)

import type { TinyFxContext } from "@tinyfx/runtime/di";

export class {{Name}} {
  // If you add reactive fields, initialize them immediately, e.g.:
  // public count = signal(0);
  // Avoid `public count!: Signal<number>` unless you assign it before use.
  constructor(public el: HTMLElement, public ctx: TinyFxContext) {}

  init(): void {
    // mounted behaviour
    console.log("{{Name}} mounted", this.el);
  }

  destroy?(): void {
    // optional teardown
  }
}

// convenience mount helper used by tinyfx.gen.ts
export function mount(el: HTMLElement, ctx: TinyFxContext) {
  const inst = new {{Name}}(el, ctx);
  inst.init();
  return inst;
}