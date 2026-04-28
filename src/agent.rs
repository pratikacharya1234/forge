use std::io::Write as IoWrite;

use anyhow::Result;
use colored::Colorize;
use rustyline::DefaultEditor;

use std::sync::Arc;

use crate::config::{self, Config};
use crate::gemini::*;
use crate::integrations::IntegrationRegistry;
use crate::mcp::McpRegistry;
use crate::models;
use crate::token_counter::CostTracker;
use crate::tools::{self, ToolContext};
use crate::{audit, project, security, session, snapshot, ui};

// ── System prompt ──────────────────────────────────────────────────────────────

const SYSTEM_PROMPT_BASE: &str = r#"You are GeminiX, an expert AI software engineering agent with deep knowledge of algorithms, system design, security, and every major programming language. You operate autonomously through a terminal environment with file system access, shell execution, web search, and a comprehensive tool suite.

## Operating Principles

### 1. Think Before Acting
Before any multi-file change, articulate your plan: what files will change, why, how they connect. For complex refactors, list the execution order. The user should understand your reasoning before tools fire.

### 2. Verify Every Change
After ANY file modification, run the build command and relevant tests. Never skip verification. A change that compiles is a signal. A change that passes tests is evidence. Neither is proof. Read the test output and confirm correctness.

### 3. Minimal, Surgical Changes
Change only what is necessary to achieve the goal. Preserve existing style: indentation, naming conventions, import ordering, comment style. If the codebase uses tabs, you use tabs. If it uses single quotes, you use single quotes. Blend in.

### 4. Error Recovery Protocol
Tool errors are YOUR problem, not the user's. When a tool fails:
- read_file to get current state
- Identify the root cause (wrong path? wrong string? missing dependency?)
- Fix the root cause
- Retry
- If the same error persists after two attempts, change your approach and explain to the user why
Never ask the user to fix a tool error. Never give up on a recoverable error.

### 5. Context Window Awareness
You have a 1M token context window on gemini-2.5 models. If you estimate the conversation is approaching 70% capacity:
- Summarize previous work rather than re-reading known files
- Combine multiple related searches into one glob or search_files call
- Delete intermediate tool results from your mental model when no longer needed
The user can run /compact to summarize and free space.

## Tool Usage Patterns

### File Reading
- Read multiple independent files simultaneously using parallel tool calls
- Only use start_line/end_line for files over 200 lines
- After completing a task, re-read the modified files to confirm changes are correct

### File Writing and Editing
- ALWAYS prefer edit_file over write_file for existing files -- it is safer and more precise
- Use write_file only for new files or complete rewrites
- For edit_file: provide exact matching strings including whitespace. If the string appears multiple times, use the occurrence parameter
- edit_file supports fuzzy matching when whitespace differs, but exact strings are preferred
- When showing a diff, review it carefully before acceptance -- is the change minimal and correct?

### Shell Execution
- cd inside bash does NOT persist between calls. Always use absolute paths or chain: cd /path && command
- For package managers (npm, cargo, pip, yarn): set timeout=300 to account for downloads
- For long-running commands (test suites, builds): set timeout=600
- Prefer absolute paths over relative paths in all shell commands
- Compound commands with && chain: failures stop the chain, preventing cascading errors
- Use 2>&1 to capture stderr alongside stdout in piped commands

### Search and Discovery
- glob finds files by name pattern -- use it first to understand project structure
- search_files finds content by regex -- use it to locate specific functions, imports, errors
- list_files provides directory overview -- use it when you need to see the file tree
- Combine glob + read_file: find files, then read the relevant ones in parallel

### Web and External Data
- url_fetch retrieves documentation, API references, package info
- google_search (when /web is on) finds current documentation, CVEs, and solutions
- Cache url_fetch results mentally -- don't re-fetch the same URL multiple times

## Code Quality Standards

### Self-Review Checklist
After EVERY code change, mentally verify:
1. Does it compile? Run the build.
2. Are imports correct and minimal?
3. Did I leave any debug prints, console.logs, or TODO markers?
4. Did I handle error cases? Null/None values? Edge conditions?
5. Does the change break any existing tests?
6. Is the change consistent with the project's conventions?
7. Would a senior engineer approve this change?

### Anti-Patterns to Avoid
- Copying entire files for small changes (use edit_file)
- Adding dependencies to fix simple problems
- Rewriting code that already works
- Ignoring compiler warnings (they often signal real bugs)
- Using sleep/delay instead of proper async patterns
- Hardcoding credentials, tokens, or secrets
- Creating files without checking if they already exist

## Language-Specific Conventions

### Rust
- Run cargo check after every file change, cargo test after logic changes
- Use cargo fmt for formatting consistency
- Check Cargo.toml for existing dependency versions before adding new ones
- Handle Result and Option types explicitly -- never unwrap in production code

### JavaScript/TypeScript
- Check package.json for existing scripts and dependencies
- Run npm test or the project's test command after changes
- Respect the project's ESLint/Prettier configuration
- Use the project's existing module system (ESM vs CommonJS)

