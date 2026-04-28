# GeminiX

Terminal AI coding agent powered by Google Gemini. Written in Rust.

v0.0.1 — first public release.

Made by [pratikacharya1234](https://github.com/pratikacharya1234).

## What It Does

Runs an agentic loop: you give it a task, it reads files, writes code, runs shell commands, searches the web, and iterates until done. Streaming output in real time with thinking token display (gemini-2.5+).

## Features

### Agentic Loop
- Streaming Gemini API with real-time token display
- Thinking/reasoning token visualization (gemini-2.5+)
- Parallel tool execution via Tokio
- Configurable iteration limits with pause/resume
- Auto-apply mode for non-interactive use
- Single-prompt mode for scripting

### Built-in Tools (16)
read_file, write_file, edit_file (fuzzy matching + occurrence param), append_file, bash (streaming + safety), list_files, search_files (regex), glob, create_directory, delete_file, move_file, copy_file, url_fetch (cached), git_snapshot

### Safety
- 4-level risk classification: Allow / Warn / Confirm / Deny
- Pipe-to-shell detection and blocking
- Per-project safety.toml policy overrides
- All destructive operations require confirmation

### Diff & Undo
- Unified diff preview before file writes
- Per-hunk interactive review (accept/reject/skip per hunk)
- In-memory undo stack with /undo and /undo N

### Context Management
- Token usage display per turn (prompt + output + thinking)
- Context window progress bar with configurable warnings
- Auto-compaction at threshold (summarizes history via Gemini)
- Manual /compact for token savings

### Session Persistence
- Binary save/restore of full conversation history
- /session save, load, list, delete
- Auto-save after each turn

### Cost Tracking
- Per-model pricing (2.5-pro, 2.5-flash, 2.0-flash, 2.0-flash-lite)
- Per-session cost accumulation with USD display
- Daily budget support with warning at 80%

### MCP Support
Full JSON-RPC 2.0 MCP client over stdio (protocol 2025-03-26).
Auto-discovers tools from any MCP-compliant server.
Parallel server startup, 5s write / 60s response timeout safety.

### Native Integrations (33 tools)
| Service | Tools | Auth |
|---------|-------|------|
| GitHub | list_repos, get_repo, create_issue, list_issues, get_issue, comment_issue, close_issue, create_pr, list_prs, get_pr, search_code, list_branches | Personal Access Token |
| Discord | send_message, read_messages, list_channels, list_guilds, create_channel, delete_message, get_channel_info | Bot Token |
| Google Drive | list_files, get_file, download_file, create_folder, upload_file, search_files, delete_file | OAuth2 |
| Gmail | send_email, list_emails, get_email, search_emails, list_labels, mark_read, delete_email | OAuth2 |

### Project Awareness
- .geminix/project.md auto-loaded into system prompt
- /load: load directory tree into context
- /learn: clone public git repo and load for Q&A

### Security
- /security: cargo audit + npm audit + static secret scan + Gemini CVE analysis
- /screenshot: Vision-based bug finding and fixes

### Profiles
Named configuration profiles in ~/.geminix/config.toml with /profile command.

## Slash Commands

/quit (/q, /exit), /clear (/c), /compact, /undo, /undo N, /snapshot, /rollback
/diff, /tokens, /audit, /think, /web, /apply, /load, /learn, /screenshot
/pr, /security, /cd, /model, /models, /save, /session, /cost, /profile
/history, /debug, /help (/h)

## Quick Install

```bash
# Via install script (recommended)
curl -fsSL https://raw.githubusercontent.com/pratikacharya1234/geminix/main/install.sh | bash

# Or build from source
cargo install --git https://github.com/pratikacharya1234/geminix
```

## Prerequisites

- Rust 1.75+ (for source builds)
- Gemini API key (free at https://aistudio.google.com/apikey)
- Optional: gh CLI, cargo-audit (for /security)

## Build from Source

```bash
git clone https://github.com/pratikacharya1234/geminix.git
cd geminix
cargo build --release
```

Binary: `target/release/geminix`

## Usage

```bash
export GEMINI_API_KEY="..."

geminix                                    # interactive session
geminix --prompt "Fix error handling"      # single prompt
geminix --think --grounding                # thinking + web search
geminix --model gemini-2.5-pro             # different model
geminix --screenshot bug.png               # vision analysis
```

## Config

~/.geminix/config.toml:

```toml
api_key = "AIza..."
model = "gemini-2.5-flash"
daily_budget_usd = 5.00

[thinking]
enabled = false
budget = 8000

[integrations.github]
token = "ghp_..."

[integrations.discord]
bot_token = "..."

[integrations.google]
client_id = "..."
client_secret = "..."
refresh_token = "..."
gdrive_enabled = true
gmail_enabled = true

[mcp_servers.postgres]
command = "npx"
args = ["-y", "@modelcontextprotocol/server-postgres"]

[profiles.work]
model = "gemini-2.5-pro"
thinking = true
grounding = true
daily_budget_usd = 10.0
```

Per-project .geminix/safety.toml:

```toml
[permissions]
destructive_commands = "confirm"
network_commands = "warn"
git_destructive = "confirm"
sudo_commands = "deny"
publish_commands = "confirm"

[trusted_commands]
allow = ["cargo check", "cargo build", "cargo test"]

[blocked_commands]
deny = ["rm -rf /"]
```

## Architecture

```
src/
  main.rs           CLI entry point
  agent.rs          Agentic loop, slash commands, streaming
  gemini.rs          Gemini API client (SSE streaming)
  tools.rs           16 built-in tools + dispatch
  safety.rs          4-level risk classifier + policy engine
  diff_view.rs       Unified diff + per-hunk review
  snapshot.rs        In-memory undo stack
  session.rs         Session save/restore persistence
  token_counter.rs   Cost tracking + budget management
  audit.rs           JSON audit log
  config.rs          Config loading, profiles, context windows
  project.rs         Directory loading, git clone
  security.rs        Security sweep (cargo/npm audit, CVE)
  ui.rs              Terminal UI, help, context bar
  mcp.rs             MCP client (JSON-RPC 2.0, stdio)
  integrations/
    mod.rs           Registry + dispatch
    github.rs        12 GitHub API tools
    discord.rs       7 Discord API tools
    google.rs        OAuth2 engine + 7 Drive + 7 Gmail tools
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup, project structure, and guidelines.

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for the full release history.

## License

MIT — see [LICENSE](LICENSE) for details.
