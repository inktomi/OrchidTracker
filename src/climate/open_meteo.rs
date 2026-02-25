use crate::error::AppError;

/// **What is it?**
/// A data structure representing the parsed response of an Open-Meteo weather API call.
///
/// **Why does it exist?**
/// It exists to provide a clean, standardized format for temperature, humidity, and precipitation data before it is inserted into the `habitat_weather` table.
///
/// **How should it be used?**
/// Instantiate this struct inside the `fetch_habitat_weather` function when returning a successful API reading.
pub struct HabitatReading {
    /// Temperature in Celsius.
    pub temperature_c: f64,
    /// Relative humidity percentage.
    pub humidity_pct: f64,
    /// Precipitation in millimeters.
    pub precipitation_mm: f64,
}

/// **What is it?**
/// A function that fetches the current weather conditions from the Open-Meteo API for a specific geographic coordinate pair.
///
/// **Why does it exist?**
/// It exists to provide a free, reliable source of current meteorological data for outdoor habitats or wild regions where the user does not have physical sensors.
///
/// **How should it be used?**
/// Call this from the habitat polling loop or the manual "Test Weather API" endpoint, passing the target latitude and longitude.
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
