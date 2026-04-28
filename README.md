# GeminiX

<p align="center">
  <b>The open-source, multi-model terminal coding agent. 1M token context. Free.</b>
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT License"></a>
  <a href="#"><img src="https://img.shields.io/badge/rust-1.75%2B-orange.svg" alt="Rust 1.75+"></a>
  <a href="#"><img src="https://img.shields.io/badge/context-1M%20tokens-green.svg" alt="1M Token Context"></a>
  <a href="#"><img src="https://img.shields.io/badge/models-Gemini%20%7C%20Claude%20%7C%20GPT-purple.svg" alt="Multi-model"></a>
</p>

---

GeminiX is a terminal AI coding agent that reads, writes, and edits code; runs shell commands; searches your codebase; executes tests; and iterates until the job is done. Streaming output, multi-model support, 1 million token context window — all in a single Rust binary.

**Why it exists:** Claude Code costs money. Cursor is tied to VS Code. Most coding agents lock you into one model. GeminiX is free, open-source, multi-model, and works anywhere you have a terminal.

## What Makes It Different

| | GeminiX | Claude Code | Cursor | Copilot |
|---|---|---|---|---|
| **Price** | Free | $20-200/mo | $20/mo | $10/mo |
| **Open source** | ✅ MIT | ❌ | ❌ | ❌ |
| **Multi-model** | Gemini + Claude + GPT | Claude only | Multi-model | GPT only |
| **Max context** | 1M tokens | 200K | ~200K | ~64K |
| **Interface** | Terminal | Terminal | VS Code | VS Code |
| **MCP support** | ✅ | ✅ | ✅ | ❌ |
| **Native integrations** | GitHub, Discord, Gmail, Drive | GitHub | ❌ | GitHub |
| **Privacy** | Your machine, your keys | Their servers | Their servers | Their servers |
| **Binary size** | 11MB | ~100MB+ | N/A | N/A |

## Quick Start

### One-liner install

```bash
curl -fsSL https://raw.githubusercontent.com/pratikacharya1234/geminix/main/install.sh | bash
```

### Manual install

```bash
git clone https://github.com/pratikacharya1234/geminix.git
cd geminix
bash install.sh
```

### Run

```bash
# Get a free API key → https://aistudio.google.com/apikey
export GEMINI_API_KEY="your-key"
geminix
```

Or with other models:

```bash
# Claude (requires Anthropic API key)
geminix --model claude-4-sonnet --anthropic-api-key "sk-ant-..."

# GPT (requires OpenAI API key)
geminix --model gpt-4.1 --openai-api-key "sk-..."
```

## Usage Examples

```bash
# Interactive session
geminix

# Single prompt, exit after
geminix --prompt "add rate limiting to the API endpoints"

# With thinking mode (Gemini 2.5+)
geminix --think --model gemini-2.5-pro

# Google Search grounding
geminix --grounding --prompt "what's the latest tokio API for graceful shutdown"

# Auto-apply changes (skip diff review, CI mode)
geminix --auto-apply --prompt "fix all compiler warnings"

# Screenshot-based bug fixing
geminix --screenshot bug-report.png

# Auto model routing — picks best model per task
geminix --model auto
```

## Command Reference

### Session Controls
| Command | Action |
|---|---|
| `/model <name>` | Switch model (gemini-2.5-pro, claude-4-sonnet, gpt-4.1, etc.) |
| `/model auto` | Auto-select best model per task |
| `/model list` | List all available models |
| `/think [on\|off\|budget]` | Toggle thinking/reasoning mode |
| `/grounding [on\|off]` | Toggle Google Search grounding |
| `/explain [on\|off]` | Show planned actions before executing |
| `/apply [on\|off]` | Toggle auto-apply file changes |
| `/compact` | Summarize and free context |

### Code & Testing
| Command | Action |
|---|---|
| `/test-fix [cmd] [cycles]` | Test → fix → test loop until passing |
| `/diff` | Show unified diff of last change |
| `/undo [N]` | Undo last N file changes |
| `/snapshot` | Capture current state |
| `/rollback` | Restore from snapshot |

### Memory & Context
| Command | Action |
|---|---|
| `/memorize <fact>` | Save fact to persistent memory |
| `/forget <keyword>` | Remove entries from memory |
| `/memory` | View all memorized facts |
| `/load [dir]` | Load directory tree into context |
| `/learn <repo-url>` | Clone and analyze a repo |

### Sessions & History
| Command | Action |
|---|---|
| `/session save <name>` | Save conversation to disk |
| `/session load <name>` | Restore saved session |
| `/session list` | List saved sessions |
| `/tokens` | View token usage |
| `/history [N]` | Show conversation history |
| `/cost` | Show session cost |
| `/profile <name>` | Load named config profile |

### Integrations & Security
| Command | Action |
|---|---|
| `/pr` | Auto-create GitHub PR |
| `/security` | Run full security sweep |
| `/screenshot <path>` | Vision-based code analysis |
| `/audit [N]` | View audit log |
| `/debug` | Toggle debug output |

### Navigation
| Command | Action |
|---|---|
| `/cd <path>` | Change working directory |
| `/save <file>` | Save transcript to file |
| `/quit` or `/exit` | End session |
| `/help` | Show help |

## Safety System

GeminiX classifies every operation:

| Level | Behavior | Examples |
|---|---|---|
| **Allow** | Runs immediately | `cargo check`, `ls`, `grep` |
| **Warn** | Runs with notice | `curl`, `wget`, `npm install` |
| **Confirm** | Asks before running | `rm`, `mv`, `git push` |
| **Deny** | Blocked entirely | `rm -rf /`, `> /dev/sda`, `curl \| bash` |

