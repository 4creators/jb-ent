// SPDX-License-Identifier: AGPL-3.0-only
// Copyright © 2026 Jacek Błaszczyński

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "jb-ent",
    version,
    about = "The Graph-Based Knowledge Base for Code, accessible via Vector Embeddings.",
    author = "Jacek Błaszczyński <jacek.blaszczynski@outlook.com>",
    help_template = "\
{before-help}{name} {version}
{author}
Copyright © 2026 Jacek Błaszczyński. Licensed under AGPL-3.0-only.

{about-with-newline}
{usage-heading} {usage}

{all-args}{after-help}"
)]
pub struct Cli {
    /// Enable performance profiling mode
    #[arg(long, global = true)]
    pub profile: bool,

    /// Increase logging verbosity
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug, PartialEq)]
pub enum Commands {
    /// Start the MCP server instance
    Serve {
        /// Enable the HTTP UI server
        #[arg(long, conflicts_with = "no_ui")]
        ui: bool,

        /// Disable the HTTP UI server
        #[arg(long)]
        no_ui: bool,

        /// Set the HTTP UI port
        #[arg(long, default_value_t = 9749)]
        port: u16,
    },
    
    /// Manage Universal Project Identities
    Project {
        #[command(subcommand)]
        command: Option<ProjectCommands>,
    },
    
    /// Manage configuration settings
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
    
    /// Execute an operation locally or via a running server
    Call {
        /// Show execution progress
        #[arg(long)]
        progress: bool,

        /// Force local execution, bypassing server
        #[arg(long)]
        local: bool,

        #[command(subcommand)]
        tool: CallCommands,
    },
}

#[derive(Subcommand, Debug, PartialEq)]
pub enum ProjectCommands {
    /// List all known projects
    List,
    /// Show detailed identity info
    Info {
        project_id: Option<String>,
    },
    /// Remove a project from the registry
    Delete {
        project_id: String,
    },
}

#[derive(Subcommand, Debug, PartialEq)]
pub enum ConfigCommands {
    /// List all configuration keys
    List {
        #[arg(long)]
        level: Option<String>,
    },
    /// Get a specific configuration key
    Get {
        key: String,
    },
    /// Set a configuration value
    Set {
        key: String,
        value: String,
        #[arg(long)]
        level: String,
    },
    /// Remove a configuration override
    Reset {
        key: String,
        #[arg(long)]
        level: String,
    },
}

#[derive(Subcommand, Debug, PartialEq)]
pub enum CallCommands {
    /// Index a repository
    Index,
    /// Get indexing status
    Status,
    /// Search the graph
    Search,
    /// Execute a Cypher query
    Query,
    /// Trace call paths or data flow
    CallGraph,
    /// Read exact source code
    Read,
    /// Display graph schema
    Schema,
    /// Perform a graph-augmented text search
    Grep,
    /// Detect code changes and analyze impact
    Changes,
    /// Architectural analysis
    CodeArch {
        #[command(subcommand)]
        command: CodeArchCommands,
    },
    /// Runtime tracing
    Trace {
        #[command(subcommand)]
        command: TraceCommands,
    },
}

#[derive(Subcommand, Debug, PartialEq)]
pub enum CodeArchCommands {
    /// Get architectural overview
    Analyze,
    /// Create Architecture Decision Record
    Save,
    /// Update existing ADR
    Update,
}

#[derive(Subcommand, Debug, PartialEq)]
pub enum TraceCommands {
    /// Ingest runtime execution traces
    Add,
    /// Analyze ingested trace data
    Analyze,
}
