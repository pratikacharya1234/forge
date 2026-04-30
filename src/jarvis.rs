// JARVIS — Real-time voice conversation mode for FORGE.
// Voice → Gemini transcribe (free) → FORGE agent (any model) → spd-say TTS
// Persistent memory. Continuous conversation. Speaks back.

use anyhow::Result;
use colored::Colorize;
use std::io::Write;

/// Run the JARVIS voice conversation loop.
pub async fn jarvis_loop(config: &crate::config::Config) -> Result<()> {
    println!();
    println!("  ╔══════════════════════════════════════════════╗");
    println!("  ║         🧠  JARVIS MODE ACTIVE               ║");
    println!("  ║                                              ║");
    println!("  ║  Voice → Gemini (free) → {} → 🔊          ║", 
        model_label(config));
    println!("  ║  [ENTER] speak  [s] type  [q] quit           ║");
    println!("  ╚══════════════════════════════════════════════╝");
    println!();

    // Load memory
    let mem_path = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".forge")
        .join("jarvis-memory.md");
    
    let mut memory: Vec<String> = if mem_path.exists() {
        std::fs::read_to_string(&mem_path)
            .unwrap_or_default()
            .lines()
            .map(|l| l.to_string())
            .filter(|l| !l.trim().is_empty())
            .collect()
    } else {
        Vec::new()
    };

    if !memory.is_empty() {
        println!("  {} Loaded {} past exchanges", "🧠".dimmed(), memory.len() / 2);
        println!();
    }

    // Check audio availability
    let has_audio = crate::voice::check_audio();
    if !has_audio {
        println!("  {} No mic found — text mode only", "⚠️ ".yellow());
        println!("    Install: apt install pulseaudio-utils");
        println!();
    }

    let mut voice_failed = !has_audio;
    let mut consecutive_failures = 0u32;

    loop {
        print!("  {} ", if voice_failed { "⌨️ ".dimmed() } else { "🎤".bright_red() });
        let _ = std::io::stdout().flush();
        
        let mut mode = String::new();
        std::io::stdin().read_line(&mut mode)?;
        let mode = mode.trim().to_lowercase();

        if mode == "q" || mode == "quit" || mode == "exit" {
            println!();
            println!("  {} JARVIS signing off. Memory saved.", "👋".cyan());
            save_memory(&mem_path, &memory);
            return Ok(());
        }

        if mode == "s" || mode == "type" {
            voice_failed = true; // Switch to text permanently
        }

        // Get the user message
        let user_message = if !voice_failed {
            match crate::voice::record_and_transcribe(&config.api_key, 6).await {
                Ok(text) => {
                    consecutive_failures = 0;
                    text
                }
                Err(e) => {
                    consecutive_failures += 1;
                    if consecutive_failures >= 2 {
                        println!("  {} Voice unavailable — switching to text mode", "⚠️".yellow());
                        voice_failed = true;
                    } else {
                        println!("  {} {}", "❌".red(), e.to_string().red());
                    }
                    continue;
                }
            }
        } else {
            // Text mode
            print!("  {} You: ", "💬".dimmed());
            let _ = std::io::stdout().flush();
            let mut text = String::new();
            std::io::stdin().read_line(&mut text)?;
            let text = text.trim().to_string();
            if text.is_empty() { continue; }
            text
        };

        if user_message.is_empty() { continue; }

        // Remember this
        memory.push(format!("User: {}", user_message));

        // Build context prompt
        let context_prompt = if memory.len() > 2 {
            let recent: String = memory.iter()
                .rev().take(6).collect::<Vec<_>>()
                .iter().rev().map(|s| s.as_str())
                .collect::<Vec<_>>().join("\n");
            format!(
                "You are JARVIS — FORGE's voice assistant. Be concise, helpful, conversational. 1-3 sentences max.\n\
                 Recent conversation:\n{}\n\nUser: {}", recent, user_message
            )
        } else {
            format!("You are JARVIS. Be concise, conversational, 1-3 sentences.\nUser: {}", user_message)
        };

        // Run through FORGE
        let response = match crate::agent::run_jarvis_query(config, &context_prompt).await {
            Ok(text) => text,
            Err(e) => format!("Sorry, something went wrong: {}", e),
        };

        memory.push(format!("FORGE: {}", response));

        // Speak it
        speak(&response);

        println!();
        println!("  {} {}", "🧠".bright_blue(), response.bright_white());
        println!();
    }
}

fn model_label(config: &crate::config::Config) -> String {
    if config.model.contains("claude") {
        "Claude".purple().bold().to_string()
    } else if config.model.contains("gpt") || config.model.contains("o3") || config.model.contains("o4") {
        "GPT".bright_yellow().bold().to_string()
    } else {
        "Gemini".green().bold().to_string()
    }
}

fn speak(text: &str) {
    let clean = text
        .replace('`', "").replace('*', "").replace('#', "")
        .replace("```", "").replace("___", "").trim().to_string();
    if clean.is_empty() { return; }
    let _ = std::process::Command::new("spd-say")
        .args(["-e", "-r", "0", &clean])
        .spawn();
}

fn save_memory(path: &std::path::Path, memory: &[String]) {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(path, memory.join("\n"));
}
