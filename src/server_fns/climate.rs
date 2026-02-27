use leptos::prelude::*;
use crate::orchid::{ClimateReading, HabitatWeather, HabitatWeatherSummary};

/// **What is it?**
/// A server function that retrieves the single most recent climate reading for every zone owned by the user.
///
/// **Why does it exist?**
/// It exists to quickly populate the "Current Conditions" overview on the dashboard without fetching massive amounts of historical data.
///
/// **How should it be used?**
/// Call this on dashboard load or via a polling interval to display live temperature and humidity metrics for all zones at once.
#[server]
#[tracing::instrument(level = "info", skip_all)]
pub async fn get_current_readings() -> Result<Vec<ClimateReading>, ServerFnError> {
    use crate::auth::require_auth;
    use crate::db::db;
    use crate::error::internal_error;

    let user_id = require_auth().await?;
    let owner = parse_owner(&user_id)?;

    // Get all zones for this user (includes wizard/manual readings too)
    let mut zone_resp = db()
        .query("SELECT id, name FROM growing_zone WHERE owner = $owner")
        .bind(("owner", owner))
        .await
        .map_err(|e| internal_error("Get climate zones query failed", e))?;

    let errors = zone_resp.take_errors();
    if !errors.is_empty() {
        let err_msg = errors.into_values().map(|e| e.to_string()).collect::<Vec<_>>().join("; ");
        return Err(internal_error("Get climate zones query error", err_msg));
    }

    let zones: Vec<ZoneIdRow> = zone_resp.take(0)
        .map_err(|e| internal_error("Get climate zones parse failed", e))?;

    let mut readings = Vec::new();

    for zone in &zones {
        let mut resp = db()
            .query(
                "SELECT * FROM climate_reading WHERE zone = $zone_id ORDER BY recorded_at DESC LIMIT 1"
            )
            .bind(("zone_id", zone.id.clone()))
            .await
            .map_err(|e| internal_error("Get reading query failed", e))?;

        let _ = resp.take_errors();

        let reading: Option<ReadingDbRow> = resp.take(0).unwrap_or(None);
        if let Some(row) = reading {
            readings.push(row.into_climate_reading());
        }
    }

    Ok(readings)
}

/// **What is it?**
/// A server function that retrieves the historical climate readings for a specific zone over the last specified number of hours.
///
/// **Why does it exist?**
/// It exists to provide the time-series data necessary to render line charts showing temperature, humidity, and VPD trends over time.
///
/// **How should it be used?**
/// Call this from a climate dashboard component, passing the desired `zone_id` and the `hours` lookback period (e.g., 24 or 48) to plot the historical data points.
#[server]
#[tracing::instrument(level = "info", skip_all)]
pub async fn get_zone_history(
    /// The unique identifier of the zone.
    zone_id: String, 
    /// The number of hours of history to fetch.
    hours: u32
) -> Result<Vec<ClimateReading>, ServerFnError> {
    use crate::auth::require_auth;
    use crate::db::db;
    use crate::error::internal_error;

    let _user_id = require_auth().await?;

    let zone_record = surrealdb::types::RecordId::parse_simple(&zone_id)
        .map_err(|e| internal_error("Zone ID parse failed", e))?;
    let duration_str = format!("{}h", hours);

    let mut response = db()
        .query(
            "SELECT * FROM climate_reading WHERE zone = $zone_id AND recorded_at > time::now() - $duration ORDER BY recorded_at ASC"
        )
        .bind(("zone_id", zone_record))
        .bind(("duration", duration_str))
        .await
        .map_err(|e| internal_error("Get zone history query failed", e))?;

    let errors = response.take_errors();
    if !errors.is_empty() {
        let err_msg = errors.into_values().map(|e| e.to_string()).collect::<Vec<_>>().join("; ");
        return Err(internal_error("Get zone history query error", err_msg));
    }

    let rows: Vec<ReadingDbRow> = response.take(0)
        .map_err(|e| internal_error("Get zone history parse failed", e))?;

    Ok(rows.into_iter().map(|r| r.into_climate_reading()).collect())
}

