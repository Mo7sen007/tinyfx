mod build;
mod config;
mod generate;
mod init;

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

use config::load_config;

#[derive(Debug, Parser)]
#[command(
    name = "tinyfx",
    version,
    about = "tinyfx — compile-time HTML component compiler"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Initialize a new tinyfx project
    Init {
        /// Project name (creates a directory with this name)
        name: String,

        /// Overwrite existing files
        #[arg(long, default_value_t = false)]
        force: bool,
    },

    /// Build the project (expand components, emit HTML/CSS/JS)
    Build {
        /// Path to config file
        #[arg(long, default_value = "tinyfx.toml")]
        config: PathBuf,

        #[arg(long, default_value_t = false)]
        phase_one_only: bool,
        /// Override the assets directory for this build
        #[arg(long)]
        assets_dir: Option<PathBuf>,
    },
    /// Generate code (components, services, interfaces, pages)
    G {
        #[command(subcommand)]
        generator: generate::Gen,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Init { name, force } => {
            init::run_init(&name, force)?;
        }
        Command::Build {
            config: config_path,
            phase_one_only,
            assets_dir,
        } => {
            let mut cfg = load_config(&config_path).unwrap_or_default();
            cfg.pahse_one_only = phase_one_only;
            if let Some(dir) = assets_dir {
                cfg.assets_dir = dir.to_string_lossy().into_owned();
            }
            build::run_build(&cfg)?;
        }
        Command::G { generator } => {
            generate::run_generate(&generator)?;
        }
    }

    Ok(())
}