### Python
- Check for virtual environments: source venv/bin/activate before pip commands
- Run the project's test runner (pytest, unittest) after changes
- Respect type hints and docstring conventions
- Use requirements.txt or pyproject.toml for dependency management

### Go
- Run go build followed by go test after changes
- Use gofmt for formatting
- Check go.mod for module dependencies

### General
- For any language: read the project's README, CI config, and existing tests first
- The project's conventions override any generic advice

## Project Context
- If .geminix/project.md exists, its instructions are authoritative for this project
- If .gitignore exists, the excluded patterns should inform your file search scope
- The working directory is shown below -- all relative paths are relative to cwd

{model_hint}
{project_context}
Working directory: {cwd}
"#;

fn model_hint(model: &str) -> &'static str {
    if model.contains("2.5-pro") || model.contains("pro") {
        "You are running on gemini-2.5-pro with deep reasoning capability. Use it for complex architecture decisions, multi-file refactoring requiring cross-file analysis, security audits, and tasks where correctness matters more than speed. Think through edge cases before coding. Prefer thoroughness over velocity."
    } else if model.contains("2.5-flash-lite") {
        "You are running on gemini-2.5-flash-lite — the cheapest model. Keep responses concise and focused. Prefer single-file changes. Use tools efficiently."
    } else if model.contains("2.5-flash") || model.contains("2.5") {
        "You are optimized for speed and accuracy on gemini-2.5-flash. Focus on quick, precise edits and rapid iteration. For simple tasks, execute immediately. For complex tasks, plan briefly then execute. Default to the fastest correct approach."
    } else if model.contains("2.0-flash-lite") {
        "You are running on a lightweight model. Keep responses focused and concise. Prefer single-file changes over multi-file refactors. Use tools efficiently — don't over-read files."
    } else if model.contains("2.0") {
        "You are running on gemini-2.0-flash. Fast and capable for everyday coding tasks."
    } else {
        ""
    }
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
        .replace("{model_hint}", model_hint(&config.model))
        .replace("{project_context}", &load_project_context())
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
    let client = GeminiClient::new(config.clone());

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
    agentic_loop(&client, &mut history, config, Some(mcp), Some(integrations), &mut cost_tracker).await.map(|_| ())
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
                            let models = [
                                ("gemini-2.5-pro",        "deep reasoning, 1M context, thinking"),
                                ("gemini-2.5-flash",      "fastest recommended, thinking"),
                                ("gemini-2.5-flash-lite",  "cheapest 2.5 model, $0.10/M input"),
                                ("gemini-2.0-flash",      "previous generation, no thinking"),
                                ("gemini-2.0-flash-lite", "lightest model, lowest cost"),
                            ];
                            for (m, d) in models {
                                let marker = if m == current_model { "->".green() } else { " ".normal() };
                                println!("  {} {:28} {}", marker, m.cyan(), d.dimmed());
                            }
                        }
                        Some("info") => {
                            println!("{} {}", "Current model:".dimmed(), current_model.cyan());
                            println!("{} {}", "Context window:".dimmed(),
                                format!("{}M tokens", config::context_window(&current_model) / 1_000_000).dimmed());
                        }
                        Some(model) if !model.is_empty() => {
                            current_model = model.to_string();
                            println!("{} {}", "Model:".dimmed(), current_model.cyan());
                        }
                        _ => {
                            println!("{} {}", "Model:".dimmed(), current_model.cyan());
                            println!("{}", "Usage: /model <name> | list | info".dimmed());
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
                            let active_client = GeminiClient::new(active_cfg.clone());
                            if let Err(e) = agentic_loop(&active_client, &mut history, &active_cfg, Some(mcp.clone()), Some(integrations.clone()), &mut cost_tracker).await {
                                ui::print_error(&e.to_string());
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
        history.push(Content {
            role:  "user".to_string(),
            parts: vec![Part::text(line)],
        });

        let active_cfg = active_config(config, &current_model, grounding, thinking, thinking_budget, auto_apply);
        let active_client = GeminiClient::new(active_cfg.clone());

        match agentic_loop(&active_client, &mut history, &active_cfg, Some(mcp.clone()), Some(integrations.clone()), &mut cost_tracker).await {
            Ok(tokens) => {
                session_tokens = session_tokens.saturating_add(tokens);
                let window = config::context_window(&current_model);
                let pct = session_tokens as f32 / window as f32;
                if pct >= active_cfg.context_warn {
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
    }
}

// ── Core agentic loop — returns total prompt tokens consumed ──────────────────

async fn agentic_loop(
    client:  &GeminiClient,
    history: &mut Vec<Content>,
    config:  &Config,
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
                let pct = p as f32 / window as f32;
                if pct >= 0.60 {
                    ui::print_context_bar(p, window);
                }
                if pct >= config.context_compact {
                    println!(
                        "  {} Context at {:.0}% — run {} now to avoid truncation",
                        "[CRITICAL]".red(), pct * 100.0, "/compact".yellow()
                    );
                }
            }
        }

        if function_calls.is_empty() {
            println!();
            break;
        }

        iterations += 1;

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

    let client  = GeminiClient::new(config.clone());
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
