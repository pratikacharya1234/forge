use colored::Colorize;

pub fn print_banner(grounding: bool, thinking: bool, auto_apply: bool) {
    let has_safety_toml = std::path::Path::new(".forge/safety.toml").exists();
    let width = 58;

    let border = "=".repeat(width).bright_blue();
    println!("\n  {}", border);
    println!(
        "  {}{:>width$}{}",
        "||".bright_blue(),
        "",
        "||".bright_blue(),
        width = width - 4
    );
    println!(
        "  {}  FORGE v0.9.0   {}{}",
        "||".bright_blue(),
        " ".repeat(width - 24),
        "||".bright_blue()
    );
    println!(
        "  {}  Multi-Model Terminal AI Coding Agent  {}{}",
        "||".bright_blue(),
        " ".repeat(width - 48),
        "||".bright_blue()
    );
    println!(
        "  {}{:>width$}{}",
        "||".bright_blue(),
        "",
        "||".bright_blue(),
        width = width - 4
    );
    println!("  {}", border);

    let mut flags = Vec::new();
    if grounding {
        flags.push("Grounding:ON".green().to_string());
    }
    if thinking {
        flags.push("Thinking:ON".yellow().to_string());
    }
    if auto_apply {
        flags.push("Auto-Apply:ON".cyan().to_string());
    }
    if has_safety_toml {
        flags.push("Custom-Safety".magenta().to_string());
    }
    flags.push("Safety:ON".red().to_string());

    println!("\n  {}  {}", "->".bright_blue(), flags.join("  "));
    println!(
        "  {}  Type a task. FORGE reads, writes, runs, and iterates until done.",
        "->".bright_blue().dimmed()
    );

    let cmds = [
        "/help", "/think", "/web", "/model", "/undo",
        "/compact", "/session", "/security", "/test-fix", "/quit",
    ];
    print!("  {}  ", "->".bright_blue().dimmed());
    for cmd in &cmds {
        print!("{}  ", cmd.yellow());
    }
    println!("\n");
}

pub fn print_thinking() {
    print!("\n  {} ", "THINKING".bright_yellow().bold());
    println!("{}", "FORGE is analyzing...".dimmed());
}

pub fn print_tool_call(tool_name: &str, args_display: &str) {
    let display_args = if args_display.len() > 100 {
        format!("{}...", &args_display[..97])
    } else {
        args_display.to_string()
    };
    println!(
        "  {}  {} {}",
        "TOOL".bright_cyan().bold(),
        tool_name.cyan(),
        display_args.dimmed()
    );
}

pub fn print_tool_result_ok(preview: &str) {
    let preview: String = preview
        .lines()
        .next()
        .unwrap_or("")
        .chars()
        .take(120)
        .collect();
    if preview.is_empty() {
        println!("  {}", "OK".green());
    } else {
        println!("  {} {}", "OK".green(), preview.dimmed());
    }
}

pub fn print_tool_result_err(err: &str) {
    let preview: String = err.chars().take(120).collect();
    println!("  {} {}", "ERR".red().bold(), preview.red().dimmed());
}

pub fn print_assistant_prefix() {
    println!();
    print!(
        "{} ",
        "FORGE:".bright_blue().bold()
    );
}

pub fn user_prompt_str() -> String {
    format!("{} ", "You:".bright_green().bold())
}

pub fn print_error(msg: &str) {
    eprintln!("\n{} {}", "ERROR:".red().bold(), msg);
}

pub fn print_token_usage(prompt: u32, completion: u32, total: u32, thoughts: u32) {
    let parts = if thoughts > 0 {
        format!(
            "{} prompt + {} output + {} thinking = {} tokens",
            fmt_tokens(prompt),
            fmt_tokens(completion),
            fmt_tokens(thoughts),
            fmt_tokens(total)
        )
    } else {
        format!(
            "{} prompt + {} output = {} tokens",
            fmt_tokens(prompt),
            fmt_tokens(completion),
            fmt_tokens(total)
        )
    };
    println!("  {} {}", "--".dimmed(), parts.dimmed());
}

fn fmt_tokens(n: u32) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1000 {
        format!("{}K", n / 1000)
    } else {
        n.to_string()
    }
}

pub fn print_context_bar(used_tokens: u32, window_size: u32) {
    if used_tokens == 0 || window_size == 0 {
        return;
    }
    let pct = (used_tokens as f32 / window_size as f32).min(1.0);
    let filled = (pct * 30.0) as usize;
    let bar: String = "#".repeat(filled) + &".".repeat(30 - filled);
    let pct_str = format!("{:.0}%", pct * 100.0);

    let colored_bar = if pct >= 0.90 {
        bar.red()
    } else if pct >= 0.75 {
        bar.yellow()
    } else {
        bar.green()
    };

    println!(
        "  Context: [{}] {}  ({}K window)",
        colored_bar,
        pct_str.dimmed(),
        window_size / 1000
    );
}

