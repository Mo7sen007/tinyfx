use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

/// Write a file only if it doesn't already exist (unless force=true).
/// Returns true if the file was written, false if skipped.
fn write_file(path: &Path, content: &str, force: bool) -> Result<bool> {
    if path.exists() && !force {
        return Ok(false);
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory {}", parent.display()))?;
    }
    fs::write(path, content)
        .with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(true)
}

fn print_result(path: &str, written: bool) {
    if written {
        println!("  ✔ {}", path);
    } else {
        println!("  · {} (exists, skipped)", path);
    }
}

pub fn run_init(src: &str, out: &str, force: bool) -> Result<()> {
    let base = Path::new(src);

    println!();
    println!("tinyfx init");
    println!();

    // ── tinyfx.toml ──────────────────────────────────────────────
    let config_content = format!(
        r#"# tinyfx project configuration
# All paths are relative to the project root.

src = "{src}"
out = "{out}"
components = "{src}/components"
entry = "{src}/main.ts"
"#
    );
    let w = write_file(Path::new("tinyfx.toml"), &config_content, force)?;
    print_result("tinyfx.toml", w);

    // ── Directories ──────────────────────────────────────────────
    let dirs = [
        "lib/http",
        "lib/dto",
        "lib/models",
        "lib/state",
        "components/Button",
        "styles",
    ];
    for d in &dirs {
        let dir_path = base.join(d);
        if !dir_path.exists() {
            fs::create_dir_all(&dir_path)
                .with_context(|| format!("Failed to create {}", dir_path.display()))?;
            println!("  ✔ {}/{}/", src, d);
        }
    }

    // ── index.html ───────────────────────────────────────────────
    let w = write_file(
        &base.join("index.html"),
        INDEX_HTML,
        force,
    )?;
    print_result(&format!("{}/index.html", src), w);

    // ── main.ts (entry point) ────────────────────────────────────
    let w = write_file(
        &base.join("main.ts"),
        MAIN_TS,
        force,
    )?;
    print_result(&format!("{}/main.ts", src), w);

    // ── lib/http ─────────────────────────────────────────────────
    let w = write_file(&base.join("lib/http/client.ts"), HTTP_CLIENT_TS, force)?;
    print_result(&format!("{}/lib/http/client.ts", src), w);

    let w = write_file(&base.join("lib/http/index.ts"), HTTP_INDEX_TS, force)?;
    print_result(&format!("{}/lib/http/index.ts", src), w);

    // ── lib/dto ──────────────────────────────────────────────────
    let w = write_file(&base.join("lib/dto/user.dto.ts"), USER_DTO_TS, force)?;
    print_result(&format!("{}/lib/dto/user.dto.ts", src), w);

    let w = write_file(&base.join("lib/dto/index.ts"), DTO_INDEX_TS, force)?;
    print_result(&format!("{}/lib/dto/index.ts", src), w);

    // ── lib/models ───────────────────────────────────────────────
    let w = write_file(&base.join("lib/models/user.model.ts"), USER_MODEL_TS, force)?;
    print_result(&format!("{}/lib/models/user.model.ts", src), w);

    let w = write_file(&base.join("lib/models/index.ts"), MODELS_INDEX_TS, force)?;
    print_result(&format!("{}/lib/models/index.ts", src), w);

    // ── lib/state ────────────────────────────────────────────────
    let w = write_file(&base.join("lib/state/app.state.ts"), APP_STATE_TS, force)?;
    print_result(&format!("{}/lib/state/app.state.ts", src), w);

    // ── components/Button ────────────────────────────────────────
    let w = write_file(&base.join("components/Button/Button.html"), BUTTON_HTML, force)?;
    print_result(&format!("{}/components/Button/Button.html", src), w);

    let w = write_file(&base.join("components/Button/Button.ts"), BUTTON_TS, force)?;
    print_result(&format!("{}/components/Button/Button.ts", src), w);

    // ── styles ───────────────────────────────────────────────────
    let w = write_file(&base.join("styles/main.css"), MAIN_CSS, force)?;
    print_result(&format!("{}/styles/main.css", src), w);

    // ── Summary ──────────────────────────────────────────────────
    println!();
    println!("Done! Next steps:");
    println!("  1. Edit {}/index.html", src);
    println!("  2. Run: tinyfx build");
    println!();

    Ok(())
}

