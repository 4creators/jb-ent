# jb-ent Rust CLI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement a robust, type-safe, and canonical Rust CLI using the `clap` crate (Derive API) as the single entry point for the `jb-ent` application layer.

**Architecture:** The CLI is divided into clear subcommands (`serve`, `project`, `config`, `call`) using `clap`'s Derive API. It acts as a router, either executing locally or passing commands to a running server.

**Tech Stack:** Rust, `clap` (Derive API), `anyhow`.

---

### Task 1: Fix Cargo.toml Dependencies

**Files:**
- Modify: `Cargo.toml`

- [ ] **Step 1: Correct the `mcp-rust-sdk` dependency**

```toml
# In Cargo.toml, replace the missing git revision with a standard dependency or correct path
# Assuming we will use a standard version for now to allow compilation
mcp-rust-sdk = "0.1.0" # or remove temporarily if not used in this immediate CLI scaffold
```

- [ ] **Step 2: Run `cargo check` to verify resolution**

Run: `cargo check`
Expected: Passes (or shows warnings about unused code, but no unresolved dependency errors).

- [ ] **Step 3: Commit**

```bash
git add Cargo.toml
git commit -m "build: fix mcp-rust-sdk dependency resolution"
```

### Task 2: Scaffold the Root CLI and Global Options

**Files:**
- Create/Modify: `src/cli/mod.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Write the basic CLI structure in `src/cli/mod.rs`**

```rust
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

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Start the server instance
    Serve,
}
```

- [ ] **Step 2: Integrate into `src/main.rs` to verify parsing**

```rust
use clap::Parser;
mod cli;

fn main() -> anyhow::Result<()> {
    let cli = cli::Cli::parse();

    if cli.profile {
        println!("Profiling enabled");
    }

    match &cli.command {
        Some(cli::Commands::Serve) => {
            println!("Starting server...");
        }
        None => {
            // Default behavior if no subcommand is provided is to print help
            use clap::CommandFactory;
            let mut cmd = cli::Cli::command();
            cmd.print_help()?;
        }
    }

    Ok(())
}
```

- [ ] **Step 3: Run tests/checks**

Run: `cargo run -- --help`
Expected: Output matches the defined help template with AGPL licensing.

- [ ] **Step 4: Commit**

```bash
git add src/cli/mod.rs src/main.rs
git commit -m "feat(cli): scaffold root clap parser and global options"
```

### Task 3: Implement Subcommands (`serve`, `project`, `config`)

**Files:**
- Modify: `src/cli/mod.rs`

- [ ] **Step 1: Add Server Options**

```rust
// Update the Commands enum
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Start the server instance
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
}

#[derive(Subcommand, Debug)]
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

#[derive(Subcommand, Debug)]
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
```

- [ ] **Step 2: Run `cargo check`**

Run: `cargo check`
Expected: Passes.

- [ ] **Step 3: Commit**

```bash
git add src/cli/mod.rs
git commit -m "feat(cli): add serve, project, and config subcommands"
```

### Task 4: Implement `call` and Nested Tool Commands

**Files:**
- Modify: `src/cli/mod.rs`

- [ ] **Step 1: Add Call Options and Tool Suites**

```rust
// Add to the Commands enum
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
// ... (outside Commands)
#[derive(Subcommand, Debug)]
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

#[derive(Subcommand, Debug)]
pub enum CodeArchCommands {
    /// Get architectural overview
    Analyze,
    /// Create Architecture Decision Record
    Save,
    /// Update existing ADR
    Update,
}

#[derive(Subcommand, Debug)]
pub enum TraceCommands {
    /// Ingest runtime execution traces
    Add,
    /// Analyze ingested trace data
    Analyze,
}
```

- [ ] **Step 2: Run `cargo check`**

Run: `cargo check`
Expected: Passes.

- [ ] **Step 3: Commit**

```bash
git add src/cli/mod.rs
git commit -m "feat(cli): add call subcommand with nested graph tools"
```