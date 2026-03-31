// {{Name}} component

import type { TinyFxContext } from "@tinyfx/runtime";

export class {{Name}} {
  // Reactive fields — initialize immediately, never use definite assignment (!)
  // public count = signal(0);

  constructor(public el: HTMLElement, public ctx: TinyFxContext) {}

  init(): void {
    console.log("{{Name}} mounted", this.el);
    // setup: add event listeners, bind state to DOM elements, etc.
  }

  destroy?(): void {
    // teardown: remove event listeners, cancel requests, etc.
  }
}

export function mount(el: HTMLElement, ctx: TinyFxContext) {
  const inst = new {{Name}}(el, ctx);
  inst.init();
  return inst;
}