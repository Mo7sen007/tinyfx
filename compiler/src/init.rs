use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

/// Locate the template directory (examples/basic/template).
/// Searches relative to the compiler binary location.
fn find_template_dir() -> Result<PathBuf> {
    let exe = env::current_exe()
        .with_context(|| "Failed to get current executable path")?;
    
    // Navigate up from the binary to find the project root
    let mut dir = exe.parent();
    while let Some(parent) = dir {
        let template_dir = parent.join("examples/basic/template");
        if template_dir.exists() {
            return Ok(template_dir);
        }
        // Also check one level up (for cargo target/debug structure)
        let alt = parent.parent().and_then(|p| {
            let candidate = p.join("examples/basic/template");
            if candidate.exists() { Some(candidate) } else { None }
        });
        if let Some(found) = alt {
            return Ok(found);
        }
        dir = parent.parent();
    }
    
    bail!("Template directory not found. Expected examples/basic/template in project root.")
}

/// Read template file and replace {{RUNTIME_IMPORT}} with the computed path.
fn read_template(template_dir: &Path, rel_path: &str, runtime_import: &str) -> Result<String> {
    let path = template_dir.join(rel_path);
    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read template: {}", path.display()))?;
    Ok(content.replace("{{RUNTIME_IMPORT}}", runtime_import))
}

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
    let project_root = env::current_dir()
        .with_context(|| "Failed to resolve current working directory")?;
    let runtime_path = find_runtime_dir(&project_root);
    
    // Find template directory
    let template_dir = find_template_dir()?;
    
    // Compute runtime imports for different file depths
    let main_runtime_import = compute_runtime_import_for_file(&project_root, src, "main.ts", &runtime_path);
    let http_runtime_import =
        compute_runtime_import_for_file(&project_root, src, "lib/http/client.ts", &runtime_path);
    let state_runtime_import =
        compute_runtime_import_for_file(&project_root, src, "lib/state/app.state.ts", &runtime_path);

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
    let index_html = fs::read_to_string(template_dir.join("index.html"))?;
    let w = write_file(&base.join("index.html"), &index_html, force)?;
    print_result(&format!("{}/index.html", src), w);

    // ── main.ts (entry point) ────────────────────────────────────
    let main_ts = read_template(&template_dir, "main.ts", &main_runtime_import)?;
    let w = write_file(&base.join("main.ts"), &main_ts, force)?;
    print_result(&format!("{}/main.ts", src), w);

    // ── lib/http ─────────────────────────────────────────────────
    let http_client_ts = read_template(&template_dir, "lib/http/client.ts", &http_runtime_import)?;
    let w = write_file(&base.join("lib/http/client.ts"), &http_client_ts, force)?;
    print_result(&format!("{}/lib/http/client.ts", src), w);

    let http_index = fs::read_to_string(template_dir.join("lib/http/index.ts"))?;
    let w = write_file(&base.join("lib/http/index.ts"), &http_index, force)?;
    print_result(&format!("{}/lib/http/index.ts", src), w);

    // ── lib/dto ──────────────────────────────────────────────────
    let user_dto = fs::read_to_string(template_dir.join("lib/dto/user.dto.ts"))?;
    let w = write_file(&base.join("lib/dto/user.dto.ts"), &user_dto, force)?;
    print_result(&format!("{}/lib/dto/user.dto.ts", src), w);

    let dto_index = fs::read_to_string(template_dir.join("lib/dto/index.ts"))?;
    let w = write_file(&base.join("lib/dto/index.ts"), &dto_index, force)?;
    print_result(&format!("{}/lib/dto/index.ts", src), w);

    // ── lib/models ───────────────────────────────────────────────
    let user_model = fs::read_to_string(template_dir.join("lib/models/user.model.ts"))?;
    let w = write_file(&base.join("lib/models/user.model.ts"), &user_model, force)?;
    print_result(&format!("{}/lib/models/user.model.ts", src), w);

    let models_index = fs::read_to_string(template_dir.join("lib/models/index.ts"))?;
    let w = write_file(&base.join("lib/models/index.ts"), &models_index, force)?;
    print_result(&format!("{}/lib/models/index.ts", src), w);

    // ── lib/state ────────────────────────────────────────────────
    let app_state_ts = read_template(&template_dir, "lib/state/app.state.ts", &state_runtime_import)?;
    let w = write_file(&base.join("lib/state/app.state.ts"), &app_state_ts, force)?;
    print_result(&format!("{}/lib/state/app.state.ts", src), w);

    // ── components/Button ────────────────────────────────────────
    let button_html = fs::read_to_string(template_dir.join("components/Button/Button.html"))?;
    let w = write_file(&base.join("components/Button/Button.html"), &button_html, force)?;
    print_result(&format!("{}/components/Button/Button.html", src), w);

    let button_ts = fs::read_to_string(template_dir.join("components/Button/Button.ts"))?;
    let w = write_file(&base.join("components/Button/Button.ts"), &button_ts, force)?;
    print_result(&format!("{}/components/Button/Button.ts", src), w);

    // ── styles ───────────────────────────────────────────────────
    let main_css = fs::read_to_string(template_dir.join("styles/main.css"))?;
    let w = write_file(&base.join("styles/main.css"), &main_css, force)?;
    print_result(&format!("{}/styles/main.css", src), w);

    // ── Summary ──────────────────────────────────────────────────
    println!();
    println!("Done! Next steps:");
    println!("  1. Edit {}/index.html", src);
    println!("  2. Run: tinyfx build");
    println!();

    Ok(())
}

  /// Find the nearest runtime directory by walking up from project root.
  fn find_runtime_dir(project_root: &Path) -> PathBuf {
    let mut dir = project_root;
    loop {
      let candidate = dir.join("runtime");
      if candidate.exists() {
        return candidate;
      }
      match dir.parent() {
        Some(parent) => dir = parent,
        None => break,
      }
    }
    project_root.join("runtime")
  }

  /// Compute a relative import path from a file inside `src` to `runtime/`.
  fn compute_runtime_import_for_file(
    project_root: &Path,
    src: &str,
    file_rel: &str,
    runtime_path: &Path,
  ) -> String {
    let base_dir = project_root.join(src);
    let file_path = base_dir.join(file_rel);
    let file_dir = file_path.parent().unwrap_or(base_dir.as_path());
    let rel = diff_paths(file_dir, runtime_path);
    path_to_import(&rel)
  }

  fn diff_paths(from: &Path, to: &Path) -> PathBuf {
    let from_components: Vec<_> = from.components().collect();
    let to_components: Vec<_> = to.components().collect();

    let mut i = 0usize;
    while i < from_components.len()
      && i < to_components.len()
      && from_components[i] == to_components[i]
    {
      i += 1;
    }

    let mut result = std::path::PathBuf::new();
    for _ in i..from_components.len() {
      result.push("..");
    }
    for comp in &to_components[i..] {
      result.push(comp.as_os_str());
    }
    result
  }

  fn path_to_import(path: &Path) -> String {
    let mut s = path.to_string_lossy().replace('\\', "/");
    if s.is_empty() {
      s = "runtime".to_string();
    }
    if !s.starts_with('.') {
      s = format!("./{}", s);
    }
    s
  }
