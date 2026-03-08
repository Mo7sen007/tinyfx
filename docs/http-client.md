# HTTP Client

TinyFX includes a thin, type-safe wrapper around the standard `fetch` API called `createHttp`.

## Creating a Client

You typically create an HTTP client instance inside a service factory and consume that service from components/pages.

```typescript
import { createHttp } from "@tinyfx/runtime";

const http = createHttp({
  baseUrl: "https://api.example.com",
  headers: {
    "X-App-ID": "tinyfx-demo"
  },
  timeout: 5000, // 5s timeout
  retries: 3
});
```

## Making Requests

The client supports standard HTTP methods: `get`, `post`, `put`, `patch`, and `delete`.

```typescript
// Fetching data
const users = await http.get<User[]>("/users");

// Sending data
await http.post("/users", { name: "John" });
```

## Request Options

You can pass additional options to each request:

- `params`: Query parameters object.
- `headers`: Additional headers.
- `timeout`: Override default timeout.
- `signal`: An `AbortSignal` for manual cancellation.

```typescript
const user = await http.get<User>("/user", {
  params: { id: "123" },
  headers: { "Authorization": "Bearer ..." }
});
```

## Interceptors

You can add interceptors to modify requests or responses globally.

```typescript
const http = createHttp({
  requestInterceptors: [
    async (url, options) => {
      // Add auth token to every request
      options.headers = { ...options.headers, Token: "..." };
      return { url, options };
    }
  ],
  responseInterceptors: [
    async (response) => {
      if (response.status === 401) {
        // Handle unauthorized
      }
      return response;
    }
  ]
});
```

## Error Handling

The client throws specific error classes for easier debugging:
- `HttpError`: For non-2xx responses.
- `HttpTimeoutError`: When a request times out.
- `HttpParseError`: When response JSON parsing fails.
