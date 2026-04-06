import type { TinyFxContext } from "@tinyfx/runtime";

export class GroupBadge {
  constructor(public el: HTMLElement, public ctx: TinyFxContext) {}

  init(): void {
    const marker = this.el.querySelector<HTMLElement>(".group-badge-mounted");
    if (!marker) return;
    marker.dataset.mounted = "true";
    marker.textContent = "mounted";
  }
}

export function mount(el: HTMLElement, ctx: TinyFxContext) {
  const inst = new GroupBadge(el, ctx);
  inst.init();
  return inst;
}
