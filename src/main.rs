mod archive;
mod commands;
mod config;
mod metadata;
mod report;
mod validation;
mod zenodo;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "release-scholar",
    version,
    about = "Validate, audit, and package scholarly software releases"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize metadata files for a scholarly release
    Init {
        /// Path to the project directory
        #[arg(long, default_value = ".")]
        project_dir: PathBuf,
    },
    /// Validate project readiness for release
    Check {
        /// Path to the project directory
        #[arg(long, default_value = ".")]
        project_dir: PathBuf,
    },
    /// Build release archive and metadata bundle
    Build {
        /// Path to the project directory
        #[arg(long, default_value = ".")]
        project_dir: PathBuf,
    },
    /// Publish release bundle to Zenodo
    Publish {
        /// Path to the project directory
        #[arg(long, default_value = ".")]
        project_dir: PathBuf,
        /// Use Zenodo sandbox instead of production
        #[arg(long)]
        sandbox: bool,
        /// Actually publish (without this, creates a draft only)
        #[arg(long)]
        confirm: bool,
    },
    /// Set up push mirrors from Codeberg to GitHub/GitLab
    Mirror {
        /// Path to the project directory
        #[arg(long, default_value = ".")]
        project_dir: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Commands::Init { project_dir } => commands::init::run(&project_dir),
        Commands::Check { project_dir } => commands::check::run(&project_dir),
        Commands::Build { project_dir } => commands::build::run(&project_dir),
        Commands::Publish {
            project_dir,
            sandbox,
            confirm,
        } => commands::publish::run(&project_dir, sandbox, confirm),
        Commands::Mirror { project_dir } => commands::mirror::run(&project_dir),
    };
    if let Err(e) = result {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
