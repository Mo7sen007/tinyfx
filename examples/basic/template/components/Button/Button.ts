// Optional component behavior.
// This runs at runtime for each <data-component="Button"> element.

import type { TinyFxContext } from "@tinyfx/runtime/di";

export function init(el: HTMLElement, ctx: TinyFxContext) {
  el.addEventListener("click", () => {
    console.log("Button clicked:", el.textContent?.trim());
  });
}
