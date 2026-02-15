// Button component (class-based)

import { signal,effect } from "@tinyfx/runtime";
import type { TinyFxContext } from "@tinyfx/runtime/di";

export class Button {
  public clicks = signal(0);
  public incBtn = document.getElementById("inc")!;

  constructor(public el: HTMLElement, public ctx: TinyFxContext) {}

  init(): void {
    // mounted behaviour
    console.log(`${this.constructor.name} mounted`, this.el);

    if (this.incBtn) {
      this.incBtn.addEventListener("click", () => {
        this.clicks.set(this.clicks() + 1);
      });
    }

    effect(() => {
      const el = document.getElementById("output");
      if (el) {
        el.textContent = `Count: ${this.clicks()}`;
      }
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