use leptos::prelude::*;
use crate::components::scanner::AnalysisResult;

// ── AI Provider Helpers ─────────────────────────────────────────────

/// Call Gemini API with a vision (image + text) prompt.
#[cfg(feature = "ssr")]
async fn call_gemini_vision(
    api_key: &str,
    model: &str,
    prompt: &str,
    image_base64: &str,
) -> Result<String, String> {
    let request_body = serde_json::json!({
        "contents": [{
            "parts": [
                { "text": prompt },
                { "inline_data": { "mime_type": "image/jpeg", "data": image_base64 } }
            ]
        }]
    });

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
        model
    );

    let client = reqwest::Client::new();
    let resp = client.post(&url)
        .header("x-goog-api-key", api_key)
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("Gemini network error: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Gemini API error: {} {}", status, body));
    }

    let json_resp: serde_json::Value = resp.json().await
        .map_err(|e| format!("Gemini parse error: {}", e))?;

    extract_gemini_text(&json_resp)
}

/// Call Gemini API with a text-only prompt.
#[cfg(feature = "ssr")]
async fn call_gemini_text(
    api_key: &str,
    model: &str,
    prompt: &str,
) -> Result<String, String> {
    let request_body = serde_json::json!({
        "contents": [{
            "parts": [{ "text": prompt }]
        }]
    });

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
        model
    );

    let client = reqwest::Client::new();
    let resp = client.post(&url)
        .header("x-goog-api-key", api_key)
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("Gemini network error: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Gemini API error: {} {}", status, body));
    }

    let json_resp: serde_json::Value = resp.json().await
        .map_err(|e| format!("Gemini parse error: {}", e))?;

    extract_gemini_text(&json_resp)
}

/// Extract text from a Gemini API response.
#[cfg(feature = "ssr")]
fn extract_gemini_text(json: &serde_json::Value) -> Result<String, String> {
    json.get("candidates")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("content"))
        .and_then(|c| c.get("parts"))
        .and_then(|p| p.get(0))
        .and_then(|p| p.get("text"))
        .and_then(|t| t.as_str())
        .map(|s| s.replace("```json", "").replace("```", "").trim().to_string())
        .ok_or_else(|| "Could not extract text from Gemini response".to_string())
}

/// Call Claude Messages API with a vision (image + text) prompt.
#[cfg(feature = "ssr")]
async fn call_claude_vision(
    api_key: &str,
    model: &str,
    prompt: &str,
    image_base64: &str,
) -> Result<String, String> {
    let request_body = serde_json::json!({
        "model": model,
        "max_tokens": 4096,
        "messages": [{
            "role": "user",
            "content": [
                {
                    "type": "image",
                    "source": {
                        "type": "base64",
                        "media_type": "image/jpeg",
                        "data": image_base64
                    }
                },
                {
                    "type": "text",
                    "text": prompt
                }
            ]
        }]
    });

    let client = reqwest::Client::new();
    let resp = client.post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("Claude network error: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Claude API error: {} {}", status, body));
    }

    let json_resp: serde_json::Value = resp.json().await
        .map_err(|e| format!("Claude parse error: {}", e))?;

    extract_claude_text(&json_resp)
}

/// Call Claude Messages API with a text-only prompt.
#[cfg(feature = "ssr")]
async fn call_claude_text(
    api_key: &str,
    model: &str,
    prompt: &str,
) -> Result<String, String> {
    let request_body = serde_json::json!({
        "model": model,
        "max_tokens": 1024,
        "messages": [{
            "role": "user",
            "content": prompt
        }]
    });

    let client = reqwest::Client::new();
    let resp = client.post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("Claude network error: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Claude API error: {} {}", status, body));
    }

    let json_resp: serde_json::Value = resp.json().await
        .map_err(|e| format!("Claude parse error: {}", e))?;

    extract_claude_text(&json_resp)
}

/// Extract text from a Claude Messages API response.
#[cfg(feature = "ssr")]
fn extract_claude_text(json: &serde_json::Value) -> Result<String, String> {
    json.get("content")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("text"))
        .and_then(|t| t.as_str())
        .map(|s| s.replace("```json", "").replace("```", "").trim().to_string())
        .ok_or_else(|| "Could not extract text from Claude response".to_string())
}

