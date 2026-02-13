// {{Name}} page

import type { TinyFxContext } from "@tinyfx/runtime/di";

export class {{Name}}Page {
  constructor(public el: HTMLElement, public ctx: TinyFxContext) {}

  init(): void {
    console.log("{{Name}}Page mounted", this.el);
  }

  destroy?(): void {
    // optional teardown
  }
}

// Factory used by tinyfx.gen.ts
export function mount(el: HTMLElement, ctx: TinyFxContext) {
  const inst = new {{Name}}Page(el, ctx);
  inst.init();
  return inst;
}
