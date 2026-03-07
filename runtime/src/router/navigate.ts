// @tinyfx/runtime — navigation helpers
// Hard router: navigation is always a real browser page load.

/**
 * Navigate to the given path.
 * This is a full browser navigation — not a DOM swap.
 *
 * @example
 * navigate("/blog/hello-world");
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
