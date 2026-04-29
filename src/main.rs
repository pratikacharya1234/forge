use anyhow::Result;
use clap::Parser;

mod agent;
mod audit;
mod backend;
mod config;
mod diff_view;
mod types;
mod integrations;
mod mcp;
mod models;
mod orchestrator;
mod project;
mod safety;
mod security;
mod session;
mod snapshot;
mod token_counter;
mod tools;
mod ui;

#[derive(Parser, Debug)]
#[clap(
    name    = "forge",
    about   = "FORGE — Multi-model terminal AI coding agent",
    version = "0.9.0",
    long_about = None
)]
struct Args {
    /// API key for Gemini backend (or set FORGE_API_KEY / GEMINI_API_KEY env var).
    #[clap(short = 'k', long, env = "FORGE_API_KEY")]
    api_key: Option<String>,

    /// Gemini model to use.
    #[clap(short, long, default_value = "gemini-2.5-flash")]
    model: String,

    /// Enable Google Search grounding.
    #[clap(short, long)]
    grounding: bool,

    /// Enable ThinkMode (gemini-2.5+ only).
    #[clap(short, long)]
    think: bool,

    /// ThinkMode token budget (default 8000, max 24576, 0 = unlimited).
    #[clap(long, default_value = "8000")]
    think_budget: i32,

    /// Auto-apply all file changes without diff preview.
    #[clap(long)]
    auto_apply: bool,

    /// Max tool-call iterations per turn before pausing (0 = unlimited).
    #[clap(long, default_value = "50")]
    max_iter: u32,

    /// Run a single prompt non-interactively and exit.
    #[clap(short, long)]
    prompt: Option<String>,

    /// Attach a screenshot to the initial prompt (ScreenFix).
    #[clap(long)]
    screenshot: Option<String>,

    /// Anthropic (Claude) API key.
    #[clap(long, env = "ANTHROPIC_API_KEY")]
    anthropic_api_key: Option<String>,

    /// OpenAI API key.
    #[clap(long, env = "OPENAI_API_KEY")]
    openai_api_key: Option<String>,

    /// Explain planned actions before executing tools.
    #[clap(long)]
    explain: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let file_cfg = config::Config::file_defaults();

    let api_key = args.api_key
        .or(file_cfg.api_key)
        .or_else(|| std::env::var("FORGE_API_KEY").ok())
        .or_else(|| std::env::var("GEMINI_API_KEY").ok())
        .ok_or_else(|| anyhow::anyhow!(
            "API key not found.\n\
             Set FORGE_API_KEY or use --api-key.\n\
             Free key: https://aistudio.google.com/apikey"
        ))?;

    let model = file_cfg.model.unwrap_or(args.model);

    let thinking = args.think || file_cfg.thinking;
    let budget   = if args.think { args.think_budget } else { file_cfg.thinking_budget };
    let auto_apply    = args.auto_apply  || file_cfg.auto_apply;
    let max_iterations = if args.max_iter != 50 { args.max_iter } else { file_cfg.max_iterations };

    let config = config::Config {
        api_key,
        model,
        grounding:       args.grounding || file_cfg.grounding,
        thinking,
        thinking_budget: budget,
        auto_apply,
        max_iterations:  if max_iterations == 0 { 0 } else { max_iterations.max(1) },
        context_warn:    file_cfg.context_warn,
        context_compact: file_cfg.context_compact,
        mcp_servers:     file_cfg.mcp_servers,
        integrations:    file_cfg.integrations,
        daily_budget_usd: file_cfg.daily_budget_usd,
        anthropic_api_key: args.anthropic_api_key.or(file_cfg.anthropic_api_key),
        openai_api_key: args.openai_api_key.or(file_cfg.openai_api_key),
        explain_before_execute: args.explain || file_cfg.explain_before_execute,
    };

    if let Some(prompt) = args.prompt {
        agent::run_once(&config, &prompt, args.screenshot.as_deref()).await?;
    } else {
        agent::run_interactive(&config).await?;
    }

    Ok(())
}
