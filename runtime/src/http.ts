// @tinyfx/runtime — HTTP client
// Thin, typed wrapper over fetch.

export interface HttpConfig {
  baseUrl?: string;
  headers?: Record<string, string>;
}

export function createHttp(config: HttpConfig = {}) {
  const base = config.baseUrl ?? "";
  const defaultHeaders = config.headers ?? {};

  async function request<T>(method: string, url: string, body?: unknown): Promise<T> {
    const headers: Record<string, string> = { ...defaultHeaders };
    if (body !== undefined) {
      headers["Content-Type"] = "application/json";
    }
    const res = await fetch(base + url, {
      method,
      headers,
      body: body !== undefined ? JSON.stringify(body) : undefined,
    });
    if (!res.ok) throw new Error(`${method} ${url}: ${res.status} ${res.statusText}`);
    return res.json();
  }

  return {
    get: <T>(url: string) => request<T>("GET", url),
    post: <T>(url: string, body?: unknown) => request<T>("POST", url, body),
    put: <T>(url: string, body?: unknown) => request<T>("PUT", url, body),
    del: <T>(url: string) => request<T>("DELETE", url),
  };
}
