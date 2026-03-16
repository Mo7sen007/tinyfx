// @tinyfx/runtime — router types

/**
 * Metadata for a single route produced by the compiler.
 *
 * @example
 * const route: RouteMeta = { page: "blog-post", path: "/blog/:slug" };
 */
export interface RouteMeta {
  /** The page name, derived from the source filename. */
  page: string;
  /** The declared URL pattern, e.g. "/blog/:slug". */
  path: string;
}

/**
 * The full route map emitted by `tinyfx build` into `tinyfx.gen.ts`.
 *
 * @example
 * const routes: RouteMap = {
 *   "/": { page: "index", path: "/" },
 *   "/blog/:slug": { page: "blog-post", path: "/blog/:slug" },
 * };
 */
export type RouteMap = Record<string, RouteMeta>;
