// @tinyfx/runtime — router utilities

/**
 * Metadata for a single route produced by the compiler.
 */
export interface RouteMeta {
  /** The page name, derived from the source filename. */
  page: string;
  /** The declared URL pattern, e.g. "/blog/:slug". */
  path: string;
}

/**
 * The full route map emitted by `tinyfx build` into `tinyfx.gen.ts`.
 */
export type RouteMap = Record<string, RouteMeta>;

/**
 * Navigate to the given path.
 * This is a full browser navigation — not a DOM swap.
 */
export function navigate(path: string): void {
  window.location.href = path;
}

/**
 * Go back one step in the browser history.
 */
export function goBack(): void {
  window.history.go(-1);
}

export { getParam, getParams } from "./params";
