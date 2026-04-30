// EMBER — Lightweight real-time voice AI. Pure Rust.
// Google Gemini for transcription + Google Cloud TTS for speech.
// Local memory, zero keyboard, hands-free operation.

use anyhow::Result;
use base64::Engine;
use colored::Colorize;
use std::io::Write;

/// Run EMBER voice assistant loop.
pub async fn ember_loop(config: &crate::config::Config) -> Result<bool> {
    println!();
    println!("  ╔══════════════════════════════════════════════╗");
    println!("  ║                                              ║");
    println!("  ║         🔥  E M B E R   O N L I N E          ║");
    println!("  ║                                              ║");
    println!("  ║  Voice → Gemini 2.0 Flash → {}", model_line(config));
    println!("  ╚══════════════════════════════════════════════╝");
    println!();

    let has_mic = crate::voice::check_audio();
    
    if has_mic {
        println!("  {} Mic detected — Voice mode or Text mode?", "🎤".bright_red());
        println!("  [1] Voice (EMBER)  |  [2] Text (terminal)  |  [Enter] = Voice");
        print!("  > ");
        let _ = std::io::stdout().flush();
        let mut choice = String::new();
        let _ = std::io::stdin().read_line(&mut choice);
        if choice.trim() == "2" { return Ok(true); }
        println!();
    } else {
        println!("  {} No mic. Starting text mode.", "⚠️".yellow());
        return Ok(true);
    }

    // Greet
    let proj = std::env::current_dir().ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        .unwrap_or_default();
    let greeting = format!("I'm Ember, your FORGE voice assistant. Working on {}. What do you need?", proj);
    speak(&greeting, &config.api_key).await;
    println!("  {} {}", "🔊".bright_cyan(), greeting.bright_white());
    println!();
    println!("  {} Speak now — I'm listening...", "🎤".bright_red());
    println!("  {} Say \"quit\" to stop", "  ".dimmed());
    println!();

    // Memory
    let mem_path = dirs::home_dir().unwrap_or_default().join(".forge").join("ember-memory.md");
    let mut memory: Vec<String> = if mem_path.exists() {
        std::fs::read_to_string(&mem_path).unwrap_or_default()
            .lines().map(|l| l.to_string()).filter(|l| !l.trim().is_empty()).collect()
    } else { Vec::new() };

    loop {
        let user_msg = match crate::voice::listen_and_transcribe(&config.api_key, 3).await {
            Ok(t) => t,
            Err(_) => continue,
        };
        if user_msg.is_empty() { continue; }

        println!("  {} {}", "🗣️".cyan(), user_msg.bright_white());

        let lower = user_msg.to_lowercase();
        if lower.contains("quit") || lower.contains("exit") || lower.contains("goodbye") {
            let bye = "Shutting down. Goodbye.";
            speak(bye, &config.api_key).await;
            println!("  {} {}", "👋".cyan(), bye);
            save_mem(&mem_path, &memory);
            return Ok(false);
        }

        memory.push(format!("You: {}", user_msg));
        let ctx = if memory.len() > 2 {
            let r: String = memory.iter().rev().take(4).collect::<Vec<_>>().iter().rev().map(|s| s.as_str()).collect::<Vec<_>>().join("\n");
            format!("You are EMBER, concise voice AI for FORGE. 1-2 sentences max. Recent:\n{}\n\nUser: {}", r, user_msg)
        } else {
            format!("You are EMBER. Concise. 1-2 sentences.\nUser: {}", user_msg)
        };

        let response = match crate::agent::run_jarvis_query(config, &ctx).await {
            Ok(t) => t,
            Err(e) => format!("Error: {}", e),
        };
        memory.push(format!("EMBER: {}", response));

        speak(&response, &config.api_key).await;
        println!("  {} {}", "🔥".bright_red(), response.bright_white());
        println!();
    }
}

fn model_line(config: &crate::config::Config) -> String {
    if config.model.contains("claude") {
        format!("{} → 🔊", "Claude".purple().bold())
    } else if config.model.contains("gpt") {
        format!("{} → 🔊", "GPT".yellow().bold())
    } else {
        format!("{} → 🔊", "Gemini".green().bold())
    }
}

/// Speak via Google Cloud TTS (same API key as Gemini). Lightweight, always works.
async fn speak(text: &str, api_key: &str) {
    let clean: String = text.chars().filter(|c| c.is_ascii()).collect();
    if clean.len() < 2 { return; }
    
    let body = serde_json::json!({
        "input": { "text": clean },
        "voice": { "languageCode": "en-US", "name": "en-US-Journey-O" },
        "audioConfig": { "audioEncoding": "LINEAR16", "speakingRate": 1.1 }
    });

    let url = format!("https://texttospeech.googleapis.com/v1/text:synthesize?key={}", api_key);
    
    if let Ok(resp) = reqwest::Client::new()
        .post(&url)
        .json(&body)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
    {
        if let Ok(parsed) = resp.json::<serde_json::Value>().await {
            if let Some(b64) = parsed["audioContent"].as_str() {
                if let Ok(audio) = base64::engine::general_purpose::STANDARD.decode(b64) {
                    // Write WAV and play via paplay
                    let tmp = std::env::temp_dir().join("ember_tts.wav");
                    let _ = std::fs::write(&tmp, &audio);
                    let _ = std::process::Command::new("paplay")
                        .arg(&tmp)
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null())
                        .spawn();
                }
            }
        }
    }
}

fn save_mem(path: &std::path::Path, memory: &[String]) {
    if let Some(p) = path.parent() { let _ = std::fs::create_dir_all(p); }
    let _ = std::fs::write(path, memory.join("\n"));
}

/// Check if mic is available for EMBER.
pub fn mic_available() -> bool { crate::voice::check_audio() }
