// JARVIS — Real-time voice conversation mode for FORGE.
// Voice → Gemini transcribe (free) → FORGE agent (any model) → spd-say TTS
// Persistent memory. Continuous conversation. Speaks back.

use anyhow::{Context, Result};
use colored::Colorize;
use std::io::Write;
use crate::voice;
use crate::agent;

/// Run the JARVIS voice conversation loop.
/// Press Enter to speak, type 'q' to quit, 's' to skip voice and type.
pub async fn jarvis_loop(config: &crate::config::Config) -> Result<()> {
    println!();
    println!("  ╔══════════════════════════════════════════════╗");
    println!("  ║         🧠  JARVIS MODE ACTIVE               ║");
    println!("  ║                                              ║");
    println!("  ║  Voice → Gemini (free) → {} → 🔊          ║", 
        if config.model.contains("claude") { "Claude".purple() } 
        else if config.model.contains("gpt") { "GPT".bright_yellow() } 
        else { "Gemini".green() }.bold());
    println!("  ║  Press [ENTER] to speak  ·  [s] type  ·  [q] quit ║");
    println!("  ╚══════════════════════════════════════════════╝");
    println!();

    // Load conversation memory
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
        println!("  {} Loaded {} past conversations", "🧠".dimmed(), memory.len() / 2);
        println!();
    }

    loop {
        // Wait for input mode
        print!("  {} ", "🎤".bright_red());
        let _ = std::io::stdout().flush();
        
        let mut mode = String::new();
        std::io::stdin().read_line(&mut mode)?;
        let mode = mode.trim().to_lowercase();

        if mode == "q" || mode == "quit" || mode == "exit" {
            println!();
            println!("  {} JARVIS signing off. Memory saved.", "👋".cyan());
            // Save memory
            if let Some(parent) = mem_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::write(&mem_path, memory.join("\n"));
            return Ok(());
        }

        // Get the user's message
        let user_message = if mode == "s" || mode == "type" {
            // Text mode
            print!("  {} You: ", "⌨️ ".dimmed());
            let _ = std::io::stdout().flush();
            let mut text = String::new();
            std::io::stdin().read_line(&mut text)?;
            let text = text.trim().to_string();
            if text.is_empty() { continue; }
            text
        } else {
            // Voice mode (default — Enter key starts recording)
            match voice::record_and_transcribe(&config.api_key, 8).await {
                Ok(text) => text,
                Err(e) => {
                    println!("  {} Voice failed: {} — type instead:", "❌".red(), e.to_string().red());
                    print!("  {} You: ", "⌨️ ".dimmed());
                    let _ = std::io::stdout().flush();
                    let mut text = String::new();
                    std::io::stdin().read_line(&mut text)?;
                    text.trim().to_string()
                }
            }
        };

        if user_message.is_empty() { continue; }

        // Remember this interaction
        memory.push(format!("User: {}", user_message));

        // Build context-aware prompt with memory
        let context_prompt = if memory.len() > 2 {
            let recent: String = memory.iter()
                .rev()
                .take(6) // last 3 exchanges
                .collect::<Vec<_>>()
                .iter()
                .rev()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join("\n");
            format!(
                "You are JARVIS — FORGE's voice assistant. Be concise and conversational. Respond in 1-3 sentences.\n\
                 Recent conversation:\n{}\n\n\
                 User said: {}", recent, user_message
            )
        } else {
            format!(
                "You are JARVIS — FORGE's voice assistant. Be concise and conversational. Respond in 1-3 sentences.\n\
                 User said: {}", user_message
            )
        };

        // Run through FORGE agent
        let response = match crate::agent::run_jarvis_query(config, &context_prompt).await {
            Ok(text) => text,
            Err(e) => format!("Sorry, I had trouble: {}", e),
        };

        memory.push(format!("FORGE: {}", response));

        // Speak the response
        speak(&response);

        println!();
        println!("  {} {}", "🧠".bright_blue(), response.bright_white());
        println!();
    }
}

/// Speak text using speech-dispatcher (spd-say) with a natural voice.
fn speak(text: &str) {
    // Clean text for speech — remove markdown, code blocks, etc.
    let clean = text
        .replace('`', "")
        .replace('*', "")
        .replace('#', "")
        .replace("```", "")
        .replace("___", "")
        .trim()
        .to_string();

    if clean.is_empty() { return; }

    // Use spd-say in background so FORGE doesn't wait
    let _ = std::process::Command::new("spd-say")
        .args(["-e", "-r", "0", &clean]) // -e = espeak, -r 0 = normal rate
        .spawn();
}

/// Speak a notification sound (confirm recording started)
pub fn beep() {
    let _ = std::process::Command::new("spd-say")
        .args(["-e", "-t", "female2", "listening"])
        .spawn();
}
