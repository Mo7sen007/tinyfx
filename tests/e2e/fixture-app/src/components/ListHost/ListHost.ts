import { signal } from "@tinyfx/runtime";
import type { TinyFxContext } from "@tinyfx/runtime";

type Item = {
  id: number;
  label: string;
};

export class ListHost {
  public items = signal<Item[]>([
    { id: 1, label: "Alpha" },
    { id: 2, label: "Beta" },
  ]);

  constructor(public el: HTMLElement, public ctx: TinyFxContext) {}

  init(): void {
    this.el.querySelector("#add-item")?.addEventListener("click", () => {
      const current = this.items();
      const nextId = current.length > 0 ? Math.max(...current.map((x) => x.id)) + 1 : 1;
      this.items.set([...current, { id: nextId, label: `Item ${nextId}` }]);
    });

    this.el.querySelector("#remove-item")?.addEventListener("click", () => {
      const current = this.items();
      if (current.length === 0) return;
      this.items.set(current.slice(0, -1));
    });
  }
}

export function mount(el: HTMLElement, ctx: TinyFxContext) {
  const inst = new ListHost(el, ctx);
  inst.init();
  return inst;
}
