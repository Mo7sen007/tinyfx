import { effect } from "../core";

export function bindText(el: HTMLElement, get: () => any) {
  effect(() => {
    el.textContent = String(get());
  });
}
