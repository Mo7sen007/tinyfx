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

export class HttpTimeoutError extends Error {
  constructor(public readonly url: string, public readonly timeout: number) {
    super(`Request timeout after ${timeout}ms: ${url}`);
    this.name = "HttpTimeoutError";
  }
}

export class HttpParseError extends Error {
  constructor(message: string, public readonly url: string) {
    super(message);
    this.name = "HttpParseError";
  }
}

export interface RequestInterceptor {
  (url: string, options: RequestInit): Promise<{ url: string; options: RequestInit }> | { url: string; options: RequestInit };
}

export interface ResponseInterceptor {
  (response: Response): Promise<Response> | Response;
}

export interface HttpConfig {
  baseUrl?: string;
  headers?: Record<string, string>;
  timeout?: number;
  retries?: number;
  retryDelay?: number;
  requestInterceptors?: RequestInterceptor[];
  responseInterceptors?: ResponseInterceptor[];
}

export interface RequestOptions {
  headers?: Record<string, string>;
  timeout?: number;
  signal?: AbortSignal;
  params?: Record<string, string | number | boolean>;
}