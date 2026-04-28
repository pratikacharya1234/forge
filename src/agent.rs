use std::io::Write as IoWrite;

use anyhow::Result;
use colored::Colorize;
use rustyline::DefaultEditor;

use std::sync::Arc;

use crate::backend::{self, BackendClient, Provider};
use crate::config::{self, Config};
use crate::gemini::*;
use crate::integrations::IntegrationRegistry;
use crate::mcp::McpRegistry;
use crate::models;
use crate::orchestrator;
use crate::token_counter::CostTracker;
use crate::tools::{self, ToolContext};
use crate::{audit, project, security, session, snapshot, ui};

// ── System prompt ──────────────────────────────────────────────────────────────

const SYSTEM_PROMPT_BASE: &str = r#"You are GeminiX — a powerful AI software engineering agent operating in a terminal environment with full file system access, shell execution, web search, and a comprehensive tool suite. You are fast, thorough, and produce production-ready work. You are built on Rust, powered by multiple AI models, and designed to outclass every other coding agent.

## Task Classification

Before acting, classify the user's request:

### Type A: Code Change (fix, refactor, implement, add feature)
- Plan the change, then execute
- Read only the files you need to change
- Prefer edit_file over write_file for existing files
- Run cargo check / build / tests after every file modification
- Be surgical. Change only what's required. Preserve existing style.

### Type B: Analysis & Research (compare, evaluate, benchmark, audit, document)
- Read BROADLY first. Read README, CHANGELOG, all relevant source files, config files.
- Use url_fetch and google_search to gather external data, competitor info, documentation.
- Do NOT be minimal. Be thorough. Read everything relevant before writing a single word.
- When creating reports: include hard data, numbers, comparisons, specific names — not generic categories.
- Never write a comparison document without actually checking what competitors offer.

### Type C: Discovery & Exploration (find, search, list, show me, where is)
- Use glob, search_files, grep to locate files and patterns
- Present results clearly with file paths and line numbers
- Summarize findings concisely

## Operating Principles

### 1. Think, Then Act
Before any multi-step action, state your plan: what you'll do, why, in what order. The user should understand your strategy before tools fire.

### 2. Verify Every Code Change
After ANY file modification, run the build. After logic changes, run tests. A change that compiles is a signal. A change that passes tests is evidence. Neither is proof.

### 3. Read Before You Write
For analysis/research tasks: read every relevant file in the project before creating output. Never produce a comparison, report, or evaluation without actually examining the source material. Surface-level knowledge produces surface-level work.

### 4. Error Recovery
Tool errors are YOUR problem. When a tool fails: read_file to check current state, identify the root cause, fix it, retry. If the same error persists twice, change your approach and tell the user why. Never give up on a recoverable error.

### 5. Context Window Awareness
You have a 1M token context window on Gemini 2.5 models. Use it. Read files fully unless they exceed 500+ lines. For large projects, read architecture docs and key files, then drill into specifics. When approaching 70% capacity: summarize, combine searches, compact.

## Tool Usage

### File Operations
- Read multiple independent files in parallel
- Use edit_file for existing files, write_file for new files or complete rewrites
- edit_file supports fuzzy matching — use exact strings when possible, occurrence parameter for duplicates

### Shell Commands
- cd does NOT persist between calls. Use absolute paths or chain: cd /path && command
- Package managers: timeout=300. Test suites: timeout=600.
- Chain commands with &&. Capture stderr with 2>&1.

### Search & Discovery
- glob for file patterns. search_files for content (regex). list_files for directory trees.
- Combine: glob to find files, then read the relevant ones in parallel.

### Web & External Data
- url_fetch for documentation, API references, package info
- google_search (when /web is enabled) for current docs, CVEs, competitor analysis, benchmarks
- For comparison/research tasks: web search is MANDATORY. You cannot evaluate competitors from memory.

## GeminiX Capabilities

You are running on GeminiX — the open-source, multi-model terminal coding agent. Key capabilities:
- Multi-model: Gemini, Claude, GPT. User can switch with /model.
- 1M token context on Gemini 2.5 models — the largest of any coding agent.
- Test-fix loop: /test-fix runs tests, detects failures, fixes code, repeats until pass.
- Explain-before-execute: /explain shows planned actions before running.
- Persistent memory: /memorize saves facts and preferences across sessions.
- 16 built-in tools + 33 integration tools (GitHub, Discord, Gmail, Drive).
- 4-level safety system with per-project policy overrides.
- MCP support for external tool servers.

## Code Quality Standards

After every code change, verify:
1. Does it compile? Run the build.
2. Are imports correct and minimal?
3. No debug prints, console.logs, TODO markers, or placeholder code.
4. Error cases handled. Null/None values guarded. Edge conditions covered.
5. Does it break existing tests?
6. Consistent with the project's conventions and existing style.
7. Would a senior engineer approve this?

### Anti-Patterns
- Copying entire files for small changes → use edit_file
- Adding dependencies for simple problems → solve with existing code
- Rewriting working code → fix only what's broken
- Hardcoding credentials, tokens, or secrets → use env vars or config
- Creating files without checking if they already exist

## Language Conventions

### Rust: cargo check after every change. cargo test after logic changes. Respect Cargo.toml versions. Never unwrap in production code — use proper error handling.

### JavaScript/TypeScript: Check package.json for scripts and deps. Run npm test. Respect ESLint/Prettier. Match existing module system.

### Python: Activate venv before pip. Run pytest/unittest. Match existing type hints and docstrings.

### Go: go build then go test. Use gofmt. Check go.mod.

### General: Read the project's README, CI config, and tests first. Project conventions override generic advice. When in a new codebase, spend time understanding before changing.

## Project Context
- .geminix/project.md is authoritative for this project
- .gitignore patterns inform search scope
- The working directory is shown below — all relative paths are relative to cwd
- Memory is loaded from .geminix/memory.md — follow memorized preferences

{model_hint}
{project_context}
{memory_context}
Working directory: {cwd}
"#;

fn model_hint(config: &Config) -> String {
    let model = &config.model;
    let provider = backend::detect_provider(model);

    let cap = match provider {
        Provider::Gemini => {
            if model.contains("2.5-pro") || model.contains("pro") {
                "You have deep reasoning. Use it for complex architecture, cross-file analysis, and security audits. Think through edge cases. Thoroughness over speed."
            } else if model.contains("2.5-flash-lite") {
                "You are on a lightweight model. Be concise. Prefer single-file changes. Use tools efficiently."
            } else if model.contains("2.5-flash") || model.contains("2.5") {
                "You are fast and accurate. For simple tasks, act immediately. For complex tasks, plan quickly then execute."
            } else {
                ""
            }
        }
        Provider::Anthropic => {
            if model.contains("opus") {
                "You have deep reasoning via Claude. Use extended thinking for complex multi-file tasks and architecture decisions."
            } else {
                "You are running on Claude. Strong reasoning. Use extended thinking for complex problems."
            }
        }
        Provider::OpenAI => {
            if model.contains("o3") || model.contains("o4") {
                "You have advanced reasoning via OpenAI. Excellent at complex multi-step tasks and code generation."
            } else {
                "You are running on GPT. Strong code generation and reasoning."
            }
        }
    };

    if cap.is_empty() {
        String::new()
    } else {
        format!("Model capability: {model} — {cap}")
    }
}

fn load_memory_context() -> String {
    // Load .geminix/memory.md if it exists
    let path = std::path::Path::new(".geminix/memory.md");
    if let Ok(content) = std::fs::read_to_string(path) {
        let trimmed = content.trim();
        if !trimmed.is_empty() {
            return format!("\n## Persistent Memory\n\nThe following facts, preferences, and conventions have been memorized. Follow them.\n\n{}\n", trimmed);
        }
    }
    String::new()
}

