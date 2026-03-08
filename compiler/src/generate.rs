use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::Subcommand;
use include_dir::{Dir, include_dir};

static GENERATORS: Dir = include_dir!("$CARGO_MANIFEST_DIR/../examples/generators");

#[derive(Debug, Subcommand)]
pub enum Gen {
    /// Generate a component
    #[command(name = "component", visible_alias = "c")]
    Component {
        /// Component name
        name: String,
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

pub fn run_generate(cmd: &Gen) -> Result<()> {
    match cmd {
        Gen::Component { name, force } => gen_component(name, *force),
        Gen::Service { name, force } => gen_service(name, *force),
        Gen::Interface { name, force } => gen_interface(name, *force),
        Gen::Page { name, force } => gen_page(name, *force),
    }
}

fn gen_component(name: &str, force: bool) -> Result<()> {
    let names = NameSet::new(name)?;
    let base = project_root()?;
    let component_dir = base.join("src").join("components").join(&names.pascal);

    let html = render_template("component/Component.html", &names)?;
    let ts = render_template("component/Component.ts", &names)?;

    let html_path = component_dir.join(format!("{}.html", names.pascal));
    let ts_path = component_dir.join(format!("{}.ts", names.pascal));

    write_file(&html_path, &html, force)?;
    write_file(&ts_path, &ts, force)?;

    println!("Generated component: {}", names.pascal);
    Ok(())
}

fn gen_service(name: &str, force: bool) -> Result<()> {
    let names = ServiceNameSet::new(name)?;
    let base = project_root()?;
    let dir = base.join("src").join("lib").join("services");
    let ts = read_template("service/Service.ts")?.replace("{{Name}}", &names.class_name);
    let ts_path = dir.join(format!("{}.service.ts", names.file_stem));

    write_file(&ts_path, &ts, force)?;
    println!("Generated service: {}", names.class_name);
    Ok(())
}

fn gen_interface(name: &str, force: bool) -> Result<()> {
    let names = NameSet::new(name)?;
    let base = project_root()?;
    let dir = base.join("src").join("lib");
    let ts = render_template("interface/Interface.ts", &names)?;
    let ts_path = dir.join(format!("{}.interface.ts", names.kebab));

    write_file(&ts_path, &ts, force)?;
    println!("Generated interface: {}", names.pascal);
    Ok(())
}

fn gen_page(name: &str, force: bool) -> Result<()> {
    let names = NameSet::new(name)?;
    let base = project_root()?;
    let dir = base.join("src").join("pages").join(&names.kebab);
    let html = render_template("page/Page.html", &names)?;
    let ts = render_template("page/Page.ts", &names)?;

    let html_path = dir.join("index.html");
    let ts_path = dir.join("index.ts");

    write_file(&html_path, &html, force)?;
    write_file(&ts_path, &ts, force)?;

    println!("Generated page: {}", names.pascal);
    Ok(())
}

fn project_root() -> Result<PathBuf> {
    let mut dir = std::env::current_dir().context("Failed to get current directory")?;

    loop {
        if dir.join("tinyfx.toml").exists() {
            return Ok(dir);
        }

        let Some(parent) = dir.parent() else {
            bail!("Not inside a tinyfx project (missing tinyfx.toml)");
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

fn write_file(path: &Path, content: &str, force: bool) -> Result<bool> {
    if path.exists() && !force {
        bail!(
            "Refusing to overwrite existing file: {} (use --force)",
            path.display()
        );
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    fs::write(path, content)
        .with_context(|| format!("Failed to write file: {}", path.display()))?;
    Ok(true)
}

struct NameSet {
    pascal: String,
    camel: String,
    kebab: String,
}

struct ServiceNameSet {
    class_name: String,
    file_stem: String,
}

impl NameSet {
    fn new(input: &str) -> Result<Self> {
        let input = input.trim();
        if input.is_empty() {
            bail!("Name cannot be empty");
        }

        let words = split_words(input);
        if words.is_empty() {
            bail!("Name is invalid");
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
        let class_name = if names.pascal.ends_with("Service") {
            names.pascal.clone()
        } else {
            format!("{}Service", names.pascal)
        };

        let file_stem = strip_trailing_segment(&names.kebab, "service");

        Ok(Self {
            class_name,
            file_stem,
        })
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
