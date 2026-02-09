// @tinyfx/runtime — HTTP client
// Thin, typed wrapper over fetch.

import { HttpConfig, HttpError, HttpParseError, HttpTimeoutError, RequestOptions } from "./data";



export function createHttp(config: HttpConfig = {}) {
  const base = config.baseUrl ?? "";
  const defaultHeaders = config.headers ?? {};
  const defaultTimeout = config.timeout ?? 30000;
  const maxRetries = config.retries ?? 0;
  const retryDelay = config.retryDelay ?? 1000;
  const requestInterceptors = config.requestInterceptors ?? [];
  const responseInterceptors = config.responseInterceptors ?? [];

  

  

  async function request<T>(
    method: string,
    url: string,
    body?: unknown,
    options: RequestOptions = {}
  ): Promise<T> {
    const fullUrl = buildUrl(url,base, options.params);
    const timeoutMs = options.timeout ?? defaultTimeout;
    
    let attempts = 0;
    const maxAttempts = maxRetries + 1;

    while (attempts < maxAttempts) {
      attempts++;
      
      try {
        const headers: Record<string, string> = {
          ...defaultHeaders,
          ...options.headers,
        };
        
        if (body !== undefined && !headers["Content-Type"]) {
          headers["Content-Type"] = "application/json";
        }

        let fetchOptions: RequestInit = {
          method,
          headers,
          body: body !== undefined ? JSON.stringify(body) : undefined,
          signal: options.signal,
        };

        let requestUrl = fullUrl;

        // Apply request interceptors
        for (const interceptor of requestInterceptors) {
          const result = await interceptor(requestUrl, fetchOptions);
          requestUrl = result.url;
          fetchOptions = result.options;
        }

        // Create timeout controller
        const timeoutController = new AbortController();
        const timeoutId = setTimeout(() => timeoutController.abort(), timeoutMs);

        // Combine signals if user provided one
        let combinedSignal: AbortSignal | undefined = timeoutController.signal;
        if (options.signal) {
          const combinedController = new AbortController();
          const abortHandler = () => combinedController.abort();
          options.signal.addEventListener("abort", abortHandler, { once: true });
          timeoutController.signal.addEventListener("abort", abortHandler, { once: true });
          combinedSignal = combinedController.signal;
        }

        fetchOptions.signal = combinedSignal;

        let res: Response;
        try {
          res = await fetch(base + requestUrl, fetchOptions);
        } catch (err) {
          clearTimeout(timeoutId);
          if (err instanceof Error && err.name === "AbortError") {
            if (options.signal?.aborted) {
              throw err; // User cancelled
            }
            throw new HttpTimeoutError(requestUrl, timeoutMs);
          }
          throw err;
        } finally {
          clearTimeout(timeoutId);
        }

        // Apply response interceptors
        for (const interceptor of responseInterceptors) {
          res = await interceptor(res);
        }

        if (!res.ok) {
          throw new HttpError(res.status, res.statusText, requestUrl, method, res);
        }

        // Parse response based on content type
        const contentType = res.headers.get("content-type") || "";
        
        if (contentType.includes("application/json")) {
          try {
            const text = await res.text();
            if (!text || text.trim().length === 0) {
              return undefined as T;
            }
            return JSON.parse(text) as T;
          } catch (err) {
            throw new HttpParseError(
              `Failed to parse JSON response: ${err instanceof Error ? err.message : "Unknown error"}`,
              requestUrl
            );
          }
        }

        if (contentType.includes("text/")) {
          return (await res.text()) as unknown as T;
        }

        // For other content types, try to read as text
        try {
          const text = await res.text();
          return (text || undefined) as unknown as T;
        } catch {
          return undefined as T;
        }
      } catch (err) {
        // Retry logic for network errors or 5xx errors
        const shouldRetry =
          attempts < maxAttempts &&
          (!(err instanceof HttpError) || (err.status >= 500 && err.status < 600));

        if (shouldRetry) {
          await sleep(retryDelay * attempts);
          continue;
        }

        throw err;
      }
    }

    throw new Error("Max retries exceeded");
  }

  return {
    get: <T>(url: string, options?: RequestOptions) => 
      request<T>("GET", url, undefined, options),
    post: <T>(url: string, body?: unknown, options?: RequestOptions) => 
      request<T>("POST", url, body, options),
    put: <T>(url: string, body?: unknown, options?: RequestOptions) => 
      request<T>("PUT", url, body, options),
    patch: <T>(url: string, body?: unknown, options?: RequestOptions) => 
      request<T>("PATCH", url, body, options),
    del: <T>(url: string, options?: RequestOptions) => 
      request<T>("DELETE", url, undefined, options),
    delete: <T>(url: string, options?: RequestOptions) => 
      request<T>("DELETE", url, undefined, options),
  };
}
