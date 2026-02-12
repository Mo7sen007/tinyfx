export function buildUrl(url: string, base:string, params?: Record<string, string | number | boolean>): string {
    const fullUrl = base + url;
    if (!params || Object.keys(params).length === 0) return fullUrl;
    
    const urlObj = new URL(fullUrl, typeof window !== 'undefined' ? window.location.href : 'http://localhost');
    Object.entries(params).forEach(([key, value]) => {
      urlObj.searchParams.append(key, String(value));
    });
    return urlObj.toString().replace(urlObj.origin, '');
}

export async function sleep(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
}