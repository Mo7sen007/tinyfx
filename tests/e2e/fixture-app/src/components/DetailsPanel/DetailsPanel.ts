import type { TinyFxContext } from "@tinyfx/runtime";

declare global {
  interface Window {
    __detailsMounts?: number;
  }
}

export class DetailsPanel {
  constructor(public el: HTMLElement, public ctx: TinyFxContext) {}

  init(): void {
    window.__detailsMounts = (window.__detailsMounts ?? 0) + 1;
    const counter = this.el.querySelector<HTMLElement>("#details-mounted-count");
    if (counter) {
      counter.textContent = `mount-count: ${window.__detailsMounts}`;
    }
  }
}

export function mount(el: HTMLElement, ctx: TinyFxContext) {
  const inst = new DetailsPanel(el, ctx);
  inst.init();
  return inst;
}
