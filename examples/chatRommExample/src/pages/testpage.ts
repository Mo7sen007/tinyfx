// Testpage page

import { onMount, onDestroy } from "@tinyfx/runtime";
import type { TinyFxContext } from "@tinyfx/runtime";

export function init(_el: HTMLElement, ctx: TinyFxContext): void {
  onMount(() => {
    console.log("Testpage page mounted");
    // setup: read ctx.params, bind state, initialize components, etc.
  });

  onDestroy(() => {
    // teardown: remove event listeners, cancel requests, etc.
  });
}
