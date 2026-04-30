// Voice input — record from microphone, transcribe via Gemini multimodal API.
// Uses `arecord` (Linux) or `sox` for recording. No new Rust crates needed.
use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use std::process::Command;

/// Record audio from the default microphone. Returns WAV bytes.
/// Tries sox first (better quality), falls back to arecord.
pub fn record_audio(duration_secs: u32) -> Result<Vec<u8>> {
    // Try sox first
    if Command::new("sox").arg("--version").output().is_ok() {
        let output = Command::new("sox")
            .args([
                "-d", "-t", "wav", "-r", "16000", "-c", "1",
                "-b", "16", "-",
                "trim", "0", &duration_secs.to_string(),
            ])
            .output()
            .context("Failed to record audio with sox. Install: apt install sox")?;
        return Ok(output.stdout);
    }

    // Fall back to arecord
    let output = Command::new("arecord")
        .args([
            "-f", "cd",
            "-t", "wav",
            "-d", &duration_secs.to_string(),
            "-r", "16000",
        ])
        .output()
        .context("Failed to record audio. Install sox or alsa-utils: apt install sox")?;
    Ok(output.stdout)
}

/// Transcribe audio bytes using Gemini's multimodal API.
/// Returns the transcribed text.
pub async fn transcribe_audio(audio_bytes: &[u8], api_key: &str) -> Result<String> {
    let audio_b64 = BASE64.encode(audio_bytes);

    let body = serde_json::json!({
        "contents": [{
            "parts": [
                { "text": "Transcribe this audio to text. Return ONLY the transcribed words, nothing else. If the audio is unclear, return your best guess." },
                { "inlineData": { "mimeType": "audio/wav", "data": audio_b64 } }
            ]
        }]
    });

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}",
        api_key
    );

    let client = reqwest::Client::new();
    let resp = client.post(&url).json(&body).send().await
        .context("Voice transcription request failed")?;

    let status = resp.status();
    let resp_body = resp.text().await?;

    if !status.is_success() {
        anyhow::bail!("Transcription API error {}: {}", status, &resp_body[..resp_body.len().min(300)]);
    }

    let parsed: serde_json::Value = serde_json::from_str(&resp_body)?;
    let text = parsed["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .unwrap_or("")
        .trim()
        .to_string();

    if text.is_empty() {
        anyhow::bail!("No transcription returned — try speaking more clearly");
    }

    Ok(text)
}

/// Full voice prompt flow: record → transcribe → return text.
/// Used by `--voice` flag to replace CLI text input with spoken commands.
pub async fn voice_prompt(api_key: &str, duration_secs: u32) -> Result<String> {
    use colored::Colorize;

    println!();
    println!("  {} Recording... speak now ({}s)", "🎤".bright_red(), duration_secs);
    println!("  {}", "  (Press Ctrl+C to cancel)".dimmed());

    let audio = record_audio(duration_secs)?;
    let size_kb = audio.len() as f64 / 1024.0;

    println!("  {} Recorded {:.1}KB — transcribing...", "✅".green(), size_kb);

    let text = transcribe_audio(&audio, api_key).await?;

    println!("  {} You said: {}", "🗣️".cyan(), text.bright_white().bold());
    println!();

    Ok(text)
}
