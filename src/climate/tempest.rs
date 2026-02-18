use super::{RawReading, calculate_vpd};
use crate::error::AppError;

/// Fetch the latest observation from a WeatherFlow Tempest station.
///
/// API docs: https://weatherflow.github.io/Tempest/api/
/// GET /swd/rest/observations/station/{station_id}?token={token}
/// Response `obs[0]` is an array of sensor values at documented indices:
///   Index 7: air_temperature (Celsius)
///   Index 8: relative_humidity (%)
pub async fn fetch_tempest_reading(
    client: &reqwest::Client,
    station_id: &str,
    token: &str,
) -> Result<RawReading, AppError> {
    let url = format!(
        "https://swd.weatherflow.com/swd/rest/observations/station/{}?token={}",
        station_id, token
    );

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| AppError::Network(format!("Tempest API request failed: {}", e)))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(AppError::Network(format!(
            "Tempest API error {}: {}",
            status, body
        )));
    }

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| AppError::Serialization(format!("Tempest response parse error: {}", e)))?;

    // Extract the first observation array
    let obs = json
        .get("obs")
        .and_then(|o| o.get(0))
        .ok_or_else(|| AppError::Serialization("No observations in Tempest response".into()))?;

    let temp_c = obs
        .get(7)
        .and_then(|v| v.as_f64())
        .ok_or_else(|| AppError::Serialization("Missing temperature at index 7".into()))?;

    let humidity = obs
        .get(8)
        .and_then(|v| v.as_f64())
        .ok_or_else(|| AppError::Serialization("Missing humidity at index 8".into()))?;

    let vpd = calculate_vpd(temp_c, humidity);

    Ok(RawReading {
        temperature_c: temp_c,
        humidity_pct: humidity,
        vpd_kpa: Some(vpd),
    })
}
