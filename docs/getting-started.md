# Getting Started with TinyFX

This guide helps you set up a TinyFX project and explains the typical development workflow.

## Installation

TinyFX's compiler is a Rust binary. For local development you'll need the Rust toolchain and
Node.js (for runtime tooling like esbuild). The easiest way to get started is to build the
compiler locally and use it to scaffold a new project.

1. Install Rust toolchain (if you don't have it):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. Build the compiler binary from the workspace root:

```bash
cargo build --manifest-path compiler/Cargo.toml --release
```

The compiled `tinyfx` binary will be at `compiler/target/release/tinyfx`. You can copy it into
`$PATH` or invoke it from that location while developing.

## Creating a New Project

To initialize a new project, use the `init` command:

```bash
tinyfx init my-cool-app
cd my-cool-app
```

This creates a default project structure:
- `src/pages/index.html`: Your home page.
- `src/main.ts`: Application entry point.
- `src/components/`: Directory for your UI components.
- `tinyfx.toml`: Configuration file.
- `package.json`: TypeScript and build dependencies.

Install the Node dependencies used by the runtime and tooling (esbuild, TypeScript):

```bash
npm install
```

## Basic Workflow

### 1. Define a Component

Create a new component using the generator:

```bash
tinyfx generate component MyButton
```

This will create `src/components/MyButton/MyButton.html` and `src/components/MyButton/MyButton.ts`.

#### MyButton.html
```html
<button id="btn" class="btn">
  <slot></slot>
</button>
```

#### MyButton.ts
```typescript
import type { TinyFxContext } from "@tinyfx/runtime";

export class MyButton {
  constructor(public el: HTMLElement, public ctx: TinyFxContext) {}

  init() {
    this.el.addEventListener('click', () => {
      console.log('Button clicked!');
    });
  }
}

export function mount(el: HTMLElement, ctx: TinyFxContext) {
  const inst = new MyButton(el, ctx);
  inst.init();
  return inst;
}
```

### 2. Use the Component

In your `src/pages/index.html`, use the component tag directly:

```html
<page route="/" layout="">
  <main>
    <h1>Welcome to TinyFX</h1>
    <MyButton>Click Me!</MyButton>
  </main>
</page>
```

### 3. Build the Project

Build the project (one-off build):

```bash
tinyfx build
```

The compiler performs the full build pipeline (component expansion, page processing,
glue generation, and asset bundling). Important notes:

- Pages are written to `dist/` as routed output. Nested pages map to `dist/<segments>/index.html`.
- The compiler generates `src/tinyfx.gen.ts` — this file is auto-generated and overwritten on each build.
- When the project is built, `dist/js/main.js` is produced by the asset pipeline.

### 4. Run the Development Server

Development server with live reload

```bash
tinyfx serve
```

`tinyfx serve` performs an initial build, starts an HTTP server on port `3000`, and a WebSocket
live-reload server on port `3001`. The server watches files and triggers rebuilds; browsers
connected to the dev server will automatically reload when the compiler writes new output to
`dist/`.

If you prefer to manage your own watch/build tooling you can run `tinyfx build` manually or
use your own bundler/watch pipeline. The canonical dev loop is:

```bash
# Terminal 1 — optional: run a file watcher or your own bundler
npm run watch

# Terminal 2 — start the dev server
tinyfx serve
```

Troubleshooting 404s when creating pages

- The server resolves routes to files using the convention `dist/<segments>/index.html`. If you
  request `/foo`, the server will serve `dist/foo/index.html`. If you see a 404 immediately after
  creating a page, wait a few moments for the compiler to finish writing files; we also recommend
  using the built-in `tinyfx serve` which performs an initial build and then rebuilds on changes.
