# TinyFX Documentation

Welcome to the official TinyFX documentation — the primary reference for using the
compiler, runtime, and project conventions.

## Contents

1.  [**Introduction**](introduction.md) - What TinyFX is and its design goals.
2.  [**Getting Started**](getting-started.md) - Create a project and the basic workflow.
3.  [**CLI Reference**](cli.md) - Commands for development and code generation.
4.  [**Components**](components.md) - Component templates, tokens, slots, and dynamic mounting.
5.  [**Signals & Reactivity**](signals.md) - The runtime signal primitives and patterns.
6.  [**DOM Bindings**](dom-helpers.md) - Helpers to bind signals to DOM.
7.  [**HTTP Client**](http-client.md) - `createHttp()` and error handling patterns.
8.  [**Context & Services**](dependency-injection.md) - Patterns for services and shared state.

Troubleshooting & Conventions
- The compiler writes routed pages to `dist/` using the pattern `dist/<segments>/index.html`.
	Requests to `/foo` should therefore resolve to `dist/foo/index.html` — the dev server
	supports this layout automatically.
- Dynamic components must include the `tinyfx-dynamic` marker in the component HTML
	and provide a matching `.ts` file that exports a `mount()` helper. See Components.

If you can't find what you need here, open an issue or look at the example apps in `/examples/`.

---

TinyFX is designed to be simple, fast, and maintainable. If you're coming from Angular, Vue, or React, you'll find many concepts familiar, but with a focus on build-time optimization and minimal runtime overhead.