// ── Fallback Orchestration ──────────────────────────────────────────

/// Call AI vision with automatic fallback: tries Gemini first, then Claude.
#[cfg(feature = "ssr")]
async fn call_ai_vision(prompt: &str, image_base64: &str) -> Result<String, ServerFnError> {
    use crate::config::config;
    let cfg = config();

    let has_gemini = !cfg.gemini_api_key.is_empty();
    let has_claude = !cfg.claude_api_key.is_empty();

    if !has_gemini && !has_claude {
        return Err(ServerFnError::new(
            "No AI API keys configured. Set GEMINI_API_KEY and/or CLAUDE_API_KEY in your .env file."
        ));
    }

    // Try Gemini first
    if has_gemini {
        match call_gemini_vision(&cfg.gemini_api_key, &cfg.gemini_model, prompt, image_base64).await {
            Ok(text) => return Ok(text),
            Err(e) => {
                if has_claude {
                    log::warn!("Gemini failed ({}), falling back to Claude", e);
                } else {
                    return Err(ServerFnError::new(e));
                }
            }
        }
    }

    // Fallback to Claude
    if has_claude {
        match call_claude_vision(&cfg.claude_api_key, &cfg.claude_model, prompt, image_base64).await {
            Ok(text) => return Ok(text),
            Err(e) => {
                return Err(ServerFnError::new(format!(
                    "AI analysis failed (both providers). Last error: {}", e
                )));
            }
        }
    }

    unreachable!()
}

/// Call AI text with automatic fallback: tries Gemini first, then Claude.
#[cfg(feature = "ssr")]
async fn call_ai_text(prompt: &str) -> Result<String, String> {
    use crate::config::config;
    let cfg = config();

    let has_gemini = !cfg.gemini_api_key.is_empty();
    let has_claude = !cfg.claude_api_key.is_empty();

    if !has_gemini && !has_claude {
        return Err("No AI API keys configured".to_string());
    }

    // Try Gemini first
    if has_gemini {
        match call_gemini_text(&cfg.gemini_api_key, &cfg.gemini_model, prompt).await {
            Ok(text) => return Ok(text),
            Err(e) => {
                if has_claude {
                    log::warn!("Gemini text failed ({}), falling back to Claude", e);
                } else {
                    return Err(e);
                }
            }
        }
    }

    // Fallback to Claude
    if has_claude {
        return call_claude_text(&cfg.claude_api_key, &cfg.claude_model, prompt).await;
    }

    unreachable!()
}

// ── Server Functions ────────────────────────────────────────────────

