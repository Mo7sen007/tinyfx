import { onMount, onDestroy } from "@tinyfx/runtime";
import type { TinyFxContext } from "@tinyfx/runtime";

export function init(_el: HTMLElement, _ctx: TinyFxContext): void {
  onMount(() => {
    console.log("index page mounted");
  });

  onDestroy(() => {
    // cleanup: remove event listeners, cancel requests, etc.
  });
}
