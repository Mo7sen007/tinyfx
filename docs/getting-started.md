# Getting Started with TinyFX

This guide will help you set up your first TinyFX project and understand the basic workflow.

## Installation

Currently, TinyFX is built from source. You'll need the Rust toolchain installed.

1.  Clone the repository and build the compiler:
    ```bash
    cargo build --release
    ```
2.  Add the `tinyfx` binary to your PATH or use it directly from `target/release/tinyfx`.

## Creating a New Project

To initialize a new project, use the `init` command:

```bash
tinyfx init my-cool-app
cd my-cool-app
```

This creates a default project structure:
- `src/index.html`: Your main HTML file.
- `src/main.ts`: Application entry point.
- `src/components/`: Directory for your UI components.
- `tinyfx.toml`: Configuration file.
- `package.json`: TypeScript and build dependencies.

Install the npm dependencies:

```bash
npm install
```

## Basic Workflow

### 1. Define a Component

Create a new component using the generator:

```bash
tinyfx g component MyButton
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
import { TinyFxContext } from "@tinyfx/runtime/di";

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

In your `src/index.html`, use the `<component>` tag:

```html
<!DOCTYPE html>
<html>
<body>
  <h1>Welcome to TinyFX</h1>
  <component name="MyButton">Click Me!</component>
  
  <script type="module" src="./main.ts"></script>
</body>
</html>
```

### 3. Build the Project

Run the build command to expand components and generate the registry:

```bash
tinyfx build
```

The compiler will:
1.  Search `index.html` for `<component>` tags.
2.  Replace them with the content from `src/components/MyButton/MyButton.html`.
3.  Generate `src/tinyfx.gen.ts` which imports `MyButton` and wires it up.
4.  Output the final files to the `dist` directory.

### 4. Run the Development Server

TinyFX has a built-in dev server with **automatic live reload**. After building, point it at the output directory:

```bash
tinyfx serve dist
```

This starts an HTTP server at `http://localhost:3000` and a WebSocket server at `ws://localhost:3001`. Any time a file in `dist/` changes, connected browsers reload automatically — no manual refresh needed.

For a continuous build-and-reload workflow, run the build in watch mode in one terminal and `tinyfx serve` in another:

```bash
# Terminal 1
tinyfx build && npm run watch

# Terminal 2
tinyfx serve dist
```