// ─── File templates ──────────────────────────────────────────────────────────

const INDEX_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>tinyfx app</title>
  <link rel="stylesheet" href="./styles/main.css" />
</head>
<body>

  <!-- Example: use a compiled component -->
  <component name="Button" variant="primary">
    Click me
  </component>

  <p id="output"></p>

  <!-- Entry point — tinyfx build will bundle this -->
  <script type="module" src="./main.ts"></script>
</body>
</html>
"#;

const MAIN_TS: &str = r#"// tinyfx entry point
// This file runs after the page loads and initializes
// any compiled components that have behavior attached.

import { initComponents } from "tinyfx/runtime";
import { signal, effect } from "tinyfx/runtime";

// Initialize all <data-component> elements on the page
document.addEventListener("DOMContentLoaded", () => {
  initComponents();
});

// Example: reactive counter
const count = signal(0);

effect(() => {
  const el = document.getElementById("output");
  if (el) {
    el.textContent = `Count: ${count()}`;
  }
});
"#;

const HTTP_CLIENT_TS: &str = r#"// HTTP client — thin typed wrapper over fetch.
// Configure your base URL and use api.get/post throughout the app.

import { createHttp } from "tinyfx/runtime";

export const api = createHttp({
  baseUrl: "/api",
});
"#;

const HTTP_INDEX_TS: &str = r#"export { api } from "./client";
"#;

const USER_DTO_TS: &str = r#"// DTO: raw shape coming from the API.
// This matches the backend JSON contract exactly.

export interface UserDto {
  id: number;
  name: string;
  created_at: string;
}
"#;

const DTO_INDEX_TS: &str = r#"export * from "./user.dto";
"#;

const USER_MODEL_TS: &str = r#"// Model: frontend-safe object built from a DTO.
// Handles type conversions (e.g. string → Date).

import type { UserDto } from "../dto/user.dto";

export class User {
  constructor(
    public id: number,
    public name: string,
    public createdAt: Date,
  ) {}

  static from(dto: UserDto): User {
    return new User(dto.id, dto.name, new Date(dto.created_at));
  }
}
"#;

const MODELS_INDEX_TS: &str = r#"export * from "./user.model";
"#;

const APP_STATE_TS: &str = r#"// Application-level reactive state.
// Import signals from the runtime and define shared state here.

import { signal } from "tinyfx/runtime";

// Example: a global counter signal
export const count = signal(0);
"#;

const BUTTON_HTML: &str = r#"<!-- Button component template.
     {{variant}} is replaced at compile time from the usage attributes.
     <slot></slot> is replaced with the children of the <component> tag. -->
<button class="btn btn-{{variant}}" data-component="Button">
  <slot></slot>
</button>
"#;

const BUTTON_TS: &str = r#"// Optional component behavior.
// This runs at runtime for each <data-component="Button"> element.

export function init(el: HTMLElement) {
  el.addEventListener("click", () => {
    console.log("Button clicked:", el.textContent?.trim());
  });
}
"#;

const MAIN_CSS: &str = r#"/* tinyfx — base styles */

*,
*::before,
*::after {
  box-sizing: border-box;
}

body {
  font-family: system-ui, -apple-system, sans-serif;
  margin: 2rem;
  line-height: 1.5;
}

.btn {
  display: inline-block;
  padding: 0.5rem 1rem;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 1rem;
}

.btn-primary {
  background: #3b82f6;
  color: #fff;
}

.btn-primary:hover {
  background: #2563eb;
}
"#;
