import type { TinyFxContext } from "@tinyfx/runtime";

export class AlertTag {
  constructor(public el: HTMLElement, public ctx: TinyFxContext) {}

  init(): void {
    const marker = this.el.querySelector<HTMLElement>(".alert-tag-mounted");
    if (!marker) return;
    marker.dataset.mounted = "true";
    marker.textContent = "mounted";
  }
}

export function mount(el: HTMLElement, ctx: TinyFxContext) {
  const inst = new AlertTag(el, ctx);
  inst.init();
  return inst;
}
