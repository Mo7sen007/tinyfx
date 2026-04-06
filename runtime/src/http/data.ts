/**
 * Discriminated union type for HTTP errors.
 *
 * @example
 * try {
 *   await http.get("/posts");
 * } catch (error) {
 *   if (error.kind === "http") {
 *     console.error(error.status, error.url);
 *   } else if (error.kind === "timeout") {
 *     console.error(error.timeout);
 *   } else if (error.kind === "parse") {
 *     console.error(error.url);
 *   }
 * }
 */
export type HttpError =
  | { kind: "http"; status: number; statusText: string; url: string; method: string; response?: Response }
  | { kind: "timeout"; url: string; timeout: number }
  | { kind: "parse"; message: string; url: string };

export function httpError(
  status: number,
  statusText: string,
  url: string,
  method: string,
  response?: Response,
): HttpError {
  return { kind: "http", status, statusText, url, method, response };
}

export function timeoutError(url: string, timeout: number): HttpError {
  return { kind: "timeout", url, timeout };
}

export function parseError(message: string, url: string): HttpError {
  return { kind: "parse", message, url };
}

export function isHttpError(err: unknown): err is Extract<HttpError, { kind: "http" }> {
  return typeof err === "object" && err !== null && "kind" in err && err.kind === "http";
}

export function isTimeoutError(err: unknown): err is Extract<HttpError, { kind: "timeout" }> {
  return typeof err === "object" && err !== null && "kind" in err && err.kind === "timeout";
}

export function isParseError(err: unknown): err is Extract<HttpError, { kind: "parse" }> {
  return typeof err === "object" && err !== null && "kind" in err && err.kind === "parse";
}

/**
 * Hook that can rewrite an outgoing request before `fetch` runs.
 *
 * @example
 * const addToken: RequestInterceptor = (url, options) => ({
 *   url,
 *   options: {
 *     ...options,
 *     headers: { ...options.headers, Authorization: "Bearer token" },
 *   },
 * });
 */
export interface RequestInterceptor {
  (url: string, options: RequestInit): Promise<{ url: string; options: RequestInit }> | { url: string; options: RequestInit };
}

/**
 * Hook that can inspect or replace a response before TinyFX handles it.
 *
 * @example
 * const logResponse: ResponseInterceptor = (response) => {
 *   console.log(response.status);
 *   return response;
 * };
 */
export interface ResponseInterceptor {
  (response: Response): Promise<Response> | Response;
}

/**
 * Default options for an HTTP client created by `createHttp()`.
 *
 * @example
 * const config: HttpConfig = {
 *   baseUrl: "/api",
 *   timeout: 5000,
 *   retries: 1,
 * };
 */
export interface HttpConfig {
  baseUrl?: string;
  headers?: Record<string, string>;
  timeout?: number;
  retries?: number;
  retryDelay?: number;
  requestInterceptors?: RequestInterceptor[];
  responseInterceptors?: ResponseInterceptor[];
}

/**
 * Per-request overrides for an HTTP call.
 *
 * @example
 * const options: RequestOptions = {
 *   params: { page: 2 },
 *   timeout: 1000,
 * };
 */
export interface RequestOptions {
  headers?: Record<string, string>;
  timeout?: number;
  signal?: AbortSignal;
  params?: Record<string, string | number | boolean>;
  json?: boolean;
}