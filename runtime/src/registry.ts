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
 * Mount all registered components found within a root element.
 *
 * @param ctx - The TinyFx context to pass to each component
 * @param root - The root element to scan (defaults to document)
 * @returns Array of mounted component instances
 */
export function mountComponents(
  ctx: TinyFxContext,
  root: Document | HTMLElement = document,
): any[] {
  const instances: any[] = [];
  root.querySelectorAll<HTMLElement>("[data-component]").forEach((el) => {
    const name = el.getAttribute("data-component");
    if (!name) return;
    const factory = factories.get(name);
    if (!factory) return;
    const inst = factory(el, ctx);
    if (inst) instances.push(inst);
  });
  return instances;
}
