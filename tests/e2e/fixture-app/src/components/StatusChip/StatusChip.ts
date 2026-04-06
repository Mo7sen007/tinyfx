import type { TinyFxContext } from "@tinyfx/runtime";

export class StatusChip {
  constructor(public el: HTMLElement, public ctx: TinyFxContext) {}

  init(): void {
    const marker = this.el.querySelector<HTMLElement>(".status-chip-mounted");
    if (!marker) return;
    marker.dataset.mounted = "true";
    marker.textContent = "mounted";
  }
}

export function mount(el: HTMLElement, ctx: TinyFxContext) {
  const inst = new StatusChip(el, ctx);
  inst.init();
  return inst;
}
