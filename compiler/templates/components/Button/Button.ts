// Button component

import { signal } from "@tinyfx/runtime";
import type { TinyFxContext } from "@tinyfx/runtime";

export class Button {
  constructor(public el: HTMLElement, public ctx: TinyFxContext) {}

  init(): void {
    console.log("Button mounted", this.el);
    void signal;

    this.el.addEventListener("click", () => {
      console.log("Button clicked");
    });
  }

  destroy?(): void {
    // teardown: remove event listeners, cancel requests, etc.
  }
}

export function mount(el: HTMLElement, ctx: TinyFxContext) {
  const inst = new Button(el, ctx);
  inst.init();
  return inst;
}
