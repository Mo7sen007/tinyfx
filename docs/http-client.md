# HTTP Client

TinyFX runtime provides `createHttp()`.

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
// src/lib/services/user.service.ts
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

## Request Features
- query params via `options.params`
- custom headers via `options.headers`
- per-request timeout override via `options.timeout`
- abort signal via `options.signal`
- request interceptors
- response interceptors

## Error Handling
The HTTP client uses a discriminated union type for errors. Check the `kind` field to determine the error type:

```ts
import { isHttpError, isTimeoutError, isParseError } from "@tinyfx/runtime";

try {
  const user = await http.get("/users/1");
} catch (err) {
  if (isHttpError(err)) {
    // HTTP error with status, statusText, url, method, response
    console.error(err.status, err.statusText);
  } else if (isTimeoutError(err)) {
    // Timeout error with url and timeout
    console.error("timed out after", err.timeout, "ms");
  } else if (isParseError(err)) {
    // JSON parse error with message and url
    console.error("parse error:", err.message);
  }
}
```

Error kinds:
- `"http"` — non-2xx response. Fields: `status`, `statusText`, `url`, `method`, `response?`.
- `"timeout"` — request exceeded timeout. Fields: `url`, `timeout`.
- `"parse"` — JSON parsing failed. Fields: `message`, `url`.