#[server]
pub async fn analyze_orchid_image(
    image_base64: String,
    existing_species: Option<Vec<String>>,
    climate_summary: String,
    zone_names: Option<Vec<String>>,
) -> Result<AnalysisResult, ServerFnError> {
    use crate::auth::require_auth;

    require_auth().await?;

    let existing_species = existing_species.unwrap_or_default();
    let zone_names = zone_names.unwrap_or_default();

    // Cap base64 payload at ~15MB to prevent abuse
    if image_base64.len() > 15 * 1024 * 1024 {
        return Err(ServerFnError::new("Image too large (max 15MB)"));
    }

    let zone_list = if zone_names.is_empty() {
        "No zones configured".to_string()
    } else {
        zone_names.join(", ")
    };

    let prompt = format!(
        "Identify the plant species from this image. This is most likely an orchid but could be any houseplant or companion plant (e.g. Rhipsalis, Hoya, fern, Tillandsia). \
        Think step-by-step: \
        1. Identify the species with high confidence (look for tags). \
        2. Analyze its natural habitat and care requirements. \
        3. Compare requirements against my conditions: {}. \
        4. Consider my growing zones: {}. \
        5. Check if I own it: {:?}. \
        6. Determine the plant's native habitat region and approximate center-point coordinates for its primary native range. \
        Then, evaluate the fit. \
        Finally, return ONLY valid JSON with this structure (no markdown): \
        {{ \"species_name\": \"...\", \"fit_category\": \"Good Fit\", \"reason\": \"...\", \"already_owned\": false, \"water_freq\": 7, \"light_req\": \"Medium\", \"temp_range\": \"18-28C\", \"temp_min\": 18.0, \"temp_max\": 28.0, \"humidity_min\": 50.0, \"humidity_max\": 80.0, \"placement_suggestion\": \"...\", \"conservation_status\": \"CITES II\", \"native_region\": \"Cloud forests of Ecuador\", \"native_latitude\": -1.83, \"native_longitude\": -78.18 }} \
        Allowed fit_categories: 'Good Fit', 'Bad Fit', 'Caution Fit'. \
        For light_req, choose from: 'High', 'Medium', 'Low'. \
        For placement_suggestion, choose from my zones: {}. \
        For conservation_status, use 'CITES I', 'CITES II', 'Endangered', 'Vulnerable', or null if unknown/common. \
        For native_region, provide a brief description of where this species naturally grows. \
        For native_latitude and native_longitude, provide approximate decimal coordinates for the center of its native range. Set to null if unknown. \
        For temp_min/temp_max, provide the ideal temperature range in Celsius as numeric values (e.g. 18.0 and 28.0). \
        For humidity_min/humidity_max, provide the ideal humidity range as percentages (e.g. 50.0 and 80.0). Set to null if unknown. \
        Also include seasonal care data in Northern Hemisphere terms: \
        \"rest_start_month\": 11, \"rest_end_month\": 2, \"bloom_start_month\": 3, \"bloom_end_month\": 5, \
        \"rest_water_multiplier\": 0.3, \"rest_fertilizer_multiplier\": 0.0, \
        \"active_water_multiplier\": 1.0, \"active_fertilizer_multiplier\": 1.0 \
        Months are 1-12. Multipliers are 0.0-1.0 (0.3 = 30% of normal frequency, 0.0 = stop entirely). \
        Set seasonal fields to null if the species has no distinct rest period or seasonal cycle.",
        climate_summary,
        zone_list,
        existing_species,
        zone_list,
    );

    let text = call_ai_vision(&prompt, &image_base64).await?;

    let result: AnalysisResult = serde_json::from_str(&text)
        .map_err(|e| ServerFnError::new(format!("Failed to parse AI response: {}", e)))?;

    Ok(result)
}

