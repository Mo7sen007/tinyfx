# CLI Reference

The `tinyfx` CLI is the main tool for managing your project and building your application.

## Configuration: `tinyfx.toml`

The behavior of the CLI is controlled by a `tinyfx.toml` file in your project root.

```toml
# Main HTML entry point
entry_html = "src/index.html"

# TypeScript entry point (where setupComponents is called)
entry_ts   = "src/main.ts"

# Output directory for the build
out_dir    = "dist"

# Path to the components directory
components = "src/components"

# Path to static assets/styles (optional)
styles     = "src/styles"

# Pages directory for the file-based router (optional, defaults to "src/pages")
pages      = "src/pages"
```

## Commands

### `init <name>`
Initializes a new TinyFX project in the specified directory. It creates the folder structure, default configuration, and basic boilerplate files.

```bash
tinyfx init my-app
```

### `build`
Compiles the project. It performs the following steps:
1.  **Template Expansion**: Scans the `entry_html` for `<component>` tags and replaces them with corresponding template files from the `components` directory.
2.  **Property Interpolation**: Replaces `{{ prop }}` placeholders with attribute values.
3.  **Slot Injection**: Places children of component tags into `<slot></slot>` locations.
4.  **Code Generation**: Scans for `.ts` files in the component directories and generates `tinyfx.gen.ts`.
5.  **Page Routing**: If `src/pages/` exists, scans it for `.html` page files, expands components, wraps in layouts, and writes each page as a real HTML file under `dist/`. The route map is embedded in `tinyfx.gen.ts`.
6.  **Output**: Writes the final HTML and generated TypeScript files to the `out_dir`.

```bash
tinyfx build
```

### `g <type> <name>`
Generates new files based on templates.

- **`g component <name>`**: Creates a new component folder with HTML and TS files.
- **`g service <name>`**: Creates a new service file.
- **`g interface <name>`**: Creates a new TypeScript interface.
- **`g page <name>`**: Creates a new page component.

```bash
tinyfx g component UserProfile
tinyfx g service AuthService
```

### `serve <dir>`

Starts a local development server with **live reload**.

```bash
tinyfx serve dist
```

| Port | Purpose |
|------|---------|
| `3000` | HTTP file server (static assets) |
| `3001` | WebSocket endpoint for live reload |

How it works:
1. An HTTP server on port `3000` serves static files from `<dir>`.
2. A WebSocket server on port `3001` is started for browser notifications.
3. TinyFX watches `<dir>` recursively for any file change.
4. A small inline `<script>` is injected into every HTML response that connects to the WebSocket and calls `location.reload()` when a `"reload"` message is received.
5. When a file change is detected (with a 100 ms debounce to coalesce bursts), all connected browsers reload automatically.

Typical development loop:

```bash
# Terminal 1 — rebuild on source change (use your bundler's watch mode)
tinyfx build && npm run watch

# Terminal 2 — serve and live-reload
tinyfx serve dist
```

---

## Build Phases

TinyFX can sometimes be run in "Phase 1 only" mode (mostly for internal use) which just expands components without doing a full build.

```bash
tinyfx build --phase-one-only
```
