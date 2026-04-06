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
<page route="/blog/:slug" title="Blog">
  <head>
    <meta name="description" content="My blog">
  </head>
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

### Page Attributes

| Attribute | Required | Purpose |
|---|---|---|
| `route` | No | URL path pattern. If omitted, derived from filename. |
| `title` | No | Text used in the `<title>` tag. Falls back to filename-derived title. |
| `layout` | No | Wrap page body in a layout component. |

### `<head>` Block

Pages can include a `<head>` block inside the `<page>` element. The compiler merges user head content with required essentials:

```html
<page route="/blog/:slug" title="Blog Post">
  <head>
    <meta name="description" content="A post about routing">
    <meta property="og:image" content="/assets/og.png">
    <link rel="stylesheet" href="/css/blog.css">
  </head>
  <main>...</main>
</page>
```

The compiler always ensures:
- `<meta charset="UTF-8">` is present
- `<meta name="viewport">` is present
- `<title>` is set (from `title` attr, user `<title>`, or filename)
- `<link rel="stylesheet" href="/css/main.css">` is present (duplicates removed)

All other user head content is passed through as-is.

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
