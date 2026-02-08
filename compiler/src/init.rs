use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};
use include_dir::{include_dir, Dir};

/// Templates are embedded at compile time from examples/basic/template/
static TEMPLATES: Dir = include_dir!("$CARGO_MANIFEST_DIR/../examples/basic/template");

/// Write a file only if it doesn't already exist (unless force=true).
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

/// Read an embedded template file.
fn read_template(rel_path: &str) -> Result<String> {
    let file = TEMPLATES
        .get_file(rel_path)
        .with_context(|| format!("Embedded template not found: {}", rel_path))?;
    Ok(String::from_utf8_lossy(file.contents()).into_owned())
}

pub fn run_init(name: &str, force: bool) -> Result<()> {
    let base = Path::new(name);

    println!();
    println!("tinyfx init → {}/", name);
    println!();

    // ── tinyfx.toml ──────────────────────────────────────────────
    let config_content = r#"# tinyfx project configuration

entry_html = "src/index.html"
entry_ts   = "src/main.ts"
out_dir    = "dist"
components = "src/components"
styles     = "src/styles"
"#;
    let w = write_file(&base.join("tinyfx.toml"), config_content, force)?;
    print_result(&format!("{}/tinyfx.toml", name), w);

    // ── package.json ─────────────────────────────────────────────
    let pkg = format!(
        r#"{{
  "name": "{}",
  "version": "0.1.0",
  "private": true,
  "type": "module",
  "scripts": {{
    "build": "tinyfx build"
  }},
  "dependencies": {{
    "@tinyfx/runtime": "^0.1.0"
  }},
  "devDependencies": {{
    "esbuild": "^0.27.0"
  }}
}}
"#,
        name
    );
    let w = write_file(&base.join("package.json"), &pkg, force)?;
    print_result(&format!("{}/package.json", name), w);

    // ── tsconfig.json ────────────────────────────────────────────
    let tsconfig = r#"{
  "compilerOptions": {
    "target": "ES2015",
    "lib": ["ES2015", "DOM"],
    "module": "ESNext",
    "moduleResolution": "node",
    "strict": true,
    "outDir": "./dist"
  },
  "include": ["src/**/*.ts"]
}
"#;
    let w = write_file(&base.join("tsconfig.json"), tsconfig, force)?;
    print_result(&format!("{}/tsconfig.json", name), w);

    // ── Directories ──────────────────────────────────────────────
    let dirs = [
        "src/components/Button",
        "src/lib/http",
        "src/lib/dto",
        "src/lib/models",
        "src/lib/state",
        "src/styles",
    ];
    for d in &dirs {
        let dir_path = base.join(d);
        if !dir_path.exists() {
            fs::create_dir_all(&dir_path)
                .with_context(|| format!("Failed to create {}", dir_path.display()))?;
        }
    }
    println!("  ✔ src/ directory structure");

    // ── Copy embedded template files ─────────────────────────────
    let template_files = [
        ("index.html",                    "src/index.html"),
        ("main.ts",                       "src/main.ts"),
        ("lib/http/client.ts",            "src/lib/http/client.ts"),
        ("lib/http/index.ts",             "src/lib/http/index.ts"),
        ("lib/dto/user.dto.ts",           "src/lib/dto/user.dto.ts"),
        ("lib/dto/index.ts",              "src/lib/dto/index.ts"),
        ("lib/models/user.model.ts",      "src/lib/models/user.model.ts"),
        ("lib/models/index.ts",           "src/lib/models/index.ts"),
        ("lib/state/app.state.ts",        "src/lib/state/app.state.ts"),
        ("components/Button/Button.html", "src/components/Button/Button.html"),
        ("components/Button/Button.ts",   "src/components/Button/Button.ts"),
        ("styles/main.css",               "src/styles/main.css"),
    ];

    for (template_path, dest_path) in &template_files {
        let content = read_template(template_path)?;
        let w = write_file(&base.join(dest_path), &content, force)?;
        print_result(&format!("{}/{}", name, dest_path), w);
    }

    // ── Run npm install ──────────────────────────────────────────
    println!();
    println!("  Installing dependencies...");
    let npm_status = Command::new("npm")
        .arg("install")
        .current_dir(base)
        .status();

    match npm_status {
        Ok(status) if status.success() => {
            println!("  ✔ npm install complete");
        }
        Ok(_) => {
            println!("  ⚠ npm install failed — run it manually:");
            println!("    cd {} && npm install", name);
        }
        Err(_) => {
            println!("  ⚠ npm not found — install dependencies manually:");
            println!("    cd {} && npm install", name);
        }
    }

    // ── Summary ──────────────────────────────────────────────────
    println!();
    println!("Done! Next steps:");
    println!("  cd {}", name);
    println!("  tinyfx build");
    println!();

    Ok(())
}
