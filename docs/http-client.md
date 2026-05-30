# HTTP Client

TinyFX runtime provides `createHttp()` — a thin, typed wrapper over `fetch`.

## Create a Client

```ts
// src/lib/services/user.service.ts
import { createHttp } from "@tinyfx/runtime";

const http = createHttp({
  baseUrl: "/api",
  timeout: 5000,
  retries: 1,
});
```

## Typed Requests

```ts
type UserDto = { id: number; name: string };

export function createUserService() {
  return {
    async getUser(id: number): Promise<UserDto> {
      return http.get<UserDto>(`/users/${id}`);
    },
  };
}
```

## Supported Methods

- `get<T>(url, options?)`
- `post<T>(url, body?, options?)`
- `put<T>(url, body?, options?)`
- `patch<T>(url, body?, options?)`
- `del<T>(url, options?)`
- `delete<T>(url, options?)`

## Request Options

| Option | Type | Description |
|---|---|---|
| `params` | `Record<string, string \| number \| boolean>` | Query parameters appended to the URL |
| `headers` | `Record<string, string>` | Per-request headers merged with defaults |
| `timeout` | `number` | Timeout in ms, overrides the client default |
| `signal` | `AbortSignal` | Abort signal to cancel the request |
| `json` | `boolean` | Serialize body as JSON (default `true`) |

## Body Serialization

By default the client serializes bodies as JSON and sets `Content-Type: application/json`. Set `json: false` to pass raw `BodyInit` such as `FormData`, `Blob`, or plain strings:

```ts
// File upload
const formData = new FormData();
formData.append("file", file);
await http.post("/upload", formData, { json: false });

// Plain text
await http.post("/webhook", "raw payload", {
  json: false,
  headers: { "Content-Type": "text/plain" },
});
```

## Interceptors

### Request Interceptors

Receive the URL and `RequestInit` and must return a `{ url, options }` object. Use them to attach auth headers or rewrite URLs.

```ts
const http = createHttp({
  baseUrl: "/api",
  requestInterceptors: [
    (url, options) => ({
      url,
      options: {
        ...options,
        headers: { ...options.headers, Authorization: `Bearer ${getToken()}` },
      },
    }),
  ],
});
```

### Response Interceptors

Receive a **clone** of the response for read-only inspection (logging, metrics, analytics). The return value is not used — response interceptors cannot transform the response.

```ts
const http = createHttp({
  baseUrl: "/api",
  responseInterceptors: [
    (response) => {
      console.log(`${response.status} ${response.url}`);
    },
  ],
});
```

The clone allows interceptors to call `response.json()` or `response.text()` freely without consuming the body that the client needs for parsing.

## Abort / Cancellation

Pass an `AbortSignal` to cancel a request. If the signal is already aborted when the request is made, the client throws immediately without sending the request.

```ts
const controller = new AbortController();

// Cancel after 2 seconds
setTimeout(() => controller.abort(), 2000);

try {
  const data = await http.get("/slow-endpoint", { signal: controller.signal });
} catch (err) {
  if (err instanceof DOMException && err.name === "AbortError") {
    console.log("request cancelled");
  }
}
```

## Error Handling

The client throws a discriminated union for all error cases. Check the `kind` field:

```ts
import { isHttpError, isTimeoutError, isParseError } from "@tinyfx/runtime";

try {
  const user = await http.get<UserDto>("/users/1");
} catch (err) {
  if (isHttpError(err)) {
    // Non-2xx response: err.status, err.statusText, err.url, err.method, err.response
    console.error(err.status, err.statusText);
  } else if (isTimeoutError(err)) {
    // Request exceeded timeout: err.url, err.timeout
    console.error("timed out after", err.timeout, "ms");
  } else if (isParseError(err)) {
    // JSON parse failure: err.message, err.url
    console.error("parse error:", err.message);
  }
}
```

## Retry

```ts
const http = createHttp({
  baseUrl: "/api",
  retries: 2,       // retry up to 2 times (3 attempts total)
  retryDelay: 500,  // initial delay in ms, multiplied by attempt number
});
```

Only network errors and 5xx responses are retried. 4xx errors are not retried.