fn load_project_context() -> String {
    // Look for .geminix/project.md in current dir
    let path = std::path::Path::new(".geminix/project.md");
    if let Ok(content) = std::fs::read_to_string(path) {
        let trimmed = content.trim();
        if !trimmed.is_empty() {
            return format!("\n## Project Instructions\n\n{}\n", trimmed);
        }
    }
    String::new()
}

fn system_prompt(config: &Config) -> String {
    let cwd = std::env::current_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| ".".to_string());

    // Escape curly braces that could break format/replace
    let cwd_safe = cwd.replace('{', "｛").replace('}', "｝");

    let grounding_line = if config.grounding {
        "- google_search — built-in Google Search grounding\n"
    } else {
        ""
    };

    SYSTEM_PROMPT_BASE
        .replace("{model_hint}", &model_hint(config))
        .replace("{project_context}", &load_project_context())
        .replace("{memory_context}", &load_memory_context())
        .replace("{cwd}", &cwd_safe)
        + grounding_line
}

// ── Tool list / config ─────────────────────────────────────────────────────────

fn build_tools(grounding: bool, mcp: Option<&McpRegistry>, integrations: Option<&IntegrationRegistry>) -> Vec<serde_json::Value> {
    let mut tools = vec![serde_json::json!({
        "functionDeclarations": tools::get_tool_declarations()
    })];
    if grounding {
        tools.push(serde_json::json!({ "googleSearch": {} }));
    }
    // Extend with integration tools if available
    if let Some(ref ireg) = integrations {
        let idecls = ireg.function_declarations();
        if !idecls.is_empty() {
            if let Some(first) = tools.first_mut() {
                if let Some(obj) = first.as_object_mut() {
                    if let Some(arr) = obj.get_mut("functionDeclarations") {
                        if let Some(decls) = arr.as_array_mut() {
                            for decl in idecls {
                                decls.push(serde_json::json!(decl));
                            }
                        }
                    }
                }
            }
        }
    }
    // Extend with MCP tools if available
    if let Some(ref mcp) = mcp {
        let mcp_decls = mcp.function_declarations();
        if !mcp_decls.is_empty() {
            if let Some(first) = tools.first_mut() {
                if let Some(obj) = first.as_object_mut() {
                    if let Some(arr) = obj.get_mut("functionDeclarations") {
                        if let Some(decls) = arr.as_array_mut() {
                            for decl in mcp_decls {
                                decls.push(serde_json::json!(decl));
                            }
                        }
                    }
                }
            }
        }
    }
    tools
}

fn build_tool_config() -> ToolConfig {
    ToolConfig {
        function_calling_config: FunctionCallingConfig { mode: "AUTO".to_string() },
    }
}

fn build_generation_config(thinking: bool, thinking_budget: i32) -> GenerationConfig {
    GenerationConfig {
        temperature:       Some(0.7),
        max_output_tokens: Some(8192),
        thinking_config: if thinking {
            Some(ThinkingConfig { thinking_budget, include_thoughts: true })
        } else {
            None
        },
    }
}

// ── Public entry points ────────────────────────────────────────────────────────

pub async fn run_once(config: &Config, prompt: &str, screenshot: Option<&str>) -> Result<()> {
    let client = BackendClient::new(config)?;

    // Initialize MCP servers and integrations
    let mcp = Arc::new(McpRegistry::startup(&config.mcp_servers).await);
    let integrations = Arc::new(IntegrationRegistry::from_config(&config.integrations));
    if mcp.server_count() > 0 { mcp.print_status(); }
    if integrations.service_count() > 0 { integrations.print_status(); }

    let mut cost_tracker = CostTracker::new(&config.model, config.daily_budget_usd);
    let mut parts = vec![Part::text(prompt)];
    if let Some(path) = screenshot {
        match encode_image(path) {
            Ok((mime, data)) => parts.push(Part::image(mime, data)),
            Err(e) => eprintln!("  ~ Could not load screenshot: {}", e),
        }
    }

    let mut history = vec![Content { role: "user".to_string(), parts }];
    agentic_loop(&client, &mut history, config, false, Some(mcp), Some(integrations), &mut cost_tracker).await.map(|_| ())
}

