// NULLVOID Terminal Theme for FORGE
// Phantom-Hacker Aesthetic — "Wraith-Core Engine"
// Raw ANSI, no emoji, pure Unicode geometric glyphs.
// Integrated with FORGE agent loop.

#![allow(dead_code)]

use std::io::{self, Write};

// ═══════════════════ ANSI primitives ═══════════════════

pub const RESET: &str = "\x1b[0m";
pub const BOLD: &str = "\x1b[1m";
pub const DIM: &str = "\x1b[2m";
const CLRLN: &str = "\x1b[2K";

fn cursor_up(n: usize) -> String { format!("\x1b[{}A", n) }
fn cursor_col_1() -> &'static str { "\x1b[1G" }

// ═══════════════════ NULLVOID Palette ═══════════════════

pub const MINT: &str = "\x1b[38;2;61;255;154m";
pub const FIRE: &str = "\x1b[38;2;255;92;26m";
pub const PLASMA: &str = "\x1b[38;2;0;212;255m";
pub const VIOLET: &str = "\x1b[38;2;123;81;255m";
pub const AMBER: &str = "\x1b[38;2;255;184;108m";
pub const BRIGHT: &str = "\x1b[38;2;205;214;244m";
pub const TEXT: &str = "\x1b[38;2;136;146;176m";
pub const MUTED: &str = "\x1b[38;2;58;64;96m";
pub const GHOST: &str = "\x1b[38;2;18;24;52m";

// ═══════════════════ Icons (Unicode, no emoji) ═══════════

pub const I_MARK: &str = "◈";
pub const I_TARGET: &str = "⌖";
pub const I_PROC: &str = "⎔";
pub const I_GRID: &str = "⊞";
pub const I_BRANCH: &str = "⟟";
pub const I_PROMPT: &str = "⊢";
pub const I_OUT: &str = "◆";
pub const I_STREAM: &str = "∷";
pub const I_WARN: &str = "⌬";
pub const I_ACTIVE: &str = "⊛";
pub const I_ERROR: &str = "⊗";
pub const I_ADD: &str = "⊕";
pub const I_LIVE: &str = "◉";
pub const I_RULE: &str = "─";
pub const I_DOT: &str = "·";

pub const SPINNER: [&str; 4] = ["◐", "◓", "◑", "◒"];

// ═══════════════════ FORGE Ghost Logo ═══════════════════

const FORGE_GHOST: [&str; 6] = [
    " ░███████░ ░██████░ ░██████░ ░██████░ ░███████░",
    " ░██░░░░░ ░██░░██░ ░██░░██░ ░██░░██░ ░██░░░░░░",
    " ░█████░ ░██░░██░ ░██████░ ░██░░██░ ░█████░░░",
    " ░██░░░░ ░██░░██░ ░██░░██░ ░██░░██░ ░██░░░░░░",
    " ░██░ ░████░ ░██░░██░ ░████░ ░███████░",
    " ",
];

type Segment = (&'static str, &'static str);

fn forge_lines() -> [[Segment; 5]; 6] {
    [
        [(FIRE, "███████╗"), (PLASMA, " ██████╗ "), (BRIGHT, "██████╗ "), (VIOLET, " ██████╗ "), (MINT, "███████╗")],
        [(FIRE, "██╔════╝"), (PLASMA, "██╔═══██╗"), (BRIGHT, "██╔══██╗ "), (VIOLET, "██╔════╝ "), (MINT, "██╔════╝")],
        [(FIRE, "█████╗ "), (PLASMA, "██║ ██║"), (BRIGHT, "██████╔╝ "), (VIOLET, "██║ ███╗"), (MINT, "█████╗ ")],
        [(FIRE, "██╔══╝ "), (PLASMA, "██║ ██║"), (BRIGHT, "██╔══██╗ "), (VIOLET, "██║ ██║"), (MINT, "██╔══╝ ")],
        [(FIRE, "██║ "), (PLASMA, "╚██████╔╝"), (BRIGHT, "██║ ██║ "), (VIOLET, "╚██████╔╝"), (MINT, "███████╗")],
        [(MUTED, "╚═╝ "), (PLASMA, " ╚═════╝ "), (BRIGHT, "╚═╝ ╚═╝ "), (VIOLET, " ╚═════╝ "), (MUTED, "╚══════╝")],
    ]
}

// ═══════════════════ Public API ═══════════════════

