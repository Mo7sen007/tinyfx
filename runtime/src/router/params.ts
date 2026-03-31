// @tinyfx/runtime — route param resolution

/** Module-level store for the current page's resolved params. */
let currentParams: Record<string, string> = {};

/**
 * Set resolved route params (called internally by init()).
 *
 * @param params - The resolved URL parameters
 */
export function setParams(params: Record<string, string>): void {
  currentParams = params;
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
