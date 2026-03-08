// @tinyfx/runtime — router entry point

import type { RouteMap } from "./types";
import { navigate, goBack } from "./navigate";
import { getParam, resolveParams } from "./params";
import { initLifecycle } from "./lifecycle";
import { highlightActiveLinks } from "./active-links";

export type { RouteMap };
export { navigate, goBack, getParam };

/**
 * Initialise the TinyFX router for the current page.
 *
 * Called automatically from the compiler-generated `tinyfx.gen.ts` on every
 * page load.  Matches `window.location.pathname` against the compiled route
 * map, resolves URL params, starts lifecycle hooks, and highlights active
 * navigation links.
 *
 * @param routes - The route map emitted by `tinyfx build`.
 *
 * @example
 * // tinyfx.gen.ts (auto-generated — do not edit)
 * initRouter({
 *   "/":          { page: "home",      path: "/" },
 *   "/blog":      { page: "blog",      path: "/blog" },
 *   "/blog/:slug":{ page: "blog-post", path: "/blog/:slug" },
 * });
 */
export function initRouter(routes: RouteMap): void {
  const pathname = window.location.pathname;

  for (const pattern of Object.keys(routes)) {
    if (resolveParams(pattern, pathname)) {
      // Match found
      initLifecycle();
      highlightActiveLinks(pathname);
      return;
    }
  }

  // No route matched — log a warning and do nothing else.
  console.warn(
    `[tinyfx] No route matched for pathname: "${pathname}". ` +
    "Check that a page file exists for this URL in src/pages/."
  );
}
