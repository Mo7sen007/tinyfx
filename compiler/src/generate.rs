//! # generate
//!
//! Code generation commands for `tinyfx generate`.
//!
//! ## Responsibilities
//! - Resolve embedded generator templates
//! - Render names into generated files
//! - Write generated files into the current TinyFX project
//!
//! ## What this module does NOT do
//! - It does not build the project
//! - It does not parse runtime HTML
//! - It does not serve or watch files

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::Subcommand;
use include_dir::{Dir, include_dir};

static GENERATORS: Dir = include_dir!("$CARGO_MANIFEST_DIR/templates/generate");

struct TemplatePaths;

impl TemplatePaths {
    const COMPONENT: &'static str = "component/Component.html";
    const COMPONENT_TS: &'static str = "component/Component.ts";
    const SERVICE: &'static str = "service/Service.ts";
    const MODEL: &'static str = "model/Model.ts";
    const DTO: &'static str = "dto/Dto.ts";
    const INTERFACE: &'static str = "interface/Interface.ts";
    const PAGE_HTML: &'static str = "page/Page.html";
    const PAGE_TS: &'static str = "page/Page.ts";
}

/// Supported `tinyfx generate` subcommands.
///
/// Each variant maps to a generator template under `compiler/templates/generate`.
#[derive(Debug, Subcommand)]
pub enum Gen {
    /// Generate a component
    #[command(name = "component", visible_alias = "c")]
    Component {
        /// Component name
        name: String,

        /// Generate a dynamic (interactive) component with lifecycle hooks and signals
        #[arg(long, short = 'd', default_value_t = false)]
        dynamic: bool,
        /// Overwrite existing files
        #[arg(long, default_value_t = false)]
        force: bool,
    },
    /// Generate a service
    #[command(name = "service", visible_alias = "s")]
    Service {
        /// Service name
        name: String,
        /// Overwrite existing files
        #[arg(long, default_value_t = false)]
        force: bool,
    },
    /// Generate a model
    #[command(name = "model", visible_alias = "m")]
    Model {
        /// Model name
        name: String,
        /// Overwrite existing files
        #[arg(long, default_value_t = false)]
        force: bool,
    },
    /// Generate a dto
    #[command(name = "dto", visible_alias = "d")]
    Dto {
        /// DTO name
        name: String,
        /// Overwrite existing files
        #[arg(long, default_value_t = false)]
        force: bool,
    },
    /// Generate an interface (legacy)
    /// Generate an interface
    #[command(name = "interface", visible_alias = "i")]
    Interface {
        /// Interface name
        name: String,
        /// Overwrite existing files
        #[arg(long, default_value_t = false)]
        force: bool,
    },
    /// Generate a page
    #[command(name = "page", visible_alias = "p")]
    Page {
        /// Page name
        name: String,
        /// Overwrite existing files
        #[arg(long, default_value_t = false)]
        force: bool,
    },
}

/// Dispatch a `tinyfx generate` subcommand.
///
/// This resolves the requested generator template, normalizes names, and
/// writes the generated files into the current TinyFX project.
///
/// # Errors
/// Returns an error when the current directory is not inside a TinyFX project,
/// when a target file already exists without `--force`, or when a generator
/// template cannot be read or written.
pub fn run_generate(cmd: &Gen) -> Result<()> {
    match cmd {
        Gen::Component {
            name,
            force,
            dynamic,
        } => gen_component(name, *force, *dynamic),
        Gen::Service { name, force } => gen_service(name, *force),
        Gen::Model { name, force } => gen_model(name, *force),
        Gen::Dto { name, force } => gen_dto(name, *force),
        Gen::Interface { name, force } => gen_interface(name, *force),
        Gen::Page { name, force } => gen_page(name, *force),
    }
}

