import { signal } from "@tinyfx/runtime";
import type { TinyFxContext } from "@tinyfx/runtime";

type AlertRow = {
  id: number;
  label: string;
  alertText: string;
};

export class ListWithConditionalChildren {
  public showAlerts = signal(true);

  public items = signal<AlertRow[]>([
    {
      id: 1,
      label: "Item One",
      alertText: "warn one",
    },
    {
      id: 2,
      label: "Item Two",
      alertText: "warn two",
    },
    {
      id: 3,
      label: "Item Three",
      alertText: "warn three",
    },
  ]);

  constructor(public el: HTMLElement, public ctx: TinyFxContext) {}

  init(): void {
    this.el.querySelector("#list-conditional-toggle-alerts")?.addEventListener("click", () => {
      this.showAlerts.set(!this.showAlerts());
    });

    this.el.querySelector("#list-conditional-show-alerts")?.addEventListener("click", () => {
      this.showAlerts.set(true);
    });
  }
}

export function mount(el: HTMLElement, ctx: TinyFxContext) {
  const inst = new ListWithConditionalChildren(el, ctx);
  inst.init();
  return inst;
}
