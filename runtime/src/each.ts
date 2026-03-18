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
  const inst = __componentInstances.get(el);
  inst?.destroy?.();
  __componentInstances.delete(el);

  el.querySelectorAll("[data-component]").forEach((child) => {
    const childEl = child as HTMLElement;
    const childInst = __componentInstances.get(childEl);
    childInst?.destroy?.();
    __componentInstances.delete(childEl);
  });
}