/// **What is it?**
/// A server function that builds a formatted climate summary string containing the latest readings from all user zones.
///
/// **Why does it exist?**
/// It exists to inject the user's real-time environmental context into the AI scanner prompt, ensuring plant care suggestions are tailored to actual conditions.
///
/// **How should it be used?**
/// Call this internally right before invoking the Gemini or Claude APIs in the scanner process to construct the `climate_summary` prompt section.
#[server]
#[tracing::instrument(level = "info", skip_all)]
pub async fn get_climate_summary_for_scanner() -> Result<String, ServerFnError> {
    let readings = get_current_readings().await?;

    if readings.is_empty() {
        return Ok("No live climate data available".to_string());
    }

    let parts: Vec<String> = readings
        .iter()
        .map(|r| {
            let vpd_str = r.vpd.map(|v| format!(", {:.2} kPa VPD", v)).unwrap_or_default();
            format!(
                "{}: {:.1}C, {:.1}% Humidity{}",
                r.zone_name, r.temperature, r.humidity, vpd_str
            )
        })
        .collect();

    Ok(parts.join(" | "))
}

/// **What is it?**
/// A server function that tests a data source connection by attempting to fetch a live reading.
///
/// **Why does it exist?**
/// It exists to immediately validate user-provided API keys or connection strings, providing feedback before a broken configuration is saved to the database.
///
/// **How should it be used?**
/// Call this when the user clicks a "Test Connection" button in a zone's configuration form, providing the unencrypted config JSON.
#[server]
#[tracing::instrument(level = "info", skip_all)]
pub async fn test_data_source(
    /// The name of the data provider (e.g., "tempest", "ac_infinity").
    provider: String, 
    /// The JSON configuration string for the data source.
    config_json: String
) -> Result<String, ServerFnError> {
    use crate::auth::require_auth;

    require_auth().await?;

    let client = reqwest::Client::new();

    match provider.as_str() {
        "tempest" => {
            let config: crate::climate::poller::TempestConfig = serde_json::from_str(&config_json)
                .map_err(|e| ServerFnError::new(format!("Invalid Tempest config: {}", e)))?;

            let reading = crate::climate::tempest::fetch_tempest_reading(
                &client,
                &config.station_id,
                &config.token,
            )
            .await
            .map_err(|e| ServerFnError::new(format!("Tempest connection failed: {}", e)))?;

            let vpd_str = reading.vpd_kpa.map(|v| format!(", {:.2} kPa VPD", v)).unwrap_or_default();
            Ok(format!(
                "Connected! Current: {:.1}C, {:.1}% Humidity{}",
                reading.temperature_c, reading.humidity_pct, vpd_str
            ))
        }
        "ac_infinity" => {
            let config: crate::climate::poller::AcInfinityConfig = serde_json::from_str(&config_json)
                .map_err(|e| ServerFnError::new(format!("Invalid AC Infinity config: {}", e)))?;

            let reading = crate::climate::ac_infinity::fetch_ac_infinity_reading(
                &client,
                &config.email,
                &config.password,
                &config.device_id,
                config.port,
            )
            .await
            .map_err(|e| ServerFnError::new(format!("AC Infinity connection failed: {}", e)))?;

            let vpd_str = reading.vpd_kpa.map(|v| format!(", {:.2} kPa VPD", v)).unwrap_or_default();
            Ok(format!(
                "Connected! Current: {:.1}C, {:.1}% Humidity{}",
                reading.temperature_c, reading.humidity_pct, vpd_str
            ))
        }
        "weather_api" => {
            let config: crate::climate::poller::WeatherApiConfig = serde_json::from_str(&config_json)
                .map_err(|e| ServerFnError::new(format!("Invalid Weather API config: {}", e)))?;

            let reading = crate::climate::open_meteo::fetch_habitat_weather(
                &client,
                config.latitude,
                config.longitude,
            )
            .await
            .map_err(|e| ServerFnError::new(format!("Weather API connection failed: {}", e)))?;

            Ok(format!(
                "Connected! Current: {:.1}C, {:.1}% Humidity, {:.1}mm precip",
                reading.temperature_c, reading.humidity_pct, reading.precipitation_mm
            ))
        }
        _ => Err(ServerFnError::new(format!("Unknown provider: {}", provider))),
    }
}

