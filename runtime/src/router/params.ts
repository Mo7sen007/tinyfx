// @tinyfx/runtime — route param resolution

/** Module-level store for the current page's resolved params. */
let currentParams: Record<string, string> = {};

/**
 * Match `pattern` (e.g. "/blog/:slug") against `pathname` (e.g. "/blog/hello").
 *
 * Returns true if they match, and stores any `:param` captures in the
 * module-level store so `getParam()` can retrieve them.
 *
 * Static segments must match exactly; `:param` segments match any non-empty
 * path segment.
 *
 * @param pattern - The route pattern produced by the compiler
 * @param pathname - The current browser pathname
 * @returns `true` when the pathname matches the pattern, otherwise `false`
 *
 * @example
 * resolveParams("/blog/:slug", "/blog/hello-world");
 */
export function resolveParams(pattern: string, pathname: string): boolean {
  const patternParts = pattern.split("/").filter(Boolean);
  const pathParts = pathname.split("/").filter(Boolean);

  if (patternParts.length !== pathParts.length) return false;

  const captured: Record<string, string> = {};

  for (let i = 0; i < patternParts.length; i++) {
    const seg = patternParts[i];
    if (seg.startsWith(":")) {
      // Dynamic segment — capture the value
      captured[seg.slice(1)] = decodeURIComponent(pathParts[i]);
    } else if (seg !== pathParts[i]) {
      // Static segment mismatch
      return false;
    }
  }

  currentParams = captured;
  return true;
}

/**
 * Retrieve a URL parameter resolved for the current page.
 *
 * @param key - The route parameter name to read
 * @returns The decoded parameter value, or `undefined` when it is missing
 *
 * @example
 * // On the page with route "/blog/:slug" and URL "/blog/hello-world"
 * getParam("slug"); // → "hello-world"
 */
export function getParam(key: string): string | undefined {
  return currentParams[key];
}

/**
 * Returns a shallow copy of all currently resolved route params.
 *
 * @returns An object containing the current route parameters
 *
 * @example
 * const params = getParams();
 * console.log(params.slug);
 */
export function getParams(): Record<string, string> {
  return { ...currentParams };
}
