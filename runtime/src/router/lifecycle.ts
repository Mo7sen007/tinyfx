// @tinyfx/runtime — page lifecycle hooks

type LifecycleCallback = () => void;

const mountCallbacks: LifecycleCallback[] = [];
const destroyCallbacks: LifecycleCallback[] = [];

/**
 * Register a callback to run when the page has mounted (DOM ready).
 *
 * - If the document is still loading, the callback runs after DOMContentLoaded.
 * - If the document is already loaded, the callback runs immediately.
 */
export function onMount(fn: LifecycleCallback): void {
  mountCallbacks.push(fn);
}

/**
 * Register a callback to run when the user navigates away from this page.
 * Fires on the browser's `pagehide` event.
 */
export function onDestroy(fn: LifecycleCallback): void {
  destroyCallbacks.push(fn);
}

/**
 * Trigger all registered `onMount` callbacks.
 * Called by `initRouter` after matching the current route.
 */
export function flushMount(): void {
  const run = () => mountCallbacks.forEach((fn) => fn());

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", run, { once: true });
  } else {
    run();
  }
}

/**
 * Wire up registered `onDestroy` callbacks to the `pagehide` event.
 * Called by `initRouter` after matching the current route.
 */
export function initLifecycle(): void {
  flushMount();

  window.addEventListener(
    "pagehide",
    () => destroyCallbacks.forEach((fn) => fn()),
    { once: true }
  );
}