pub async fn run_interactive(config: &Config) -> Result<()> {
    ui::print_banner(config.grounding, config.thinking, config.auto_apply);

    // Initialize MCP servers and integrations
    let mcp = Arc::new(McpRegistry::startup(&config.mcp_servers).await);
    let integrations = Arc::new(IntegrationRegistry::from_config(&config.integrations));
    if mcp.server_count() > 0 { mcp.print_status(); println!(); }
    if integrations.service_count() > 0 { integrations.print_status(); println!(); }

    let mut cost_tracker = CostTracker::new(&config.model, config.daily_budget_usd);

    let mut history:         Vec<Content> = Vec::new();
    let mut current_model    = config.model.clone();
    let mut grounding        = config.grounding;
    let mut thinking         = config.thinking;
    let mut thinking_budget  = config.thinking_budget;
    let mut auto_apply       = config.auto_apply;
    let mut explain_exec     = config.explain_before_execute;
    let mut debug            = false;
    let mut session_tokens   = 0u32;

    // Announce project.md if found
    if std::path::Path::new(".geminix/project.md").exists() {
        println!(
            "  {} Loaded project instructions from {}",
            "[OK]".green(),
            ".geminix/project.md".cyan()
        );
        println!();
    }

    let history_path = dirs::home_dir()
        .map(|h| h.join(".geminix-history"))
        .unwrap_or_else(|| std::path::PathBuf::from(".geminix-history"));

    let mut rl = DefaultEditor::new()?;
    let _ = rl.load_history(&history_path);

    loop {
        let prompt_str = ui::user_prompt_str();

        let line = match rl.readline(&prompt_str) {
            Ok(l) => l,
            Err(rustyline::error::ReadlineError::Interrupted) => {
                println!();
                println!("{}", "(Ctrl-C — type /quit to exit)".dimmed());
                continue;
            }
            Err(rustyline::error::ReadlineError::Eof) => { println!(); break; }
            Err(e) => { ui::print_error(&e.to_string()); break; }
        };

        let line = line.trim().to_string();
        if line.is_empty() { continue; }
        let _ = rl.add_history_entry(&line);

        // ── Slash commands ─────────────────────────────────────────────────────
        if line.starts_with('/') {
            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            match parts[0] {

                "/quit" | "/exit" | "/q" => {
                    println!("{}", "Goodbye!".bright_blue());
                    break;
                }

                "/clear" | "/c" => {
                    history.clear();
                    session_tokens = 0;
                    println!("{}", "Conversation cleared.".dimmed());
                }

                "/help" | "/h" => ui::print_help(),

                "/history" => {
                    let n: usize = parts.get(1)
                        .and_then(|s| s.trim().parse().ok())
                        .unwrap_or(5);
                    let start = history.len().saturating_sub(n * 2);
                    for msg in &history[start..] {
                        let label = if msg.role == "user" { "You".green() } else { "GeminiX".blue() };
                        for part in &msg.parts {
                            if let Part::Text { text, .. } = part {
                                let preview: String = text.chars().take(200).collect();
                                println!("{}: {}{}", label, preview, if text.len() > 200 { "…" } else { "" });
                            }
                        }
                    }
                }

                "/model" => {
                    match parts.get(1).map(|s| s.trim()) {
                        Some("list") | Some("ls") => {
                            println!();
                            println!("  {}:", "Gemini".cyan());
                            let gemini_models = [
                                ("gemini-2.5-pro",        "deep reasoning, 1M context, thinking"),
                                ("gemini-2.5-flash",      "fastest recommended, thinking"),
                                ("gemini-2.5-flash-lite",  "cheapest 2.5, $0.10/M input"),
                                ("gemini-2.0-flash",      "previous gen, no thinking"),
                                ("gemini-2.0-flash-lite", "lightest, lowest cost"),
                            ];
                            for (m, d) in gemini_models {
                                let marker = if m == current_model { "->".green() } else { " ".normal() };
                                println!("  {} {:28} {}", marker, m.cyan(), d.dimmed());
                            }
                            println!();
                            println!("  {}:", "Claude".cyan());
                            let claude = ["claude-4-opus", "claude-4-sonnet", "claude-3.5-sonnet"];
                            for m in claude {
                                let marker = if m == current_model { "->".green() } else { " ".normal() };
                                println!("  {} {}", marker, m.cyan());
                            }
                            println!();
                            println!("  {}:", "OpenAI".cyan());
                            let openai = ["gpt-4.1", "gpt-4o", "o3", "o4-mini"];
                            for m in openai {
                                let marker = if m == current_model { "->".green() } else { " ".normal() };
                                println!("  {} {}", marker, m.cyan());
                            }
                            println!();
                            println!("  {} /model auto — auto-select best model for each task", "Tip:".dimmed());
                        }
                        Some("info") => {
                            let provider = backend::detect_provider(&current_model);
                            let prov_name = match provider {
                                Provider::Gemini => "Gemini",
                                Provider::Anthropic => "Anthropic",
                                Provider::OpenAI => "OpenAI",
                            };
                            println!("{} {}", "Current model:".dimmed(), current_model.cyan());
                            println!("{} {}", "Provider:".dimmed(), prov_name.dimmed());
                            println!("{} {}", "Context window:".dimmed(),
                                format!("{}M tokens", config::context_window(&current_model) / 1_000_000).dimmed());
                        }
                        Some("auto") => {
                            println!("{} Auto-routing enabled. The agent will select the best model for each task.", "[AUTO]".cyan());
                            println!("  {} complex tasks (refactor, architecture, security) → reasoning model", "▸".dimmed());
                            println!("  {} normal tasks (fix, add, implement) → balanced model", "▸".dimmed());
                            println!("  {} simple tasks (find, read, explain) → fast/cheap model", "▸".dimmed());
                            current_model = "auto".to_string();
                        }
                        Some(model) if !model.is_empty() => {
                            let new_provider = backend::detect_provider(model);
                            let current_provider = backend::detect_provider(&current_model);
                            if new_provider != current_provider {
                                let key_needed = match new_provider {
                                    Provider::Anthropic => config.anthropic_api_key.is_none(),
                                    Provider::OpenAI => config.openai_api_key.is_none(),
                                    _ => false,
                                };
                                if key_needed {
                                    println!("{} API key required for {}. Set via --anthropic-api-key / --openai-api-key or config.toml.", "!".yellow(), model);
                                } else {
                                    current_model = model.to_string();
                                    println!("{} {} (provider: {})", "Model:".dimmed(), current_model.cyan(),
                                        match new_provider { Provider::Gemini => "Gemini", Provider::Anthropic => "Claude", Provider::OpenAI => "OpenAI" }.dimmed());
                                }
                            } else {
                                current_model = model.to_string();
                                println!("{} {}", "Model:".dimmed(), current_model.cyan());
                            }
                        }
                        _ => {
                            println!("{} {}", "Model:".dimmed(), current_model.cyan());
                            println!("{}", "Usage: /model <name> | auto | list | info".dimmed());
                        }
                    }
                }

                "/models" => {
                    println!("{} Fetching available Gemini models...", "--".dimmed());
                    match models::fetch_available_models(&config.api_key).await {
                        Ok(all) => {
                            let coding = models::filter_coding_models(&all);
                            if coding.is_empty() {
                                println!("  No Gemini models found. Check your API key.");
                            } else {
                                println!("  Available models (auto-detected):");
                                for m in coding {
                                    let marker = if m.name.contains(&current_model) || current_model.contains(&m.name) {
                                        "->".green()
                                    } else {
                                        "  ".normal()
                                    };
                                    let display = m.display_name.as_deref().unwrap_or(&m.name);
                                    let tokens = m.input_token_limit
                                        .map(|t| format!("{}K tokens", t / 1000))
                                        .unwrap_or_default();
                                    println!(
                                        "  {} {:<45} {}",
                                        marker,
                                        display.cyan(),
                                        tokens.dimmed()
                                    );
                                }
                            }
                        }
                        Err(e) => println!("  Failed to fetch models: {}", e),
                    }
                }

                "/cost" => {
                    let status = cost_tracker.format_status();
                    println!("  {} {}", "$".dimmed(), status);
                    if let Some(warning) = cost_tracker.budget_warning() {
                        println!("  {} {}", "!".yellow(), warning.yellow());
                    }
                }

                "/profile" => {
                    match parts.get(1).map(|s| s.trim()) {
                        Some(name) if !name.is_empty() => {
                            // Apply profile settings
                            if let Some(profile) = config::load_profile(name) {
                                if let Some(ref m) = profile.model { current_model = m.clone(); }
                                if let Some(g) = profile.grounding { grounding = g; }
                                if let Some(t) = profile.thinking { thinking = t; }
                                if let Some(tb) = profile.thinking_budget { thinking_budget = tb; }
                                if let Some(aa) = profile.auto_apply { auto_apply = aa; }
                                if let Some(b) = profile.daily_budget_usd {
                                    cost_tracker = CostTracker::new(&current_model, Some(b));
                                }
                                println!("  Profile '{}' applied.", name.cyan());
                            } else {
                                println!("  Profile '{}' not found in config.", name.red());
                            }
                        }
                        _ => println!("Usage: /profile <name>  (configured in ~/.geminix/config.toml [profiles] section)"),
                    }
                }

                "/web" => {
                    grounding = !grounding;
                    println!("{}", if grounding {
                        "Google Search grounding ENABLED.".green().to_string()
                    } else {
                        "Google Search grounding DISABLED.".yellow().to_string()
                    });
                }

                "/think" => {
                    match parts.get(1).map(|s| s.trim()) {
                        None | Some("on") => {
                            thinking = true;
                            println!("{} ThinkMode ON — budget: {} tokens", "[THINK]".yellow(), thinking_budget);
                        }
                        Some("off") => {
                            thinking = false;
                            println!("{}", "ThinkMode OFF.".dimmed());
                        }
                        Some(arg) if arg.starts_with("budget=") => {
                            if let Ok(n) = arg[7..].parse::<i32>() {
                                thinking_budget = n.clamp(0, 24576);
                                thinking = true;
                                println!("{} ThinkMode ON — budget: {} tokens", "[THINK]".yellow(), thinking_budget);
                            } else {
                                println!("Usage: /think budget=8000");
                            }
                        }
                        _ => println!("Usage: /think  /think off  /think budget=8000"),
                    }
                }

                "/apply" => {
                    match parts.get(1).map(|s| s.trim()) {
                        Some("on")  => { auto_apply = true;  println!("{}", "Auto-apply ON — diffs accepted without prompt.".yellow()); }
                        Some("off") => { auto_apply = false; println!("{}", "Auto-apply OFF — diff preview enabled.".green()); }
                        _           => {
                            auto_apply = !auto_apply;
                            println!("{}", if auto_apply {
                                "Auto-apply ON.".yellow().to_string()
                            } else {
                                "Auto-apply OFF — diff preview enabled.".green().to_string()
                            });
                        }
                    }
                }

                "/debug" => {
                    debug = !debug;
                    println!("{}", if debug { "Debug ON.".yellow().to_string() } else { "Debug OFF.".dimmed().to_string() });
                }

                "/memorize" => {
                    let fact = parts.get(1).map(|s| s.trim()).unwrap_or("");
                    if fact.is_empty() {
                        println!("{}", "Usage: /memorize <fact> — save a fact or preference to persistent memory".dimmed());
                    } else {
                        let now = chrono::Local::now();
                        let entry = format!("- [{}] {}", now.format("%Y-%m-%d"), fact);
                        let path = std::path::Path::new(".geminix/memory.md");
                        let mut content = std::fs::read_to_string(path).unwrap_or_default();
                        if !content.is_empty() && !content.ends_with('\n') { content.push('\n'); }
                        content.push_str(&entry);
                        content.push('\n');
                        if let Err(e) = std::fs::write(path, &content) {
                            println!("{} Failed to save: {}", "[ERR]".red(), e);
                        } else {
                            println!("{} {}", "[MEM]".magenta(), entry);
                        }
                    }
                }

                "/forget" => {
                    let keyword = parts.get(1).map(|s| s.trim()).unwrap_or("");
                    if keyword.is_empty() {
                        println!("{}", "Usage: /forget <keyword> — remove matching entries from memory".dimmed());
                    } else {
                        let path = std::path::Path::new(".geminix/memory.md");
                        match std::fs::read_to_string(path) {
                            Ok(content) => {
                                let filtered: Vec<&str> = content.lines()
                                    .filter(|line| !line.to_lowercase().contains(&keyword.to_lowercase()))
                                    .collect();
                                let removed = content.lines().count() - filtered.len();
                                if removed == 0 {
                                    println!("{} No entries matching '{}'", "[MEM]".magenta(), keyword);
                                } else {
                                    let new_content = filtered.join("\n") + "\n";
                                    if let Err(e) = std::fs::write(path, &new_content) {
                                        println!("{} Failed to update: {}", "[ERR]".red(), e);
                                    } else {
                                        println!("{} Removed {} entr{} matching '{}'", "[MEM]".magenta(), removed, if removed == 1 { "y" } else { "ies" }, keyword);
                                    }
                                }
                            }
                            Err(_) => {
                                println!("{} No memory file found.", "[MEM]".magenta());
                            }
                        }
                    }
                }

                "/memory" => {
                    let path = std::path::Path::new(".geminix/memory.md");
                    match std::fs::read_to_string(path) {
                        Ok(content) if !content.trim().is_empty() => {
                            println!("\n{} Persistent Memory:", "[MEM]".magenta());
                            for line in content.lines() {
                                if !line.trim().is_empty() {
                                    println!("  {}", line.dimmed());
                                }
                            }
                            println!();
                        }
                        _ => println!("{} No memories yet. Use {} to add one.", "[MEM]".magenta(), "/memorize <fact>".cyan()),
                    }
                }

                "/explain" => {
                    match parts.get(1).map(|s| s.trim()) {
                        Some("on")  => { explain_exec = true;  println!("{}", "Explain-before-execute ON — agent will summarize planned actions before running.".green()); }
                        Some("off") => { explain_exec = false; println!("{}", "Explain-before-execute OFF.".dimmed()); }
                        _ => {
                            explain_exec = !explain_exec;
                            println!("{}", if explain_exec { "Explain-before-execute ON.".green().to_string() } else { "Explain-before-execute OFF.".dimmed().to_string() });
                        }
                    }
                }

                "/test-fix" => {
                    let test_cmd = parts.get(1).map(|s| s.trim()).unwrap_or("cargo test");
                    let max_cycles: u32 = parts.get(2).and_then(|s| s.trim().parse().ok()).unwrap_or(3);
                    println!("{} Test-fix mode: '{}' (max {} cycles)", "[TEST]".cyan(), test_cmd, max_cycles);
                    let active_cfg = active_config(config, &current_model, grounding, thinking, thinking_budget, auto_apply);
                    match BackendClient::new(&active_cfg) {
                        Ok(active_client) => {
                            if let Err(e) = test_fix_loop(&active_client, &mut history, &active_cfg, test_cmd, max_cycles, Some(mcp.clone()), Some(integrations.clone()), &mut cost_tracker).await {
                                ui::print_error(&e.to_string());
                            }
                        }
                        Err(e) => ui::print_error(&e.to_string()),
                    }
                }

                "/task" => {
                    let task_req = parts.get(1..).map(|s| s.join(" ")).unwrap_or_default();
                    if task_req.is_empty() {
                        println!("{}", "Usage: /task <requirement> — full pipeline: research → decompose → dispatch → consensus → merge".dimmed());
                    } else {
                        let active_cfg = active_config(config, &current_model, grounding, thinking, thinking_budget, auto_apply);
                        let mut orch = orchestrator::TaskOrchestrator::new(&active_cfg);
                        match orch.run(&task_req, Some(mcp.clone()), Some(integrations.clone())).await {
                            Ok(report) => {
                                // Add the task result to conversation history
                                history.push(Content {
                                    role: "user".to_string(),
                                    parts: vec![Part::text(&format!("Task completed: {}", task_req))],
                                });
                                history.push(Content {
                                    role: "model".to_string(),
                                    parts: vec![Part::text(&report)],
                                });
                            }
                            Err(e) => ui::print_error(&format!("Task orchestration failed: {}", e)),
                        }
                    }
                }

                "/undo" => {
                    let n: usize = parts.get(1).and_then(|s| s.trim().parse().ok()).unwrap_or(1);
                    let mut count = 0;
                    for _ in 0..n {
                        match snapshot::undo() {
                            Some(desc) => {
                                println!("{} Undone: {}", "↩".bright_yellow(), desc.dimmed());
                                count += 1;
                            }
                            None => {
                                if count == 0 {
                                    println!("{}", "Nothing to undo.".dimmed());
                                }
                                break;
                            }
                        }
                    }
                    if count > 0 {
                        println!("{} {} change(s) reverted.", "[OK]".green(), count);
                    }
                }

                "/diff" => {
                    let snaps = snapshot::list();
                    if snaps.is_empty() {
                        println!("{}", "No file snapshots in this session.".dimmed());
                    } else {
                        println!("{}", "Snapshot stack (most recent first):".dimmed());
                        for (i, (path, desc)) in snaps.iter().enumerate() {
                            println!("  {:2}. {} — {}", i + 1, path.cyan(), desc.dimmed());
                        }
                    }
                }

                "/tokens" => {
                    let window = config::context_window(&current_model);
                    println!("{} {} tokens used this session", "◦".dimmed(), session_tokens.to_string().cyan());
                    ui::print_context_bar(session_tokens, window);
                }

                "/audit" => {
                    let n: usize = parts.get(1).and_then(|s| s.trim().parse().ok()).unwrap_or(10);
                    let entries = audit::tail(n);
                    if entries.is_empty() {
                        println!("{}", "No audit entries yet.".dimmed());
                    } else {
                        println!();
                        for e in &entries {
                            let icon = if e.success { "[OK]".green() } else { "[ERR]".red() };
                            println!(
                                "  {} {} {}  {}",
                                icon,
                                e.timestamp.dimmed(),
                                e.action.cyan(),
                                e.detail.chars().take(80).collect::<String>().dimmed()
                            );
                        }
                        println!();
                    }
                }

                "/snapshot" => {
                    let label = parts.get(1).map(|s| s.trim()).unwrap_or("manual");
                    let args = serde_json::json!({ "name": label });
                    let ctx  = ToolContext { stream_output: false, auto_apply, mcp: Some(mcp.clone()), integrations: Some(integrations.clone()) };
                    let result = tools::execute_tool("git_snapshot", &args, &ctx).await;
                    if result.is_error {
                        ui::print_tool_result_err(&result.output);
                    } else {
                        println!("{} {}", "[OK]".green(), result.output.dimmed());
                    }
                }

                "/rollback" => {
                    println!("{} Rolling back to last git stash...", "[BUSY]".bright_yellow());
                    let out = tokio::process::Command::new("git")
                        .args(["stash", "pop"])
                        .output().await;
                    match out {
                        Ok(o) if o.status.success() => {
                            let msg = String::from_utf8_lossy(&o.stdout).trim().to_string();
                            println!("{} Rolled back: {}", "[OK]".green(), msg.dimmed());
                        }
                        Ok(o) => {
                            let err = String::from_utf8_lossy(&o.stderr).trim().to_string();
                            ui::print_error(&format!("git stash pop failed: {}", err));
                        }
                        Err(e) => ui::print_error(&format!("git not available: {}", e)),
                    }
                }

                "/cd" => {
                    if let Some(dir) = parts.get(1).map(|s| s.trim()) {
                        match std::env::set_current_dir(dir) {
                            Ok(_) => {
                                let cwd = std::env::current_dir()
                                    .map(|p| p.display().to_string())
                                    .unwrap_or_else(|_| dir.to_string());
                                println!("{} {}", "cwd:".dimmed(), cwd.cyan());
                            }
                            Err(e) => ui::print_error(&format!("cd: {}", e)),
                        }
                    } else {
                        println!("{}", std::env::current_dir()
                            .map(|p| p.display().to_string())
                            .unwrap_or_else(|_| "?".to_string()));
                    }
                }

                "/load" => {
                    let path = parts.get(1).map(|s| s.trim()).unwrap_or(".");
                    println!("{} Loading project from '{}'...", "[BUSY]".bright_yellow(), path);
                    match project::load_project(path, None) {
                        Ok(proj) => {
                            history.push(Content {
                                role:  "user".to_string(),
                                parts: vec![Part::text(&proj.context_block)],
                            });
                            history.push(Content {
                                role:  "model".to_string(),
                                parts: vec![Part::text(format!(
                                    "OK Loaded {} files (~{} tokens). Ask me anything.",
                                    proj.file_count, proj.token_estimate
                                ))],
                            });
                            println!(
                                "{} Loaded {} files (~{} tokens) into context.",
                                "[OK]".green(), proj.file_count, proj.token_estimate
                            );
                        }
                        Err(e) => ui::print_error(&e.to_string()),
                    }
                }

                "/learn" => {
                    let url = match parts.get(1).map(|s| s.trim()) {
                        Some(u) if !u.is_empty() => u,
                        _ => { println!("Usage: /learn <git-url>"); continue; }
                    };
                    println!("{} Cloning {}...", "[BUSY]".bright_yellow(), url);
                    match project::clone_and_load(url).await {
                        Ok(proj) => {
                            history.push(Content {
                                role:  "user".to_string(),
                                parts: vec![Part::text(format!(
                                    "I cloned {} and want to learn about it:\n\n{}",
                                    url, proj.context_block
                                ))],
                            });
                            history.push(Content {
                                role:  "model".to_string(),
                                parts: vec![Part::text(format!(
                                    "OK Loaded {} ({} files, ~{} tokens). Ask me anything.",
                                    url, proj.file_count, proj.token_estimate
                                ))],
                            });
                            println!("{} Loaded {} files (~{} tokens).", "[OK]".green(), proj.file_count, proj.token_estimate);
                        }
                        Err(e) => ui::print_error(&format!("clone failed: {}", e)),
                    }
                }

                "/pr" => {
                    let desc = parts.get(1).map(|s| s.trim()).unwrap_or("Update code");
                    auto_pr(desc).await;
                }

                "/security" => {
                    let sec_cfg = Config {
                        api_key:         config.api_key.clone(),
                        model:           current_model.clone(),
                        grounding:       true,
                        ..Config::default()
                    };
                    if let Err(e) = security::sweep(&sec_cfg).await {
                        ui::print_error(&e.to_string());
                    }
                }

                "/screenshot" => {
                    let path = match parts.get(1).map(|s| s.trim()) {
                        Some(p) if !p.is_empty() => p,
                        _ => { println!("Usage: /screenshot <path>"); continue; }
                    };
                    match encode_image(path) {
                        Ok((mime, data)) => {
                            history.push(Content {
                                role: "user".to_string(),
                                parts: vec![
                                    Part::text("Analyze this screenshot, identify any visible bugs, find the relevant source code, and fix it."),
                                    Part::image(mime, data),
                                ],
                            });
                            let active_cfg = active_config(config, &current_model, grounding, thinking, thinking_budget, auto_apply);
                            match BackendClient::new(&active_cfg) {
                                Ok(active_client) => {
                                    if let Err(e) = agentic_loop(&active_client, &mut history, &active_cfg, explain_exec, Some(mcp.clone()), Some(integrations.clone()), &mut cost_tracker).await {
                                        ui::print_error(&e.to_string());
                                    }
                                }
                                Err(e) => ui::print_error(&e.to_string()),
                            }
                        }
                        Err(e) => ui::print_error(&format!("Cannot load image '{}': {}", path, e)),
                    }
                }

                "/compact" => {
                    if history.is_empty() {
                        println!("{}", "Nothing to compact.".dimmed());
                        continue;
                    }
                    println!("{}", "Compacting conversation...".dimmed());
                    let compact_cfg = Config { api_key: config.api_key.clone(), model: current_model.clone(), ..Config::default() };
                    match compact_history(&compact_cfg, &history).await {
                        Ok(summary) => {
                            history.clear();
                            history.push(Content { role: "user".to_string(), parts: vec![Part::text("Summary of previous conversation:")] });
                            history.push(Content { role: "model".to_string(), parts: vec![Part::text(&summary)] });
                            session_tokens = 0;
                            println!("{} Compacted ({} chars).", "[OK]".green(), summary.len());
                        }
                        Err(e) => ui::print_error(&format!("Compact failed: {}", e)),
                    }
                }

                "/session" => {
                    match parts.get(1).map(|s| s.trim()) {
                        Some("save") => {
                            let name = parts.get(2).map(|s| s.trim()).unwrap_or("default");
                            match session::save_session(name, &history, &current_model, grounding, thinking, thinking_budget) {
                                Ok(_) => println!("  Session '{}' saved ({} turns).", name.cyan(), history.len()),
                                Err(e) => println!("  Save failed: {}", e),
                            }
                        }
                        Some("load") => {
                            let name = parts.get(2).map(|s| s.trim()).unwrap_or("default");
                            match session::load_session(name) {
                                Ok(s) => {
                                    history = s.history;
                                    current_model = s.model;
                                    grounding = s.grounding;
                                    thinking = s.thinking;
                                    thinking_budget = s.thinking_budget;
                                    session_tokens = 0;
                                    cost_tracker = CostTracker::new(&current_model, config.daily_budget_usd);
                                    println!("  Session '{}' loaded ({} turns).", name.cyan(), history.len());
                                }
                                Err(e) => println!("  Load failed: {}", e),
                            }
                        }
                        Some("list") | Some("ls") => {
                            let sessions = session::list_sessions();
                            if sessions.is_empty() {
                                println!("  No saved sessions.");
                            } else {
                                for s in sessions {
                                    println!("  {}  {}  {} turns  {}KB", s.name.cyan(), s.created.dimmed(), s.turns, s.size_bytes / 1024);
                                }
                            }
                        }
                        Some("delete") | Some("rm") => {
                            if let Some(name) = parts.get(2).map(|s| s.trim()) {
                                match session::delete_session(name) {
                                    Ok(_) => println!("  Session '{}' deleted.", name.cyan()),
                                    Err(e) => println!("  Delete failed: {}", e),
                                }
                            } else {
                                println!("Usage: /session delete <name>");
                            }
                        }
                        _ => {
                            println!("  /session save <name>  — save session to disk");
                            println!("  /session load <name>  — restore saved session");
                            println!("  /session list         — list saved sessions");
                            println!("  /session delete <name> — remove a saved session");
                        }
                    }
                }

                "/save" => {
                    let filename = parts.get(1).map(|s| s.trim()).unwrap_or("geminix-session.md");
                    match save_session(filename, &history) {
                        Ok(_)  => println!("{} '{}'", "Saved session to".green(), filename),
                        Err(e) => ui::print_error(&format!("Save failed: {}", e)),
                    }
                }

                unknown => {
                    println!("{} '{}' — type /help", "Unknown command:".yellow(), unknown);
                }
            }
            continue;
        }

        // ── Regular message ────────────────────────────────────────────────────
        let message_text = line;

        // Auto-routing: select best model based on task complexity
        let effective_model = if current_model == "auto" {
            let (model, reason) = auto_route_model(config, &message_text);
            println!("  {} routed to {} — {}", "[AUTO]".cyan(), model.yellow(), reason.dimmed());
            model.to_string()
        } else {
            current_model.clone()
        };

        history.push(Content {
            role:  "user".to_string(),
            parts: vec![Part::text(message_text)],
        });

        let active_cfg = active_config(config, &effective_model, grounding, thinking, thinking_budget, auto_apply);
        let active_client = match BackendClient::new(&active_cfg) {
            Ok(c) => c,
            Err(e) => { ui::print_error(&e.to_string()); continue; }
        };

        match agentic_loop(&active_client, &mut history, &active_cfg, explain_exec, Some(mcp.clone()), Some(integrations.clone()), &mut cost_tracker).await {
            Ok(tokens) => {
                session_tokens = session_tokens.saturating_add(tokens);
                let window = config::context_window(&current_model);
                let pct = session_tokens as f32 / window as f32;
                if session_tokens > 0 && pct >= active_cfg.context_warn {
                    ui::print_context_warning(pct);
                }
                // Cost tracking
                let cost_status = cost_tracker.format_status();
                println!("  {} {}", "$".dimmed(), cost_status.dimmed());
                if let Some(warning) = cost_tracker.budget_warning() {
                    println!("  {} {}", "!".yellow(), warning.yellow());
                }
                // Auto-compaction at threshold
                if pct >= active_cfg.context_compact && history.len() > 4 {
                    println!(
                        "\n  {} Context at {:.0}% — auto-compacting...",
                        "&".yellow(),
                        pct * 100.0
                    );
                    match compact_history(&active_cfg, &history).await {
                        Ok(summary) => {
                            history.clear();
                            history.push(Content {
                                role: "user".to_string(),
                                parts: vec![Part::text(format!(
                                    "[Conversation compacted — summary of prior context follows]\n\n{}",
                                    summary
                                ))],
                            });
                            session_tokens = 0;
                            println!("  {} Context compacted.", "+".green());
                        }
                        Err(e) => {
                            println!("  {} Auto-compact failed: {}", "x".red(), e);
                        }
                    }
                }
                // Auto-save session after successful turns
                let _ = session::save_session("auto", &history, &current_model, grounding, thinking, thinking_budget);

                if debug {
                    println!("  {} session tokens: {}", "dbg".dimmed(), session_tokens);
                }
            }
            Err(e) => ui::print_error(&e.to_string()),
        }
    }

    let _ = rl.save_history(&history_path);
    Ok(())
}

