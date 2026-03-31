import { describe, it, expect } from "vitest";
import { markMounted, isMounted } from "../mount-state";

describe("markMounted", () => {
  it("returns true on first call", () => {
    const el = document.createElement("div");
    expect(markMounted(el)).toBe(true);
  });

  it("returns false on second call for same element", () => {
    const el = document.createElement("div");
    markMounted(el);
    expect(markMounted(el)).toBe(false);
  });

  it("returns true for different elements", () => {
    const el1 = document.createElement("div");
    const el2 = document.createElement("div");
    markMounted(el1);
    expect(markMounted(el2)).toBe(true);
  });
});

describe("isMounted", () => {
  it("returns false for unmarked element", () => {
    const el = document.createElement("div");
    expect(isMounted(el)).toBe(false);
  });

  it("returns true after markMounted", () => {
    const el = document.createElement("div");
    markMounted(el);
    expect(isMounted(el)).toBe(true);
  });
});
