// @tinyfx/runtime — component registry
// Decoupled component registration for tree-shakable bundles.

import type { TinyFxContext } from "./context";

type ComponentFactory = (el: HTMLElement, ctx: TinyFxContext) => any;

const factories = new Map<string, ComponentFactory>();

/**
 * Register a component factory by name.
 *
 * @param name - The component name (matches `data-component` attribute)
 * @param factory - A function that mounts the component and returns an instance
 * @returns Nothing
 *
 * @example
 * registerComponent("Button", (el, ctx) => {
 *   return ButtonModule.mount(el, ctx);
 * });
 */
export function registerComponent(name: string, factory: ComponentFactory): void {
  factories.set(name, factory);
}

/**
 * Retrieve a registered component factory.
 *
 * @param name - The component name to look up
 * @returns The factory function, or undefined if not registered
 */
export function getComponentFactory(name: string): ComponentFactory | undefined {
  return factories.get(name);
}

/**
 * Mount a single component by name on a specific element.
 *
 * This is the primary mounting primitive. The generated code uses this
 * internally — you should only call it directly when inserting HTML
 * dynamically at runtime (e.g. after fetching a fragment from the server).
 *
 * @param name - The component name (matches `data-component` attribute)
 * @param el - The element that has `data-component="${name}"`
 * @param ctx - The TinyFx context
 * @returns The mounted instance, or null if the factory is not registered
 *
 * @example
 * // After inserting HTML that contains <div data-component="Chat">
 * const chatEl = container.querySelector<HTMLElement>("[data-component='Chat']");
 * if (chatEl) mountComponentByName("Chat", chatEl, ctx);
 */
export function mountComponentByName(
  name: string,
  el: HTMLElement,
  ctx: TinyFxContext,
): any | null {
  const factory = factories.get(name);
  if (!factory) return null;
  return factory(el, ctx);
}

/**
 * Mount all known dynamic components found within a subtree.
 *
 * This is an escape hatch for runtime-inserted HTML. The compiler generates
 * direct mount calls for page-level components — you only need this when you
 * inject HTML after the initial page load (e.g. via innerHTML or a fetch
 * response) and want TinyFX to mount any `[data-component]` elements inside.
 *
 * @param root - The root element to scan
 * @param ctx - The TinyFx context
 * @param names - Optional list of component names to look for. When omitted,
 *                all registered components are scanned. Providing names makes
 *                the query more targeted.
 * @returns Array of mounted component instances
 *
 * @example
 * // Mount all components in a freshly inserted fragment
 * container.innerHTML = serverHtml;
 * mountComponentsIn(container, ctx);
 *
 * @example
 * // Mount only specific components (more efficient)
 * mountComponentsIn(container, ctx, ["Chat", "UserProfile"]);
 */
export function mountComponentsIn(
  root: HTMLElement,
  ctx: TinyFxContext,
  names?: string[],
): any[] {
  const instances: any[] = [];
  const selector = names
    ? names.map(n => `[data-component="${n}"]`).join(",")
    : "[data-component]";

  root.querySelectorAll<HTMLElement>(selector).forEach((el) => {
    const name = el.getAttribute("data-component");
    if (!name) return;
    const factory = factories.get(name);
    if (!factory) return;
    const inst = factory(el, ctx);
    if (inst) instances.push(inst);
  });
  return instances;
}

/**
 * @deprecated Use generated mount calls instead.
 *
 * This function scans the DOM at runtime to discover components. The compiler
 * now generates direct mount calls for every known component, making this
 * discovery pass redundant. It is kept only for edge cases where you need to
 * manually mount components outside the normal build pipeline.
 *
 * For runtime-inserted HTML prefer `mountComponentsIn()`.
 *
 * @param ctx - The TinyFx context
 * @param root - The root element to scan (defaults to document)
 * @returns Array of mounted component instances
 */
export function mountComponents(
  ctx: TinyFxContext,
  root: Document | HTMLElement = document,
): any[] {
  return mountComponentsIn(root as HTMLElement, ctx);
}
