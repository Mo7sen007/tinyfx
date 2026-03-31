// @tinyfx/runtime — mount state tracking
// WeakSet-based mount tracking that is safe for minifier mangling.

const mounted = new WeakSet<HTMLElement>();

/**
 * Mark an element as mounted. Returns false if already mounted.
 *
 * @param el - The element to mark
 * @returns true if newly marked, false if already mounted
 *
 * @example
 * if (!markMounted(el)) return undefined;
 */
export function markMounted(el: HTMLElement): boolean {
  if (mounted.has(el)) return false;
  mounted.add(el);
  return true;
}

/**
 * Check if an element has been marked as mounted.
 *
 * @param el - The element to check
 * @returns true if the element is marked as mounted
 */
export function isMounted(el: HTMLElement): boolean {
  return mounted.has(el);
}
