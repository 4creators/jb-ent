// SPDX-License-Identifier: AGPL-3.0-only
// Copyright © 2026 Jacek Błaszczyński

use clap::{CommandFactory, Parser};
use jb_ent::cli::{CallCommands, Cli, Commands};

#[test]
fn test_cli_debug_assert() {
    Cli::command().debug_assert();
}

#[test]
fn test_parse_serve_command() {
    let args = vec!["jb-ent", "serve", "--ui", "--port", "8080"];
    let cli = Cli::try_parse_from(args).unwrap();
    assert_eq!(cli.command, Some(Commands::Serve { ui: true, no_ui: false, port: 8080 }));
}

#[test]
fn test_parse_global_flags() {
    let args = vec!["jb-ent", "--profile", "-vv", "serve"];
    let cli = Cli::try_parse_from(args).unwrap();
    assert!(cli.profile);
    assert_eq!(cli.verbose, 2);
    assert!(matches!(cli.command, Some(Commands::Serve { .. })));
}

#[test]
fn test_parse_call_index() {
    let args = vec!["jb-ent", "call", "index"];
    let cli = Cli::try_parse_from(args).unwrap();
    assert_eq!(
        cli.command,
        Some(Commands::Call {
            progress: false,
            local: false,
            tool: CallCommands::Index,
        })
    );
}

#[test]
fn test_parse_call_search_with_progress() {
    let args = vec!["jb-ent", "call", "--progress", "search"];
    let cli = Cli::try_parse_from(args).unwrap();
    assert_eq!(
        cli.command,
        Some(Commands::Call {
            progress: true,
            local: false,
            tool: CallCommands::Search,
        })
    );
}

#[test]
fn test_parse_call_code_arch_analyze() {
    let args = vec!["jb-ent", "call", "--local", "code-arch", "analyze"];
    let cli = Cli::try_parse_from(args).unwrap();
    
    use jb_ent::cli::CodeArchCommands;
    assert_eq!(
        cli.command,
        Some(Commands::Call {
            progress: false,
            local: true,
            tool: CallCommands::CodeArch { command: CodeArchCommands::Analyze },
        })
    );
}