#[server]
pub async fn analyze_orchid_by_name(
    species_name: String,
    existing_species: Option<Vec<String>>,
    climate_summary: String,
    zone_names: Option<Vec<String>>,
) -> Result<AnalysisResult, ServerFnError> {
    use crate::auth::require_auth;

    require_auth().await?;

    let species_name = species_name.trim().to_string();
    if species_name.is_empty() {
        return Err(ServerFnError::new("Please enter a species name"));
    }

    let existing_species = existing_species.unwrap_or_default();
    let zone_names = zone_names.unwrap_or_default();

    let zone_list = if zone_names.is_empty() {
        "No zones configured".to_string()
    } else {
        zone_names.join(", ")
    };

    let prompt = format!(
        "I'm considering getting a plant: '{}'. \
        Think step-by-step: \
        1. Confirm the species exists and determine its full botanical name. If the name is ambiguous, pick the most common match for this species name. Do not assume it is an orchid \u{2014} identify the correct genus and family. \
        2. Analyze its natural habitat and care requirements. \
        3. Compare requirements against my conditions: {}. \
        4. Consider my growing zones: {}. \
        5. Check if I own it: {:?}. \
        6. Determine the plant's native habitat region and approximate center-point coordinates for its primary native range. \
        Then, evaluate the fit. \
        Finally, return ONLY valid JSON with this structure (no markdown): \
        {{ \"species_name\": \"...\", \"fit_category\": \"Good Fit\", \"reason\": \"...\", \"already_owned\": false, \"water_freq\": 7, \"light_req\": \"Medium\", \"temp_range\": \"18-28C\", \"temp_min\": 18.0, \"temp_max\": 28.0, \"humidity_min\": 50.0, \"humidity_max\": 80.0, \"placement_suggestion\": \"...\", \"conservation_status\": \"CITES II\", \"native_region\": \"Cloud forests of Ecuador\", \"native_latitude\": -1.83, \"native_longitude\": -78.18 }} \
        Allowed fit_categories: 'Good Fit', 'Bad Fit', 'Caution Fit'. \
        For light_req, choose from: 'High', 'Medium', 'Low'. \
        For placement_suggestion, choose from my zones: {}. \
        For conservation_status, use 'CITES I', 'CITES II', 'Endangered', 'Vulnerable', or null if unknown/common. \
        For native_region, provide a brief description of where this species naturally grows. \
        For native_latitude and native_longitude, provide approximate decimal coordinates for the center of its native range. Set to null if unknown. \
        For temp_min/temp_max, provide the ideal temperature range in Celsius as numeric values (e.g. 18.0 and 28.0). \
        For humidity_min/humidity_max, provide the ideal humidity range as percentages (e.g. 50.0 and 80.0). Set to null if unknown. \
        Also include seasonal care data in Northern Hemisphere terms: \
        \"rest_start_month\": 11, \"rest_end_month\": 2, \"bloom_start_month\": 3, \"bloom_end_month\": 5, \
        \"rest_water_multiplier\": 0.3, \"rest_fertilizer_multiplier\": 0.0, \
        \"active_water_multiplier\": 1.0, \"active_fertilizer_multiplier\": 1.0 \
        Months are 1-12. Multipliers are 0.0-1.0 (0.3 = 30% of normal frequency, 0.0 = stop entirely). \
        Set seasonal fields to null if the species has no distinct rest period or seasonal cycle.",
        species_name,
        climate_summary,
        zone_list,
        existing_species,
        zone_list,
    );

    let text = call_ai_text_server(&prompt).await?;

    let result: AnalysisResult = serde_json::from_str(&text)
        .map_err(|e| ServerFnError::new(format!("Failed to parse AI response: {}", e)))?;

    Ok(result)
}

/// Internal wrapper: call_ai_text returning ServerFnError.
#[cfg(feature = "ssr")]
async fn call_ai_text_server(prompt: &str) -> Result<String, ServerFnError> {
    call_ai_text(prompt).await.map_err(|e| ServerFnError::new(e))
}

