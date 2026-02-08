// @tinyfx/runtime — signals
// Minimal, explicit reactivity — no proxies, no magic.

type Subscriber = () => void;

const effectStack: Subscriber[] = [];

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

export function signal<T>(v: T) {
  const s = new Signal(v);
  const fn = () => s.get();
  fn.set = (v: T) => s.set(v);
  return fn as (() => T) & { set(v: T): void };
}

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
