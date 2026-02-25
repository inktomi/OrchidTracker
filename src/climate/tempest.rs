use super::{RawReading, calculate_vpd};
use crate::error::AppError;

/// **What is it?**
/// A function that fetches the latest environmental observation from a WeatherFlow Tempest station via their REST API.
///
/// **Why does it exist?**
/// It exists to integrate user-owned, high-accuracy local weather stations into the system, parsing both named-key and positional-array JSON response formats from Tempest.
///
/// **How should it be used?**
/// Call this from the background polling task, passing the configured station ID and personal access token, to extract the current temperature, humidity, and calculate the VPD.
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
        precipitation_mm: None,
    })
}
