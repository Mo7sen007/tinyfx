// @tinyfx/runtime — unified initialization
// Single entry point for TinyFX bootstrap.

import type { TinyFxContext } from "./context";
import type { RouteDef } from "./router/path-matcher";
import { matchPath, splitPath } from "./router/path-matcher";
import { initLifecycle } from "./router/lifecycle";
import { highlightActiveLinks } from "./router/active-links";
import { navigate } from "./router/index";
import { setParams } from "./router/params";
import { mountComponents } from "./registry";
import { runPageInit } from "./page-registry";

export interface InitConfig {
  routes: Record<string, RouteDef>;
  setupDirectives?: (ctx: TinyFxContext) => void;
}

export interface InitResult {
  matchedPath: string;
  params: Record<string, string>;
  instances: any[];
}

let initialized = false;

export function init(config: InitConfig): InitResult | null {
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
    console.warn(
      `[tinyfx] No route matched for pathname: "${pathname}". ` +
      "Check that a page file exists for this URL in src/pages/."
    );
    return null;
  }

  initLifecycle();
  highlightActiveLinks(pathname);

  setParams(params);

  const ctx: TinyFxContext = { params, navigate };
  const instances = mountComponents(ctx);
  if (config.setupDirectives) {
    config.setupDirectives(ctx);
  }
  runPageInit(matchedPath, ctx);

  return { matchedPath, params, instances };
}
