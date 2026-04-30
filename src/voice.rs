// Voice input — record from microphone, transcribe via Gemini multimodal API.
// Tries pw-record > parec > sox > arecord. Converts raw PCM to WAV on the fly.
use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use std::process::Command;

/// Check if any audio recording tool is available.
pub fn check_audio() -> bool {
    Command::new("pw-record").arg("--version").output().is_ok()
        || Command::new("parec").arg("--version").output().is_ok()
        || Command::new("sox").arg("--version").output().is_ok()
        || Command::new("arecord").arg("--version").output().is_ok()
}

/// Record audio from default microphone. Returns proper WAV bytes.
/// Tries: timeout pw-record > timeout parec > sox > arecord
pub fn record_audio(duration_secs: u32) -> Result<Vec<u8>> {
    let dur = duration_secs.to_string();
    let timeout = format!("{}", duration_secs + 2);

    // pw-record (PipeWire) — needs timeout wrapper since it records forever
    if Command::new("pw-record").arg("--version").output().is_ok() {
        if let Ok(output) = Command::new("timeout")
            .args([&timeout, "pw-record", "--rate", "16000", "--channels", "1", "--format", "s16", "-"])
            .output()
        {
            if !output.stdout.is_empty() {
                return Ok(raw_to_wav(&output.stdout, 16000, 1, 16));
            }
        }
    }

    // parec (PulseAudio)
    if Command::new("parec").arg("--version").output().is_ok() {
        if let Ok(output) = Command::new("timeout")
            .args([&timeout, "parec", "--rate", "16000", "--channels", "1", "--format", "s16le"])
            .output()
        {
            if !output.stdout.is_empty() {
                return Ok(raw_to_wav(&output.stdout, 16000, 1, 16));
            }
        }
    }

    // sox — self-terminating after duration
    if Command::new("sox").arg("--version").output().is_ok() {
        if let Ok(output) = Command::new("sox")
            .args(["-d", "-t", "wav", "-r", "16000", "-c", "1", "-b", "16", "-", "trim", "0", &dur])
            .output()
        {
            if !output.stdout.is_empty() { return Ok(output.stdout); }
        }
    }

    // arecord — self-terminating
    if Command::new("arecord").arg("--version").output().is_ok() {
        if let Ok(output) = Command::new("arecord")
            .args(["-f", "cd", "-t", "wav", "-d", &dur, "-r", "16000"])
            .output()
        {
            if !output.stdout.is_empty() { return Ok(output.stdout); }
        }
    }

    anyhow::bail!("No audio recorder found. Install: apt install pulseaudio-utils")
}

/// Convert raw S16LE PCM bytes to minimal WAV with header.
fn raw_to_wav(pcm: &[u8], sample_rate: u32, channels: u16, bits: u16) -> Vec<u8> {
    let data_len = pcm.len() as u32;
    let byte_rate = sample_rate * channels as u32 * (bits / 8) as u32;
    let block_align = channels * (bits / 8);
    let mut wav = Vec::with_capacity(44 + pcm.len());
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + data_len).to_le_bytes());
    wav.extend_from_slice(b"WAVE");
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes()); // PCM = 1
    wav.extend_from_slice(&channels.to_le_bytes());
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    wav.extend_from_slice(&block_align.to_le_bytes());
    wav.extend_from_slice(&bits.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_len.to_le_bytes());
    wav.extend_from_slice(pcm);
    wav
}

/// Transcribe audio bytes using Gemini multimodal API. Returns text.
pub async fn transcribe_audio(audio_bytes: &[u8], api_key: &str) -> Result<String> {
    let audio_b64 = BASE64.encode(audio_bytes);
    let body = serde_json::json!({
        "contents": [{
            "parts": [
                { "text": "Transcribe this audio to text. Return ONLY the transcribed words." },
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
        .context("Transcription request failed")?;
    let status = resp.status();
    let body_text = resp.text().await?;
    if !status.is_success() {
        anyhow::bail!("Transcription HTTP {}: {}", status, &body_text[..body_text.len().min(300)]);
    }
    let parsed: serde_json::Value = serde_json::from_str(&body_text)?;
    let text = parsed["candidates"][0]["content"]["parts"][0]["text"]
        .as_str().unwrap_or("").trim().to_string();
    if text.is_empty() {
        anyhow::bail!("No transcription — speak more clearly");
    }
    Ok(text)
}

/// Shorthand: record + transcribe, return text.
pub async fn record_and_transcribe(api_key: &str, duration_secs: u32) -> Result<String> {
    use colored::Colorize;
    println!("  {} Listening...", "🎙️".bright_red());
    let audio = record_audio(duration_secs)?;
    let text = transcribe_audio(&audio, api_key).await?;
    println!("  {} {}", "🗣️".cyan(), text.bright_white());
    Ok(text)
}

/// Full voice prompt flow for --voice flag.
pub async fn voice_prompt(api_key: &str, duration_secs: u32) -> Result<String> {
    use colored::Colorize;
    println!();
    println!("  {} Recording... speak now ({}s)", "🎤".bright_red(), duration_secs);
    println!("  {}", "  (Press Ctrl+C to cancel)".dimmed());
    let audio = record_audio(duration_secs)?;
    println!("  {} Recorded {:.1}KB — transcribing...", "✅".green(), audio.len() as f64 / 1024.0);
    let text = transcribe_audio(&audio, api_key).await?;
    println!("  {} You said: {}", "🗣️".cyan(), text.bright_white().bold());
    println!();
    Ok(text)
}
