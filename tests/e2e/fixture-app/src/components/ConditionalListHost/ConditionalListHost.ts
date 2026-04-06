import { signal } from "@tinyfx/runtime";
import type { TinyFxContext } from "@tinyfx/runtime";

type ConditionalItem = {
  id: number;
  label: string;
  status: string;
};

export class ConditionalListHost {
  public showList = signal(true);

  public allItems = signal<ConditionalItem[]>([
    { id: 1, label: "Row One", status: "ready" },
    { id: 2, label: "Row Two", status: "waiting" },
  ]);

  public visibleItems = signal<ConditionalItem[]>(this.allItems());

  constructor(public el: HTMLElement, public ctx: TinyFxContext) {}

  init(): void {
    this.el.querySelector("#conditional-toggle-list")?.addEventListener("click", () => {
      const nextShowList = !this.showList();
      this.showList.set(nextShowList);
      this.visibleItems.set(nextShowList ? this.allItems() : []);
    });

    this.el.querySelector("#conditional-add-item")?.addEventListener("click", () => {
      const current = this.allItems();
      const nextId = current.length > 0 ? Math.max(...current.map((item) => item.id)) + 1 : 1;
      const nextItems = [
        ...current,
        {
          id: nextId,
          label: `Row ${nextId}`,
          status: nextId % 2 === 0 ? "ready" : "waiting",
        },
      ];
      this.allItems.set(nextItems);
      if (this.showList()) {
        this.visibleItems.set(nextItems);
      }
    });
  }
}

export function mount(el: HTMLElement, ctx: TinyFxContext) {
  const inst = new ConditionalListHost(el, ctx);
  inst.init();
  return inst;
}