/// **What is it?**
/// A server function that saves an estimated set of temperature and humidity values to a specific zone, derived from a "wizard" or manual input process.
///
/// **Why does it exist?**
/// It exists for users who do not have automated sensors (like Tempest or AC Infinity) but still want to store an approximate baseline of their climate data to improve AI care suggestions.
///
/// **How should it be used?**
/// Call this from the zone configuration modal when a user finishes the "Estimate conditions" wizard form.
#[server]
#[tracing::instrument(level = "info", skip_all)]
pub async fn save_wizard_estimation(
    /// The unique identifier of the zone.
    zone_id: String,
    /// The name of the zone.
    zone_name: String,
    /// The estimated temperature in Celsius.
    temperature: f64,
    /// The estimated humidity percentage.
    humidity: f64,
) -> Result<(), ServerFnError> {
    use crate::auth::require_auth;
    use crate::db::db;
    use crate::error::internal_error;
    use crate::climate::calculate_vpd;

    let user_id = require_auth().await?;
    let owner = parse_owner(&user_id)?;
    let zone_record = surrealdb::types::RecordId::parse_simple(&zone_id)
        .map_err(|e| internal_error("Zone ID parse failed", e))?;

    let vpd = calculate_vpd(temperature, humidity);

    // Create climate reading and update zone text fields atomically
    let temp_range = format!("{:.0}-{:.0}C", temperature - 2.0, temperature + 2.0);
    let humidity_str = format!("{:.0}%", humidity);

    let mut resp = db()
        .query(
            "BEGIN TRANSACTION; \
             CREATE climate_reading SET \
                 zone = $zone_id, zone_name = $zone_name, \
                 temperature = $temp, humidity = $humidity, \
                 vpd = $vpd, source = $source, recorded_at = time::now(); \
             UPDATE $zone_id SET temperature_range = $temp_range, humidity = $hum WHERE owner = $owner; \
             COMMIT TRANSACTION;"
        )
        .bind(("zone_id", zone_record))
        .bind(("zone_name", zone_name))
        .bind(("temp", temperature))
        .bind(("humidity", humidity))
        .bind(("vpd", vpd))
        .bind(("source", "wizard".to_string()))
        .bind(("temp_range", temp_range))
        .bind(("hum", humidity_str))
        .bind(("owner", owner))
        .await
        .map_err(|e| internal_error("Save wizard estimation failed", e))?;

    let errors = resp.take_errors();
    if !errors.is_empty() {
        let err_msg = errors.into_values().map(|e| e.to_string()).collect::<Vec<_>>().join("; ");
        return Err(internal_error("Save wizard estimation error", err_msg));
    }

    Ok(())
}

/// **What is it?**
/// A server function that logs a one-off, manually inputted climate reading into a zone's history.
///
/// **Why does it exist?**
/// It exists to allow a user to use analog thermometers or simple, non-connected hygrometers and input their findings manually to track a zone's conditions over time.
///
/// **How should it be used?**
/// Call this from a "Quick Add Reading" button near a specific zone, allowing the user to simply enter a temperature and humidity without going through the full wizard.
#[server]
#[tracing::instrument(level = "info", skip_all)]
pub async fn log_manual_reading(
    /// The unique identifier of the zone.
    zone_id: String,
    /// The name of the zone.
    zone_name: String,
    /// The manually recorded temperature in Celsius.
    temperature: f64,
    /// The manually recorded humidity percentage.
    humidity: f64,
) -> Result<(), ServerFnError> {
    use crate::auth::require_auth;
    use crate::db::db;
    use crate::error::internal_error;
    use crate::climate::calculate_vpd;

    let _user_id = require_auth().await?;
    let zone_record = surrealdb::types::RecordId::parse_simple(&zone_id)
        .map_err(|e| internal_error("Zone ID parse failed", e))?;

    let vpd = calculate_vpd(temperature, humidity);

    let mut resp = db()
        .query(
            "CREATE climate_reading SET \
             zone = $zone_id, zone_name = $zone_name, \
             temperature = $temp, humidity = $humidity, \
             vpd = $vpd, source = $source, recorded_at = time::now()"
        )
        .bind(("zone_id", zone_record))
        .bind(("zone_name", zone_name))
        .bind(("temp", temperature))
        .bind(("humidity", humidity))
        .bind(("vpd", vpd))
        .bind(("source", "manual".to_string()))
        .await
        .map_err(|e| internal_error("Log manual reading failed", e))?;

    let errors = resp.take_errors();
    if !errors.is_empty() {
        let err_msg = errors.into_values().map(|e| e.to_string()).collect::<Vec<_>>().join("; ");
        return Err(internal_error("Log manual reading error", err_msg));
    }

    Ok(())
}

