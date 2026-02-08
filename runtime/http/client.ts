export function createHttp(config: { baseUrl?: string }) {
  return {
    async get<T>(url: string): Promise<T> {
      const res = await fetch((config.baseUrl ?? "") + url);
      if (!res.ok) throw new Error(res.statusText);
      return res.json();
    }
  };
}
