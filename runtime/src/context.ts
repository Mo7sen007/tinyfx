/**
 * Lightweight page context passed to page and component behavior.
 *
 * TinyFX uses this to expose route params and navigation helpers without a
 * dependency-injection container.
 *
 * @example
 * import type { TinyFxContext } from "@tinyfx/runtime";
 *
 * export function init(el: HTMLElement, ctx: TinyFxContext): void {
 *   console.log(ctx.params.slug);
 *   ctx.navigate("/");
 *   void el;
 * }
 */
export interface TinyFxContext {
  params: Record<string, string>;
  navigate: (path: string) => void;
}