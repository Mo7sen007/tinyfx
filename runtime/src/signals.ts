// @tinyfx/runtime — signals
// Minimal, explicit reactivity — no proxies, no magic.

type Subscriber = () => void;

const effectStack: Subscriber[] = [];

/**
 * Reactive value container used by TinyFX signals.
 *
 * Reading the value through {@link Signal.get} during an active effect records
 * that effect as a subscriber. Writing through {@link Signal.set} re-runs the
 * affected subscribers.
 *
 * @example
 * const count = new Signal(0);
 * console.log(count.get());
 * count.set(1);
 * console.log(count.get());
 */
export class Signal<T> {
  private value: T;
  private subs = new Set<Subscriber>();

  constructor(v: T) {
    this.value = v;
  }

  get(): T {
    const running = effectStack[effectStack.length - 1];
    if (running) {
      this.subs.add(running);
    }
    return this.value;
  }

  set(v: T) {
    if (Object.is(v, this.value)) return;
    this.value = v;
    this.subs.forEach((fn) => fn());
  }
}

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
  const s = new Signal(v);
  const fn = () => s.get();
  fn.set = (v: T) => s.set(v);
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