pub fn print_forge_logo() {
    let stdout = io::stdout();
    let mut w = io::BufWriter::new(stdout.lock());
    for line in &FORGE_GHOST {
        writeln!(w, "{GHOST}{DIM} {line}{RESET}").unwrap();
    }
    write!(w, "{}", cursor_up(FORGE_GHOST.len())).unwrap();
    for segments in &forge_lines() {
        write!(w, "{CLRLN}{cursor_col_1} ", cursor_col_1 = cursor_col_1()).unwrap();
        for (color, text) in segments { write!(w, "{color}{text}").unwrap(); }
        writeln!(w, "{RESET}").unwrap();
    }
    w.flush().unwrap();
}

pub fn print_banner(tool_count: usize, context_tokens: u32, model: &str) {
    let stdout = io::stdout();
    let mut w = io::BufWriter::new(stdout.lock());
    writeln!(w).unwrap();

    // Logo
    drop(w);
    print_forge_logo();
    let stdout = io::stdout();
    let mut w = io::BufWriter::new(stdout.lock());

    let ctx_str = if context_tokens >= 1_000_000 {
        format!("{}M Context", context_tokens / 1_000_000)
    } else {
        format!("{}K Context", context_tokens / 1_000)
    };

    writeln!(w, " {MUTED}{I_STREAM}{I_STREAM}{I_STREAM} {TEXT}NULLVOID ENGINE {MUTED}{I_MARK} {TEXT}Multi-Model Terminal Agent {MUTED}{I_MARK} {TEXT}v0.0.1 {MUTED}{I_STREAM}{I_STREAM}{I_STREAM}{RESET}").unwrap();
    writeln!(w).unwrap();
    thin_rule_w(&mut w);
    writeln!(w, " {MINT}{I_MARK}{RESET} Safety{MINT}:ON{RESET} {FIRE}{I_PROC}{RESET} {tool_count} Tools {PLASMA}{I_GRID}{RESET} {ctx_str} {VIOLET}{I_TARGET}{RESET} Multi-Model {AMBER}{I_BRANCH}{RESET} .forge/project.md").unwrap();
    thin_rule_w(&mut w);
    writeln!(w, " {MUTED}{I_PROMPT} {TEXT}/help {MUTED}{I_DOT} {TEXT}/think {MUTED}{I_DOT} {TEXT}/web {MUTED}{I_DOT} {TEXT}/model {MUTED}{I_DOT} {TEXT}/task {MUTED}{I_DOT} {TEXT}/undo {MUTED}{I_DOT} {TEXT}/compact {MUTED}{I_DOT} {TEXT}/session {MUTED}{I_DOT} {TEXT}/security {MUTED}{I_DOT} {TEXT}/quit{RESET}").unwrap();
    writeln!(w).unwrap();
    w.flush().unwrap();
}

pub fn print_model_detect(model: &str, provider: &str, count: usize) {
    let short = model.trim_start_matches("models/").split('-').take(3).collect::<Vec<_>>().join("-");
    println!(" {I_WARN} Auto-detected: {AMBER}{short}{TEXT} ({provider}, {count} models){RESET}");
}

pub fn print_project_loaded(path: &str) {
    println!("\n {MINT}{I_ADD}{RESET} Loaded project instructions from {AMBER}{path}{RESET}");
}

pub fn print_mode_selector() -> bool {
    println!("\n {MINT}{I_LIVE}{RESET} {BRIGHT}Mic detected{RESET} {MUTED}—{RESET} {TEXT}Voice mode or Text mode?{RESET}");
    println!(" {MUTED}[{MINT}1{MUTED}] {BRIGHT}Voice (EMBER){RESET} {MUTED}| [{MINT}2{MUTED}] {BRIGHT}Text (terminal){RESET} {MUTED}| [Enter] = Voice{RESET}");
    print!(" {PLASMA}{I_PROMPT}{RESET} ");
    io::stdout().flush().unwrap();
    let mut buf = String::new();
    io::stdin().read_line(&mut buf).unwrap();
    buf.trim() == "2"
}

pub fn input_prompt_str() -> String {
    format!(" {PLASMA}{I_PROMPT} {MUTED}╰─ {MINT}>{RESET} ")
}

pub fn print_thinking(model: &str) {
    let short = model.trim_start_matches("models/").split('-').take(3).collect::<Vec<_>>().join("-");
    println!("\n {VIOLET}{I_OUT} REASONING {MUTED}[{AMBER}{short}{MUTED}]{TEXT} {I_STREAM}{I_STREAM}{I_STREAM}{RESET}");
}