/// **What is it?**
/// A server function that tests a weather API connection for a specific latitude and longitude, returning a preview string.
///
/// **Why does it exist?**
/// It exists to ensure that the provided coordinates are valid and can successfully fetch habitat weather from Open-Meteo before saving them to a zone configuration.
///
/// **How should it be used?**
/// Call this from a "Test coordinates" button near the map input on a zone's configuration page.
#[server]
#[tracing::instrument(level = "info", skip_all)]
pub async fn test_weather_api(
    /// The latitude coordinate.
    latitude: f64, 
    /// The longitude coordinate.
    longitude: f64
) -> Result<String, ServerFnError> {
    use crate::auth::require_auth;

    require_auth().await?;

    let client = reqwest::Client::new();
    let reading = crate::climate::open_meteo::fetch_habitat_weather(&client, latitude, longitude)
        .await
        .map_err(|e| ServerFnError::new(format!("Weather API failed: {}", e)))?;

    Ok(format!(
        "Current: {:.1}C, {:.1}% Humidity, {:.1}mm precip",
        reading.temperature_c, reading.humidity_pct, reading.precipitation_mm
    ))
}

/// **What is it?**
/// A server function that configures a zone's data source type and encrypts its configuration string.
///
/// **Why does it exist?**
/// It exists to securely bind a zone to a specific polling data source (like a Tempest station), persisting its credentials so the backend loop can continuously pull readings.
///
/// **How should it be used?**
/// Call this from a zone settings form when the user chooses "Tempest" or "AC Infinity" and inputs their API keys or IPs.
#[server]
#[tracing::instrument(level = "info", skip_all)]
pub async fn configure_zone_data_source(
    /// The unique identifier of the zone.
    zone_id: String,
    /// The data provider name, if any.
    provider: Option<String>,
    /// The JSON configuration string for the data source.
    config_json: String,
) -> Result<(), ServerFnError> {
    use crate::auth::require_auth;
    use crate::db::db;
    use crate::error::internal_error;
    use crate::crypto::encrypt;

    let user_id = require_auth().await?;
    let owner = parse_owner(&user_id)?;
    let zone_record = surrealdb::types::RecordId::parse_simple(&zone_id)
        .map_err(|e| internal_error("Zone ID parse failed", e))?;

    let stored_config = if config_json.is_empty() {
        config_json
    } else {
        encrypt(&config_json).map_err(|e| internal_error("Encrypt config failed", e))?
    };

    let mut response = db()
        .query(
            "UPDATE $id SET data_source_type = $provider, data_source_config = $config WHERE owner = $owner RETURN *"
        )
        .bind(("id", zone_record))
        .bind(("owner", owner))
        .bind(("provider", provider))
        .bind(("config", stored_config))
        .await
        .map_err(|e| internal_error("Configure data source query failed", e))?;

    let errors = response.take_errors();
    if !errors.is_empty() {
        let err_msg = errors.into_values().map(|e| e.to_string()).collect::<Vec<_>>().join("; ");
        return Err(internal_error("Configure data source query error", err_msg));
    }

    Ok(())
}