fn active_config(
    base:           &Config,
    model:          &str,
    grounding:      bool,
    thinking:       bool,
    thinking_budget: i32,
    auto_apply:     bool,
) -> Config {
    Config {
        api_key:         base.api_key.clone(),
        model:           model.to_string(),
        grounding,
        thinking,
        thinking_budget,
        auto_apply,
        max_iterations:  base.max_iterations,
        context_warn:    base.context_warn,
        context_compact: base.context_compact,
        mcp_servers:     base.mcp_servers.clone(),
        integrations:    base.integrations.clone(),
        daily_budget_usd: base.daily_budget_usd,
        anthropic_api_key: base.anthropic_api_key.clone(),
        openai_api_key: base.openai_api_key.clone(),
        explain_before_execute: base.explain_before_execute,
    }
}

// ── Core agentic loop — returns total prompt tokens consumed ──────────────────

async fn agentic_loop(
    client:  &BackendClient,
    history: &mut Vec<Content>,
    config:  &Config,
    explain_exec: bool,
    mcp: Option<Arc<McpRegistry>>,
    integrations: Option<Arc<IntegrationRegistry>>,
    cost_tracker: &mut CostTracker,
) -> Result<u32> {
    let sys         = system_prompt(config);
    let mut total_prompt_tokens = 0u32;
    let mut iterations = 0u32;
    let mut last_error: String = String::new();
    let mut consecutive_same_error = 0u32;

    loop {
        // Progress detection: if same error repeats 3 times, the agent is stuck
        if consecutive_same_error >= 3 {
            println!(
                "\n  {} Agent appears stuck — same error repeated {} times. Pausing for review.",
                "!".yellow(),
                consecutive_same_error
            );
            println!("  {} Last error: {}", "|".dimmed(), last_error.dimmed());
            print!("  Continue? [Y/n] ");
            let _ = std::io::stdout().flush();
            let mut ans = String::new();
            let _ = std::io::stdin().read_line(&mut ans);
            if ans.trim().to_lowercase() == "n" {
                break;
            }
            consecutive_same_error = 0;
        }

        // Iteration guard
        if config.max_iterations > 0 && iterations >= config.max_iterations {
            println!(
                "\n  {} Reached {} tool-call rounds. Continue? [Y/n] ",
                "*".yellow(),
                config.max_iterations
            );
            let _ = std::io::stdout().flush();
            let mut ans = String::new();
            let _ = std::io::stdin().read_line(&mut ans);
            if ans.trim().to_lowercase() == "n" {
                break;
            }
            iterations = 0;
        }

        ui::print_thinking();

        let request = GenerateContentRequest {
            contents:           history.clone(),
            tools:              build_tools(config.grounding, mcp.as_deref(), integrations.as_deref()),
            tool_config:        Some(build_tool_config()),
            system_instruction: Some(SystemContent { parts: vec![Part::text(&sys)] }),
            generation_config:  Some(build_generation_config(config.thinking, config.thinking_budget)),
        };

        // ── Streaming ─────────────────────────────────────────────────────────
        let first_text     = std::cell::Cell::new(true);
        let thought_active = std::cell::Cell::new(false);
        let thought_buf    = std::cell::RefCell::new(String::new());

        let mut on_thought = |chunk: &str| {
            if !thought_active.get() {
                println!();
                println!("  {} {}", "[THINK]".yellow(), "THINKING...".yellow().dimmed());
                thought_active.set(true);
            }
            thought_buf.borrow_mut().push_str(chunk);
            let mut buf = thought_buf.borrow_mut();
            while let Some(pos) = buf.find('\n') {
                let line = buf[..pos].to_string();
                *buf = buf[pos + 1..].to_string();
                println!("  {} {}", "│".dimmed(), line.dimmed().yellow());
            }
            let _ = std::io::stdout().flush();
        };

        let mut on_text = |chunk: &str| {
            if thought_active.get() {
                let rem = thought_buf.borrow().trim_end().to_string();
                if !rem.is_empty() {
                    println!("  {} {}", "│".dimmed(), rem.dimmed().yellow());
                }
                thought_buf.borrow_mut().clear();
                println!("  {} {}", "[OK]".green().dimmed(), "Reasoning complete.".dimmed());
                thought_active.set(false);
            }
            if first_text.get() {
                print!("\n{} ", "GeminiX".bright_blue().bold());
                first_text.set(false);
            }
            print!("{}", chunk);
            let _ = std::io::stdout().flush();
        };

        let response = tokio::select! {
            res = client.generate_streaming(&request, &mut on_text, &mut on_thought) => res?,
            _ = tokio::signal::ctrl_c() => {
                println!("\n{}", "Interrupted.".yellow().dimmed());
                return Ok(total_prompt_tokens);
            }
        };

        // Flush remaining thought
        if thought_active.get() {
            let rem = thought_buf.borrow().trim_end().to_string();
            if !rem.is_empty() {
                println!("  {} {}", "│".dimmed(), rem.dimmed().yellow());
            }
            println!("  {} {}", "[OK]".green().dimmed(), "Reasoning complete.".dimmed());
        }
        let first_text = first_text.get();

        // ── Parse candidate ───────────────────────────────────────────────────
        let candidate = response.candidates
            .and_then(|mut c| if c.is_empty() { None } else { Some(c.remove(0)) })
            .ok_or_else(|| anyhow::anyhow!("Gemini returned no candidates"))?;

        let content = candidate.content
            .ok_or_else(|| anyhow::anyhow!("Candidate has no content"))?;

        let mut text_chunks:    Vec<String>       = Vec::new();
        let mut function_calls: Vec<FunctionCall> = Vec::new();

        for part in &content.parts {
            match part {
                Part::Text { text, thought: None | Some(false) } if !text.trim().is_empty() => {
                    text_chunks.push(text.clone());
                }
                Part::FunctionCall { function_call } => {
                    function_calls.push(function_call.clone());
                }
                _ => {}
            }
        }

        history.push(content);

        if first_text && !text_chunks.is_empty() {
            ui::print_assistant_prefix();
            println!("{}", text_chunks.join("\n"));
        } else if !first_text {
            println!();
        }

        if let Some(usage) = response.usage_metadata {
            let p  = usage.prompt_token_count.unwrap_or(0);
            let c  = usage.candidates_token_count.unwrap_or(0);
            let t  = usage.total_token_count.unwrap_or(0);
            let th = usage.thoughts_token_count.unwrap_or(0);
            if t > 0 {
                ui::print_token_usage(p, c, t, th);
                total_prompt_tokens = total_prompt_tokens.saturating_add(t);
                cost_tracker.record_usage(p, c, th);

                // Context bar when approaching limit
                let window = config::context_window(&config.model);
                let total_pct = total_prompt_tokens as f32 / window as f32;
                if total_pct >= 0.40 {
                    ui::print_context_bar(total_prompt_tokens, window);
                }
                if total_prompt_tokens > 0 && total_pct >= config.context_compact {
                    println!(
                        "  {} Context at {:.0}% — run {} now to avoid truncation",
                        "[CRITICAL]".red(), total_pct * 100.0, "/compact".yellow()
                    );
                }
            }
        }

        if function_calls.is_empty() {
            println!();
            break;
        }

        iterations += 1;

        // ── Explain-before-execute ──────────────────────────────────────────
        if explain_exec && !function_calls.is_empty() && !config.auto_apply {
            println!();
            println!("  {} Planned actions:", "[PLAN]".cyan());
            for fc in &function_calls {
                let args_summary = fmt_args_compact(&fc.args);
                println!("    {} {} {}", "▸".cyan(), fc.name.yellow(), args_summary.dimmed());
            }
            print!("  {} Proceed? [Y/n] ", "?".yellow());
            let _ = std::io::stdout().flush();
            let mut ans = String::new();
            let _ = std::io::stdin().read_line(&mut ans);
            if ans.trim().to_lowercase() == "n" {
                println!("  {} Execution skipped.", "✗".dimmed());
                break;
            }
        }

        // ── ParallelOps ───────────────────────────────────────────────────────
        if function_calls.len() > 1 {
            println!(
                "  {} Running {} tools in parallel...",
                "[BATCH]".bright_yellow(),
                function_calls.len()
            );
        }

        let solo_bash = function_calls.len() == 1 && function_calls[0].name == "bash";
        let auto_ap   = config.auto_apply;

        let handles: Vec<_> = function_calls
            .iter()
            .map(|fc| {
                let name   = fc.name.clone();
                let args   = fc.args.clone();
                let stream = solo_bash;
                let mcp_opt = mcp.clone();
                let integ_opt = integrations.clone();
                tokio::spawn(async move {
                    let ctx = ToolContext { stream_output: stream, auto_apply: auto_ap, mcp: mcp_opt, integrations: integ_opt };
                    let result = tools::execute_tool(&name, &args, &ctx).await;
                    (name, args, result)
                })
            })
            .collect();

        let mut response_parts: Vec<Part> = Vec::new();

        for handle in handles {
            let (name, args, result) = handle.await?;
            ui::print_tool_call(&name, &fmt_args(&args));

            if result.was_streamed {
                if result.is_error { ui::print_tool_result_err("(see output above)"); }
            } else if result.is_error {
                ui::print_tool_result_err(&result.output);
                // Progress detection
                let err_key = result.output.chars().take(80).collect::<String>();
                if err_key == last_error {
                    consecutive_same_error += 1;
                } else {
                    last_error = err_key;
                    consecutive_same_error = 1;
                }
            } else {
                ui::print_tool_result_ok(&result.output);
                consecutive_same_error = 0;
            }

            response_parts.push(Part::FunctionResponse {
                function_response: FunctionResponse {
                    name,
                    response: serde_json::json!({ "content": result.output }),
                    id: None,
                },
            });
        }

        history.push(Content { role: "user".to_string(), parts: response_parts });
    }

    Ok(total_prompt_tokens)
}

