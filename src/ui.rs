use colored::Colorize;

// ── FORGE logo — 6 rows × 43 visible columns ──────────────────────────────────

const LOGO: &[&str] = &[
    " ███████╗ ██████╗ ██████╗  ██████╗ ███████╗",
    " ██╔════╝██╔═══██╗██╔══██╗██╔════╝ ██╔════╝",
    " █████╗  ██║   ██║██████╔╝██║  ███╗█████╗  ",
    " ██╔══╝  ██║   ██║██╔══██╗██║   ██║██╔══╝  ",
    " ██║     ╚██████╔╝██║  ██║╚██████╔╝███████╗",
    " ╚═╝      ╚═════╝ ╚═╝  ╚═╝ ╚═════╝ ╚══════╝",
];

// Terminal geometry (80-column layout)
// ║ + space + 76 content + space + ║  =  80
const LOGO_W: usize = 43;
const INNER:  usize = 76;

// ── Box-drawing primitives ────────────────────────────────────────────────────

fn box_top() {
    println!(
        "{}",
        "╔══════════════════════════════════════════════════════════════════════════════╗"
            .bright_blue()
    );
}

fn box_bot() {
    println!(
        "{}",
        "╚══════════════════════════════════════════════════════════════════════════════╝"
            .bright_blue()
    );
}

fn box_empty() {
    println!(
        "{}{:78}{}",
        "║".bright_blue(),
        "",
        "║".bright_blue()
    );
}

// Print one 80-col bordered row where content has already been written via
// print! — caller owns the opening "║ " and passes `vis` = visible char count.
// Preferred: use box_row_pre / box_row_post pair so coloured content works.
fn box_row_pre() {
    print!("{} ", "║".bright_blue());
}
fn box_row_post(vis_len: usize) {
    let pad = INNER.saturating_sub(vis_len);
    print!("{}", " ".repeat(pad));
    println!(" {}", "║".bright_blue());
}

// ── Banner ────────────────────────────────────────────────────────────────────

pub fn print_banner(grounding: bool, thinking: bool, auto_apply: bool, tool_count: usize, integration_count: usize, context_tokens: u32) {
    let has_safety_toml = std::path::Path::new(".forge/safety.toml").exists();

    println!();
    box_top();
    box_empty();

    // Logo with forge-fire gradient: top = bright_red, base = white
    let logo_vis = [
        LOGO[0].bright_red(),
        LOGO[1].red(),
        LOGO[2].yellow(),
        LOGO[3].bright_yellow(),
        LOGO[4].yellow(),
        LOGO[5].white(),
    ];
    for c in &logo_vis {
        box_row_pre();
        print!("{}", c);
        box_row_post(LOGO_W);
    }

    box_empty();

    // Tagline + version
    // "  Multi-Model Terminal AI Coding Agent" = 38 chars
    // "v0.0.1"                                =  6 chars
    // padding between                         = 76 - 38 - 6 = 32
    box_row_pre();
    print!(
        "{}{}{}",
        "  Multi-Model Terminal AI Coding Agent".bright_white().bold(),
        " ".repeat(32),
        "v0.0.1".bright_cyan()
    );
    box_row_post(38 + 32 + 6);

    // Stats line — values come from the live runtime, not hardcoded strings
    let ctx_str = if context_tokens >= 1_000_000 {
        format!("{}M Context", context_tokens / 1_000_000)
    } else {
        format!("{}K Context", context_tokens / 1_000)
    };
    let int_str = if integration_count > 0 {
        format!("  ·  {} Integrations", integration_count)
    } else {
        String::new()
    };
    let stats = format!(
        "  Multi-Model  ·  {}  ·  {} Tools{}",
        ctx_str, tool_count, int_str
    );
    let stats_vis = stats.chars().count();
    box_row_pre();
    print!("{}", stats.cyan().dimmed());
    box_row_post(stats_vis);

    box_empty();
    box_bot();
    println!();

    // Active-mode flags — outside the box (ANSI width unpredictable inside)
    let mut flags: Vec<String> = Vec::new();
    if grounding      { flags.push("Search:ON".green().to_string()); }
    if thinking       { flags.push("Think:ON".yellow().to_string()); }
    if auto_apply     { flags.push("AutoApply:ON".bright_yellow().to_string()); }
    if has_safety_toml { flags.push("SafetyPolicy".magenta().to_string()); }
    flags.push("Safety:ON".bright_red().to_string());

    print!("  {}  ", "▸".bright_blue());
    for (i, f) in flags.iter().enumerate() {
        if i > 0 { print!("  "); }
        print!("{}", f);
    }
    println!();

    println!(
        "  {}  {}",
        "▸".bright_blue().dimmed(),
        "Type a task. FORGE reads, writes, runs, and iterates until done.".dimmed()
    );

    let cmds = [
        "/help", "/think", "/web", "/model", "/task",
        "/undo", "/compact", "/session", "/security", "/quit",
    ];
    print!("  {}  ", "▸".bright_blue().dimmed());
    for cmd in &cmds {
        print!("{}  ", cmd.yellow());
    }
    println!("\n");
}

