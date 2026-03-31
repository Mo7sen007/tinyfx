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

`src/tinyfx.gen.ts` is generated code and is overwritten on subsequent builds. Do not edit it manually.

## Project Layout
```text
my-app/
├── AGENTS.md
├── package.json
├── tinyfx.toml
├── tsconfig.json
├── src/
│   ├── main.ts
│   ├── pages/
│   │   ├── index.html
│   │   └── index.ts
│   ├── components/
│   ├── lib/
│   │   ├── dto/
│   │   ├── models/
│   │   ├── services/
│   │   └── state/
│   └── styles/
│       └── main.css
└── dist/
```

## Static vs Dynamic from Day One
- Static component: HTML only, expanded at build time. No `.ts` file.
- Dynamic component: HTML with `tinyfx-dynamic` on the root element and a same-name `.ts` module.

Compiler checks:
- `tinyfx-dynamic` without `.ts` → error.
- `.ts` without `tinyfx-dynamic` → warning.

## Useful Development Commands
```bash
tinyfx build --phase-one-only
tinyfx build --assets-dir public-assets
tinyfx serve --config tinyfx.toml
```