// ── AutoPR ────────────────────────────────────────────────────────────────────

async fn auto_pr(description: &str) {
    println!("{} Checking git status...", "[BUSY]".bright_yellow());

    let check = tokio::process::Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .output().await;

    if check.map(|o| !o.status.success()).unwrap_or(true) {
        ui::print_error("Not inside a git repository.");
        return;
    }

    let branch_out = tokio::process::Command::new("git")
        .args(["branch", "--show-current"])
        .output().await;
    let branch = branch_out
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "HEAD".to_string());

    println!("{} Pushing branch '{}'...", "[BUSY]".bright_yellow(), branch);
    let push = tokio::process::Command::new("git")
        .args(["push", "-u", "origin", &branch])
        .output().await;

    if let Ok(p) = &push {
        if !p.status.success() {
            let err = String::from_utf8_lossy(&p.stderr);
            ui::print_error(&format!("git push failed: {}", err.trim()));
            return;
        }
    }

    let body = format!(
        "## Summary\n\nAuto-generated by GeminiX 1.0.\n\n{}\n\n\
         > Created with [GeminiX](https://github.com/geminix/geminix)",
        description
    );

    println!("{} Creating PR via gh CLI...", "[BUSY]".bright_yellow());

    match tokio::process::Command::new("gh")
        .args(["pr", "create", "--title", description, "--body", &body])
        .output().await
    {
        Ok(o) if o.status.success() => {
            let url = String::from_utf8_lossy(&o.stdout).trim().to_string();
            println!("{} PR created: {}", "[OK]".green(), url.cyan());
        }
        Ok(o) => {
            let err = String::from_utf8_lossy(&o.stderr);
            ui::print_error(&format!("gh pr create failed: {}", err.trim()));
        }
        Err(e) => {
            ui::print_error(&format!("gh CLI not found ({}). Install: https://cli.github.com", e));
        }
    }
}

