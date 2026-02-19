use super::{RawReading, calculate_vpd};
use crate::error::AppError;

/// Fetch the latest observation from a WeatherFlow Tempest station.
///
/// API docs: https://weatherflow.github.io/Tempest/api/
/// GET /swd/rest/observations/station/{station_id}?token={token}
/// Response `obs[0]` can be either:
///   - An object with named keys (e.g. "air_temperature", "relative_humidity")
///   - A positional array where index 7 = air_temperature, index 8 = relative_humidity
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

    let obs = json
        .get("obs")
        .and_then(|o| o.get(0))
        .ok_or_else(|| AppError::Serialization("No observations in Tempest response".into()))?;

    let (temp_c, humidity) = if obs.is_object() {
        // Named-key format: {"air_temperature": 12.6, "relative_humidity": 60, ...}
        let temp = obs.get("air_temperature")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| AppError::Serialization(
                "Missing 'air_temperature' in Tempest observation".into(),
            ))?;
        let hum = obs.get("relative_humidity")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| AppError::Serialization(
                "Missing 'relative_humidity' in Tempest observation".into(),
            ))?;
        (temp, hum)
    } else if let Some(arr) = obs.as_array() {
        // Positional array format: index 7 = temperature, index 8 = humidity
        let temp = arr.get(7)
            .and_then(|v| v.as_f64())
            .ok_or_else(|| AppError::Serialization(format!(
                "Missing temperature at index 7 (array length={})", arr.len(),
            )))?;
        let hum = arr.get(8)
            .and_then(|v| v.as_f64())
            .ok_or_else(|| AppError::Serialization(format!(
                "Missing humidity at index 8 (array length={})", arr.len(),
            )))?;
        (temp, hum)
    } else {
        return Err(AppError::Serialization(format!(
            "Unexpected obs[0] type: {:?}", obs
        )));
    };

    let vpd = calculate_vpd(temp_c, humidity);

    Ok(RawReading {
        temperature_c: temp_c,
        humidity_pct: humidity,
        vpd_kpa: Some(vpd),
    })
}
