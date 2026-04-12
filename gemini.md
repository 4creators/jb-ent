# Gemini AI Session Handover: Unified WSL/Windows Project Path Resolution

**CRITICAL DIRECTIVE FOR AI**: When beginning a new session, follow these strict initialization steps and established patterns:
1. **MANDATORY SYNCHRONIZATION**: The absolute first action in any new session MUST be to synchronize the \main\ branch with the \origin\ remote (e.g., \git pull --rebase origin main\).
2. **CONTEXT GATHERING**: After reading this \gemini.md\ file, you MUST read \README.md\ before starting any actual work or modifying files.
3. **DEVELOPMENT WORKFLOW**: Always write code, its accompanying tests, and update the relevant documentation BEFORE committing any code changes.

---

## The Problem: MCP Project Fragmentation
During a previous session on a different project, we discovered a critical "identity crisis" occurring between Windows and WSL2 environments. When the Gemini CLI or an MCP client accesses a repository via a Windows native path (\E:\src\...\) versus a WSL path (\\\wsl.localhost\...\ or \/home/jacek/...\), the \codebase-memory-mcp\ server generates entirely different SQLite database files for the exact same repository.

This leads to:
1. "Project not found or not indexed" errors when querying the graph from a different OS context than the one that indexed it.
2. Massive database bloat as the same repository is re-indexed into multiple fragmented \.db\ files in the \~/.codebase-memory/cache/\ directory.

## Root Cause Analysis
The fragmentation originates in \src/pipeline/fqn.c\ within the \cbm_project_name_from_path(const char *abs_path)\ function.

Currently, the MCP server derives the project's unique identifier strictly by sanitizing the absolute file path (replacing \/\ and \:\ with \-\). 
*   Path A: \E:/src/ai/codebase-mcp\ -> Project ID: \E-src-ai-codebase-mcp\
*   Path B: \//wsl.localhost/ubuntu/home/jacek/src/codebase-mcp\ -> Project ID: \wsl.localhost-ubuntu-home-jacek-src-codebase-mcp\

Because the absolute paths differ across the Windows/WSL boundary, the MCP server fails to recognize them as the same physical project.

## Proposed Implementation Strategy
We need to modify \cbm_project_name_from_path\ (or the upstream pipeline logic) to support a unified, deterministic project identifier that is independent of the host OS mounting path.

**Potential Approaches to Explore:**
1.  **The \.cbm_project_name\ Override:** Modify the C code to look for a hidden \.cbm_project_name\ file in the repository root. If it exists, read the string and use it as the definitive Project ID. If not, fallback to the legacy path-sanitization logic.
2.  **Git Remote Origin Hashing:** Similar to how we fixed the \kconfig-analyzer\ cache, we could spawn a subprocess to read the \git remote get-url origin\ and hash it. (However, this introduces external dependencies to the fast C pipeline).
3.  **Git Config Reading:** Read the \epo_path/.git/config\ file natively in C to extract the remote URL or repository name.

## Next Steps for this Session
1. Evaluate the proposed approaches and implement the most robust, performant C solution in \src/pipeline/fqn.c\ (or relevant architecture).
2. Ensure no memory leaks are introduced (valgrind/asan).
3. Write comprehensive C unit tests in the \	ests/\ directory to prove that WSL, Windows, and Linux paths resolve to the exact same Project ID when the unification logic is triggered.
4. Compile the server via \Makefile.cbm\ and restart the Gemini CLI to apply the fix.
