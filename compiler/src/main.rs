mod config;
mod init;

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

use config::{ensure_out_dirs, load_config};

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
    /// Initialize a new tinyfx project structure
    Init {
        /// Source directory name (default: "frontend")
        #[arg(long, default_value = "frontend")]
        src: String,

        /// Output directory name (default: "dist")
        #[arg(long, default_value = "dist")]
        out: String,

        /// Overwrite existing files
        #[arg(long, default_value_t = false)]
        force: bool,
    },

    /// Build the project (expand components, emit HTML/CSS/JS)
    Build {
        /// Path to config file
        #[arg(long, default_value = "tinyfx.toml")]
        config: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Init { src, out, force } => {
            init::run_init(&src, &out, force)?;
        }
        Command::Build { config: config_path } => {
            let cfg = load_config(&config_path).unwrap_or_default();

            ensure_out_dirs(&cfg.out)?;

            println!("tinyfx build:");
            println!("  src:        {}", cfg.src);
            println!("  out:        {}", cfg.out);
            println!("  components: {}", cfg.components);
            println!("  entry:      {}", cfg.entry);
            println!();
            println!("  (compiler pipeline not yet implemented)");
        }
    }

    Ok(())
}
