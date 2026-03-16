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
Create a new TinyFX project in the specified directory. This scaffolds the folder structure, default config, and example files.

```bash
tinyfx init my-app
```

### `build`
Run the full compiler pipeline (component expansion, page resolution, glue generation, and asset bundling). Output is written to the `out_dir` defined in `tinyfx.toml` (defaults to `dist/`).

```bash
tinyfx build
```

Notes:
- Pages are written to `dist/<segments>/index.html` for nested routes.
- `src/tinyfx.gen.ts` is auto-generated and overwritten on each build.

### `generate` (alias: `g`)
Generate project artifacts from templates. The long-form command is `tinyfx generate <type> <Name>`; a shorthand `g` alias is also supported in the CLI.

- `tinyfx generate component <Name>` — creates `src/components/<PascalName>/<PascalName>.html` and, with `--dynamic`, a `.ts` file.
- `tinyfx generate page <name>` — creates `src/pages/<kebab-name>.html` and optional `.ts` file. `index` maps to `/`.
- `tinyfx generate service <Name>` — creates `src/lib/services/<kebab-name>.service.ts`.
- `tinyfx generate model <Name>` — creates `src/lib/models/<kebab-name>.model.ts`.
- `tinyfx generate dto <Name>` — creates `src/lib/dto/<kebab-name>.dto.ts`.

Examples:

```bash
tinyfx generate component UserProfile
tinyfx generate page about
```

### `serve`
Start the integrated development server with live reload.

```bash
tinyfx serve
```

Behavior:
- Binds an HTTP server on `127.0.0.1:3000` to serve files from the configured `out_dir`.
- Starts a WebSocket server on `127.0.0.1:3001` that notifies connected browsers to reload when builds complete.
- Performs an initial build before serving and triggers rebuilds when source files change.

Ports:

| Port | Purpose |
|------|---------|
| 3000 | HTTP file server (static assets) |
| 3001 | WebSocket endpoint for live reload |

Build-only mode (phase 1)

If you need to run only the component expansion phase (internal/diagnostics), use:

```bash
tinyfx build --phase-one-only
```
