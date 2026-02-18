use leptos::prelude::*;
use crate::orchid::ClimateReading;

/// Get the latest climate reading per zone for the current user.
#[server]
pub async fn get_current_readings() -> Result<Vec<ClimateReading>, ServerFnError> {
    use crate::auth::require_auth;
    use crate::db::db;
    use crate::error::internal_error;

    let user_id = require_auth().await?;
    let owner = parse_owner(&user_id)?;

    // Get all zones for this user that have a data source
    let mut zone_resp = db()
        .query("SELECT id, name FROM growing_zone WHERE owner = $owner AND data_source_type IS NOT NULL")
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

    let duration_str = format!("{}h", hours);

    let mut response = db()
        .query(
            "SELECT * FROM climate_reading WHERE zone = $zone_id AND recorded_at > time::now() - $duration ORDER BY recorded_at ASC"
        )
        .bind(("zone_id", zone_id))
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
        _ => Err(ServerFnError::new(format!("Unknown provider: {}", provider))),
    }
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

    let user_id = require_auth().await?;
    let owner = parse_owner(&user_id)?;

    let mut response = db()
        .query(
            "UPDATE $id SET data_source_type = $provider, data_source_config = $config WHERE owner = $owner RETURN *"
        )
        .bind(("id", zone_id.clone()))
        .bind(("owner", owner))
        .bind(("provider", provider))
        .bind(("config", config_json))
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

#[cfg(feature = "ssr")]
mod ssr_types {
    use surrealdb::types::SurrealValue;
    use crate::orchid::ClimateReading;

    #[derive(serde::Deserialize, SurrealValue)]
    #[surreal(crate = "surrealdb::types")]
    pub struct ZoneIdRow {
        pub id: String,
        pub name: String,
    }

    #[derive(serde::Deserialize, SurrealValue)]
    #[surreal(crate = "surrealdb::types")]
    pub struct ReadingDbRow {
        pub id: String,
        pub zone: String,
        pub zone_name: String,
        pub temperature: f64,
        pub humidity: f64,
        #[surreal(default)]
        pub vpd: Option<f64>,
        pub recorded_at: chrono::DateTime<chrono::Utc>,
    }

    impl ReadingDbRow {
        pub fn into_climate_reading(self) -> ClimateReading {
            ClimateReading {
                id: self.id,
                zone_id: self.zone,
                zone_name: self.zone_name,
                temperature: self.temperature,
                humidity: self.humidity,
                vpd: self.vpd,
                recorded_at: self.recorded_at,
            }
        }
    }
}

#[cfg(feature = "ssr")]
use ssr_types::*;