#[server]
pub async fn generate_care_recap(
    orchid_id: String,
    event_type: String,
) -> Result<String, ServerFnError> {
    use crate::auth::require_auth;
    use crate::db::db;
    use crate::error::internal_error;

    let user_id = require_auth().await?;

    let orchid_record = surrealdb::types::RecordId::parse_simple(&orchid_id)
        .map_err(|e| internal_error("Parse orchid ID failed", e))?;
    let owner = surrealdb::types::RecordId::parse_simple(&user_id)
        .map_err(|e| internal_error("Parse user ID failed", e))?;

    // Gather care history for past 6 months
    let mut response = db()
        .query(
            "SELECT event_type, note, timestamp FROM log_entry \
             WHERE orchid = $orchid_id AND owner = $owner \
             AND timestamp > time::now() - 6m \
             ORDER BY timestamp ASC"
        )
        .bind(("orchid_id", orchid_record.clone()))
        .bind(("owner", owner.clone()))
        .await
        .map_err(|e| internal_error("Care recap query failed", e))?;

    let _ = response.take_errors();
    let entries: Vec<serde_json::Value> = response.take(0).unwrap_or_default();

    // Gather orchid info
    let mut orchid_resp = db()
        .query(
            "SELECT name, species, light_requirement, placement FROM $orchid_id \
             WHERE owner = $owner"
        )
        .bind(("orchid_id", orchid_record))
        .bind(("owner", owner))
        .await
        .map_err(|e| internal_error("Orchid info query failed", e))?;

    let _ = orchid_resp.take_errors();
    let orchid_info: Option<serde_json::Value> = orchid_resp.take(0).unwrap_or(None);

    let species = orchid_info.as_ref()
        .and_then(|o| o.get("species"))
        .and_then(|s| s.as_str())
        .unwrap_or("Unknown");

    // Summarize care events
    let mut watering_count = 0u32;
    let mut care_events = Vec::new();
    for entry in &entries {
        let et = entry.get("event_type").and_then(|e| e.as_str()).unwrap_or("");
        let note = entry.get("note").and_then(|n| n.as_str()).unwrap_or("");
        match et {
            "Watered" => watering_count += 1,
            "" => {},
            _ => care_events.push(format!("{}: {}", et, note)),
        }
    }

    let fallback_stats = format!(
        "Over the past 6 months: {} waterings, {} care events recorded.",
        watering_count, care_events.len()
    );

    let care_summary = serde_json::json!({
        "species": species,
        "event_type": event_type,
        "watering_count_6mo": watering_count,
        "care_events": care_events,
        "total_log_entries": entries.len(),
    });

    let prompt = format!(
        "Given this {}'s care history over the past 6 months, explain in 2-3 sentences \
         what likely contributed to this {}. Be specific about which care actions helped. \
         Keep the tone warm and encouraging. Data: {}",
        species, event_type, care_summary
    );

    match call_ai_text(&prompt).await {
        Ok(text) => Ok(text),
        Err(e) => {
            log::warn!("AI care recap failed ({}), returning fallback stats", e);
            Ok(fallback_stats)
        }
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    // ── extract_gemini_text ─────────────────────────────────────────

    #[test]
    fn test_extract_gemini_text_valid_response() {
        let json = serde_json::json!({
            "candidates": [{
                "content": {
                    "parts": [{ "text": "{\"species_name\": \"Phalaenopsis bellina\"}" }]
                }
            }]
        });
        let result = extract_gemini_text(&json);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Phalaenopsis bellina"));
    }

    #[test]
    fn test_extract_gemini_text_strips_markdown_fences() {
        let json = serde_json::json!({
            "candidates": [{
                "content": {
                    "parts": [{ "text": "```json\n{\"species_name\": \"Dendrobium\"}\n```" }]
                }
            }]
        });
        let result = extract_gemini_text(&json).unwrap();
        assert!(!result.contains("```"));
        assert!(result.contains("Dendrobium"));
    }

    #[test]
    fn test_extract_gemini_text_missing_candidates() {
        let json = serde_json::json!({});
        assert!(extract_gemini_text(&json).is_err());
    }

    #[test]
    fn test_extract_gemini_text_empty_candidates() {
        let json = serde_json::json!({ "candidates": [] });
        assert!(extract_gemini_text(&json).is_err());
    }

    #[test]
    fn test_extract_gemini_text_missing_parts() {
        let json = serde_json::json!({
            "candidates": [{ "content": {} }]
        });
        assert!(extract_gemini_text(&json).is_err());
    }

    // ── extract_claude_text ─────────────────────────────────────────

    #[test]
    fn test_extract_claude_text_valid_response() {
        let json = serde_json::json!({
            "content": [{
                "type": "text",
                "text": "{\"species_name\": \"Oncidium sharry baby\"}"
            }]
        });
        let result = extract_claude_text(&json);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Oncidium sharry baby"));
    }

    #[test]
    fn test_extract_claude_text_strips_markdown_fences() {
        let json = serde_json::json!({
            "content": [{
                "type": "text",
                "text": "```json\n{\"species_name\": \"Cattleya\"}\n```"
            }]
        });
        let result = extract_claude_text(&json).unwrap();
        assert!(!result.contains("```"));
        assert!(result.contains("Cattleya"));
    }

    #[test]
    fn test_extract_claude_text_missing_content() {
        let json = serde_json::json!({});
        assert!(extract_claude_text(&json).is_err());
    }

    #[test]
    fn test_extract_claude_text_empty_content() {
        let json = serde_json::json!({ "content": [] });
        assert!(extract_claude_text(&json).is_err());
    }

    #[test]
    fn test_extract_claude_text_missing_text_field() {
        let json = serde_json::json!({
            "content": [{ "type": "text" }]
        });
        assert!(extract_claude_text(&json).is_err());
    }
}