// ── /compact ──────────────────────────────────────────────────────────────────

async fn compact_history(config: &Config, history: &[Content]) -> Result<String> {
    let transcript: String = history.iter().map(|msg| {
        let role = if msg.role == "user" { "User" } else { "Assistant" };
        let text: String = msg.parts.iter().filter_map(|p| {
            if let Part::Text { text, .. } = p { Some(text.as_str()) } else { None }
        }).collect::<Vec<_>>().join(" ");
        format!("{}: {}", role, text)
    }).collect::<Vec<_>>().join("\n\n");

    let prompt = format!(
        "Produce a dense technical summary of this conversation. \
         Preserve all code snippets, file paths, commands, errors, and decisions. \
         This summary replaces the full history — nothing important should be lost.\n\n---\n{}",
        transcript
    );

    let client  = BackendClient::new(config)?;
    let request = GenerateContentRequest {
        contents: vec![Content { role: "user".to_string(), parts: vec![Part::text(&prompt)] }],
        tools:    vec![],
        tool_config:        None,
        system_instruction: None,
        generation_config: Some(GenerationConfig {
            temperature: Some(0.3),
            max_output_tokens: Some(4096),
            thinking_config: None,
        }),
    };

    let resp = client.generate(request).await?;
    resp.candidates.and_then(|mut c| c.pop())
        .and_then(|c| c.content)
        .and_then(|c| c.parts.into_iter().find_map(|p| {
            if let Part::Text { text, .. } = p { Some(text) } else { None }
        }))
        .ok_or_else(|| anyhow::anyhow!("No summary returned"))
}

