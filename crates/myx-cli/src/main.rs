mod cli;
mod commands;
mod exit;
mod export;
mod non_interactive;
mod util;

use clap::Parser;
use serde_json::json;

use crate::cli::{Cli, Commands};
use crate::exit::{fail, CliExit};
use crate::util::chrono_like_timestamp;

fn run(cli: Cli) -> Result<(), CliExit> {
    match cli.command {
        Commands::Init { path, force } => {
            commands::command_init(path, force).map_err(|e| fail(3, e))
        }
        Commands::Add {
            package,
            config,
            non_interactive,
            json,
        } => commands::command_add(&package, config, non_interactive, json),
        Commands::Inspect {
            package,
            config,
            json,
        } => commands::command_inspect(&package, config, json),
        Commands::Build {
            target,
            package,
            config,
            json,
        } => commands::command_build(&target, package, config, json),
    }
}

fn main() {
    let cli = Cli::parse();
    if let Err(err) = run(cli) {
        let message = json!({
            "command": "myx",
            "ok": false,
            "timestamp": chrono_like_timestamp(),
            "error": {
                "code": err.code,
                "message": err.message
            }
        });
        eprintln!(
            "{}",
            serde_json::to_string_pretty(&message).unwrap_or_else(|_| {
                "{\"ok\":false,\"error\":{\"code\":1,\"message\":\"failed to serialize error\"}}"
                    .to_string()
            })
        );
        std::process::exit(err.code);
    }
}
