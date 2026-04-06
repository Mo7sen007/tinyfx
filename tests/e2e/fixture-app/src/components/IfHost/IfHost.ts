import { signal } from "@tinyfx/runtime";
import type { TinyFxContext } from "@tinyfx/runtime";

export class IfHost {
  public showDetails = signal(false);

  constructor(public el: HTMLElement, public ctx: TinyFxContext) {}

  init(): void {
    this.el.querySelector("#toggle-details")?.addEventListener("click", () => {
      this.showDetails.set(!this.showDetails());
    });
  }
}

export function mount(el: HTMLElement, ctx: TinyFxContext) {
  const inst = new IfHost(el, ctx);
  inst.init();
  return inst;
}
