# Getting Started

## Install and Scaffold
```bash
cargo install --path compiler
tinyfx init my-app
cd my-app
tinyfx serve
```

`tinyfx serve` runs an initial build, starts the dev HTTP server, starts the reload WebSocket server, and rebuilds on source changes.

## First Build Workflow
```bash
tinyfx build
```

Build output:
- Resolved page HTML under `dist/`.
- Bundled JS under `dist/js/main.js`.
- Copied CSS under `dist/css/`.
- Generated glue file at `src/tinyfx.gen.ts` during the build process.

`src/tinyfx.gen.ts` is generated code and is overwritten on subsequent builds.

## Project Layout
```text
my-app/
├── tinyfx.toml
├── src/
│   ├── main.ts
│   ├── pages/
│   ├── components/
│   ├── lib/
│   └── styles/
└── dist/
```

## Static vs Dynamic from Day One
- Static component: HTML only, expanded at build time.
- Dynamic component: HTML with `tinyfx-dynamic` and same-name `.ts` module.

Compiler checks:
- `tinyfx-dynamic` without `.ts` -> error.
- `.ts` without `tinyfx-dynamic` -> warning.

## Useful Development Commands
```bash
tinyfx build --phase-one-only
tinyfx build --assets-dir public-assets
tinyfx serve --config tinyfx.toml
```
