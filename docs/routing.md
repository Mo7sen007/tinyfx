# Routing

TinyFX currently implements file-based hard routing.

## Hard Routing Model
- Pages compile to real HTML files in `dist/`.
- Navigation uses real browser page loads (`window.location.href`).
- Generated glue initializes everything through a single `init()` call.

Soft routing is planned and not currently implemented.

## Page Routing
Pages live in `src/pages/` with `<page>` root.

```html
<!-- src/pages/blog.html -->
<page route="/blog/:slug" layout="">
  <main>
    <h1>Blog</h1>
  </main>
</page>
```

```ts
// src/pages/blog.ts
import { onMount } from "@tinyfx/runtime";
import type { TinyFxContext } from "@tinyfx/runtime";

export function init(_el: HTMLElement, ctx: TinyFxContext): void {
  onMount(() => {
    console.log("slug", ctx.params.slug);
  });
}
```

If `route` is omitted, the route is derived from the page file path.

## Runtime Routing Helpers
From `@tinyfx/runtime`:
- `getParam(name)` — get a single route parameter.
- `getParams()` — get all route parameters.
- `navigate(path)` — trigger a full browser navigation.
- `goBack()` — go back one step in browser history.
- `onMount(fn)` — register a mount callback.
- `onDestroy(fn)` — register a destroy callback.

## Generated Glue Responsibilities
`tinyfx.gen.ts` generates and uses:
- Route table with pre-split static segments and param names (`RouteDef`).
- Page module registration via `registerPage()`.
- Component registration via `registerComponent()`.
- Bootstrap through `init({ routes, setupDirectives })`.

The `init()` function replaces the old `initRouter()` API. It handles route matching, param extraction, component mounting, and page initialization in a single call.