/// **What is it?**
/// A utility function that parses the "table:key" user_id string into a SurrealDB RecordId.
///
/// **Why does it exist?**
/// It exists to provide a common, error-checked way to extract the authenticated user's ID for database constraints across the climate module.
///
/// **How should it be used?**
/// Call this inside server functions after `require_auth` to obtain the `RecordId` needed for the `owner` field in database queries.
#[cfg(feature = "ssr")]
pub(crate) fn parse_owner(user_id: &str) -> Result<surrealdb::types::RecordId, ServerFnError> {
    use crate::error::internal_error;
    surrealdb::types::RecordId::parse_simple(user_id)
        .map_err(|e| internal_error("Owner ID parse failed", e))
}

/// **What is it?**
/// A server function that retrieves the latest habitat weather reading for a specific geographic coordinate pair.
///
/// **Why does it exist?**
/// It exists to provide the frontend with the live weather conditions (temperature, humidity, precipitation) of a specific outdoor area, like a plant's native habitat.
///
/// **How should it be used?**
/// Call this from the "Native Habitat" view of an orchid to display what the weather is doing *right now* in that plant's natural range.
#[server]
#[tracing::instrument(level = "info", skip_all)]
pub async fn get_habitat_current(
    /// The latitude coordinate.
    latitude: f64,
    /// The longitude coordinate.
    longitude: f64,
) -> Result<Option<HabitatWeather>, ServerFnError> {
    use crate::auth::require_auth;
    use crate::db::db;
    use crate::error::internal_error;

    require_auth().await?;

    // Round to 2 decimals to match poller grouping
    let lat = (latitude * 100.0).round() / 100.0;
    let lon = (longitude * 100.0).round() / 100.0;

    let mut response = db()
        .query(
            "SELECT temperature, humidity, precipitation, recorded_at \
             FROM habitat_weather \
             WHERE latitude = $lat AND longitude = $lon \
             ORDER BY recorded_at DESC LIMIT 1"
        )
        .bind(("lat", lat))
        .bind(("lon", lon))
        .await
        .map_err(|e| internal_error("Get habitat current query failed", e))?;

    let _ = response.take_errors();

    let row: Option<HabitatWeatherDbRow> = response.take(0).unwrap_or(None);
    Ok(row.map(|r| r.into_habitat_weather()))
}

/// **What is it?**
/// A server function that retrieves the habitat weather history summaries (like average, minimum, maximum temperature) over a specified number of days for a given coordinate pair.
///
/// **Why does it exist?**
/// It exists to provide the time-series summary data necessary to render long-term trend charts of a plant's native habitat conditions.
///
/// **How should it be used?**
/// Call this from a climate trend component, passing the `latitude`, `longitude`, and the `days` lookback period to plot the historical data points.
#[server]
#[tracing::instrument(level = "info", skip_all)]
pub async fn get_habitat_history(
    /// The latitude coordinate.
    latitude: f64,
    /// The longitude coordinate.
    longitude: f64,
    /// The number of days of history to fetch.
    days: u32,
) -> Result<Vec<HabitatWeatherSummary>, ServerFnError> {
    use crate::auth::require_auth;
    use crate::db::db;
    use crate::error::internal_error;

    require_auth().await?;

    let lat = (latitude * 100.0).round() / 100.0;
    let lon = (longitude * 100.0).round() / 100.0;
    let duration_str = format!("{}d", days);

    let mut response = db()
        .query(
            "SELECT period_type, period_start, avg_temperature, min_temperature, \
                    max_temperature, avg_humidity, total_precipitation, sample_count \
             FROM habitat_weather_summary \
             WHERE latitude = $lat AND longitude = $lon \
                   AND period_start > time::now() - $duration \
             ORDER BY period_start DESC"
        )
        .bind(("lat", lat))
        .bind(("lon", lon))
        .bind(("duration", duration_str))
        .await
        .map_err(|e| internal_error("Get habitat history query failed", e))?;

    let errors = response.take_errors();
    if !errors.is_empty() {
        let err_msg = errors.into_values().map(|e| e.to_string()).collect::<Vec<_>>().join("; ");
        return Err(internal_error("Get habitat history query error", err_msg));
    }

    let rows: Vec<HabitatSummaryDbRow> = response.take(0)
        .map_err(|e| internal_error("Get habitat history parse failed", e))?;

    Ok(rows.into_iter().map(|r| r.into_summary()).collect())
}

