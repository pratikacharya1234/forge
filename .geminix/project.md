# GeminiX — Self-Project

## Overview
GeminiX is a terminal-based AI coding agent powered by Google Gemini, written in Rust.

## Architecture
- Main entry: `src/main.rs` — CLI argument parsing with clap
- Core loop: `src/agent.rs` — streaming agentic loop, slash commands, history management
- Gemini API: `src/gemini.rs` — HTTP client, SSE streaming, request/response types
- Tools: `src/tools.rs` — all tool implementations + function declarations
- Safety: `src/safety.rs` — risk classification, safety.toml loading
- Diff: `src/diff_view.rs` — unified diff display with accept/reject
- Snapshot: `src/snapshot.rs` — in-memory file snapshot stack for undo
- Audit: `src/audit.rs` — JSON audit log
- Config: `src/config.rs` — Config struct, file loading, context window sizes
- Project: `src/project.rs` — directory loading, git clone+load
- Security: `src/security.rs` — cargo/npm audit, static secret scan, Gemini CVE analysis
- UI: `src/ui.rs` — colored terminal output, banner, help, context bar

## Key Conventions
- All tool functions return `ToolResult { output, is_error, was_streamed }`
- New tools: add to `execute_tool()`, `get_tool_declarations()`, and the agent dispatch
- Safety: 4 levels — Allow/Warn/Confirm/Deny
- Config: loaded from `~/.geminix/config.toml` with CLI overrides
- Per-project: `.geminix/project.md`, `.geminix/safety.toml`

## Dependencies
- tokio (async runtime, full features)
- reqwest (HTTP client with JSON + stream)
- serde/serde_json (serialization)
- chrono (timestamps)
- rustyline (REPL input)
- clap (CLI args)
- colored (terminal colors)
- similar (diff engine via TextDiff)
- walkdir (directory walking)
- regex (pattern matching)
- base64 (image encoding)
- toml (config parsing)