fn gen_component(name: &str, force: bool, dynamic: bool) -> Result<()> {
    let names = NameSet::new(name)?;
    let base = project_root()?;
    let mut files = Vec::new();

    let component_dir = base.join("src").join("components").join(&names.pascal);

    if component_dir.exists() && !force {
        bail!(
            "refusing to overwrite existing component directory '{}'\n  → pass --force to replace the generated files",
            component_dir.display()
        );
    }

    let html = render_component_template(TemplatePaths::COMPONENT, &names, dynamic)?;
    let html_path = component_dir.join(format!("{}.html", names.pascal));
    files.push((html_path, html));
    if dynamic {
        let ts = render_template(TemplatePaths::COMPONENT_TS, &names)?;
        let ts_path = component_dir.join(format!("{}.ts", names.pascal));
        files.push((ts_path, ts));
    }

    for (path, content) in files {
        write_file(&path, &content, force)?;
    }

    println!("Generated component: {}", names.pascal);
    Ok(())
}

fn gen_service(name: &str, force: bool) -> Result<()> {
    let names = ServiceNameSet::new(name)?;
    let base = project_root()?;
    let dir = base.join("src").join("lib").join("services");
    let ts = read_template(TemplatePaths::SERVICE)?.replace("{{Name}}", &names.display_name);
    let ts_path = dir.join(format!("{}.service.ts", names.file_stem));

    write_file(&ts_path, &ts, force)?;
    println!("Generated service: {}", names.service_type_name());
    Ok(())
}

fn gen_model(name: &str, force: bool) -> Result<()> {
    let names = NameSet::new(name)?;
    let base = project_root()?;
    let dir = base.join("src").join("lib").join("models");
    let ts = render_template(TemplatePaths::MODEL, &names)?;
    let ts_path = dir.join(format!("{}.model.ts", names.kebab));

    write_file(&ts_path, &ts, force)?;
    println!("Generated model: {}", names.pascal);
    Ok(())
}

fn gen_dto(name: &str, force: bool) -> Result<()> {
    let names = NameSet::new(name)?;
    let base = project_root()?;
    let dir = base.join("src").join("lib").join("dto");
    let ts = render_template(TemplatePaths::DTO, &names)?;
    let ts_path = dir.join(format!("{}.dto.ts", names.kebab));

    write_file(&ts_path, &ts, force)?;
    println!("Generated dto: {}", names.pascal);
    Ok(())
}

fn gen_interface(name: &str, force: bool) -> Result<()> {
    let names = NameSet::new(name)?;
    let base = project_root()?;
    let dir = base.join("src").join("lib");
    let ts = render_template(TemplatePaths::INTERFACE, &names)?;
    let ts_path = dir.join(format!("{}.interface.ts", names.kebab));

    write_file(&ts_path, &ts, force)?;
    println!("Generated interface: {}", names.pascal);
    Ok(())
}

fn gen_page(name: &str, force: bool) -> Result<()> {
    let names = NameSet::new(name)?;
    let base = project_root()?;
    let pages_dir = base.join("src").join("pages");
    let route = page_route(&names.kebab);
    let html = render_template(TemplatePaths::PAGE_HTML, &names)?.replace("{{route}}", &route);
    let ts = render_template(TemplatePaths::PAGE_TS, &names)?;

    let html_path = pages_dir.join(format!("{}.html", names.kebab));
    let ts_path = pages_dir.join(format!("{}.ts", names.kebab));

    if (html_path.exists() || ts_path.exists()) && !force {
        bail!(
            "refusing to overwrite existing page files '{}' and '{}'\n  → pass --force to replace the generated files",
            html_path.display(),
            ts_path.display()
        );
    }

    write_file(&html_path, &html, force)?;
    write_file(&ts_path, &ts, force)?;

    println!("Generated page: {}", names.pascal);
    Ok(())
}

fn page_route(kebab_name: &str) -> String {
    if kebab_name == "index" {
        "/".to_string()
    } else {
        format!("/{}", kebab_name)
    }
}

fn project_root() -> Result<PathBuf> {
    let mut dir = std::env::current_dir().context("Failed to get current directory")?;

    loop {
        if dir.join("tinyfx.toml").exists() {
            return Ok(dir);
        }

        let Some(parent) = dir.parent() else {
            bail!(
                "not inside a TinyFX project\n  → run this command from a directory containing tinyfx.toml"
            );
        };

        dir = parent.to_path_buf();
    }
}

