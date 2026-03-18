# CLI Reference

## `tinyfx init`
```bash
tinyfx init <name> [--force]
```
Creates a new TinyFX project from compiler templates.

## `tinyfx build`
```bash
tinyfx build [--config tinyfx.toml] [--phase-one-only] [--assets-dir <path>]
```
Runs the full build pipeline.

Current phases:
1. Dependency graph
2. Component compilation
3. Page processing
4. Glue generation
5. Asset pipeline

## `tinyfx serve`
```bash
tinyfx serve [--config tinyfx.toml]
```
Runs build + HTTP server + WebSocket reload + file watch loop.

## `tinyfx generate` (alias: `tinyfx g`)
```bash
tinyfx generate component <Name> [--dynamic] [--force]
tinyfx generate page <name> [--force]
tinyfx generate service <Name> [--force]
tinyfx generate model <Name> [--force]
tinyfx generate dto <Name> [--force]
tinyfx generate interface <Name> [--force]
```

Subcommand aliases:
- `component` -> `c`
- `service` -> `s`
- `model` -> `m`
- `dto` -> `d`
- `interface` -> `i`
- `page` -> `p`

## Config Fields (`tinyfx.toml`)
Active fields in compiler config model:
- `entry`
- `entry_ts`
- `out_dir`
- `components`
- `styles`
- `static_dirs`
- `assets_dir`
- `pages`
- `phase_one_only`

The scaffold may include additional sections (such as server/router placeholders) that are not consumed by the current config parser.
