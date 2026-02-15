# CLI Reference

The `tinyfx` CLI is the main tool for managing your project and building your application.

## Configuration: `tinyfx.toml`

The behavior of the CLI is controlled by a `tinyfx.toml` file in your project root.

```toml
# Main HTML entry point
entry_html = "src/index.html"

# TypeScript entry point (where setupComponents is called)
entry_ts   = "src/main.ts"

# Output directory for the build
out_dir    = "dist"

# Path to the components directory
components = "src/components"

# Path to static assets/styles (optional)
styles     = "src/styles"
```

## Commands

### `init <name>`
Initializes a new TinyFX project in the specified directory. It creates the folder structure, default configuration, and basic boilerplate files.

```bash
tinyfx init my-app
```

### `build`
Compiles the project. It performs the following steps:
1.  **Template Expansion**: Scans the `entry_html` for `<component>` tags and replaces them with corresponding template files from the `components` directory.
2.  **Property Interpolation**: Replaces `{{ prop }}` placeholders with attribute values.
3.  **Slot Injection**: Places children of component tags into `<slot></slot>` locations.
4.  **Code Generation**: Scans for `.ts` files in the component directories and generates `tinyfx.gen.ts`.
5.  **Output**: Writes the final HTML and generated TypeScript files to the `out_dir`.

```bash
tinyfx build
```

### `g <type> <name>`
Generates new files based on templates.

- **`g component <name>`**: Creates a new component folder with HTML and TS files.
- **`g service <name>`**: Creates a new service file.
- **`g interface <name>`**: Creates a new TypeScript interface.
- **`g page <name>`**: Creates a new page component.

```bash
tinyfx g component UserProfile
tinyfx g service AuthService
```

## Build Phases

TinyFX can sometimes be run in "Phase 1 only" mode (mostly for internal use) which just expands components without doing a full build.

```bash
tinyfx build --phase-one-only
```
