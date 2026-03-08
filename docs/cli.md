# CLI Reference

The `tinyfx` CLI is the main tool for managing your project and building your application.

## Configuration: `tinyfx.toml`

The behavior of the CLI is controlled by a `tinyfx.toml` file in your project root.

```toml
# Pages directory
pages      = "src/pages"

# TypeScript entry point
entry_ts   = "src/main.ts"

# Output directory for the build
out_dir    = "dist"

# Path to the components directory
components = "src/components"

# Path to static assets/styles (optional)
styles     = "src/styles"

```

## Commands

### `init <name>`
Initializes a new TinyFX project in the specified directory. It creates the folder structure, default configuration, and basic boilerplate files.

```bash
tinyfx init my-app
```

### `build`
Compiles the project. It performs the following steps:
1.  **Component Discovery**: Scans `src/components/` recursively for component templates.
2.  **Property Interpolation**: Replaces `{{ prop }}` placeholders with attribute values.
3.  **Slot Injection**: Places children of component tags into `<slot></slot>` locations.
4.  **Page Processing**: Scans all `.html` pages in `src/pages/`, expands components, and writes routed output into `dist/`.
5.  **Asset Pipeline**: Bundles `src/main.ts` to `dist/js/main.js`, copies styles/assets.
6.  **Glue Generation**: Generates `src/tinyfx.gen.ts` route/component wiring from discovered pages and components.

```bash
tinyfx build
```

### `g <type> <name>`
Generates new files based on templates.

- **`g component <name>`**: Creates a new component folder with HTML and TS files.
- **`g service <name>`**: Creates `src/lib/services/<name>.service.ts` service factory.
- **`g model <name>`**: Creates `src/lib/models/<name>.model.ts`.
- **`g dto <name>`**: Creates `src/lib/dto/<name>.dto.ts`.
- **`g page <name>`**: Creates `src/pages/<name>.html` and `src/pages/<name>.ts`.

```bash
tinyfx g component UserProfile
tinyfx g service AuthService
```

### `serve`

Starts a local development server with **live reload**.

```bash
tinyfx serve
```

| Port | Purpose |
|------|---------|
| `3000` | HTTP file server (static assets) |
| `3001` | WebSocket endpoint for live reload |

How it works:
1. An HTTP server on port `3000` serves static files from `out_dir` in `tinyfx.toml`.
2. A WebSocket server on port `3001` is started for browser notifications.
3. TinyFX watches `src/` recursively for source changes.
4. A small inline `<script>` is injected into every HTML response that connects to the WebSocket and calls `location.reload()` when a `"reload"` message is received.
5. When a file change is detected (with a 100 ms debounce to coalesce bursts), all connected browsers reload automatically.

Typical development loop:

```bash
# Terminal 1 — rebuild on source change (use your bundler's watch mode)
tinyfx build && npm run watch

# Terminal 2 — serve and live-reload
tinyfx serve
```

---

## Build Phases

TinyFX can sometimes be run in "Phase 1 only" mode (mostly for internal use) which just expands components without doing a full build.

```bash
tinyfx build --phase-one-only
```
