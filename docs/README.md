# TinyFX Docs

This folder contains topic-focused documentation that matches the current implementation in the compiler and runtime.

## Sections
- `introduction.md`: Framework model, architecture split, and design intent.
- `getting-started.md`: Install, scaffold, run, and first build workflow.
- `cli.md`: Current CLI commands, flags, and generator behavior.
- `components.md`: Static vs dynamic components, wrappers, `<each>`, and `<if>`.
- `signals.md`: `signal()` and `effect()` patterns used by runtime and generated glue.
- `dom-helpers.md`: `bindText`, `bindAttr`, and `bindClass` helpers.
- `http-client.md`: `createHttp()` usage and error classes.
- `routing.md`: Hard routing model and route param helpers.
- `dependency-injection.md`: Current context and service-factory approach.

## Source of Truth
When docs conflict, follow implementation in:
- `compiler/src/build/mod.rs`
- `compiler/src/build/glue.rs`
- `compiler/src/build/wrapper.rs`
- `runtime/src/index.ts`

The generated `src/tinyfx.gen.ts` in an actual TinyFX app reflects real emitted behavior and should be used to verify runtime wiring details.
