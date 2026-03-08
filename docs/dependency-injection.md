# Context & Services

TinyFX no longer uses a runtime dependency injection container. Instead, pages and components receive a lightweight `TinyFxContext` and services are created explicitly with factory functions.

## TinyFxContext

`TinyFxContext` is imported from `@tinyfx/runtime` and contains route-aware helpers:

```typescript
import type { TinyFxContext } from "@tinyfx/runtime";

export function init(el: HTMLElement, ctx: TinyFxContext) {
  console.log(ctx.params);
  ctx.navigate("/");
}
```

## Service factories

Create services via factory functions in `src/lib/services/`.

```typescript
import { createHttp } from "@tinyfx/runtime";

const http = createHttp({ baseUrl: "/api" });

export function createUserService() {
  return {
    async getUser(id: number) {
      return http.get(`/users/${id}`);
    },
  };
}
```

Use them directly where needed:

```typescript
const userService = createUserService();
```
