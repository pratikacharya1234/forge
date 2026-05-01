# Changelog

All notable changes to FORGE are documented in this file.

## [0.0.2] — 2026-04-30

### Added
- **Domain Bootstrap:** Interactive project-type selector (15 domains) with real-time DuckDuckGo web search. Shows latest web findings alongside embedded blueprint. Custom domain input option `[C]` for any domain. Visible spinner and elapsed time during search.
- **15 embedded domain blueprints:** Mobile, Web, AI/ML, Deep Learning, Desktop, Hardware/IoT, Game Dev, DevOps, Data Engineering, Blockchain/Web3, Cybersecurity, CLI/Dev Tools, API/Backend, Scientific/HPC, General. Production-ready knowledge for each domain.
- **NULLVOID Terminal Theme:** Complete visual overhaul — spectral ghost logo with 5-color gradient, protocol header, pipe-separated stats bar, Unicode geometric glyphs (zero emoji). Phantom-hacker aesthetic.
- **Streaming thought rendering:** Nullvoid-styled reasoning blocks with line-by-line amber text streaming and box-drawn borders.
- **Response blocks:** Box-drawn output panels with header/body formatting for all AI responses.
- **Model auto-fallback:** Automatically switches to an alternative model when the current one hits rate limits, auth errors, or server failures. Falls back: same-provider → cross-provider (Gemini free tier preferred).
- **Gemini free tier indicator:** Cost display now shows "FREE (Gemini tier)" when using Gemini models.
- **Free tier guide in README:** Prominent section explaining how FORGE is 100% free with Gemini's 1,500 req/day tier.
- **Demo script:** `demo.sh` — runs a live 2-minute FORGE demo showing code generation and testing.
- **Branch protection:** Main branch protected from force pushes and deletions.

### Changed
- **EMBER voice AI:** Disabled for v0.0.2 — under active development, shipping in v0.0.3. CLI flags `--ember` and `--voice` hidden. Voice modules preserved, ready to re-enable.
- **Auto-detect notifications:** Now use nullvoid styling instead of raw stderr.
- **User echo:** All user input echoed in nullvoid style before processing.
- **All emoji purged:** Replaced with nullvoid Unicode glyphs (◈ ⎔ ⊗ ◉ ⊕ ⊢ ⊞ ⌬).
- Updated comparison table with "Free tier" row.

### Fixed
- **BufWriter flush bug:** FORGE ghost logo now renders correctly — protocol header was stuck in buffer, breaking cursor-overlay alignment.
- **Separator rendering:** Replaced `*` with `·` in protocol header to fix `©` glyph on some terminal fonts.
- **NULLVOID::PROTOCOL visibility:** Header text brightened from MUTED (#3A4060) to TEXT (#8892B0) for readability on dark terminals.

## [0.0.1] — 2026-04-28

### First Public Release

FORGE v0.0.1 — the open-source, multi-model terminal coding agent. 1M token context. Free. Previously developed as GeminiX, rebranded to FORGE to reflect universal multi-model support and independence from any single AI provider.

**Multi-Model Support**
- Gemini 2.5 Pro/Flash/Lite, 2.0 Flash
- Claude 4 Opus/Sonnet, Claude 3.5 Sonnet (Anthropic API)
- GPT-4.1, GPT-4o, o3, o4-mini (OpenAI API)
- Backend abstraction layer with canonical message format
- SSE streaming per provider with tool call round-trips
- Provider auto-detection from model name

**Auto Model Routing**
- `/model auto` analyzes task complexity and picks the best model
- Complex (refactor, architecture, security) → reasoning model
- Simple (find, read, explain) → fast/cheap model
- Everything else → balanced model
- Provider-aware: uses available API keys intelligently

**Explain-Before-Execute**
- `/explain on|off` toggles pre-execution summaries
- Shows planned file changes, shell commands, expected impact
- Asks for confirmation before running tools
- `--explain` CLI flag

**Test-Fix Loop**
- `/test-fix <command> [max-cycles]`
- Runs tests, detects failures, feeds errors to model
- Model fixes code, re-tests, loops until passing
- Handles truncated output for large test suites

**Persistent Memory**
- `/memorize <fact>` saves to `.forge/memory.md`
- `/forget <keyword>` removes matching entries
- `/memory` displays all memorized facts
- Auto-injected into system prompt each turn

**Agentic Loop**
- Streaming Gemini API with real-time token display
- Thinking/reasoning token visualization (Gemini 2.5+)
- Parallel tool execution via Tokio
- Configurable iteration limits with pause/resume
- Auto-apply mode for non-interactive use
- Single-prompt mode for scripting

**Built-in Tools (16)**
`read_file`, `write_file`, `edit_file` (fuzzy matching + occurrence parameter), `append_file`, `bash` (streaming + safety classification), `list_files`, `search_files` (regex), `glob`, `create_directory`, `delete_file`, `move_file`, `copy_file`, `url_fetch` (cached), `git_snapshot`

**Safety System**
- 4-level risk classification: Allow, Warn, Confirm, Deny
- Pipe-to-shell detection and blocking
- Per-project `.forge/safety.toml` with category-level overrides
- Trusted/blocked command lists

**Diff & Undo**
- Unified diff preview before file writes
- Per-hunk interactive review (accept/reject/skip per change)
- In-memory undo stack with `/undo` and `/undo N`
- Git snapshot creation and rollback

**Context Management**
- Token usage display per turn (prompt + output + thinking)
- Context window progress bar with configurable warnings
- Auto-compaction at threshold (summarizes history via model)
- Manual `/compact` command

**Session Persistence**
- Binary save/restore of full conversation history
- `/session save`, `load`, `list`, `delete`
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
- Configurable profiles in `~/.forge/config.toml`
- Per-profile model, thinking, grounding, auto_apply, budget
- `/profile` command for switching

**Security Sweep**
- Cargo audit + npm audit integration
- Static secret scanning (API keys, tokens, passwords)
- Gemini-powered CVE analysis

**Additional Commands**
- `/learn`: Clone and load public git repos for Q&A
- `/load`: Load directory tree into context
- `/screenshot`: Vision-based bug analysis
- `/pr`: Auto-create pull requests
- `/models`: List available Gemini models
- `/debug`: Debug information dump
- `/history`: Show conversation history
- `/cost`: Show session cost breakdown
