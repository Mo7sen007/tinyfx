// @tinyfx/runtime — page lifecycle hooks

type LifecycleCallback = () => void;

const mountCallbacks: LifecycleCallback[] = [];
const destroyCallbacks: LifecycleCallback[] = [];

/**
 * Register a callback to run when the page has mounted (DOM ready).
 *
 * - If the document is still loading, the callback runs after DOMContentLoaded.
 * - If the document is already loaded, the callback runs immediately.
 *
 * @param fn - The callback to run on mount
 * @returns Nothing
 *
 * @example
 * onMount(() => {
 *   console.log("page ready");
 * });
 */
export function onMount(fn: LifecycleCallback): void {
  if (document.readyState === "loading") {
    mountCallbacks.push(fn);
    return;
  }

  fn();
}

/**
 * Register a callback to run when the user navigates away from this page.
 * Fires on the browser's `pagehide` event.
 *
 * @param fn - The callback to run during page teardown
 * @returns Nothing
 *
 * @example
 * onDestroy(() => {
 *   console.log("page leaving");
 * });
 */
export function onDestroy(fn: LifecycleCallback): void {
  destroyCallbacks.push(fn);
}

/**
 * Trigger all registered `onMount` callbacks.
 * Called by `init()` after matching the current route.
 *
 * @returns Nothing
 *
 * @example
 * flushMount();
 */
export function flushMount(): void {
  const run = () => {
    mountCallbacks.forEach((fn) => fn());
    mountCallbacks.length = 0;
  };

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", run, { once: true });
  } else {
    run();
  }
}

/**
 * Wire up registered `onDestroy` callbacks to the `pagehide` event.
 * Called by `init()` after matching the current route.
 *
 * @returns Nothing
 *
 * @example
 * initLifecycle();
 */
export function initLifecycle(): void {
  flushMount();

  window.addEventListener(
    "pagehide",
    () => {
      destroyCallbacks.forEach((fn) => fn());
      destroyCallbacks.length = 0;
    },
    { once: true }
  );
}
