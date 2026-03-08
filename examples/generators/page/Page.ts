// {{Name}} page

import { onMount, onDestroy } from "@tinyfx/runtime";
import type { TinyFxContext } from "@tinyfx/runtime";

export function init(el: HTMLElement, ctx: TinyFxContext): void {
  onMount(() => {
    console.log("{{Name}} page mounted", el);
    // setup: read ctx.params, bind state, initialize components, etc.
    void ctx;
  });

  onDestroy(() => {
    // teardown: remove event listeners, cancel requests, etc.
  });
}
