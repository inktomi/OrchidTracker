use leptos::prelude::*;
use crate::orchid::{ClimateReading, HabitatWeather, HabitatWeatherSummary};

/// Get the latest climate reading per zone for the current user.
#[server]
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

/// Get historical readings for a specific zone within the last N hours.
#[server]
pub async fn get_zone_history(zone_id: String, hours: u32) -> Result<Vec<ClimateReading>, ServerFnError> {
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

/// Build a formatted climate summary string for the AI scanner.
#[server]
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

/// Test a data source connection by attempting to fetch a reading.
/// Returns a formatted result string on success or an error message.
#[server]
pub async fn test_data_source(provider: String, config_json: String) -> Result<String, ServerFnError> {
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

/// Save a wizard estimation as a climate reading and update zone fields.
#[server]
pub async fn save_wizard_estimation(
    zone_id: String,
    zone_name: String,
    temperature: f64,
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

    // Create climate reading with wizard source
    let mut resp = db()
        .query(
            "CREATE climate_reading SET \
             zone = $zone_id, zone_name = $zone_name, \
             temperature = $temp, humidity = $humidity, \
             vpd = $vpd, source = $source, recorded_at = time::now()"
        )
        .bind(("zone_id", zone_record.clone()))
        .bind(("zone_name", zone_name))
        .bind(("temp", temperature))
        .bind(("humidity", humidity))
        .bind(("vpd", vpd))
        .bind(("source", "wizard".to_string()))
        .await
        .map_err(|e| internal_error("Save wizard reading failed", e))?;

    let errors = resp.take_errors();
    if !errors.is_empty() {
        let err_msg = errors.into_values().map(|e| e.to_string()).collect::<Vec<_>>().join("; ");
        return Err(internal_error("Save wizard reading error", err_msg));
    }

    // Update zone's text fields too
    let temp_range = format!("{:.0}-{:.0}C", temperature - 2.0, temperature + 2.0);
    let humidity_str = format!("{:.0}%", humidity);

    let mut zone_resp = db()
        .query(
            "UPDATE $id SET temperature_range = $temp_range, humidity = $hum WHERE owner = $owner"
        )
        .bind(("id", zone_record))
        .bind(("temp_range", temp_range))
        .bind(("hum", humidity_str))
        .bind(("owner", owner))
        .await
        .map_err(|e| internal_error("Update zone fields failed", e))?;

    let _ = zone_resp.take_errors();

    Ok(())
}

/// Log a manual climate reading for a zone.
#[server]
pub async fn log_manual_reading(
    zone_id: String,
    zone_name: String,
    temperature: f64,
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

/// Test weather API for a lat/lon pair, returning a preview string.
#[server]
pub async fn test_weather_api(latitude: f64, longitude: f64) -> Result<String, ServerFnError> {
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

/// Configure a zone's data source type and config.
#[server]
pub async fn configure_zone_data_source(
    zone_id: String,
    provider: Option<String>,
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

/// Parse the "table:key" user_id string into a SurrealDB RecordId
#[cfg(feature = "ssr")]
fn parse_owner(user_id: &str) -> Result<surrealdb::types::RecordId, ServerFnError> {
    use crate::error::internal_error;
    surrealdb::types::RecordId::parse_simple(user_id)
        .map_err(|e| internal_error("Owner ID parse failed", e))
}

/// Get the latest habitat weather reading for a coordinate pair.
#[server]
pub async fn get_habitat_current(
    latitude: f64,
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

/// Get habitat weather history (summaries + recent raw) for a coordinate pair.
#[server]
pub async fn get_habitat_history(
    latitude: f64,
    longitude: f64,
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

#[cfg(feature = "ssr")]
mod ssr_types {
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
    pub struct ReadingDbRow {
        pub id: surrealdb::types::RecordId,
        pub zone: surrealdb::types::RecordId,
        pub zone_name: String,
        pub temperature: f64,
        pub humidity: f64,
        #[surreal(default)]
        pub vpd: Option<f64>,
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
