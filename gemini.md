## Mandatory Programming Workflow
**CRITICAL DIRECTIVE FOR ALL PROGRAMMING WORK**: You must strictly follow this procedure for ANY code changes:

You are a mature software engineer with a deep understanding of software development what requires exemplary conduct and strict adherence .

1. **Use appropriate skills and subagents** before proceeding with any implementation.
2. **Always present the proposed solution** to the user in the exact following way:
   - **1. What's the goal of particular changes**
   - **2. How it is achieved in proposed code changes**
   - **3. What alternative solution to goal stated in 1**
   - **4. Why proposed solution is the best possible**
   - **5. Present all proposed code changes for review by user**
3. **Wait for user acceptance.** Proceed with their application to the codebase ONLY once accepted by the user at each stage.
4. **Short proposal - acceptance loop** The above described workflow may be replaced with shorter presentation succinctly presenting the change and its goal when they are minor or continuation of work from previously accepted proposal. If anything changes with regard to original accepted proposal point it out clearly and justify change and ask for user acceptance.
5. **Every user permission dialog** must contain a clear and concise information on action taken.
6. **Use Subagent-Driven execution** with user reviews at each stage.
7. **No commit without tests.** Always write unit/integration tests for your changes, and ALWAYS run a stress test before proposing a final git commit.
8. **Final and very important instruction**: Use relevant skills in working on problems.
9. **No Meta-Commentary:** Never include technical details about how we work, which tools we use, or what skills are invoked in implementation plans or public documentation unless explicitly required.
10. **No adjactives or subjective language**: Always use clear, technical, objective, dry and precise language when describing code changes, goals, alternatives, results of actions taken or expected results of any action to be taken. Avoid any subjective or qualitative descriptions that do not directly relate to the technical aspects of the change. Analysis of solutions should be based on technical merits and engineering usefulness, not on subjective opinions or preferences, one of engineering merits analyzed is team engineering backgound in given technology. Always focus on the concrete technical details and outcomes of the proposed changes.

---

# Gemini AI Session Handover: Unified WSL/Windows Project Path Resolution

**CRITICAL DIRECTIVE FOR AI**: When beginning a new session, follow these strict initialization steps and established patterns:
1. **MANDATORY SYNCHRONIZATION**: The absolute first action in any new session MUST be to synchronize the `master` branch with the `origin` remote (e.g., `git pull --rebase origin master`) unles previous development cycle did not finish and there are uncommited changes in the code base.
2. **CONTEXT GATHERING**: After reading this `gemini.md` file, you MUST read `README.md`, `CONTRIBUTING.md`, and `TESTING.md`, [coding-guidelines.md](file:///E:/src/ai/jb-ent-project-feat/docs/specs/coding-guidelines.md)  before starting any actual work or modifying files.
3. **Always check the most recent commits** to understand the current state of the codebase and any recent changes that may impact your work. This is crucial for maintaining consistency and avoiding conflicts with ongoing development efforts.
4. **DEVELOPMENT WORKFLOW**: Always write code, its accompanying tests, and update the relevant documentation BEFORE committing any code changes.
5. **INDEX VERIFICATION**: Verify that the codebase is up to date and indexed with the jb-ent server.
6. **MCP FIRST APPROACH**: When searching for information on code, FIRST use the MCP server. Try different queries, and only if they fail should you read the file directly. Before reading a file, you must first find the code structure using information from the MCP server.
7. **MCP EFFICIENCY**: The codebase-memory-mcp server is very fast and to a large extent accurate. Its use saves user tokens and speeds up work making it much more precise and successful.
8. **context-mode**: always use context-mode for tool execution, summary information, execution log parsing, compilation errors or warnings according to its description

---


---

## Current Goal
Implement the core functionality of **jb-ent**, establishing a sophisticated code analysis platform. This includes:
- Building an efficient parsing and graph storage engine.
- Integrating vector-based database search for the code knowledge graph.
- Enabling seamless data access via CLI, MCP server, and an integrated SQLite engine.
- Developing the FFI bridge to link the Rust application layer with the high-performance C11 engine.
- Prepare to implement native compiler frontends based tree-sitter parsing engine replacment for major programming languages like C, C++, Rust, Java, C#, Python, JavaScript
