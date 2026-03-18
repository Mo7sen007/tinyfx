# Context and Services

TinyFX does not implement a runtime dependency injection container.

## `TinyFxContext`
`TinyFxContext` is passed into page and component mount/init functions and includes route params and `navigate`.

```ts
// src/pages/profile.ts
import type { TinyFxContext } from "@tinyfx/runtime";

export function init(el: HTMLElement, ctx: TinyFxContext): void {
  console.log(ctx.params);
  void el;
}
```

## Service Pattern
Use explicit factory functions in `src/lib/services/`.

```ts
// src/lib/services/post.service.ts
import { createHttp } from "@tinyfx/runtime";

const http = createHttp({ baseUrl: "/api" });

export function createPostService() {
  return {
    async list() {
      return http.get("/posts");
    },
  };
}
```

## Shared State Pattern
Use shared signals in `src/lib/state/` for cross-component state.

```ts
// src/lib/state/session.state.ts
import { signal } from "@tinyfx/runtime";

export const currentUserId = signal<number | null>(null);
```
