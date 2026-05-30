/**
 * Builds a request URL by combining a base path with optional query params.
 *
 * @param url - The request path or URL fragment
 * @param base - The configured base URL prefix
 * @param params - Optional query parameters to append
 * @returns The final request URL string
 *
 * @example
 * buildUrl("/posts", "/api", { page: 2, published: true });
 */
export function buildUrl(url: string, base:string, params?: Record<string, string | number | boolean>): string {
    const separator = base.endsWith("/") || url.startsWith("/") ? "" : "/";
    const fullUrl = base + separator + url;
    if (!params || Object.keys(params).length === 0) return fullUrl;
    
    const urlObj = new URL(fullUrl, typeof window !== 'undefined' ? window.location.href : 'http://localhost');
    Object.entries(params).forEach(([key, value]) => {
      urlObj.searchParams.append(key, String(value));
    });
    return urlObj.toString().replace(urlObj.origin, '');
}

/**
 * Waits for a number of milliseconds.
 *
 * @param ms - The delay duration in milliseconds
 * @returns A promise that resolves after the delay completes
 *
 * @example
 * await sleep(250);
 */
export async function sleep(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
}