// @tinyfx/runtime — DOM helpers
// Helpers for binding reactive state to the DOM.

import { effect } from "./signals";

export function bindText(el: HTMLElement, get: () => any) {
  effect(() => {
    el.textContent = String(get());
  });
}

export function bindAttr(el: HTMLElement, attr: string, get: () => string) {
  effect(() => {
    el.setAttribute(attr, get());
  });
}

export function bindClass(el: HTMLElement, cls: string, get: () => boolean) {
  effect(() => {
    el.classList.toggle(cls, get());
  });
}