// ── /save ─────────────────────────────────────────────────────────────────────

fn save_session(filename: &str, history: &[Content]) -> Result<()> {
    let mut out = String::from("# GeminiX Session\n\n");
    for msg in history {
        out.push_str(if msg.role == "user" { "## You\n" } else { "## GeminiX\n" });
        for part in &msg.parts {
            match part {
                Part::Text { text, .. } => { out.push_str(text); out.push('\n'); }
                Part::FunctionCall { function_call } => {
                    out.push_str(&format!("\n**Tool:** `{}` `{}`\n", function_call.name, function_call.args));
                }
                Part::FunctionResponse { function_response } => {
                    out.push_str(&format!("\n**Result:** `{}`\n", function_response.response));
                }
                _ => {}
            }
        }
        out.push('\n');
    }
    std::fs::write(filename, &out)?;
    Ok(())
}

// ── Image encoding ─────────────────────────────────────────────────────────────

fn encode_image(path: &str) -> Result<(String, String)> {
    use base64::Engine as _;
    let bytes = std::fs::read(path)?;
    let mime = match path.rsplit('.').next().unwrap_or("").to_lowercase().as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png"          => "image/png",
        "gif"          => "image/gif",
        "webp"         => "image/webp",
        "bmp"          => "image/bmp",
        _              => "image/jpeg",
    };
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Ok((mime.to_string(), b64))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn fmt_args(args: &serde_json::Value) -> String {
    let Some(obj) = args.as_object() else { return args.to_string() };
    obj.iter().map(|(k, v)| {
        let val = match v {
            serde_json::Value::String(s) => {
                let s = s.replace('\n', "↵");
                if s.chars().count() > 60 {
                    format!("{}…", s.chars().take(60).collect::<String>())
                } else { s }
            }
            _ => v.to_string(),
        };
        format!("{}={}", k, val)
    }).collect::<Vec<_>>().join("  ")
}

