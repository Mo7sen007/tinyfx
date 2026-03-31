import { describe, it, expect } from "vitest";
import { httpError, timeoutError, parseError } from "../http/data";

describe("httpError", () => {
  it("creates http error with all fields", () => {
    const err = httpError(404, "Not Found", "/api/test", "GET");
    expect(err.kind).toBe("http");
    if (err.kind === "http") {
      expect(err.status).toBe(404);
      expect(err.statusText).toBe("Not Found");
      expect(err.url).toBe("/api/test");
      expect(err.method).toBe("GET");
    }
  });

  it("includes optional response", () => {
    const response = new Response();
    const err = httpError(500, "Server Error", "/api", "POST", response);
    if (err.kind === "http") {
      expect(err.response).toBe(response);
    }
  });
});

describe("timeoutError", () => {
  it("creates timeout error with url and timeout", () => {
    const err = timeoutError("/api/slow", 5000);
    expect(err.kind).toBe("timeout");
    if (err.kind === "timeout") {
      expect(err.url).toBe("/api/slow");
      expect(err.timeout).toBe(5000);
    }
  });
});

describe("parseError", () => {
  it("creates parse error with message and url", () => {
    const err = parseError("Unexpected token", "/api/json");
    expect(err.kind).toBe("parse");
    if (err.kind === "parse") {
      expect(err.message).toBe("Unexpected token");
      expect(err.url).toBe("/api/json");
    }
  });
});

describe("error discrimination", () => {
  it("allows discriminating by kind", () => {
    const errors = [
      httpError(400, "Bad Request", "/a", "GET"),
      timeoutError("/b", 1000),
      parseError("Bad JSON", "/c"),
    ];

    const httpErr = errors.find((e) => e.kind === "http");
    expect(httpErr?.kind).toBe("http");
    if (httpErr?.kind === "http") {
      expect(httpErr.status).toBe(400);
    }

    const timeoutErr = errors.find((e) => e.kind === "timeout");
    expect(timeoutErr?.kind).toBe("timeout");
    if (timeoutErr?.kind === "timeout") {
      expect(timeoutErr.timeout).toBe(1000);
    }

    const parseErr = errors.find((e) => e.kind === "parse");
    expect(parseErr?.kind).toBe("parse");
    if (parseErr?.kind === "parse") {
      expect(parseErr.message).toBe("Bad JSON");
    }
  });
});
