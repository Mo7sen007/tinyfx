// @tinyfx/runtime — signals
// Minimal, explicit reactivity — no proxies, no magic.
//
// Effects track which signal subscriber sets they belong to. On each re-run
// they unsubscribe from the previous run's signals before re-subscribing.
// Calling the returned handle disposes the effect permanently.

type Subscriber = () => void;

/**
 * An effect handle. Calling it disposes the effect, unsubscribing it from
 * all signals it reads. After disposal the effect will never run again.
 *
 * @example
 * const count = signal(0);
 * const dispose = effect(() => console.log(count()));
 * // later, when the effect is no longer needed:
 * dispose();
 */
export type EffectHandle = () => void;

/**
 * Internal effect frame pushed onto the stack while an effect callback runs.
 * Tracks both the effect function itself and the set of signal subscriber
 * sets it has been added to, so it can unsubscribe on re-run or disposal.
 */
interface EffectFrame {
  run: Subscriber;
  subs: Set<Set<Subscriber>>;
}

const effectStack: EffectFrame[] = [];

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
    const current = effectStack[effectStack.length - 1];
    if (current) {
      subs.add(current.run);
      current.subs.add(subs);
    }
    return value;
  };

  fn.set = (next: T) => {
    if (Object.is(next, value)) return;
    value = next;
    [...subs].forEach((s) => s());
  };

  return fn as (() => T) & { set(v: T): void };
}

/**
 * Registers and runs a reactive effect.
 *
 * Every signal read while `fn` runs is tracked, and the effect is re-run when
 * any of those signal values change. On each re-run the effect first
 * unsubscribes from the signals it read in the previous run, then re-runs
 * `fn` and subscribes to the new set of signals.
 *
 * @param fn - The reactive callback to execute and subscribe
 * @returns An {@link EffectHandle} — calling it disposes the effect
 *          permanently, unsubscribing it from all signals.
 *
 * @example
 * const count = signal(0);
 * const dispose = effect(() => {
 *   console.log(`count = ${count()}`);
 * });
 * count.set(1);
 * // When done:
 * dispose();
 */
export function effect(fn: () => void): EffectHandle {
  const subs = new Set<Set<Subscriber>>();
  let disposed = false;

  const run = () => {
    if (disposed) return;
    // Unsubscribe from the previous run's signals
    for (const s of subs) s.delete(run);
    subs.clear();

    effectStack.push({ run, subs });
    try {
      fn();
    } finally {
      effectStack.pop();
    }
  };

  run();

  const dispose = () => {
    disposed = true;
    for (const s of subs) s.delete(run);
    subs.clear();
  };

  return dispose;
}
