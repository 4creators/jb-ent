# Design Specification: jb-ent Rust CLI Implementation (v2.0)

**Date:** 2026-04-17
**Author:** Jacek Błaszczyński
**Assistance:** Gemini CLI
**Copyright:** © 2026 Jacek Błaszczyński
**License:** AGPL-3.0-only

## 1. Goal
Replace the legacy C-based CLI parsing in `src-c/main.c` with a robust, type-safe, and canonical Rust CLI using the `clap` crate (Derive API). This new CLI will act as the single entry point for the `jb-ent` orchestration layer, managing server lifecycle, direct tool execution, configuration, and universal project identity.

## 2. Core Principles
*   **Explicit Actions:** The application will no longer default to starting the MCP server if no arguments are provided. It will strictly print help text and exit.
*   **Canonical Hierarchy:** Grouping related functionality into clear subcommands (`serve`, `call`, `config`, `project`).
*   **Lifecycle Delegation:** Installation and updating commands are removed from the internal CLI, deferring to external scripts or package managers.
*   **JSON-Native Config:** All `config` commands will expect and output JSON to ensure consistency and simplicity across the 3-tier configuration system (System/Global, User, Project).
*   **MCP Security Boundaries:** Security-sensitive actions (like deleting projects or mutating global/user configuration) are strictly CLI-only and will never be exposed via the MCP API.

## 3. Command Line Interface Hierarchy

### 3.1 Top-Level (`jb-ent`)
The root command provides global execution context.
*   `-h`, `--help`: Prints comprehensive help including copyright and AGPL-3.0-only license attribution.
*   `-v`: Prints the short numeric version (e.g., `0.1.0`).
*   `--version`: Prints the long version including the git commit hash (e.g., `0.1.0-a1b2c3d`).
*   `--profile`: Global flag to enable performance profiling.
*   `--verbose`: Global flag to increase logging verbosity (can be stacked).

### 3.2 `serve`
Starts the MCP server instance.
*   `--ui`: Explicitly enable the HTTP UI server for this run.
*   `--no-ui`: Explicitly disable the HTTP UI server.
*   `--port <PORT>`: Override the default (9749) HTTP UI port.

### 3.3 `project` (Formerly `id`, `list_projects`, `delete_project`)
Manages Universal Project Identities and their associated graph databases.
*   *(No subcommand)*: Prints the Universal ID of the current working directory.
*   `list`: Lists all known projects and their registered OS paths from the global registry.
*   `info [PROJECT_ID]`: Shows detailed identity, path mapping, and `.cbm` database location for a given ID (or the current directory if omitted).
*   `delete [PROJECT_ID]`: Removes a project from the registry and deletes its graph database. *(CLI ONLY - Not exposed via MCP).*

### 3.4 `config`
Manages the JSON-based 3-tier settings.
*   `list`: Lists all resolved configuration keys as JSON.
    *   `--level <global|user|project>`: Filter output to a specific tier.
*   `get <KEY>`: Gets the resolved JSON value of a specific key.
*   `set <KEY> <JSON_VALUE>`: Sets a configuration value. *(CLI ONLY)*
    *   `--level <global|user|project>`: Required to specify which tier is being modified.
*   `reset <KEY>`: Removes an override. *(CLI ONLY)*
    *   `--level <global|user|project>`: Required to specify which tier to clear.

### 3.5 `call`
Executes an MCP tool or graph operation. It acts as an intelligent router: if a server is running, it proxies the call; if not, or if the tool is computationally heavy, it executes the logic directly in the current CLI process.

*   `--progress`: Show execution progress on stderr.
*   `--local`: Force local execution, bypassing any running server instance.

**Core Graph Operations:**
*   `index`: Index a repository into the knowledge graph.
*   `status`: Get the indexing status and statistics of a project.
*   `search`: Search the knowledge graph for functions, classes, routes, and variables.
*   `query`: Execute a raw Cypher query against the graph for complex multi-hop patterns.
*   `read`: Read the exact source code for a specific symbol.
*   `schema`: Display the knowledge graph schema.
*   `grep`: Perform a graph-augmented regex/text search across the codebase.
*   `changes`: Detect code changes against the worktree, a specific commit, staged changes, or PRs, and analyze their graph impact.

**Hierarchical Analysis:**
*   `call-graph`: Trace call paths or data flow through the codebase (callers/callees).

**Architecture (`code-arch`):**
*   `analyze`: Get a high-level architectural overview of packages, services, and dependencies.
*   `save`: Create an Architecture Decision Record (ADR) based on graph context.
*   `update`: Update an existing ADR.

**Runtime Tracing (`trace`):**
*   `add`: Ingest runtime execution traces to enrich the knowledge graph with dynamic data.
*   `analyze`: Analyze ingested trace data against the static graph.

## 4. Implementation Strategy (Clap Derive API)
1.  Define the root `Cli` struct deriving `Parser` in `src/cli/mod.rs`.
2.  Define a `Commands` enum deriving `Subcommand` to model `Serve`, `Call`, `Config`, and `Project`.
3.  Implement nested enums for `CallCommands`, `ConfigCommands`, and `ProjectCommands`.
4.  Implement standard matching logic to route to respective (currently empty) handler functions.
5.  Resolve the `mcp-rust-sdk` git dependency issue in `Cargo.toml` so the project compiles.