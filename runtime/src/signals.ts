// @tinyfx/runtime — signals
// Minimal, explicit reactivity — no proxies, no magic.

type Subscriber = () => void;

const effectStack: Subscriber[] = [];

/**
 * Creates a reactive signal function.
 *
 * @param v - The initial value stored by the signal
 * @returns A callable signal that reads the current value and exposes `.set()`
 *          to update it
 *
 * @example
 * const count = signal(0);
 * console.log(count());
 * count.set(1);
 */
export function signal<T>(v: T) {
  let value = v;
  const subs = new Set<Subscriber>();

  const fn = () => {
    const running = effectStack[effectStack.length - 1];
    if (running) subs.add(running);
    return value;
  };

  fn.set = (next: T) => {
    if (Object.is(next, value)) return;
    value = next;
    subs.forEach((s) => s());
  };

  return fn as (() => T) & { set(v: T): void };
}

/**
 * Registers and runs a reactive effect.
 *
 * Every signal read while `fn` runs is tracked, and the effect is re-run when
 * any of those signal values change.
 *
 * @param fn - The reactive callback to execute and subscribe
 * @returns Nothing
 *
 * @example
 * const count = signal(0);
 * effect(() => {
 *   console.log(`count = ${count()}`);
 * });
 * count.set(1);
 */
export function effect(fn: () => void) {
  const run = () => {
    effectStack.push(run);
    try {
      fn();
    } finally {
      effectStack.pop();
    }
  };
  run();
}
