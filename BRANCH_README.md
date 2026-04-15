# Branch: fix-msvc-windows-build

## Branch Objectives & Current State

This branch represents a divergence from `main` to establish a robust, high-performance foundation for `codebase-memory-mcp`.

### Critical Architecture & Development Assumptions

1. **Primary Toolchain:** Windows MSVC via CMake is our primary development and build toolchain.
2. **Portability:** We intend to maintain OS and architecture portability (Linux, macOS, Windows), though currently, our testing focus is heavily on the Windows MSVC environment.
3. **Memory Management Challenges:** We recognize that a major weakness of the current C11 implementation is the immense memory management burden. Parsing and analyzing large codebases require significant amounts of memory, making manual memory tracking, OOM safeguards, and arena/interning optimizations necessary but highly complex.
4. **Feasibility of Rust Port:** We are actively analyzing the feasibility of porting the engine piece by piece from C11 to Rust. Rust's ownership model would eliminate our current memory management issues while maintaining or exceeding C11's performance. The fact that the entire program was previously successfully ported from Go to C11 provides strong encouragement that a gradual port to Rust is highly achievable.