// SPDX-License-Identifier: AGPL-3.0-only
// Copyright © 2026 Jacek Błaszczyński

use clap::{CommandFactory, Parser};
use jb_ent::cli;

fn main() -> anyhow::Result<()> {
    let cli_args = cli::Cli::parse();

    if cli_args.profile {
        println!("Profiling enabled");
    }

    match &cli_args.command {
        Some(cli::Commands::Serve { ui, no_ui, port }) => {
            println!("Starting server (ui: {}, no_ui: {}, port: {})...", ui, no_ui, port);
        }
        Some(cli::Commands::Project { command }) => {
            println!("Project command: {:?}", command);
        }
        Some(cli::Commands::Config { command }) => {
            println!("Config command: {:?}", command);
        }
        Some(cli::Commands::Call { progress, local, tool }) => {
            println!("Call command: tool={:?}, progress={}, local={}", tool, progress, local);
        }
        None => {
            // Default behavior: print help
            let mut cmd = cli::Cli::command();
            cmd.print_help()?;
        }
    }

    Ok(())
}