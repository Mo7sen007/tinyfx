# TinyFX

A hybrid web framework. The Rust compiler handles component expansion, directive transformation, and glue generation at build time. The TypeScript runtime provides the browser primitives that remain necessary after compile time.

## Install

```bash
cargo install --path compiler
```

## Quick start

```bash
tinyfx init my-app
cd my-app
tinyfx serve      # builds, watches, and serves on http://localhost:3000
```

## How it works

```
src/components/ + src/pages/
  → Phase 0: dependency graph
  → Phase 1: component compilation
  → Phase 2: page processing  
  → Phase 3: tinyfx.gen.ts generation
  → Phase 4: esbuild + asset copy
  → dist/
```

Static components are expanded to plain HTML at build time — no JavaScript shipped for them. Dynamic components opt in with `tinyfx-dynamic` on the root element and a sibling `.ts` file.

## CLI

| Command | Description |
|---|---|
| `tinyfx init <name>` | Scaffold a new project |
| `tinyfx build` | Production build |
| `tinyfx serve` | Dev server with live reload |
| `tinyfx generate c <Name>` | Static component |
| `tinyfx generate c <Name> --dynamic` | Dynamic component |
| `tinyfx generate p <name>` | Page |
| `tinyfx generate s <Name>` | Service |

Full CLI reference → [docs/cli.md](docs/cli.md)

## Documentation

- [Introduction & architecture](docs/introduction.md)
- [Getting started](docs/getting-started.md)
- [CLI reference](docs/cli.md)
- [Components](docs/components.md)
- [Signals & reactivity](docs/signals.md)
- [DOM helpers](docs/dom-helpers.md)
- [HTTP client](docs/http-client.md)
- [Routing](docs/routing.md)
- [Dependency injection](docs/dependency-injection.md)
- [Component mounting](docs/component-mounting.md)

## Status

| Feature | Status |
|---|---|
| Static component expansion | ✅ |
| Dynamic components | ✅ |
| File-based hard router | ✅ |
| Signals and effects | ✅ |
| `<each>` directive | ✅ |
| `<if>` / `<else>` directive | ✅ |
| Dev server with live reload | ✅ |
| Soft router | 📋 Planned |
| Lazy component loading | 📋 Planned |
