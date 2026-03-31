// @tinyfx/runtime — path matcher
// Unified path resolution with pre-split array support for compiler optimization.

export interface RouteDef {
  /** Static segment values (null for dynamic segments) */
  staticSegments: (string | null)[];
  /** Param names at dynamic positions */
  paramNames: string[];
}

/**
 * Match a pre-compiled route definition against path segments.
 *
 * @param def - The route definition from the compiler
 * @param pathSegments - Pre-split path segments from splitPath()
 * @returns Captured params on match, null on mismatch
 *
 * @example
 * const def = { staticSegments: ["blog", null], paramNames: ["slug"] };
 * matchPath(def, ["blog", "hello-world"]);
 * // Returns: { slug: "hello-world" }
 */
export function matchPath(
  def: RouteDef,
  pathSegments: string[],
): Record<string, string> | null {
  const { staticSegments, paramNames } = def;
  if (staticSegments.length !== pathSegments.length) return null;

  const params: Record<string, string> = {};
  let paramIdx = 0;

  for (let i = 0; i < staticSegments.length; i++) {
    const expected = staticSegments[i];
    const actual = pathSegments[i];
    if (expected === null) {
      params[paramNames[paramIdx++]] = decodeURIComponent(actual);
    } else if (expected !== actual) {
      return null;
    }
  }

  return params;
}

/**
 * Split a pathname into segments for matching.
 *
 * @param pathname - The browser pathname (e.g., "/blog/hello")
 * @returns Array of non-empty segments
 *
 * @example
 * splitPath("/blog/hello-world");
 * // Returns: ["blog", "hello-world"]
 */
export function splitPath(pathname: string): string[] {
  return pathname.split("/").filter(Boolean);
}
