// JARVIS — Iron Man style real-time voice AI assistant.
// Auto-greet, voice-first, zero keyboard. Pure Rust audio via cpal.
// Memory persists. Speaks back via spd-say TTS.

use anyhow::Result;
use colored::Colorize;
use std::io::Write;

/// Check if a microphone is available for JARVIS mode.
pub fn mic_available() -> bool {
    crate::voice::check_audio()
}

pub async fn jarvis_loop(config: &crate::config::Config) -> Result<bool> {
    // Returns true if user wants to fall through to text mode
    // ── Greeting ───────────────────────────────────────────────────────────
    println!();
    println!("  ╔══════════════════════════════════════════════╗");
    println!("  ║                                              ║");
    println!("  ║         🧠  J.A.R.V.I.S.  ONLINE             ║");
    println!("  ║                                              ║");
    println!("  ║  {}    {}", "Voice-driven".bright_cyan().bold(), config.model.dimmed());
    println!("  ╚══════════════════════════════════════════════╝");
    println!();

    let has_mic = crate::voice::check_audio();

    if !has_mic {
        println!("  {} No mic detected. Starting text mode.", "⚠️ ".yellow());
        println!();
    } else {
        // Ask: voice or text?
        println!("  {} Mic detected — Voice mode or Text mode?", "🎤".bright_red());
        println!("  [1] Voice (JARVIS)  |  [2] Text (terminal)  |  [Enter] = Voice");
        print!("  > ");
        let _ = std::io::stdout().flush();
        let mut choice = String::new();
        let _ = std::io::stdin().read_line(&mut choice);
        if choice.trim() == "2" {
            return Ok(true); //
        }
        println!();
    }

    if !has_mic {
        return Ok(true); //
    }

    // Greet the user
    let greeting = format!(
        "Hello. I'm FORGE, your AI assistant. I see you're in the {} project. How can I help?",
        get_project_name()
    );
    println!("  {} {}", "🔊".bright_cyan(), greeting.bright_white());
    speak(&greeting);
    println!();
    println!("  {} Speak now — I'm listening...", "🎤".bright_red());
    println!("  {} Say \"quit\" or \"exit\" to stop | Hold for 4 seconds to speak", "  ".dimmed());
    println!();

    // ── Memory ─────────────────────────────────────────────────────────────
    let mem_path = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".forge")
        .join("jarvis-memory.md");
    
    let mut memory: Vec<String> = if mem_path.exists() {
        std::fs::read_to_string(&mem_path).unwrap_or_default()
            .lines().map(|l| l.to_string()).filter(|l| !l.trim().is_empty()).collect()
    } else { Vec::new() };

    // ── Voice loop ─────────────────────────────────────────────────────────
    let mut silence_count = 0u32;
    loop {
        // Listen — 4 second window
        let user_message = match crate::voice::listen_and_transcribe(&config.api_key, 4).await {
            Ok(text) => {
                silence_count = 0;
                text
            }
            Err(_) => {
                silence_count += 1;
                // Only print "listening" indicator every 5 failures
                if silence_count % 5 == 0 {
                    print!("  {} Still listening...\r", "🎙️".dimmed());
                    let _ = std::io::stdout().flush();
                }
                continue;
            }
        };

        if user_message.is_empty() { continue; }

        println!("  {} {}", "🗣️".cyan(), user_message.bright_white());

        // Check for exit commands
        let lower = user_message.to_lowercase();
        if lower.contains("quit") || lower.contains("exit") || lower.contains("goodbye") || lower.contains("bye") {
            let farewell = "Goodbye. Shutting down JARVIS.";
            speak(farewell);
            println!("  {} {}", "👋".cyan(), farewell.bright_white());
            save_memory(&mem_path, &memory);
            return Ok(false);
        }

        memory.push(format!("User: {}", user_message));

        // Build context
        let context = build_context(&memory, &user_message);

        // Process
        let response = match crate::agent::run_jarvis_query(config, &context).await {
            Ok(text) => text,
            Err(e) => format!("Sorry, I encountered an error: {}", e),
        };

        memory.push(format!("FORGE: {}", response));

        // Speak it
        speak(&response);

        println!("  {} {}", "🧠".bright_blue(), response.bright_white());
        println!();
    }
}

fn build_context(memory: &[String], user_msg: &str) -> String {
    if memory.len() <= 2 {
        return format!("You are JARVIS — FORGE's voice AI. Be concise, helpful, conversational. 1-3 sentences.\nUser: {}", user_msg);
    }
    let recent: String = memory.iter().rev().take(6)
        .collect::<Vec<_>>().iter().rev().map(|s| s.as_str())
        .collect::<Vec<_>>().join("\n");
    format!(
        "You are JARVIS — FORGE's voice AI. Be concise, conversational, 1-3 sentences.\nRecent:\n{}\n\nUser: {}", 
        recent, user_msg
    )
}

fn get_project_name() -> String {
    std::env::current_dir().ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        .unwrap_or_else(|| "current".to_string())
}

/// Text-to-speech via spd-say (speech-dispatcher).
fn speak(text: &str) {
    let clean = text.replace('`', "").replace('*', "").replace('#', "")
        .replace("```", "").replace("___", "").trim().to_string();
    if clean.is_empty() { return; }
    // Block until spoken — uses output() instead of spawn()
    let _ = std::process::Command::new("spd-say")
        .args(["-e", "-r", "0", &clean])
        .output();
}

fn save_memory(path: &std::path::Path, memory: &[String]) {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(path, memory.join("\n"));
}
