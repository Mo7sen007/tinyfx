// @tinyfx/runtime — DOM helpers
// Helpers for binding reactive state to the DOM.
//
// Each binding function returns an {@link EffectHandle}. Calling it disposes
// the underlying effect, stopping further DOM updates. This is useful when
// tearing down a component or removing a binding at runtime.

import { effect, type EffectHandle } from "./signals";

/**
 * Binds an element's text content to a reactive getter.
 *
 * @param el - The DOM element whose text content should be updated
 * @param get - A reactive getter that returns the next text value
 * @returns An {@link EffectHandle} — calling it stops the binding
 *
 * @example
 * const count = signal(0);
 * const dispose = bindText(span, () => `Count: ${count()}`);
 * // later:
 * dispose();
 */
export function bindText(el: HTMLElement, get: () => unknown): EffectHandle {
  return effect(() => {
    el.textContent = String(get());
  });
}

/**
 * Binds an attribute to a reactive getter.
 *
 * @param el - The DOM element whose attribute should be updated
 * @param attr - The attribute name to write
 * @param get - A reactive getter that returns the attribute value
 * @returns An {@link EffectHandle} — calling it stops the binding
 *
 * @example
 * const isDark = signal(false);
 * const dispose = bindAttr(document.body, "data-theme", () => (isDark() ? "dark" : "light"));
 * // later:
 * dispose();
 */
export function bindAttr(el: HTMLElement, attr: string, get: () => string): EffectHandle {
  return effect(() => {
    el.setAttribute(attr, get());
  });
}

/**
 * Toggles a CSS class from a reactive boolean getter.
 *
 * @param el - The DOM element whose class list should be updated
 * @param cls - The class name to toggle
 * @param get - A reactive getter that returns whether the class should exist
 * @returns An {@link EffectHandle} — calling it stops the binding
 *
 * @example
 * const open = signal(false);
 * const dispose = bindClass(panel, "is-open", () => open());
 * // later:
 * dispose();
 */
export function bindClass(el: HTMLElement, cls: string, get: () => boolean): EffectHandle {
  return effect(() => {
    el.classList.toggle(cls, get());
  });
}
