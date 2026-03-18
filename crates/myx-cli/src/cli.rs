use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "myx")]
#[command(about = "myx Rust MVP CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Init {
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,
        #[arg(long, default_value_t = false)]
        force: bool,
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    Add {
        package: String,
        #[arg(long)]
        config: Option<PathBuf>,
        #[arg(
            long,
            default_value_t = false,
            help = "Disable interactive policy prompts"
        )]
        non_interactive: bool,
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    Inspect {
        package: String,
        #[arg(long)]
        config: Option<PathBuf>,
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    Build {
        #[arg(long)]
        target: String,
        #[arg(long)]
        package: Option<String>,
        #[arg(long)]
        config: Option<PathBuf>,
        #[arg(long, default_value_t = false)]
        json: bool,
    },
}
