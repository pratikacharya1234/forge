# Changelog

All notable changes to GeminiX are documented in this file.

## [0.0.1] — 2026-04-28

### Initial Release

First public release of GeminiX — a terminal AI coding agent powered by Google Gemini, written in Rust.

**Agentic Loop**
- Streaming Gemini API with real-time token display
- Thinking/reasoning token visualization (gemini-2.5+)
- Parallel tool execution via Tokio
- Configurable iteration limits with pause/resume
- Auto-apply and single-prompt modes

**Built-in Tools (16)**
- read_file, write_file, edit_file (fuzzy matching + occurrence parameter)
- append_file, bash (streaming + safety classification), list_files
- search_files (regex), glob, create_directory, delete_file
- move_file, copy_file, url_fetch (cached), git_snapshot

**Safety System**
- 4-level risk classification: Allow, Warn, Confirm, Deny
- Pipe-to-shell detection and blocking
- Per-project safety.toml with category-level overrides
- Trusted/blocked command lists

**Diff & Undo**
- Unified diff preview before file writes
- Per-hunk interactive review (accept/reject per change)
- In-memory undo stack with /undo and /undo N
- Git snapshot creation and rollback

**Context Management**
- Token usage display per turn (prompt + output + thinking)
- Context window progress bar with configurable warnings
- Auto-compaction at threshold (summarizes history via Gemini)
- Manual /compact command

**Session Persistence**
- Binary save/restore of full conversation history
- /session save, load, list, delete
- Auto-save after each successful turn

**Cost Tracking**
- Per-model pricing with USD display
- Session cost accumulation
- Daily budget support with 80% warning

**MCP Support**
- Full JSON-RPC 2.0 MCP client over stdio
- Protocol 2025-03-26 compliance
- Auto-discovers tools from any MCP-compliant server
- Parallel server startup with timeout safety

**Native Integrations (33 tools)**
- GitHub: 12 tools (issues, PRs, repos, code search, branches)
- Discord: 7 tools (messages, channels, guilds, embeds)
- Google Drive: 7 tools (files, folders, upload, search)
- Gmail: 7 tools (send, list, search, labels, read status)

**Named Profiles**
- Configurable profiles in ~/.geminix/config.toml
- Per-profile model, thinking, grounding, auto_apply, budget
- /profile command for switching

**Security Sweep**
- Cargo audit + npm audit integration
- Static secret scanning
- Gemini-powered CVE analysis

**Additional Commands**
- /learn: Clone and load public git repos for Q&A
- /load: Load directory tree into context
- /screenshot: Vision-based bug analysis
- /pr: Create pull requests
- /models: List available Gemini models
- /debug: Debug information dump
