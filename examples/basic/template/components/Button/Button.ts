// Button component

import type { TinyFxContext } from "@tinyfx/runtime";

export class Button {
  constructor(public el: HTMLElement, public ctx: TinyFxContext) {}

  init(): void {
    console.log("Button mounted", this.el);
  }

  destroy?(): void {
    // optional teardown
  }
}

// convenience mount helper used by tinyfx.gen.ts
export function mount(el: HTMLElement, ctx: TinyFxContext) {
  const inst = new Button(el, ctx);
  inst.init();
  return inst;
}