// ── Thinking indicator ────────────────────────────────────────────────────────

pub fn print_thinking() {
    println!();
    println!(
        "  {} {}",
        "◆ REASONING".bright_yellow().bold(),
        "···".dimmed()
    );
}

pub fn print_thinking_with_model(model: &str) {
    let short_model = model.trim_start_matches("models/").split('-').take(3).collect::<Vec<_>>().join("-");
    println!();
    println!(
        "  {} [{}] {}",
        "◆ REASONING".bright_yellow().bold(),
        short_model.bright_blue(),
        "···".dimmed()
    );
}

// ── Tool output ───────────────────────────────────────────────────────────────

pub fn print_tool_call(tool_name: &str, args_display: &str) {
    let display = if args_display.len() > 80 {
        let end = if args_display.is_char_boundary(77) { 77 } else { args_display.floor_char_boundary(77) };
        format!("{}…", &args_display[..end])
    } else {
        args_display.to_string()
    };
    println!(
        "  {}  {}  {}",
        "◈".bright_cyan(),
        tool_name.cyan().bold(),
        display.dimmed()
    );
}

pub fn print_tool_call_with_model(tool_name: &str, args_display: &str, model: &str) {
    let short_model = model.trim_start_matches("models/")
        .split('-')
        .filter(|s| !s.is_empty())
        .take(3)
        .collect::<Vec<_>>()
        .join("-");
    let display = if args_display.len() > 70 {
        let end = if args_display.is_char_boundary(67) { 67 } else { args_display.floor_char_boundary(67) };
        format!("{}…", &args_display[..end])
    } else {
        args_display.to_string()
    };
    println!(
        "  {} [{}] {}  {}",
        "◈".bright_cyan(),
        short_model.bright_blue(),
        tool_name.cyan().bold(),
        display.dimmed()
    );
}

pub fn print_tool_result_ok(preview: &str) {
    let preview: String = preview
        .lines()
        .next()
        .unwrap_or("")
        .chars()
        .take(100)
        .collect();
    if preview.is_empty() {
        println!("  {}  {}", "└".bright_black(), "done".green().dimmed());
    } else {
        println!("  {}  {}", "└".bright_black(), preview.dimmed());
    }
}

pub fn print_tool_result_err(err: &str) {
    let preview: String = err.chars().take(100).collect();
    println!("  {}  {}", "✗".red().bold(), preview.red().dimmed());
}

// ── Response prefix / prompt ──────────────────────────────────────────────────

pub fn print_assistant_prefix() {
    println!();
    print!("{} ", "FORGE".bright_blue().bold());
}

pub fn user_prompt_str() -> String {
    format!("{} ", "▸".bright_green().bold())
}

// ── Error / info ──────────────────────────────────────────────────────────────

pub fn print_error(msg: &str) {
    eprintln!("\n  {}  {}", "ERR".red().bold(), msg);
}

// ── Token usage ───────────────────────────────────────────────────────────────

pub fn print_token_usage(prompt: u32, completion: u32, total: u32, thoughts: u32) {
    let parts = if thoughts > 0 {
        format!(
            "{}p  {}c  {}t  =  {} tokens",
            fmt_tokens(prompt),
            fmt_tokens(completion),
            fmt_tokens(thoughts),
            fmt_tokens(total)
        )
    } else {
        format!(
            "{}p  {}c  =  {} tokens",
            fmt_tokens(prompt),
            fmt_tokens(completion),
            fmt_tokens(total)
        )
    };
    println!("  {}  {}", "◦".dimmed(), parts.dimmed());
}

fn fmt_tokens(n: u32) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{}K", n / 1_000)
    } else {
        n.to_string()
    }
}

// ── Context bar ───────────────────────────────────────────────────────────────

pub fn print_context_bar(used_tokens: u32, window_size: u32) {
    if used_tokens == 0 || window_size == 0 {
        return;
    }
    let pct = (used_tokens as f32 / window_size as f32).min(1.0);
    let filled = (pct * 28.0) as usize;
    let empty  = 28usize.saturating_sub(filled);
    let bar_f  = "█".repeat(filled);
    let bar_e  = "░".repeat(empty);
    let pct_s  = format!("{:.0}%", pct * 100.0);
    let win_s  = format!("{}M", window_size / 1_000_000);

    if pct >= 0.90 {
        println!(
            "  {}  {}{}  {}  [{}]",
            "CTX".bright_black(),
            bar_f.red(),
            bar_e.bright_black(),
            pct_s.red(),
            win_s.dimmed()
        );
    } else if pct >= 0.75 {
        println!(
            "  {}  {}{}  {}  [{}]",
            "CTX".bright_black(),
            bar_f.yellow(),
            bar_e.bright_black(),
            pct_s.yellow(),
            win_s.dimmed()
        );
    } else {
        println!(
            "  {}  {}{}  {}  [{}]",
            "CTX".bright_black(),
            bar_f.green(),
            bar_e.bright_black(),
            pct_s.green(),
            win_s.dimmed()
        );
    }
}

