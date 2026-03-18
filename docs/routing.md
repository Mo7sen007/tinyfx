# Routing

TinyFX currently implements file-based hard routing.

## Hard Routing Model
- Pages compile to real HTML files in `dist/`.
- Navigation uses browser page loads.
- Generated glue initializes route helpers on load.

Soft routing is planned and not currently implemented.

## Page Routing
Pages live in `src/pages/` with `<page>` root.

```html
<!-- src/pages/blog/[slug].html (example naming style) -->
<page route="/blog/:slug" layout="">
  <main>
    <h1>Blog</h1>
  </main>
</page>
```

If `route` is omitted, route is derived from page path.

## Runtime Routing Helpers
From `@tinyfx/runtime`:
- `initRouter`
- `getParam`
- `navigate`
- `goBack`
- `onMount`
- `onDestroy`

```ts
// src/pages/blog.ts
import { getParam, onMount } from "@tinyfx/runtime";

export function init(el: HTMLElement): void {
  onMount(() => {
    console.log("slug", getParam("slug"));
    void el;
  });
}
```

## Generated Glue Responsibilities
`tinyfx.gen.ts` generates and uses:
- route table
- route matching helpers
- param extraction
- page module registry
- bootstrap call (`initRouter(routes)` + page init)
