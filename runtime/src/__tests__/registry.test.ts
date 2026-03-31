import { describe, it, expect, beforeEach } from "vitest";
import { registerComponent, getComponentFactory, mountComponents } from "../registry";
import type { TinyFxContext } from "../context";

describe("registerComponent / getComponentFactory", () => {
  beforeEach(() => {
    registerComponent("__test_cleanup__", () => {});
  });

  it("returns undefined for unregistered component", () => {
    expect(getComponentFactory("NonExistent")).toBeUndefined();
  });

  it("returns factory after registration", () => {
    const factory = (el: HTMLElement) => ({ mounted: true });
    registerComponent("TestComp", factory);
    expect(getComponentFactory("TestComp")).toBe(factory);
  });
});

describe("mountComponents", () => {
  it("mounts components found in root element", () => {
    const root = document.createElement("div");
    const el1 = document.createElement("div");
    el1.setAttribute("data-component", "CompA");
    const el2 = document.createElement("div");
    el2.setAttribute("data-component", "CompB");
    root.appendChild(el1);
    root.appendChild(el2);

    const results: string[] = [];
    registerComponent("CompA", () => {
      results.push("CompA");
      return { name: "CompA" };
    });
    registerComponent("CompB", () => {
      results.push("CompB");
      return { name: "CompB" };
    });

    const ctx: TinyFxContext = { params: {}, navigate: () => {} };
    const instances = mountComponents(ctx, root);

    expect(results).toContain("CompA");
    expect(results).toContain("CompB");
    expect(instances).toHaveLength(2);
  });

  it("skips elements without data-component", () => {
    const root = document.createElement("div");
    const el = document.createElement("div");
    root.appendChild(el);

    const ctx: TinyFxContext = { params: {}, navigate: () => {} };
    const instances = mountComponents(ctx, root);

    expect(instances).toHaveLength(0);
  });

  it("skips unregistered component names", () => {
    const root = document.createElement("div");
    const el = document.createElement("div");
    el.setAttribute("data-component", "Unknown");
    root.appendChild(el);

    const ctx: TinyFxContext = { params: {}, navigate: () => {} };
    const instances = mountComponents(ctx, root);

    expect(instances).toHaveLength(0);
  });

  it("does not include undefined returns in instances", () => {
    const root = document.createElement("div");
    const el = document.createElement("div");
    el.setAttribute("data-component", "NoReturn");
    root.appendChild(el);

    registerComponent("NoReturn", () => undefined);

    const ctx: TinyFxContext = { params: {}, navigate: () => {} };
    const instances = mountComponents(ctx, root);

    expect(instances).toHaveLength(0);
  });
});
