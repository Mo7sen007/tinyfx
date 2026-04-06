import { signal } from "@tinyfx/runtime";
import type { TinyFxContext } from "@tinyfx/runtime";

type Member = {
  id: number;
  label: string;
};

type Group = {
  id: number;
  name: string;
};

export class NestedListHost {
  public showMembers = signal(true);

  public members = signal<Member[]>([
    { id: 11, label: "A-One" },
    { id: 12, label: "A-Two" },
    { id: 21, label: "B-One" },
  ]);

  public groups = signal<Group[]>([
    {
      id: 1,
      name: "Team A",
    },
  ]);

  constructor(public el: HTMLElement, public ctx: TinyFxContext) {}

  init(): void {
    this.el.querySelector("#nested-add-inner")?.addEventListener("click", () => {
      const current = this.members();
      const nextId = current.length > 0 ? Math.max(...current.map((member) => member.id)) + 1 : 11;
      this.members.set([...current, { id: nextId, label: `A-${nextId}` }]);
    });

    this.el.querySelector("#nested-toggle-members")?.addEventListener("click", () => {
      this.showMembers.set(!this.showMembers());
    });

    this.el.querySelector("#nested-clear-members")?.addEventListener("click", () => {
      this.members.set([]);
    });
  }
}

export function mount(el: HTMLElement, ctx: TinyFxContext) {
  const inst = new NestedListHost(el, ctx);
  inst.init();
  return inst;
}