fn fmt_args_compact(args: &serde_json::Value) -> String {
    let Some(obj) = args.as_object() else { return args.to_string() };
    let parts: Vec<String> = obj.iter().take(3).map(|(k, v)| {
        let val = match v {
            serde_json::Value::String(s) => {
                let s = s.replace('\n', "↵");
                if s.chars().count() > 40 {
                    format!("{}…", s.chars().take(40).collect::<String>())
                } else { s }
            }
            _ => v.to_string(),
        };
        format!("{}={}", k, val)
    }).collect();
    let mut s = parts.join(" ");
    if obj.len() > 3 { s.push_str(" …"); }
    s
}

// ── Auto model routing ──────────────────────────────────────────────────────

/// Classify task complexity and return (model_name, reason).
/// Falls back to available providers based on configured API keys.
fn auto_route_model(config: &Config, message: &str) -> (&'static str, &'static str) {
    let lower = message.to_lowercase();

    // Complexity signals — high → reasoning model
    let complex_signals = [
        "refactor", "architecture", "migrate", "rewrite", "redesign",
        "security audit", "vulnerability", "optimize", "scale", "multi-thread",
        "async", "concurrent", "race condition", "deadlock", "memory leak",
        "design pattern", "microservice", "distributed", "database schema",
        "api design", "system design", "protocol", "encryption", "auth",
        "jwt", "oauth", "deploy", "ci/cd", "pipeline",
    ];

    // Simple signals — low → fast/cheap model
    let simple_signals = [
        "what is", "how do", "explain", "show me", "list", "find",
        "read", "check", "describe", "tell me", "lookup", "where is",
        "document", "search for", "grep", "locate",
    ];

    let complexity = if complex_signals.iter().any(|s| lower.contains(s)) {
        "high"
    } else if simple_signals.iter().any(|s| lower.starts_with(s)) {
        "low"
    } else {
        "medium"
    };

    // Pick the best available model
    let has_anthropic = config.anthropic_api_key.as_deref().map_or(false, |k| !k.is_empty());
    let has_openai = config.openai_api_key.as_deref().map_or(false, |k| !k.is_empty());

    match complexity {
        "high" => {
            if has_anthropic {
                ("claude-4-sonnet", "complex task → Claude balanced reasoning")
            } else if has_openai {
                ("o3", "complex task → OpenAI reasoning")
            } else {
                ("gemini-2.5-pro", "complex task → Gemini deep reasoning")
            }
        }
        "low" => {
            // Cheapest available option
            if has_openai {
                ("gpt-4o", "simple task → GPT fast/affordable")
            } else if has_anthropic {
                ("claude-4-sonnet", "simple task → Claude")
            } else {
                ("gemini-2.5-flash-lite", "simple task → cheapest Gemini")
            }
        }
        _ => {
            // Medium complexity — balanced
            if has_anthropic {
                ("claude-4-sonnet", "normal task → Claude")
            } else if has_openai {
                ("gpt-4.1", "normal task → GPT balanced")
            } else {
                ("gemini-2.5-flash", "normal task → Gemini balanced")
            }
        }
    }
}

// ── /test-fix loop ───────────────────────────────────────────────────────────

async fn test_fix_loop(
    client: &BackendClient,
    history: &mut Vec<Content>,
    config: &Config,
    test_command: &str,
    max_cycles: u32,
    mcp: Option<Arc<McpRegistry>>,
    integrations: Option<Arc<IntegrationRegistry>>,
    cost_tracker: &mut CostTracker,
) -> Result<()> {
    for cycle in 1..=max_cycles {
        println!(
            "\n  {} Test-fix cycle {}/{} — running '{}'...",
            "[TEST]".cyan(), cycle, max_cycles, test_command.dimmed()
        );

        let output = tokio::process::Command::new("sh")
            .args(["-c", test_command])
            .output().await;

        match &output {
            Ok(o) if o.status.success() => {
                let stdout = String::from_utf8_lossy(&o.stdout);
                println!("  {} All tests passed!", "[OK]".green());
                if !stdout.trim().is_empty() {
                    println!("{}", stdout.dimmed());
                }
                return Ok(());
            }
            Ok(o) => {
                let stdout = String::from_utf8_lossy(&o.stdout);
                let stderr = String::from_utf8_lossy(&o.stderr);
                let combined = format!("{}\n{}", stdout, stderr);

                // Truncate if too long (keep last 4000 chars — that's where failures are)
                let truncated: String = if combined.len() > 4000 {
                    let start = combined.len().saturating_sub(4000);
                    format!("…(truncated)…\n{}", &combined[start..])
                } else {
                    combined.clone()
                };

                println!("  {} Tests FAILED. Feeding errors to model...", "[FAIL]".red());

                let prompt = format!(
                    "The test command '{}' failed. Here is the output:\n\n```\n{}\n```\n\n\
                     Analyze the failures. Find the root cause(s). Fix the code. \
                     Be surgical — only fix what's broken. Don't refactor passing code.\n\
                     Key rules:\n\
                     - Read the files that have failing tests before editing\n\
                     - Make minimal changes to fix the errors\n\
                     - Run cargo check after each file change\n\
                     - Don't add new dependencies unless absolutely necessary",
                    test_command, truncated
                );

                history.push(Content {
                    role: "user".to_string(),
                    parts: vec![Part::text(&prompt)],
                });

                let tokens = agentic_loop(client, history, config, false, mcp.clone(), integrations.clone(), cost_tracker).await?;
                let _ = tokens;
            }
            Err(e) => {
                println!("  {} Failed to run tests: {}", "[ERR]".red(), e);
                return Err(anyhow::anyhow!("Test command failed: {}", e));
            }
        }
    }

    println!(
        "\n  {} Test-fix loop ended after {} cycles — tests still failing.",
        "[WARN]".yellow(), max_cycles
    );
    Ok(())
}
