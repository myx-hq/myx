use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "myx")]
#[command(about = "myx Rust MVP CLI scaffold")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Init,
    Add { package: String },
    Inspect { package: String },
    Build { #[arg(long)] target: String },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => println!("init scaffold: TODO"),
        Commands::Add { package } => println!("add package: {package}"),
        Commands::Inspect { package } => println!("inspect package: {package}"),
        Commands::Build { target } => println!("build target: {target}"),
    }
}
