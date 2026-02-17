use leptos::prelude::*;
use crate::components::scanner::AnalysisResult;

#[server]
pub async fn analyze_orchid_image(
    image_base64: String,
    existing_species: Vec<String>,
    climate_summary: String,
) -> Result<AnalysisResult, ServerFnError> {
    use crate::config::config;

    let cfg = config();
    let api_key = &cfg.gemini_api_key;
    let model = &cfg.gemini_model;

    if api_key.is_empty() {
        return Err(ServerFnError::new("Gemini API key not configured on server"));
    }

    let prompt = format!(
        "Identify the orchid species from this image. \
        Think step-by-step: \
        1. Identify the species with high confidence (look for tags). \
        2. Analyze its natural habitat and care requirements. \
        3. Compare requirements against my conditions: {}. \
        4. Consider outdoor conditions (90606, partial shade/rack). Outdoor Rack: High Sun with shade cloth. Patio: Morning Sun/Afternoon Shade. \
        5. Check if I own it: {:?}. \
        Then, evaluate the fit. \
        Finally, return ONLY valid JSON with this structure (no markdown): \
        {{ \"species_name\": \"...\", \"fit_category\": \"Good Fit\", \"reason\": \"...\", \"already_owned\": false, \"water_freq\": 7, \"light_req\": \"Medium\", \"temp_range\": \"18-28C\", \"placement_suggestion\": \"Medium\", \"conservation_status\": \"CITES II\" }} \
        Allowed fit_categories: 'Good Fit', 'Bad Fit', 'Caution Fit'. \
        For light_req, choose from: 'High', 'Medium', 'Low'. \
        For placement_suggestion, choose from: 'High', 'Medium', 'Low', 'Patio', 'OutdoorRack'. \
        For conservation_status, use 'CITES I', 'CITES II', 'Endangered', 'Vulnerable', or null if unknown/common.",
        climate_summary,
        existing_species
    );

    let request_body = serde_json::json!({
        "contents": [{
            "parts": [
                { "text": prompt },
                { "inline_data": { "mime_type": "image/jpeg", "data": image_base64 } }
            ]
        }]
    });

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        model, api_key
    );

    let client = reqwest::Client::new();
    let resp = client.post(&url)
        .json(&request_body)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Network error: {}", e)))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(ServerFnError::new(format!("Gemini API error: {} - {}", status, body)));
    }

    let json_resp: serde_json::Value = resp.json().await
        .map_err(|e| ServerFnError::new(format!("Parse error: {}", e)))?;

    let text = json_resp
        .get("candidates")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("content"))
        .and_then(|c| c.get("parts"))
        .and_then(|p| p.get(0))
        .and_then(|p| p.get("text"))
        .and_then(|t| t.as_str())
        .map(|s| s.replace("```json", "").replace("```", "").trim().to_string())
        .ok_or_else(|| ServerFnError::new("Could not extract text from Gemini response"))?;

    let result: AnalysisResult = serde_json::from_str(&text)
        .map_err(|e| ServerFnError::new(format!("Failed to parse AI response: {}", e)))?;

    Ok(result)
}