pub fn print_response_header() {
    println!();
    println!(" {MUTED}╭── {PLASMA}{}{RESET}", I_RULE.repeat(44));
    println!(" {MUTED}│ {VIOLET}{I_OUT}{BRIGHT} OUTPUT{RESET}");
    println!(" {PLASMA}├── {MUTED}{}{RESET}", I_RULE.repeat(44));
}

pub fn print_response_body(text: &str) {
    for line in text.lines() {
        if line.is_empty() {
            println!(" {MUTED}│{RESET}");
        } else {
            println!(" {MUTED}│ {TEXT}{line}{RESET}");
        }
    }
    println!(" {MUTED}╰{}{RESET}", I_RULE.repeat(47));
}

pub fn print_thinking_line(line: &str) {
    println!(" {MUTED}│ {AMBER}{line}{RESET}");
}

pub fn print_thinking_close() {
    println!(" {MUTED}╰{}{RESET}", I_RULE.repeat(47));
}

pub fn print_token_stats(prompt_k: u32, completion: u32, thinking: u32, session_usd: f64, _input_total: u32, _output_total: u32, turn: u32) {
    let total = prompt_k * 1000 + completion + thinking;
    println!(" {MUTED}{I_ACTIVE} {TEXT}{prompt_k}Kp {completion}c {thinking}t {MUTED}{I_STREAM} {TEXT}{}K tokens{RESET}", total / 1000);
    println!(" {MUTED}{I_WARN} Session: {AMBER}${session_usd:.4} {MUTED}{I_STREAM} {TEXT}Turns: {turn}{RESET}");
}

pub fn print_tool_call(tool: &str, args_summary: &str) {
    println!(" {AMBER}{I_PROC}{RESET} {BRIGHT}{tool}{RESET} {MUTED}{args_summary}{RESET}");
}

pub fn print_tool_result(success: bool, summary: &str) {
    let (icon, color) = if success { (I_ADD, MINT) } else { (I_ERROR, FIRE) };
    println!(" {color}{icon}{RESET} {TEXT}{summary}{RESET}");
}

pub fn print_error(msg: &str) {
    eprintln!("\n {FIRE}{I_ERROR} ERROR{RESET} {TEXT}{msg}{RESET}");
}

pub fn print_warning(msg: &str) {
    println!(" {AMBER}{I_WARN} {TEXT}{msg}{RESET}");
}

pub fn print_info(msg: &str) {
    println!(" {PLASMA}{I_MARK} {TEXT}{msg}{RESET}");
}

pub fn print_input_prompt() { print!(" {PLASMA}{I_PROMPT} {MUTED}╰─ {MINT}>{RESET} "); }
pub fn print_user_echo(text: &str) { println!("\n {MUTED}{I_PROMPT} ╰─ {BRIGHT}>{RESET} {BRIGHT}{text}{RESET}"); }
pub fn print_session_summary(turns: u32, total_in: u32, total_out: u32, cost_usd: f64) {
    println!();
    thin_rule_stdout();
    println!(" {VIOLET}{I_MARK} SESSION SUMMARY{RESET}");
    println!(" {MUTED}{I_STREAM} Turns: {BRIGHT}{turns}{RESET}");
    println!(" {MUTED}{I_STREAM} Input: {BRIGHT}{total_in}K tokens{RESET}");
    println!(" {MUTED}{I_STREAM} Output: {BRIGHT}{total_out}K tokens{RESET}");
    println!(" {MUTED}{I_STREAM} Cost: {AMBER}${cost_usd:.4}{RESET}");
    thin_rule_stdout();
    println!();
}
pub fn print_security_status(safety_on: bool) {
    let (icon, color, label) = if safety_on { (I_LIVE, MINT, "ENABLED") } else { (I_ERROR, FIRE, "DISABLED — CAUTION") };
    println!("\n {color}{icon} Safety Policy: {BRIGHT}{label}{RESET}");
}
pub fn print_quit(session_usd: f64) {
    println!();
    thin_rule_stdout();
    println!(" {MUTED}{I_MARK} {TEXT}Session closed {MUTED}{I_STREAM} {AMBER}Total: ${session_usd:.4}{RESET}");
    thin_rule_stdout();
    println!();
}

fn thin_rule_w(w: &mut impl Write) { writeln!(w, " {MUTED}{}{RESET}", I_RULE.repeat(68)).unwrap(); }
fn thin_rule_stdout() { println!(" {MUTED}{}{RESET}", I_RULE.repeat(68)); }
