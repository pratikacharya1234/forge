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