/// **What is it?**
/// A server function that retrieves climate snapshots (aggregated 48-hour data) for all zones the current user owns.
///
/// **Why does it exist?**
/// It exists to provide a high-level, performant overview of the recent environmental conditions across all of a user's growing locations at once, without running N separate queries.
///
/// **How should it be used?**
/// Call this from a dashboard component that needs to display aggregate metrics like average temperature, VPD, or humidity for all zones.
#[server]
#[tracing::instrument(level = "info", skip_all)]
pub async fn get_all_zone_snapshots() -> Result<Vec<crate::watering::ClimateSnapshot>, ServerFnError> {
    use crate::auth::require_auth;
    use crate::db::db;
    use crate::error::internal_error;
    use std::collections::HashMap;

    let user_id = require_auth().await?;
    let owner = parse_owner(&user_id)?;

    // Get all zones for this user with their location type
    let mut zone_resp = db()
        .query("SELECT id, name, location_type FROM growing_zone WHERE owner = $owner")
        .bind(("owner", owner))
        .await
        .map_err(|e| internal_error("Get zones for snapshots failed", e))?;

    let _ = zone_resp.take_errors();
    let zones: Vec<ZoneWithType> = zone_resp.take(0)
        .map_err(|e| internal_error("Parse zones for snapshots failed", e))?;

    if zones.is_empty() {
        return Ok(Vec::new());
    }

    // Batch query: get all readings from the last 48 hours for all of the user's zones
    let zone_ids: Vec<surrealdb::types::RecordId> = zones.iter().map(|z| z.id.clone()).collect();
    let mut reading_resp = db()
        .query(
            "SELECT * FROM climate_reading WHERE zone IN $zone_ids AND recorded_at > time::now() - 48h ORDER BY recorded_at DESC"
        )
        .bind(("zone_ids", zone_ids))
        .await
        .map_err(|e| internal_error("Get readings for snapshots failed", e))?;

    let _ = reading_resp.take_errors();
    let rows: Vec<ReadingDbRow> = reading_resp.take(0).unwrap_or_default();

    // Group readings by zone_id
    let mut by_zone: HashMap<String, Vec<crate::orchid::ClimateReading>> = HashMap::new();
    for row in rows {
        let reading = row.into_climate_reading();
        by_zone.entry(reading.zone_id.clone()).or_default().push(reading);
    }

    // Build location_type lookup by zone ID
    let zone_outdoor: HashMap<String, bool> = zones.iter().map(|z| {
        let is_outdoor = z.location_type.as_deref() == Some("Outdoor");
        (crate::server_fns::auth::record_id_to_string(&z.id), is_outdoor)
    }).collect();

    // Build snapshots
    let mut snapshots = Vec::new();
    for (zone_id, readings) in &by_zone {
        let is_outdoor = zone_outdoor.get(zone_id).copied().unwrap_or(false);
        let zone_name = readings.first().map(|r| r.zone_name.as_str()).unwrap_or("Unknown");
        if let Some(snap) = crate::watering::ClimateSnapshot::from_readings(zone_name, readings, is_outdoor) {
            snapshots.push(snap);
        }
    }

    Ok(snapshots)
}

#[cfg(feature = "ssr")]
pub(crate) mod ssr_types {
    use surrealdb::types::SurrealValue;
    use crate::orchid::{ClimateReading, HabitatWeather, HabitatWeatherSummary};

    use crate::server_fns::auth::record_id_to_string;

    #[derive(serde::Deserialize, SurrealValue)]
    #[surreal(crate = "surrealdb::types")]
    pub struct ZoneIdRow {
        pub id: surrealdb::types::RecordId,
        pub name: String,
    }

