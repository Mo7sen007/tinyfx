import { describe, it, expect, beforeEach } from "vitest";
import { signal, effect } from "../signals";

describe("signal (inlined)", () => {
  it("returns current value when called", () => {
    const count = signal(0);
    expect(count()).toBe(0);
  });

  it("updates value via set()", () => {
    const count = signal(0);
    count.set(5);
    expect(count()).toBe(5);
  });

  it("does not notify subscribers when value is the same", () => {
    const count = signal(0);
    let runs = 0;
    effect(() => {
      count();
      runs++;
    });
    expect(runs).toBe(1);
    count.set(0);
    expect(runs).toBe(1);
  });

  it("notifies subscribers on change", () => {
    const count = signal(0);
    let value = 0;
    effect(() => {
      value = count();
    });
    expect(value).toBe(0);
    count.set(10);
    expect(value).toBe(10);
  });

  it("supports multiple subscribers", () => {
    const count = signal(0);
    let a = 0;
    let b = 0;
    effect(() => { a = count(); });
    effect(() => { b = count() * 2; });
    count.set(3);
    expect(a).toBe(3);
    expect(b).toBe(6);
  });

  it("works with objects", () => {
    const user = signal({ name: "Alice" });
    expect(user().name).toBe("Alice");
    user.set({ name: "Bob" });
    expect(user().name).toBe("Bob");
  });

  it("does not re-run effect when unrelated signal changes", () => {
    const a = signal(0);
    const b = signal(0);
    let runs = 0;
    effect(() => {
      a();
      runs++;
    });
    expect(runs).toBe(1);
    b.set(1);
    expect(runs).toBe(1);
  });
});

describe("effect", () => {
  it("runs immediately on creation", () => {
    let ran = false;
    effect(() => { ran = true; });
    expect(ran).toBe(true);
  });

  it("re-runs when tracked signal changes", () => {
    const count = signal(0);
    let runs = 0;
    effect(() => {
      count();
      runs++;
    });
    expect(runs).toBe(1);
    count.set(1);
    expect(runs).toBe(2);
  });

  it("supports nested signal reads", () => {
    const a = signal(1);
    const b = signal(2);
    let sum = 0;
    effect(() => {
      sum = a() + b();
    });
    expect(sum).toBe(3);
    a.set(10);
    expect(sum).toBe(12);
    b.set(5);
    expect(sum).toBe(15);
  });
});
