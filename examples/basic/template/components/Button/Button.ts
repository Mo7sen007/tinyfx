// Button component (class-based)

import { signal } from "@tinyfx/runtime";
import type { TinyFxContext } from "@tinyfx/runtime";

export class Button {
  constructor(public el: HTMLElement, public ctx: TinyFxContext) {}
  public clicks = signal(0);

  init(): void {
    console.log("Button mounted", this.el);
    this.el.addEventListener("click", () => {
      this.clicks.set(this.clicks() + 1);
      console.log("Button clicked", this.clicks());
    });
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