pub fn print_context_warning(pct: f32) {
    if pct >= 0.90 {
        println!(
            "\n  {} Context at {:.0}% -- run /compact to free space",
            "WARN".red().bold(),
            pct * 100.0
        );
    } else if pct >= 0.75 {
        println!(
            "\n  {} Context at {:.0}% -- consider /compact soon",
            "WARN".yellow(),
            pct * 100.0
        );
    }
}

pub fn print_help() {
    println!();
    println!(
        "{}",
        "----- COMMANDS --------------------------------------".bold()
    );

    let cmds: &[(&str, &str)] = &[
        // Core
        ("/quit  /exit",               "exit FORGE"),
        ("/clear",                     "clear conversation history"),
        ("",                            ""),
        // Model & Thinking
        ("/model <name|auto|list|info>", "switch model: auto-routing, list, info"),
        ("/models",                    "fetch available models from Gemini API"),
        ("/think [on|off|budget=N]",   "toggle thinking mode (gemini-2.5+)"),
        ("/web",                       "toggle Google Search grounding"),
        ("/apply [on|off]",            "toggle auto-apply (skip diff prompts)"),
        ("/explain [on|off]",          "show planned actions before executing"),
        ("",                            ""),
        // Code & Testing
        ("/task <requirement>",         "full pipeline: research → decompose → dispatch → consensus"),
        ("/test-fix <cmd> [cycles]",    "test -> fix -> retest loop"),
        ("/pr <title>",                "push branch and create GitHub PR"),
        ("/compact",                   "summarize history to save tokens"),
        ("/undo",                      "revert last file change"),
        ("/undo N",                    "revert last N file changes"),
        ("/snapshot",                  "create git stash snapshot"),
        ("/rollback",                  "restore from last git stash"),
        ("/diff",                      "show pending snapshot list"),
        ("",                            ""),
        // Memory
        ("/memorize <fact>",           "save fact to persistent memory"),
        ("/forget <keyword>",          "remove entries from memory"),
        ("/memory",                    "view all memorized facts"),
        ("",                            ""),
        // Context & Data
        ("/load [path]",               "load project into context"),
        ("/learn <git-url>",           "clone and load any OSS repo"),
        ("/tokens",                    "show context window usage"),
        ("/cost",                      "show session token costs and budget"),
        ("",                            ""),
        // Sessions
        ("/session save|load|list|del","manage saved sessions"),
        ("/save [file]",               "export session as Markdown"),
        ("/history [N]",               "show last N conversation turns"),
        ("",                            ""),
        // Safety & Debug
        ("/security",                  "security sweep with CVE scan"),
        ("/audit [N]",                 "show last N actions (default 10)"),
        ("/profile <name>",            "apply named config profile"),
        ("/screenshot <path>",         "vision analysis for bug finding"),
        ("/debug",                     "toggle debug output"),
        ("/cd <dir>",                  "change working directory"),
        ("/help",                      "show this help"),
    ];

    for (cmd, desc) in cmds {
        println!(
            "  {:<42} {}",
            cmd.yellow(),
            desc.dimmed()
        );
    }

    println!();
    println!(
        "{}",
        "----- MODELS ----------------------------------------".bold()
    );
    let models: &[(&str, &str)] = &[
        ("gemini-2.5-flash",       "fastest recommended  (default)"),
        ("gemini-2.5-flash-lite",  "cheapest at $0.10/M input"),
        ("gemini-2.5-pro",         "deep reasoning, 1M context"),
        ("gemini-2.0-flash",       "previous generation"),
        ("gemini-2.0-flash-lite",  "lightest model"),
    ];
    for (m, d) in models {
        println!("  {:<32} {}", m.cyan(), d.dimmed());
    }

    println!();
    println!(
        "{}",
        "----- SAFETY ----------------------------------------".bold()
    );
    println!(
        "  {}   safe commands run silently",
        "ALLOW  ".green().bold().to_string()
    );
    println!(
        "  {}   side-effect commands print warning",
        "WARN   ".yellow().bold().to_string()
    );
    println!(
        "  {}   destructive commands require confirmation",
        "CONFIRM".red().bold().to_string()
    );
    println!(
        "  {}    catastrophic commands blocked outright",
        "DENY   ".red().bold().to_string()
    );

    println!();
    println!(
        "{}",
        "----- TOOLS -----------------------------------------".bold()
    );
    println!("  read_file  write_file  edit_file(occurrence=N)  append_file");
    println!("  bash  list_files  search_files  glob  url_fetch");
    println!("  create_directory  delete_file  move_file  copy_file");
    println!("  git_snapshot");
    println!("  edit_file supports fuzzy matching (whitespace-agnostic)");

    println!();
    println!(
        "{}",
        "----- CONFIG ----------------------------------------".bold()
    );
    println!("  ~/.forge/config.toml  and  .forge/project.md (per-project)");
    println!("  .forge/safety.toml -- per-project permission policies");
    println!(
        "  {}",
        "auto_apply = false     # ask before overwriting files".dimmed()
    );
    println!(
        "  {}",
        "max_iterations = 50    # pause if loop exceeds".dimmed()
    );
    println!(
        "  {}",
        "context_warn = 0.75    # warn at context usage".dimmed()
    );
    println!();
}
