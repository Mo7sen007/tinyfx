// @tinyfx/runtime — page module registry
// Decoupled page module registration for tree-shakable bundles.

import type { TinyFxContext } from "./context";

type PageModule = {
  init?: (el: HTMLElement, ctx: TinyFxContext) => void;
};

const modules = new Map<string, PageModule>();

/**
 * Register a page module by its route path.
 *
 * @param path - The route path (e.g., "/blog/:slug")
 * @param module - The page module object with optional init()
 * @returns Nothing
 *
 * @example
 * registerPage("/blog/:slug", PageModule);
 */
export function registerPage(path: string, module: PageModule): void {
  modules.set(path, module);
}

/**
 * Retrieve a registered page module.
 *
 * @param path - The route path to look up
 * @returns The page module, or undefined if not registered
 */
export function getPageModule(path: string): PageModule | undefined {
  return modules.get(path);
}

/**
 * Run the init() function for a registered page module.
 *
 * @param path - The route path of the page to initialize
 * @param ctx - The TinyFx context to pass to page init
 * @returns Nothing
 */
export function runPageInit(path: string, ctx: TinyFxContext): void {
  const mod = modules.get(path);
  if (typeof mod?.init === "function") {
    mod.init(document.body, ctx);
  }
}