pub fn print_context_warning(pct: f32) {
    if pct >= 0.90 {
        println!(
            "\n  {}  Context at {:.0}% — run {} immediately",
            "▲ WARN".red().bold(),
            pct * 100.0,
            "/compact".yellow()
        );
    } else if pct >= 0.75 {
        println!(
            "\n  {}  Context at {:.0}% — consider {} soon",
            "▲".yellow(),
            pct * 100.0,
            "/compact".yellow()
        );
    }
}

// ── Help screen ───────────────────────────────────────────────────────────────

/// All three `live_*` slices are optional — pass `None` to fall back to built-in lists.
pub fn print_help(
    live_gemini:  Option<&[(String, String)]>,
    live_claude:  Option<&[(String, String)]>,
    live_openai:  Option<&[(String, String)]>,
) {
    fn section(title: &str) {
        // "  ── TITLE ──...──" filling to col 80
        // 2 (indent) + 2 (──) + 1 (sp) + title + 1 (sp) + dashes = 80
        let dashes = "─".repeat(74usize.saturating_sub(title.len()));
        println!();
        println!(
            "  {} {} {}",
            "──".bright_blue(),
            title.bright_white().bold(),
            dashes.bright_blue()
        );
        println!();
    }

    section("COMMANDS");

    let cmds: &[(&str, &str)] = &[
        ("/quit  /exit  /q",              "exit FORGE"),
        ("/clear",                         "clear conversation history"),
        ("",                               ""),
        ("/model <name|auto|list|info>",   "switch model, list all providers, or view info"),
        ("/models",                        "fetch live model list from Gemini API"),
        ("/think [on|off|budget=N]",       "toggle extended reasoning mode"),
        ("/web",                           "toggle Google Search grounding"),
        ("/apply [on|off]",                "toggle auto-apply (skip diff review)"),
        ("/explain [on|off]",              "preview planned actions before executing"),
        ("",                               ""),
        ("/task <requirement>",            "research → decompose → dispatch → consensus"),
        ("/test-fix <cmd> [cycles]",       "test → fix → retest loop"),
        ("/pr <title>",                    "push branch and open GitHub PR"),
        ("/compact",                       "summarize history to reclaim context"),
        ("/undo [N]",                      "revert last N file changes"),
        ("/snapshot [label]",              "create git stash snapshot"),
        ("/rollback",                      "restore from last git stash"),
        ("/diff",                          "list pending file snapshots"),
        ("",                               ""),
        ("/memorize <fact>",               "save fact to persistent memory"),
        ("/forget <keyword>",              "remove matching memory entries"),
        ("/memory",                        "view all memorized facts"),
        ("/learnings",                     "view auto-learned error patterns"),
        ("/dna",                           "show auto-detected project conventions"),
        ("",                               ""),
        ("/load [path]",                   "load project files into context"),
        ("/learn <git-url>",               "clone and load any OSS repo"),
        ("/tokens",                        "show context window usage"),
        ("/cost",                          "show session token costs and daily budget"),
        ("",                               ""),
        ("/session save|load|list|del",    "manage named sessions"),
        ("/save [file]",                   "export conversation as Markdown"),
        ("/history [N]",                   "show last N turns"),
        ("",                               ""),
        ("/security",                      "security sweep with CVE scan"),
        ("/audit [N]",                     "show last N actions from audit log"),
        ("/profile <name>",                "apply config profile"),
        ("/screenshot <path>",             "vision analysis — find and fix bugs"),
        ("/debug",                         "toggle debug output"),
        ("/cd <dir>",                      "change working directory"),
        ("/help",                          "show this help"),
    ];

    for (cmd, desc) in cmds {
        if cmd.is_empty() {
            println!();
        } else {
            println!("  {:<42} {}", cmd.yellow(), desc.dimmed());
        }
    }

    section("MODELS");

    println!("  {}  {}:", "▸".bright_blue(), "Gemini".bright_white().bold());
    if let Some(models) = live_gemini {
        // Real-time list from the API
        for (name, desc) in models {
            println!("      {:<34} {}", name.as_str().cyan(), desc.as_str().dimmed());
        }
    } else {
        // Fallback when offline or key unavailable
        let gemini: &[(&str, &str)] = &[
            ("gemini-3.1-pro",         "latest — 80.6% SWE-bench"),
            ("gemini-3-flash",         "latest fast model"),
            ("gemini-2.5-pro",         "deep reasoning, 1M context"),
            ("gemini-2.5-flash",       "fast, recommended"),
            ("gemini-2.5-flash-lite",  "cheapest — $0.10/M input"),
        ];
        for (m, d) in gemini {
            println!("      {:<34} {}", m.cyan(), d.dimmed());
        }
    }

    println!();
    println!("  {}  {}:", "▸".bright_blue(), "Claude".bright_white().bold());
    if let Some(models) = live_claude {
        for (name, desc) in models {
            println!("      {:<34} {}", name.as_str().cyan(), desc.as_str().dimmed());
        }
    } else {
        let claude: &[(&str, &str)] = &[
            ("claude-4-opus",     "200K ctx  max reasoning"),
            ("claude-4-sonnet",   "200K ctx  balanced"),
            ("claude-3.5-sonnet", "200K ctx  fast, capable"),
        ];
        for (m, d) in claude {
            println!("      {:<34} {}", m.cyan(), d.dimmed());
        }
    }

    println!();
    println!("  {}  {}:", "▸".bright_blue(), "OpenAI".bright_white().bold());
    if let Some(models) = live_openai {
        for (name, desc) in models {
            println!("      {:<34} {}", name.as_str().cyan(), desc.as_str().dimmed());
        }
    } else {
        let openai: &[(&str, &str)] = &[
            ("gpt-4.1",  "1M ctx  strong code generation"),
            ("gpt-4o",   "128K ctx  fast, multimodal"),
            ("o3",       "200K ctx  advanced reasoning"),
            ("o4-mini",  "200K ctx  reasoning, cost-efficient"),
        ];
        for (m, d) in openai {
            println!("      {:<34} {}", m.cyan(), d.dimmed());
        }
    }

    println!();
    println!(
        "  {}  {}",
        "Tip".dimmed(),
        "/model auto — route each task to the best available model".dimmed()
    );

    section("SAFETY");

    let levels: &[(&str, &str, &str)] = &[
        ("ALLOW  ", "green",       "safe commands run silently"),
        ("WARN   ", "yellow",      "side-effect commands print a warning"),
        ("CONFIRM", "bright_red",  "destructive commands require confirmation"),
        ("DENY   ", "red",         "catastrophic commands blocked outright"),
    ];
    for (label, color, desc) in levels {
        let colored_label = match *color {
            "green"      => label.green().bold().to_string(),
            "yellow"     => label.yellow().bold().to_string(),
            "bright_red" => label.bright_red().bold().to_string(),
            _            => label.red().bold().to_string(),
        };
        println!("  {}   {}", colored_label, desc.dimmed());
    }

    section("TOOLS");

    println!("  {}", "File system:".bright_white());
    println!("    {}", "read_file  write_file  edit_file  append_file  delete_file  copy_file  move_file".cyan());
    println!();
    println!("  {}", "Search:".bright_white());
    println!("    {}", "list_files  glob  search_files  create_directory".cyan());
    println!();
    println!("  {}", "Shell:".bright_white());
    println!("    {}", "bash  (streaming output, safety-checked, timeout up to 600s)".cyan());
    println!();
    println!("  {}", "Web:".bright_white());
    println!("    {}", "url_fetch  (HTML → plain text, 1-hour cache, 128KB max)".cyan());
    println!();
    println!("  {}", "Git:".bright_white());
    println!("    {}", "git_snapshot  (creates stash for /rollback)".cyan());
    println!();
    println!(
        "  {}",
        "edit_file uses fuzzy (whitespace-agnostic) matching; occurrence=N for duplicates."
            .dimmed()
    );

    section("CONFIG");

    println!("  {:<42} {}", "~/.forge/config.toml".cyan(), "global settings and API keys".dimmed());
    println!("  {:<42} {}", ".forge/project.md".cyan(),    "per-project instructions (auto-loaded)".dimmed());
    println!("  {:<42} {}", ".forge/safety.toml".cyan(),   "per-project permission policy overrides".dimmed());
    println!("  {:<42} {}", ".forge/memory.md".cyan(),     "persistent facts written by /memorize".dimmed());
    println!();
    println!("  {}", "auto_apply = false          # ask before overwriting files".bright_black());
    println!("  {}", "max_iterations = 50         # pause when tool loop exceeds limit".bright_black());
    println!("  {}", "context_warn = 0.75         # warn when context reaches 75%".bright_black());
    println!("  {}", "daily_budget_usd = 5.00     # alert when session cost exceeds".bright_black());
    println!();
    println!(
        "  {}",
        "╚══════════════════════════════════════════════════════════════════════════════╝"
            .bright_blue()
    );
    println!();
}