fn read_template(path: &str) -> Result<String> {
    let file = GENERATORS
        .get_file(path)
        .with_context(|| format!("Generator template not found: {}", path))?;
    let contents = file
        .contents_utf8()
        .with_context(|| format!("Template is not valid UTF-8: {}", path))?;
    Ok(contents.to_string())
}

fn render_template(path: &str, names: &NameSet) -> Result<String> {
    let template = read_template(path)?;
    Ok(template
        .replace("{{Name}}", &names.pascal)
        .replace("{{name}}", &names.kebab)
        .replace("{{camelName}}", &names.camel))
}

fn render_component_template(path: &str, names: &NameSet, dynamic: bool) -> Result<String> {
    let template = read_template(path)?;
    let root_marker = if dynamic { " tinyfx-dynamic" } else { "" };
    Ok(template
        .replace("{{Name}}", &names.pascal)
        .replace("{{name}}", &names.kebab)
        .replace("{{camelName}}", &names.camel)
        .replace("{{rootMarker}}", root_marker))
}

fn write_file(path: &Path, content: &str, force: bool) -> Result<()> {
    if path.exists() && !force {
        bail!(
            "refusing to overwrite existing file '{}'\n  → pass --force to replace it",
            path.display()
        );
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    fs::write(path, content)
        .with_context(|| format!("Failed to write file: {}", path.display()))?;
    Ok(())
}

struct NameSet {
    pascal: String,
    camel: String,
    kebab: String,
}

struct ServiceNameSet {
    display_name: String,
    file_stem: String,
}

impl NameSet {
    fn new(input: &str) -> Result<Self> {
        let input = input.trim();
        if input.is_empty() {
            bail!("name cannot be empty\n  → provide a non-empty resource name");
        }

        let words = split_words(input);
        if words.is_empty() {
            bail!(
                "name '{}' is invalid\n  → use letters or numbers in the resource name",
                input
            );
        }

        let pascal = words.iter().map(|w| capitalize(w)).collect::<String>();

        let camel = {
            let mut iter = words.iter();
            let first = iter.next().map(|w| w.to_lowercase()).unwrap_or_default();
            let rest = iter.map(|w| capitalize(w)).collect::<String>();
            format!("{}{}", first, rest)
        };

        let kebab = words
            .iter()
            .map(|w| w.to_lowercase())
            .collect::<Vec<_>>()
            .join("-");

        Ok(Self {
            pascal,
            camel,
            kebab,
        })
    }
}

impl ServiceNameSet {
    fn new(input: &str) -> Result<Self> {
        let names = NameSet::new(input)?;
        let display_name = if names.pascal.ends_with("Service") {
            names
                .pascal
                .strip_suffix("Service")
                .unwrap_or(&names.pascal)
                .to_string()
        } else {
            names.pascal.clone()
        };

        let file_stem = strip_trailing_segment(&names.kebab, "service");

        Ok(Self {
            display_name,
            file_stem,
        })
    }

    fn service_type_name(&self) -> String {
        format!("{}Service", self.display_name)
    }
}

fn strip_trailing_segment(value: &str, segment: &str) -> String {
    value
        .strip_suffix(&format!("-{}", segment))
        .unwrap_or(value)
        .to_string()
}

fn split_words(input: &str) -> Vec<String> {
    let mut words: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut prev_is_lower = false;

    for ch in input.chars() {
        if !ch.is_ascii_alphanumeric() {
            if !current.is_empty() {
                words.push(current.clone());
                current.clear();
            }
            prev_is_lower = false;
            continue;
        }

        let is_upper = ch.is_ascii_uppercase();
        if is_upper && prev_is_lower && !current.is_empty() {
            words.push(current.clone());
            current.clear();
        }

        current.push(ch);
        prev_is_lower = ch.is_ascii_lowercase();
    }

    if !current.is_empty() {
        words.push(current);
    }

    words
}

fn capitalize(word: &str) -> String {
    let mut chars = word.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    format!(
        "{}{}",
        first.to_ascii_uppercase(),
        chars.as_str().to_ascii_lowercase()
    )
}
