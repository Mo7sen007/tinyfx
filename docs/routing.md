# File-Based Routing

TinyFX includes a file-based **hard router**. Every page is a real HTML file written to `dist/`. Navigation is a genuine browser page load — no virtual DOM swapping, no client-side route interception.

## How it Works

1. You create HTML files in `src/pages/`.
2. `tinyfx build` expands components, wraps each page in its layout, and writes a real `.html` file under `dist/`.
3. The compiler emits a route map into the generated `tinyfx.gen.ts`.
4. `initRouter()` runs in the browser on every page load to resolve URL params, run lifecycle hooks, and highlight active links.

## Page Format

Pages are HTML files with a `<page>` root element.

```html
<!-- src/pages/blog.html -->
<page route="/blog" layout="MainLayout">

  <component name="PostList" />

</page>
```

### Attributes

| Attribute | Required | Description |
|-----------|----------|-------------|
| `route`   | No       | URL pattern this page responds to. Defaults to the file path. Supports `:param` segments. |
| `layout`  | No       | Name of a component from `src/components/` to wrap the page body in. |

### Deriving Routes from Filenames

If `route` is omitted, TinyFX derives it from the file path relative to `src/pages/`:

| File                     | Route         |
|--------------------------|---------------|
| `src/pages/index.html`   | `/`           |
| `src/pages/blog.html`    | `/blog`       |
| `src/pages/blog/post.html` | `/blog/post` |

### Script Blocks

An optional `<script type="tinyfx">` block can appear inside `<page>`. It may **call** functions, but must not **define** them. Function definitions belong in `src/lib/`.

```html
<page route="/blog/:slug">

  <script type="tinyfx">
    const post = await fetchPost(getParam("slug"));
  </script>

  <component name="PostDetail" title="{{post.title}}" />

</page>
```

> **Warning**: If TinyFX detects a function definition inside the script block, it will print a build warning.

## Output File Structure

| Route          | Output path            |
|----------------|------------------------|
| `/`            | `dist/index.html`      |
| `/blog`        | `dist/blog/index.html` |
| `/blog/:slug`  | `dist/blog/index.html` |

Dynamic `:param` segments exist only in the route map. The static HTML file is served for all matching paths; params are resolved in the browser by `getParam()`.

## Configuration

Add a `pages` entry to `tinyfx.toml` (defaults to `src/pages`):

```toml
pages = "src/pages"
```

## Runtime API

### `initRouter(routes)`

Called automatically by the generated `tinyfx.gen.ts`. You never call this directly.

### `getParam(key)`

Retrieve a URL parameter for the current page.

```typescript
import { getParam } from "@tinyfx/runtime";

// On a page with route "/blog/:slug" and URL "/blog/hello"
const slug = getParam("slug"); // → "hello"
```

### `onMount(fn)`

Register a callback to run after the page is ready.

```typescript
import { onMount } from "@tinyfx/runtime";

onMount(() => {
  console.log("Page is ready");
});
```

### `onDestroy(fn)`

Register a callback to run when the user navigates away.

```typescript
import { onDestroy } from "@tinyfx/runtime";

onDestroy(() => {
  // clean up timers, subscriptions, etc.
});
```

### `navigate(path)`

Perform a full-page navigation to the given path.

```typescript
import { navigate } from "@tinyfx/runtime";

navigate("/blog");
```

### `goBack()`

Navigate back in the browser history.

```typescript
import { goBack } from "@tinyfx/runtime";

goBack();
```

## Active Link Highlighting

`initRouter` automatically sets `data-active="true"` on any `<a>` tag whose `href` matches the current pathname. Use this in CSS:

```css
a[data-active="true"] {
  font-weight: bold;
  color: var(--color-accent);
}
```

## Design Constraints

- **No link interception.** TinyFX never hijacks `<a>` clicks. Every navigation is a real HTTP request.
- **No DOM swapping.** Each page is its own HTML file. There is no SPA shell.
- **Params are runtime-only.** Dynamic segments like `:slug` are resolved by `getParam()` in the browser; they do not affect the static file path.
- **Components are not pages.** Only files in `src/pages/` are treated as routes.
