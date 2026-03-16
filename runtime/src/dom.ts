// @tinyfx/runtime — DOM helpers
// Helpers for binding reactive state to the DOM.

import { effect } from "./signals";

/**
 * Binds an element's text content to a reactive getter.
 *
 * @param el - The DOM element whose text content should be updated
 * @param get - A reactive getter that returns the next text value
 * @returns Nothing
 *
 * @example
 * const count = signal(0);
 * bindText(span, () => `Count: ${count()}`);
 */
export function bindText(el: HTMLElement, get: () => any) {
  effect(() => {
    el.textContent = String(get());
  });
}

/**
 * Binds an attribute to a reactive getter.
 *
 * @param el - The DOM element whose attribute should be updated
 * @param attr - The attribute name to write
 * @param get - A reactive getter that returns the attribute value
 * @returns Nothing
 *
 * @example
 * const isDark = signal(false);
 * bindAttr(document.body, "data-theme", () => (isDark() ? "dark" : "light"));
 */
export function bindAttr(el: HTMLElement, attr: string, get: () => string) {
  effect(() => {
    el.setAttribute(attr, get());
  });
}

/**
 * Toggles a CSS class from a reactive boolean getter.
 *
 * @param el - The DOM element whose class list should be updated
 * @param cls - The class name to toggle
 * @param get - A reactive getter that returns whether the class should exist
 * @returns Nothing
 *
 * @example
 * const open = signal(false);
 * bindClass(panel, "is-open", () => open());
 */
export function bindClass(el: HTMLElement, cls: string, get: () => boolean) {
  effect(() => {
    el.classList.toggle(cls, get());
  });
}
