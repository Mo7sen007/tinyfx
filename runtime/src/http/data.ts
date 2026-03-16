/**
 * Error thrown for non-success HTTP responses.
 *
 * @example
 * try {
 *   await http.get("/posts");
 * } catch (error) {
 *   if (error instanceof HttpError) {
 *     console.error(error.status, error.url);
 *   }
 * }
 */
export class HttpError extends Error {
  constructor(
    public readonly status: number,
    public readonly statusText: string,
    public readonly url: string,
    public readonly method: string,
    public readonly response?: Response
  ) {
    super(`HTTP ${status} ${statusText}: ${method} ${url}`);
    this.name = "HttpError";
  }
}

/**
 * Error thrown when a request times out.
 *
 * @example
 * try {
 *   await http.get("/slow", { timeout: 100 });
 * } catch (error) {
 *   if (error instanceof HttpTimeoutError) {
 *     console.error(error.timeout);
 *   }
 * }
 */
export class HttpTimeoutError extends Error {
  constructor(public readonly url: string, public readonly timeout: number) {
    super(`Request timeout after ${timeout}ms: ${url}`);
    this.name = "HttpTimeoutError";
  }
}

/**
 * Error thrown when a JSON response cannot be parsed.
 *
 * @example
 * try {
 *   await http.get("/broken-json");
 * } catch (error) {
 *   if (error instanceof HttpParseError) {
 *     console.error(error.url);
 *   }
 * }
 */
export class HttpParseError extends Error {
  constructor(message: string, public readonly url: string) {
    super(message);
    this.name = "HttpParseError";
  }
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
}