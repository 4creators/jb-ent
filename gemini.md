## Mandatory Programming Workflow
**CRITICAL DIRECTIVE FOR ALL PROGRAMMING WORK**: You must strictly follow this procedure for ANY code changes:

1. **Use appropriate skills and subagents** before proceeding with any implementation.
2. **Always present the proposed solution** to the user in the exact following way:
   - **1. What's the goal of particular changes**
   - **2. How it is achieved in proposed code changes**
   - **3. What alternative solution to goal stated in 1**
   - **4. Why proposed solution is the best possible**
   - **5. Present all proposed code changes for review by user**
3. **Wait for user acceptance.** Proceed with their application to the codebase ONLY once accepted by the user at each stage.
4. **Use Subagent-Driven execution** with user reviews at each stage.
5. **No commit without tests.** Always write unit/integration tests for your changes, and ALWAYS run a stress test before proposing a final git commit.
6. **Final and very important instruction**: Use relevant skills in working on problems.
7. **No Meta-Commentary:** Never include technical details about how we work, which tools we use, or what skills are invoked in implementation plans or public documentation.

---

# Gemini AI Session Handover: Unified WSL/Windows Project Path Resolution

**CRITICAL DIRECTIVE FOR AI**: When beginning a new session, follow these strict initialization steps and established patterns:
1. **MANDATORY SYNCHRONIZATION**: The absolute first action in any new session MUST be to synchronize the `master` branch with the `origin` remote (e.g., `git pull --rebase origin master`).
2. **CONTEXT GATHERING**: After reading this `gemini.md` file, you MUST read `README.md` before starting any actual work or modifying files.
3. **DEVELOPMENT WORKFLOW**: Always write code, its accompanying tests, and update the relevant documentation BEFORE committing any code changes.
4. **INDEX VERIFICATION**: Verify that the codebase is up to date and indexed with the jb-ent server.
5. **MCP FIRST APPROACH**: When searching for information on code, FIRST use the MCP server. Try different queries, and only if they fail should you read the file directly. Before reading a file, you must first find the code structure using information from the MCP server.
6. **MCP EFFICIENCY**: The jb-ent server is very fast and to a large extent accurate. Its use saves user tokens and speeds up work making it much more precise and successful.

---

## The Problem: MCP Project Fragmentation
During the initial development phase, we discovered a critical "identity crisis" between Windows and WSL2 environments. When the Gemini CLI or an MCP client accesses a repository via a Windows native path versus a WSL path, the server generates entirely different SQLite database files for the exact same repository.

## Resolution Strategy: jb-ent Unified Identity
The **jb-ent** project addresses this by implementing a deterministic project identifier independent of the host OS mounting path.

**Implementation Details:**
1. **Primary ID:** The first git commit hash (`git rev-list --max-parents=0 HEAD`).
2. **Fallback:** A persistent UUID stored in the project's configuration directory.
3. **Rust Orchestration:** The `jb-ent` Rust-led architecture handles cross-platform path normalization and identity resolution before passing requests to the legacy C11 engine.

## Current Project Structure
- `src/`: Rust orchestration layer (MCP server, CLI, Configuration, Identity).
- `src-c/`: Consolidated legacy C11 engine (structural analysis, graph storage).
- `docs/`: Design specifications and migration history.

---

## Current Goal
Implement the core functionality of **jb-ent**, establishing a sophisticated code analysis platform. This includes:
- Building an efficient parsing and graph storage engine.
- Integrating vector-based database search for the code knowledge graph.
- Enabling seamless data access via CLI, MCP server, and an integrated SQLite engine.
- Developing the FFI bridge to link the Rust orchestration layer with the high-performance C11 engine.
