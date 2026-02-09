// Optional component behavior for {{Name}}.

import type { TinyFxContext } from "@tinyfx/runtime/di";

export function init(el: HTMLElement, ctx: TinyFxContext) {
  // Access services via: const myService = ctx.container.inject(MyService);
  console.log("{{Name}} mounted", el);
}
