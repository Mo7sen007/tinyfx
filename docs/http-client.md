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
- `get`
- `post`
- `put`
- `patch`
- `delete`

## Request Features
- query params
- custom headers
- per-request timeout override
- abort signal
- request interceptors
- response interceptors

## Error Types
Runtime HTTP package exposes error classes including:
- `HttpError`
- `HttpTimeoutError`
- `HttpParseError`
