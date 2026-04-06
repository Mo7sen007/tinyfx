import type { TinyFxContext } from "@tinyfx/runtime";

export class ListItem {
  constructor(public el: HTMLElement, public ctx: TinyFxContext) {}

  init(): void {
    const marker = this.el.querySelector<HTMLElement>(".list-item-mounted");
    if (!marker) return;
    marker.dataset.mounted = "true";
    marker.textContent = "mounted";
  }
}

export function mount(el: HTMLElement, ctx: TinyFxContext) {
  const inst = new ListItem(el, ctx);
  inst.init();
  return inst;
}
