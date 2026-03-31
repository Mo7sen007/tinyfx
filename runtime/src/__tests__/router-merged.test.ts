import { describe, it, expect, vi } from "vitest";
import { navigate, goBack, RouteMeta, RouteMap } from "../router/index";

describe("navigate", () => {
  it("is a function that accepts a path string", () => {
    expect(typeof navigate).toBe("function");
    expect(() => navigate("/test/path")).not.toThrow();
  });
});

describe("goBack", () => {
  it("calls history.go(-1)", () => {
    const spy = vi.spyOn(window.history, "go");
    goBack();
    expect(spy).toHaveBeenCalledWith(-1);
    spy.mockRestore();
  });
});

describe("RouteMeta type", () => {
  it("accepts valid route metadata", () => {
    const meta: RouteMeta = { page: "blog-post", path: "/blog/:slug" };
    expect(meta.page).toBe("blog-post");
    expect(meta.path).toBe("/blog/:slug");
  });
});

describe("RouteMap type", () => {
  it("accepts route map object", () => {
    const routes: RouteMap = {
      "/": { page: "index", path: "/" },
      "/blog/:slug": { page: "blog-post", path: "/blog/:slug" },
    };
    expect(Object.keys(routes)).toHaveLength(2);
    expect(routes["/blog/:slug"].page).toBe("blog-post");
  });
});
