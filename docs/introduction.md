# Introduction to TinyFX

TinyFX is a lightweight, compiler-augmented web framework designed for building efficient, reactive web applications with minimal overhead. It combines a Rust-based compiler for static template expansion with a tiny TypeScript runtime for reactivity and behavior.

## Key Features

- **Zero-Runtime Templates**: Components are expanded at build time, resulting in standard HTML with no template parsing in the browser.
- **Explicit Reactivity**: A simple, powerful signal-based reactivity system (similar to SolidJS or Angular Signals) without proxies or magic.
- **TypeScript First**: Designed with first-class TypeScript support for runtime behavior and typed context.
- **Minimal Footprint**: The runtime is extremely small, providing only what you need: signals, DOM bindings, and a typed HTTP client.
- **Component-Based Architecture**: Build encapsulated components with their own HTML templates and TypeScript logic.

## How it Works

TinyFX uses a unique "compile-then-hydrate" approach:

1.  **Authoring**: You write HTML templates using `<component>` tags and TypeScript classes for behavior.
2.  **Compilation**: The TinyFX compiler (written in Rust) processes your HTML. It replaces `<component>` tags with their actual template content, injects properties, and handles slots—all at build time.
3.  **Generation**: The compiler generates a `tinyfx.gen.ts` file that wires up your TypeScript components to the DOM elements produced in the compilation step.
4.  **Runtime**: In the browser, TinyFX finds elements with the `data-component` attribute and "hydrates" them by instantiating your TypeScript classes.

## Project Structure

A typical TinyFX project looks like this:

- `src/components/`: Your components (each in its own folder with `.html` and `.ts` files).
- `src/pages/`: Page files with a `<page route="...">` root element.
- `src/main.ts`: App-level bootstrap code.
- `tinyfx.toml`: Configuration for the compiler.
