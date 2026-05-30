// @tinyfx/runtime — each lifecycle helpers

const __componentInstances = new WeakMap<HTMLElement, { destroy?: () => void }>();

/**
 * Register a mounted component instance by root element.
 */
export function registerInstance(
  el: HTMLElement,
  instance: { destroy?: () => void },
): void {
  __componentInstances.set(el, instance);
}

/**
 * Destroy a mounted component instance on a node and nested component nodes.
 */
export function destroyNode(el: HTMLElement): void {
  // Snapshot children before destroying the parent — parent's destroy() may remove
  // them from the DOM, causing a subsequent querySelectorAll to find nothing.
  // Reverse for bottom-up order (deepest descendants destroyed first).
  const children = Array.from(
    el.querySelectorAll<HTMLElement>("[data-component]"),
  ).reverse();

  children.forEach((childEl) => {
    __componentInstances.get(childEl)?.destroy?.();
    __componentInstances.delete(childEl);
  });

  __componentInstances.get(el)?.destroy?.();
  __componentInstances.delete(el);
}