    #[derive(serde::Deserialize, SurrealValue)]
    #[surreal(crate = "surrealdb::types")]
    pub struct ZoneWithType {
        pub id: surrealdb::types::RecordId,
        pub name: String,
        #[surreal(default)]
        pub location_type: Option<String>,
    }

    #[derive(serde::Deserialize, SurrealValue)]
    #[surreal(crate = "surrealdb::types")]
    pub struct ReadingDbRow {
        pub id: surrealdb::types::RecordId,
        pub zone: surrealdb::types::RecordId,
        pub zone_name: String,
        pub temperature: f64,
        pub humidity: f64,
        #[surreal(default)]
        pub vpd: Option<f64>,
        #[surreal(default)]
        pub precipitation: Option<f64>,
        #[surreal(default)]
        pub source: Option<String>,
        pub recorded_at: chrono::DateTime<chrono::Utc>,
    }

    impl ReadingDbRow {
        pub fn into_climate_reading(self) -> ClimateReading {
            ClimateReading {
                id: record_id_to_string(&self.id),
                zone_id: record_id_to_string(&self.zone),
                zone_name: self.zone_name,
                temperature: self.temperature,
                humidity: self.humidity,
                vpd: self.vpd,
                precipitation: self.precipitation,
                source: self.source,
                recorded_at: self.recorded_at,
            }
        }
    }

    #[derive(serde::Deserialize, SurrealValue)]
    #[surreal(crate = "surrealdb::types")]
    pub struct HabitatWeatherDbRow {
        pub temperature: f64,
        pub humidity: f64,
        #[surreal(default)]
        pub precipitation: f64,
        pub recorded_at: chrono::DateTime<chrono::Utc>,
    }

    impl HabitatWeatherDbRow {
        pub fn into_habitat_weather(self) -> HabitatWeather {
            HabitatWeather {
                temperature: self.temperature,
                humidity: self.humidity,
                precipitation: self.precipitation,
                recorded_at: self.recorded_at,
            }
        }
    }

    #[derive(serde::Deserialize, SurrealValue)]
    #[surreal(crate = "surrealdb::types")]
    pub struct HabitatSummaryDbRow {
        pub period_type: String,
        pub period_start: chrono::DateTime<chrono::Utc>,
        pub avg_temperature: f64,
        pub min_temperature: f64,
        pub max_temperature: f64,
        pub avg_humidity: f64,
        #[surreal(default)]
        pub total_precipitation: f64,
        pub sample_count: i64,
    }

    impl HabitatSummaryDbRow {
        pub fn into_summary(self) -> HabitatWeatherSummary {
            HabitatWeatherSummary {
                period_type: self.period_type,
                period_start: self.period_start,
                avg_temperature: self.avg_temperature,
                min_temperature: self.min_temperature,
                max_temperature: self.max_temperature,
                avg_humidity: self.avg_humidity,
                total_precipitation: self.total_precipitation,
                sample_count: self.sample_count as u32,
            }
        }
    }
}

#[cfg(feature = "ssr")]
use ssr_types::*;

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::ssr_types::ZoneWithType;
    use surrealdb::engine::local::Mem;
    use surrealdb::Surreal;

    #[tokio::test]
    async fn test_zone_with_type_deserialization() {
        let db = Surreal::new::<Mem>(()).await.unwrap();
        db.use_ns("test").use_db("test").await.unwrap();

        // This simulates the exact state that caused the crash!
        let mut response = db.query("CREATE type::record('growing_zone', '1') SET name = 'Test', location_type = 'Outdoor'")
            .await
            .unwrap();
            
        let _ = response.take_errors();
        
        // This line used to crash due to LocationType enum deserialization rules!
        let mut get_resp = db.query("SELECT * FROM type::record('growing_zone', '1')").await.unwrap();
        let zone: Option<ZoneWithType> = get_resp.take(0).unwrap();
        
        let z = zone.unwrap();
        assert_eq!(z.name, "Test");
        assert_eq!(z.location_type, Some("Outdoor".to_string()));
    }
}
