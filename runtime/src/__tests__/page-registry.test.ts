import { describe, it, expect, beforeEach } from "vitest";
import { registerPage, getPageModule, runPageInit } from "../page-registry";
import type { TinyFxContext } from "../context";

describe("registerPage / getPageModule", () => {
  it("returns undefined for unregistered page", () => {
    expect(getPageModule("/unknown")).toBeUndefined();
  });

  it("returns module after registration", () => {
    const mod = { init: () => {} };
    registerPage("/test", mod);
    expect(getPageModule("/test")).toBe(mod);
  });
});

describe("runPageInit", () => {
  it("calls init on page module", () => {
    let called = false;
    let receivedEl: HTMLElement | undefined;
    let receivedCtx: TinyFxContext | undefined;

    registerPage("/init-test", {
      init: (el, ctx) => {
        called = true;
        receivedEl = el;
        receivedCtx = ctx;
      },
    });

    const ctx: TinyFxContext = { params: { id: "1" }, navigate: () => {} };
    runPageInit("/init-test", ctx);

    expect(called).toBe(true);
    expect(receivedEl).toBe(document.body);
    expect(receivedCtx).toBe(ctx);
  });

  it("does nothing for unregistered page", () => {
    expect(() => runPageInit("/nonexistent", { params: {}, navigate: () => {} })).not.toThrow();
  });

  it("does nothing for page without init", () => {
    registerPage("/no-init", {});
    expect(() => runPageInit("/no-init", { params: {}, navigate: () => {} })).not.toThrow();
  });
});