Per-project overrides in `.geminix/safety.toml`:

```toml
[permissions]
destructive_commands = "confirm"
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
  main.rs           CLI entry point, argument parsing
  agent.rs          Agentic loop, slash commands, streaming (1685 lines)
  backend.rs        Multi-model backend: Gemini, Anthropic, OpenAI (1215 lines)
  gemini.rs         Gemini API types + client
  tools.rs          16 built-in tools + dispatch (887 lines)
  safety.rs         4-level risk classifier + policy engine + per-project config
  diff_view.rs      Unified diff + per-hunk interactive review
  snapshot.rs       In-memory undo/redo stack
  session.rs        Binary session save/restore persistence
  token_counter.rs  Cost tracking + budget management
  audit.rs          JSON audit logging
  config.rs         Config loading, profiles, context windows
  project.rs        Directory loading, git clone for /learn
  security.rs       Security sweep: cargo/npm audit, secret scan, CVE analysis
  ui.rs             Terminal UI, help output, context bar
  mcp.rs            MCP client: JSON-RPC 2.0 over stdio (572 lines)
  models.rs         Model resolution and discovery
  integrations/
    mod.rs          Registry + dispatch
    github.rs       12 GitHub API tools (639 lines)
    discord.rs      7 Discord API tools (409 lines)
    google.rs       OAuth2 engine + 7 Drive + 7 Gmail tools (992 lines)
```

## Features

### Agentic Loop
- Streaming output with real-time token display
- Thinking/reasoning token visualization
- Parallel tool execution via Tokio
- Configurable iteration limits with pause/resume
- Per-hunk diff review (accept/reject/skip individual changes)
- Stuck detection: pauses after 3 consecutive identical errors
- In-memory undo stack

### Multi-Model Support
- Gemini 2.5 Pro/Flash/Lite, 2.0 Flash
- Claude 4 Opus/Sonnet, Claude 3.5 Sonnet
- GPT-4.1, GPT-4o, o3, o4-mini
- Auto-routing: picks best model based on task complexity
- Provider-aware model hints in system prompt
- SSE streaming with proper tool call round-trips per provider

### Built-in Tools (16)
`read_file`, `write_file`, `edit_file` (fuzzy matching + occurrence), `append_file`, `bash` (streaming), `list_files`, `search_files` (regex), `glob`, `create_directory`, `delete_file`, `move_file`, `copy_file`, `url_fetch` (cached), `git_snapshot`

### Native Integrations (33 tools)
- **GitHub:** repos, issues, PRs, code search, branches (12 tools)
- **Discord:** messages, channels, guilds, embeds (7 tools)
- **Google Drive:** files, folders, upload, search (7 tools)
- **Gmail:** send, list, search, labels (7 tools)

### MCP Support
Full JSON-RPC 2.0 MCP client (protocol 2025-03-26) over stdio. Auto-discovers tools from any MCP-compliant server. Parallel startup with timeout safety.

### Context & Memory
- 1M token context window (Gemini 2.5 models)
- Auto-compaction at configurable threshold
- Persistent memory via `/memorize` and `/forget`
- Project instructions via `.geminix/project.md`

### Testing & Quality
- Test-fix loop: run tests → detect failures → fix → repeat
- Explain-before-execute: preview planned actions
- Auto-apply mode for CI/CD pipelines
- Context window progress bar with warnings

### Security
- 4-level risk classification
- Per-project safety policy overrides
- Pipe-to-shell detection
- Security sweep: cargo audit + npm audit + secret scan + CVE analysis
- All API keys from env vars, never hardcoded

### Profiles & Cost
- Named configuration profiles
- Per-model cost tracking with USD display
- Daily budget support with warnings
- `/cost` command for session stats

## Configuration

`~/.geminix/config.toml`:

```toml
api_key = "AIza..."
model = "gemini-2.5-flash"
daily_budget_usd = 5.00

[thinking]
enabled = false
budget = 8000

# Multi-model keys
anthropic_api_key = "sk-ant-..."
openai_api_key = "sk-..."

[integrations.github]
token = "ghp_..."

[integrations.discord]
bot_token = "..."

[integrations.google]
client_id = "..."
client_secret = "..."
refresh_token = "..."

[mcp_servers.postgres]
command = "npx"
args = ["-y", "@modelcontextprotocol/server-postgres"]

[profiles.work]
model = "gemini-2.5-pro"
thinking = true
grounding = true
daily_budget_usd = 10.0
```

## Prerequisites

- Rust 1.75+ (for source builds)
- A Gemini, Anthropic, or OpenAI API key
- Optional: `gh` CLI, `cargo-audit`, `npm` for security sweeps

## Build

```bash
git clone https://github.com/pratikacharya1234/geminix.git
cd geminix
cargo build --release
# Binary: target/release/geminix (11MB)
```

Or use the installer:

```bash
bash install.sh

# Options:
bash install.sh --dir /usr/local/bin    # custom install path
bash install.sh --version 1.0.0         # specific version
GEMINIX_FROM_SOURCE=1 bash install.sh   # force source build
```

Prebuilt binaries available on [GitHub Releases](https://github.com/pratikacharya1234/geminix/releases).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup, project structure, and guidelines for adding tools, integrations, and slash commands.

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for the full release history.

## License

MIT — see [LICENSE](LICENSE) for details.

---

<p align="center">
  <b>Built with Rust. Powered by Gemini. Free forever.</b>
</p>
