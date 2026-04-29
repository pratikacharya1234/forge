#![allow(dead_code)]

use anyhow::Result;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct ModelInfo {
    pub name: String,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(rename = "inputTokenLimit")]
    pub input_token_limit: Option<u32>,
    #[serde(rename = "outputTokenLimit")]
    pub output_token_limit: Option<u32>,
    #[serde(rename = "supportedGenerationMethods")]
    pub supported_methods: Option<Vec<String>>,
}

#[derive(Deserialize, Debug)]
struct ModelListResponse {
    models: Vec<ModelInfo>,
}

/// Query the Gemini API for available models. Returns the full list.
pub async fn fetch_available_models(api_key: &str) -> Result<Vec<ModelInfo>> {
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models?key={}",
        api_key
    );

    let client = reqwest::Client::new();
    let resp = client.get(&url).send().await?;
    let body = resp.text().await?;

    let parsed: ModelListResponse = serde_json::from_str(&body)?;

    Ok(parsed.models)
}

/// Filter to coding-relevant models only.
pub fn filter_coding_models(models: &[ModelInfo]) -> Vec<ModelInfo> {
    let code_relevant: Vec<&str> = vec![
        "gemini-3.1",
        "gemini-3",
        "gemini-2.5-pro",
        "gemini-2.5-flash",
        "gemini-2.0-flash",
    ];

    // Junk display names from Gemini API (internal codenames)
    let skip_displays: Vec<&str> = vec![
        "nano banana", "nano", "banana",
    ];

    let mut filtered: Vec<ModelInfo> = models
        .iter()
        .filter(|m| {
            let name = m.name.to_lowercase();
            let display = m.display_name.as_deref().unwrap_or("").to_lowercase();
            let has_gemini = name.contains("gemini");
            let is_chat_model = m
                .supported_methods
                .as_ref()
                .map(|m| m.iter().any(|s| s == "generateContent"))
                .unwrap_or(false);
            let is_relevant = code_relevant.iter().any(|p| name.contains(p));
            let is_latest = !name.contains("1.0") && !name.contains("1.5");
            let has_junk_display = skip_displays.iter().any(|j| display.contains(*j));
            has_gemini && is_chat_model && is_relevant && is_latest && !has_junk_display
        })
        .cloned()
        .collect();

    if filtered.is_empty() {
        filtered = models
            .iter()
            .filter(|m| {
                let display = m.display_name.as_deref().unwrap_or("").to_lowercase();
                let has_junk = skip_displays.iter().any(|j| display.contains(*j));
                m.supported_methods
                    .as_ref()
                    .map(|m| m.iter().any(|s| s == "generateContent"))
                    .unwrap_or(false)
                    && !has_junk
            })
            .cloned()
            .collect();
    }

    filtered.sort_by(|a, b| b.name.cmp(&a.name));
    filtered.dedup_by(|a, b| a.name == b.name);
    filtered
}

/// Fetch Claude models from the Anthropic API.
/// Returns `(model_id, context_description)` pairs filtered to generation models.
pub async fn fetch_anthropic_models(api_key: &str) -> Result<Vec<(String, String)>> {
    #[derive(serde::Deserialize)]
    struct AnthropicModel {
        id: String,
        display_name: Option<String>,
    }
    #[derive(serde::Deserialize)]
    struct AnthropicListResponse {
        data: Vec<AnthropicModel>,
    }

    let client = reqwest::Client::new();
    let resp = client
        .get("https://api.anthropic.com/v1/models")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .send()
        .await?;

    let body = resp.text().await?;
    let parsed: AnthropicListResponse = serde_json::from_str(&body)?;

    let skip = ["claude-2", "claude-instant", "claude-1"];
    let mut out: Vec<(String, String)> = parsed
        .data
        .into_iter()
        .filter(|m| !skip.iter().any(|s| m.id.contains(s)))
        .map(|m| {
            let ctx = if m.id.contains("claude-4") || m.id.contains("claude-3") {
                "200K ctx".to_string()
            } else {
                m.display_name.unwrap_or_default()
            };
            (m.id, ctx)
        })
        .collect();

    out.sort_by(|a, b| b.0.cmp(&a.0));
    out.dedup_by(|a, b| a.0 == b.0);
    Ok(out)
}

