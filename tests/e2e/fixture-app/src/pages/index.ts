import { onMount } from "@tinyfx/runtime";
import type { TinyFxContext } from "@tinyfx/runtime";

export function init(_el: HTMLElement, _ctx: TinyFxContext): void {
  onMount(() => {
    console.log("E2E test page mounted");
  });
}
