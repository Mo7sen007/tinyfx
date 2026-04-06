// @tinyfx/runtime — unified initialization
// Single entry point for TinyFX bootstrap.

import type { TinyFxContext } from "./context";
import type { RouteDef } from "./router/path-matcher";
import { matchPath, splitPath } from "./router/path-matcher";
import { initLifecycle } from "./router/lifecycle";
import { highlightActiveLinks } from "./router/active-links";
import { navigate } from "./router/index";
import { setParams } from "./router/params";
import { runPageInit } from "./page-registry";

declare const __DEV__: boolean;

export interface InitConfig {
  routes: Record<string, RouteDef>;
  setupDirectives?: (ctx: TinyFxContext) => void;
}

let initialized = false;

/**
 * Bootstrap the application for the current URL.
 *
 * Matches the pathname against registered routes, initializes lifecycle hooks,
 * highlights active links, sets route params, runs directive setup, and calls
 * page-level init().
 *
 * Component mounting is handled by the generated code — `init()` returns a
 * `TinyFxContext` that the generated bootstrap uses to mount components
 * directly.
 *
 * @param config - Route map and optional directive setup
 * @returns A TinyFxContext for component mounting, or null if no route matched
 */
export function init(config: InitConfig): TinyFxContext | null {
  if (initialized) return null;
  initialized = true;

  const pathname = window.location.pathname;
  const pathSegments = splitPath(pathname);

  let matchedPath: string | null = null;
  let params: Record<string, string> = {};

  for (const [path, def] of Object.entries(config.routes)) {
    const result = matchPath(def, pathSegments);
    if (result) {
      matchedPath = path;
      params = result;
      break;
    }
  }

  if (!matchedPath) {
    if (__DEV__) {
      console.warn(
        `[tinyfx] No route matched for pathname: "${pathname}". ` +
        "Check that a page file exists for this URL in src/pages/."
      );
    }
    return null;
  }

  initLifecycle();
  highlightActiveLinks(pathname);

  setParams(params);

  const ctx: TinyFxContext = { params, navigate };
  if (config.setupDirectives) {
    config.setupDirectives(ctx);
  }
  runPageInit(matchedPath, ctx);

  return ctx;
}
