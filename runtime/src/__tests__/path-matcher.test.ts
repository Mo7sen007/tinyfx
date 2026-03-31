import { describe, it, expect } from "vitest";
import { matchPath, splitPath } from "../router/path-matcher";

describe("splitPath", () => {
  it("splits root path", () => {
    expect(splitPath("/")).toEqual([]);
  });

  it("splits nested path", () => {
    expect(splitPath("/blog/hello-world")).toEqual(["blog", "hello-world"]);
  });

  it("handles trailing slash", () => {
    expect(splitPath("/blog/")).toEqual(["blog"]);
  });
});

describe("matchPath", () => {
  it("matches static route exactly", () => {
    const def = { staticSegments: ["blog"], paramNames: [] };
    expect(matchPath(def, ["blog"])).toEqual({});
  });

  it("returns null for static mismatch", () => {
    const def = { staticSegments: ["blog"], paramNames: [] };
    expect(matchPath(def, ["about"])).toBeNull();
  });

  it("returns null for length mismatch", () => {
    const def = { staticSegments: ["blog"], paramNames: [] };
    expect(matchPath(def, ["blog", "extra"])).toBeNull();
  });

  it("extracts single param", () => {
    const def = { staticSegments: ["blog", null], paramNames: ["slug"] };
    expect(matchPath(def, ["blog", "hello-world"])).toEqual({ slug: "hello-world" });
  });

  it("extracts multiple params", () => {
    const def = { staticSegments: [null, "posts", null], paramNames: ["user", "id"] };
    expect(matchPath(def, ["alice", "posts", "42"])).toEqual({ user: "alice", id: "42" });
  });

  it("decodes URI params", () => {
    const def = { staticSegments: ["blog", null], paramNames: ["slug"] };
    expect(matchPath(def, ["blog", "hello%20world"])).toEqual({ slug: "hello world" });
  });

  it("matches root route", () => {
    const def = { staticSegments: [], paramNames: [] };
    expect(matchPath(def, [])).toEqual({});
  });
});