/// Fetch GPT/O-series models from the OpenAI API.
/// Returns `(model_id, context_description)` pairs filtered to relevant chat models.
pub async fn fetch_openai_models(api_key: &str) -> Result<Vec<(String, String)>> {
    #[derive(serde::Deserialize)]
    struct OpenAIModel {
        id: String,
    }
    #[derive(serde::Deserialize)]
    struct OpenAIListResponse {
        data: Vec<OpenAIModel>,
    }

    let client = reqwest::Client::new();
    let resp = client
        .get("https://api.openai.com/v1/models")
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await?;

    let body = resp.text().await?;
    let parsed: OpenAIListResponse = serde_json::from_str(&body)?;

    let relevant = ["gpt-4", "gpt-4o", "gpt-4.1", "o1", "o3", "o4"];
    let skip = ["instruct", "vision", "realtime", "audio", "search", "mini-2024", "preview-2024", "0125", "0613", "1106"];
    let mut out: Vec<(String, String)> = parsed
        .data
        .into_iter()
        .filter(|m| {
            relevant.iter().any(|p| m.id.starts_with(p))
                && !skip.iter().any(|s| m.id.contains(s))
        })
        .map(|m| {
            let ctx = if m.id.contains("gpt-4.1") {
                "1M ctx".to_string()
            } else if m.id.contains("o3") || m.id.contains("o4") || m.id.contains("o1") {
                "200K ctx".to_string()
            } else {
                "128K ctx".to_string()
            };
            (m.id, ctx)
        })
        .collect();

    out.sort_by(|a, b| b.0.cmp(&a.0));
    out.dedup_by(|a, b| a.0 == b.0);
    Ok(out)
}

/// Get the latest available model matching a preference.
pub fn pick_best_model(models: &[ModelInfo], preferred: &str) -> String {
    let find = |name: &str| -> Option<String> {
        models
            .iter()
            .find(|m| {
                m.name
                    .to_lowercase()
                    .contains(&name.to_lowercase())
            })
            .map(|m| m.name.clone())
    };

    if !preferred.is_empty() {
        if let Some(m) = find(preferred) {
            return m;
        }
    }

    find("gemini-2.5-flash-latest")
        .or_else(|| find("gemini-2.5-flash"))
        .or_else(|| find("gemini-2.5-pro"))
        .or_else(|| find("gemini-2.0-flash"))
        .unwrap_or_else(|| "gemini-2.5-flash".to_string())
}

/// Detect the actual model version from a model name prefix.
/// E.g., "gemini-2.5-flash" might resolve to "models/gemini-2.5-flash-preview-05-20"
pub fn resolve_model_name(fetched: &[ModelInfo], requested: &str) -> String {
    let normalized = requested
        .trim_start_matches("models/")
        .trim_start_matches("tunedModels/");

    let exact = fetched.iter().find(|m| {
        m.name == normalized
            || m.name == format!("models/{}", normalized)
    });

    if let Some(m) = exact {
        return m.name.clone();
    }

    let prefix_match = fetched
        .iter()
        .filter(|m| m.name.contains(normalized))
        .max_by_key(|m| m.name.len());

    if let Some(m) = prefix_match {
        return m.name.clone();
    }

    requested.to_string()
}

/// Auto-detect the best available Gemini model from the API response.
/// Prefers gemini-2.5-flash variants, falls back to latest available.
pub fn resolve_best_model(fetched: &[ModelInfo]) -> String {
    let coding = filter_coding_models(fetched);

    // Priority order for default model selection
    let preferences = [
        "gemini-3.1-flash",
        "gemini-3-flash",
        "gemini-2.5-flash",
        "gemini-3.1-pro",
        "gemini-3-pro",
        "gemini-2.5-pro",
        "gemini-2.0-flash",
    ];

    for pref in &preferences {
        if let Some(m) = coding.iter().find(|m| m.name.contains(pref)) {
            return m.name.clone();
        }
    }

    // No preferred model found — use the first coding model available
    if let Some(m) = coding.first() {
        return m.name.clone();
    }

    // Nothing found at all — return the API default
    "gemini-2.5-flash".to_string()
}
