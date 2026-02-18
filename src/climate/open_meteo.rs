use crate::error::AppError;

/// A raw habitat weather reading from Open-Meteo.
pub struct HabitatReading {
    pub temperature_c: f64,
    pub humidity_pct: f64,
    pub precipitation_mm: f64,
}

/// Fetch current weather conditions from Open-Meteo for a given coordinate pair.
/// Free API, no key required.
pub async fn fetch_habitat_weather(
    client: &reqwest::Client,
    latitude: f64,
    longitude: f64,
) -> Result<HabitatReading, AppError> {
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,relative_humidity_2m,precipitation",
        latitude, longitude
    );

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| AppError::Network(format!("Open-Meteo request failed: {}", e)))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(AppError::Network(format!(
            "Open-Meteo API error {}: {}",
            status, body
        )));
    }

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| AppError::Serialization(format!("Open-Meteo parse error: {}", e)))?;

    let current = json
        .get("current")
        .ok_or_else(|| AppError::Serialization("Missing 'current' in Open-Meteo response".into()))?;

    let temperature = current
        .get("temperature_2m")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);

    let humidity = current
        .get("relative_humidity_2m")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);

    let precipitation = current
        .get("precipitation")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);

    Ok(HabitatReading {
        temperature_c: temperature,
        humidity_pct: humidity,
        precipitation_mm: precipitation,
    })
}
