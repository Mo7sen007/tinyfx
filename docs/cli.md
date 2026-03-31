# CLI Reference

## `tinyfx init`
```bash
tinyfx init <name>
```
Creates a new TinyFX project from compiler templates. Writes scaffold files including pages, components, lib folders, styles, and configuration.

## `tinyfx build`
```bash
tinyfx build [--config tinyfx.toml] [--phase-one-only] [--assets-dir <path>]
```
Runs the full build pipeline.

Current phases:
1. Dependency graph — scans `src/components/` and `src/pages/` for HTML files and component references.
2. Component compilation — reads templates in dependency order, expands nested components, detects dynamic components, parses `<each>` and `<if>` sites.
3. Page processing — compiles pages, expands components, processes page-level directives, writes resolved HTML to `dist/`.
4. Glue generation — generates `src/tinyfx.gen.ts` with component/page registration, directive helpers, wrappers, routes, and `init()` bootstrap.
5. Asset pipeline — bundles `src/main.ts` with esbuild, copies styles and static assets.

## `tinyfx serve`
```bash
tinyfx serve [--config tinyfx.toml]
```
Runs build + HTTP server + WebSocket reload + file watch loop.

## `tinyfx generate` (alias: `tinyfx g`)
```bash
tinyfx generate component <Name> [--dynamic]
tinyfx generate page <name>
tinyfx generate service <Name>
tinyfx generate model <Name>
tinyfx generate dto <Name>
tinyfx generate interface <Name>
```

Subcommand aliases:
- `component` → `c`
- `service` → `s`
- `model` → `m`
- `dto` → `d`
- `interface` → `i`
- `page` → `p`

Generated files:
- `component <Name>` → `src/components/<Name>/<Name>.html` (+ `<Name>.ts` if `--dynamic`)
- `page <name>` → `src/pages/<name>.html` + `src/pages/<name>.ts`
- `service <Name>` → `src/lib/services/<kebab-name>.service.ts`
- `model <Name>` → `src/lib/models/<kebab-name>.model.ts`
- `dto <Name>` → `src/lib/dto/<kebab-name>.dto.ts`
- `interface <Name>` → `src/lib/models/<kebab-name>.interface.ts`

## Config Fields (`tinyfx.toml`)
Active fields in compiler config model:

| Field | Type | Default | Description |
|---|---|---|---|
| `entry` | `string` | `"src"` | Present in config model, not used in build phases |
| `entry_ts` | `string` | `"src/main.ts"` | TypeScript entry point for bundling |
| `out_dir` | `string` | `"dist"` | Build output directory |
| `components` | `string` | `"src/components"` | Component templates directory |
| `styles` | `string` | `"src/styles"` | CSS source directory |
| `static_dirs` | `string[]` | `[]` | Additional static asset directories |
| `assets_dir` | `string` | `"src/assets"` | Assets directory |
| `pages` | `string` | `"src/pages"` | Page templates directory |
| `phase_one_only` | `bool` | `false` | Stop after component compilation (debug) |

The scaffold may include additional sections (such as `[server]` and `[router]` placeholders) that are not consumed by the current config parser.
