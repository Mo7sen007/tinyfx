// @tinyfx/runtime — router types

/** Metadata for a single route, produced by the compiler. */
export interface RouteMeta {
  /** The page name, derived from the source filename. */
  page: string;
  /** The declared URL pattern, e.g. "/blog/:slug". */
  path: string;
}

/** The full route map emitted by `tinyfx build` into tinyfx.gen.ts. */
export type RouteMap = Record<string, RouteMeta>;
