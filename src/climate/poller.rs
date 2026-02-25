use crate::db::db;
use surrealdb::types::SurrealValue;
use super::{tempest, ac_infinity, open_meteo};

/// Poll all zones that have a configured data source, fetch readings, and store them.
/// Called periodically by the background task in main.rs.
///
/// Two-phase approach:
///   Phase A: Device-linked zones — grouped by hardware_device, one API call per device
///   Phase B: Legacy zones — individual zones with data_source_type/data_source_config
pub async fn poll_all_zones() {
    let db = db();
    let client = reqwest::Client::new();

    // ── Phase A: Device-linked zones ──────────────────────────────
    poll_device_linked_zones(db, &client).await;

    // ── Phase B: Legacy zones (data_source_type set, no hardware_device) ──
    poll_legacy_zones(db, &client).await;

    // Prune readings older than 30 days
    if let Err(e) = db
        .query("DELETE climate_reading WHERE recorded_at < time::now() - 30d")
        .await
    {
        tracing::warn!("Climate poll: failed to prune old readings: {}", e);
    }

    tracing::info!("Climate poll completed, checking alerts...");

    // Check condition alerts after storing new readings
    super::alerts::check_and_send_alerts().await;
}

/// Phase A: Fetch all hardware devices, group linked zones by device, make one API call per device.
async fn poll_device_linked_zones(
    db: &surrealdb::Surreal<surrealdb::engine::remote::ws::Client>,
    client: &reqwest::Client,
) {
    // Get all hardware devices
    let mut dev_response = match db
        .query("SELECT id, device_type, config FROM hardware_device")
        .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("Climate poll: failed to query hardware devices: {}", e);
            return;
        }
    };

    let errors = dev_response.take_errors();
    if !errors.is_empty() {
        tracing::debug!("Climate poll: hardware_device query errors (may not exist yet): {:?}", errors);
        return;
    }

    let devices: Vec<DeviceRow> = match dev_response.take(0) {
        Ok(d) => d,
        Err(e) => {
            tracing::debug!("Climate poll: failed to parse hardware devices: {}", e);
            return;
        }
    };

    if devices.is_empty() {
        tracing::debug!("Climate poll: no hardware devices configured");
        return;
    }

    for device in &devices {
        // Get zones linked to this device
        let mut zone_response = match db
            .query("SELECT id, name, hardware_port FROM growing_zone WHERE hardware_device = $dev_id")
            .bind(("dev_id", device.id.clone()))
            .await
        {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!("Climate poll: failed to query zones for device {:?}: {}", device.id, e);
                continue;
            }
        };

        let _ = zone_response.take_errors();
        let linked_zones: Vec<DeviceZoneRow> = match zone_response.take(0) {
            Ok(z) => z,
            Err(e) => {
                tracing::warn!("Climate poll: failed to parse linked zones: {}", e);
                continue;
            }
        };

        if linked_zones.is_empty() {
            continue;
        }

        let config_str = crate::crypto::decrypt_or_raw(&device.config);

        match device.device_type.as_str() {
            "tempest" => {
                let config: TempestConfig = match serde_json::from_str(&config_str) {
                    Ok(c) => c,
                    Err(e) => {
                        tracing::warn!("Climate poll: bad tempest config for device {:?}: {}", device.id, e);
                        continue;
                    }
                };

                // One fetch for all linked zones
                match tempest::fetch_tempest_reading(client, &config.station_id, &config.token).await {
                    Ok(raw) => {
                        tracing::info!(
                            "Climate poll: Tempest device fetch OK, distributing to {} zones",
                            linked_zones.len()
                        );
                        for zone in &linked_zones {
                            store_reading(db, &zone.id, &zone.name, &raw, "tempest").await;
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Climate poll: Tempest fetch failed for device: {}", e);
                    }
                }
            }
            "ac_infinity" => {
                let config: AcInfinityConfig = match serde_json::from_str(&config_str) {
                    Ok(c) => c,
                    Err(e) => {
                        tracing::warn!("Climate poll: bad ac_infinity config for device {:?}: {}", device.id, e);
                        continue;
                    }
                };

                // One fetch, all ports
                match ac_infinity::fetch_ac_infinity_all_ports(
                    client,
                    &config.email,
                    &config.password,
                    &config.device_id,
                ).await {
                    Ok(port_readings) => {
                        tracing::info!(
                            "Climate poll: AC Infinity device fetch OK ({} ports), distributing to {} zones",
                            port_readings.len(),
                            linked_zones.len()
                        );
                        for zone in &linked_zones {
                            let port = zone.hardware_port.unwrap_or(1) as u32;
                            if let Some(raw) = port_readings.get(&port) {
                                store_reading(db, &zone.id, &zone.name, raw, "ac_infinity").await;
                            } else {
                                tracing::warn!(
                                    "Climate poll: no reading for port {} on AC Infinity device for zone '{}'",
                                    port, zone.name
                                );
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Climate poll: AC Infinity fetch failed for device: {}", e);
                    }
                }
            }
            other => {
                tracing::warn!("Climate poll: unknown device type '{}' for device", other);
            }
        }
    }
}

/// Phase B: Poll legacy zones that have data_source_type set but no hardware_device.
async fn poll_legacy_zones(
    db: &surrealdb::Surreal<surrealdb::engine::remote::ws::Client>,
    client: &reqwest::Client,
) {
    let mut response = match db
        .query(
            "SELECT id, name, data_source_type, data_source_config FROM growing_zone \
             WHERE data_source_type IS NOT NULL AND hardware_device IS NONE"
        )
        .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("Climate poll: failed to query legacy zones: {}", e);
            return;
        }
    };

    let errors = response.take_errors();
    if !errors.is_empty() {
        tracing::warn!("Climate poll: legacy zone query errors: {:?}", errors);
        return;
    }

    let zones: Vec<ZoneRow> = match response.take(0) {
        Ok(z) => z,
        Err(e) => {
            tracing::warn!("Climate poll: failed to parse legacy zones: {}", e);
            return;
        }
    };

    if zones.is_empty() {
        tracing::debug!("Climate poll: no legacy zones with data sources configured");
        return;
    }

    tracing::info!("Climate poll: polling {} legacy zones", zones.len());

    for zone in &zones {
        let zone_id = &zone.id;
        let zone_name = &zone.name;
        let source_type = match &zone.data_source_type {
            Some(t) => t.as_str(),
            None => continue,
        };
        let config_str = crate::crypto::decrypt_or_raw(&zone.data_source_config);

        let reading = match source_type {
            "tempest" => {
                let config: TempestConfig = match serde_json::from_str(&config_str) {
                    Ok(c) => c,
                    Err(e) => {
                        tracing::warn!("Climate poll: bad tempest config for zone '{}': {}", zone_name, e);
                        continue;
                    }
                };
                tempest::fetch_tempest_reading(client, &config.station_id, &config.token).await
            }
            "ac_infinity" => {
                let config: AcInfinityConfig = match serde_json::from_str(&config_str) {
                    Ok(c) => c,
                    Err(e) => {
                        tracing::warn!("Climate poll: bad ac_infinity config for zone '{}': {}", zone_name, e);
                        continue;
                    }
                };
                ac_infinity::fetch_ac_infinity_reading(
                    client,
                    &config.email,
                    &config.password,
                    &config.device_id,
                    config.port,
                )
                .await
            }
            "weather_api" => {
                let config: WeatherApiConfig = match serde_json::from_str(&config_str) {
                    Ok(c) => c,
                    Err(e) => {
                        tracing::warn!("Climate poll: bad weather_api config for zone '{}': {}", zone_name, e);
                        continue;
                    }
                };
                open_meteo::fetch_habitat_weather(client, config.latitude, config.longitude)
                    .await
                    .map(|h| super::RawReading {
                        temperature_c: h.temperature_c,
                        humidity_pct: h.humidity_pct,
                        vpd_kpa: Some(super::calculate_vpd(h.temperature_c, h.humidity_pct)),
                        precipitation_mm: Some(h.precipitation_mm),
                    })
            }
            other => {
                tracing::warn!("Climate poll: unknown data source type '{}' for zone '{}'", other, zone_name);
                continue;
            }
        };

        match reading {
            Ok(raw) => {
                store_reading(db, zone_id, zone_name, &raw, source_type).await;
            }
            Err(e) => {
                tracing::warn!("Climate poll: failed to fetch reading for zone '{}': {}", zone_name, e);
            }
        }
    }
}

/// Shared helper: store a climate reading in the database.
async fn store_reading(
    db: &surrealdb::Surreal<surrealdb::engine::remote::ws::Client>,
    zone_id: &surrealdb::types::RecordId,
    zone_name: &str,
    raw: &super::RawReading,
    source: &str,
) {
    if let Err(e) = db
        .query(
            "CREATE climate_reading SET \
             zone = $zone_id, zone_name = $zone_name, \
             temperature = $temp, humidity = $humidity, \
             vpd = $vpd, precipitation = $precip, \
             source = $source, recorded_at = time::now()",
        )
        .bind(("zone_id", zone_id.clone()))
        .bind(("zone_name", zone_name.to_string()))
        .bind(("temp", raw.temperature_c))
        .bind(("humidity", raw.humidity_pct))
        .bind(("vpd", raw.vpd_kpa))
        .bind(("precip", raw.precipitation_mm))
        .bind(("source", source.to_string()))
        .await
    {
        tracing::warn!("Climate poll: failed to store reading for zone '{}': {}", zone_name, e);
    } else {
        tracing::info!(
            "Climate poll: stored reading for '{}': {:.1}C, {:.1}%",
            zone_name,
            raw.temperature_c,
            raw.humidity_pct
        );
    }
}

// ── Internal structs ──────────────────────────────────────────────

#[derive(serde::Deserialize, surrealdb::types::SurrealValue)]
#[surreal(crate = "surrealdb::types")]
struct DeviceRow {
    id: surrealdb::types::RecordId,
    device_type: String,
    config: String,
}

#[derive(serde::Deserialize, surrealdb::types::SurrealValue)]
#[surreal(crate = "surrealdb::types")]
struct DeviceZoneRow {
    id: surrealdb::types::RecordId,
    name: String,
    #[surreal(default)]
    hardware_port: Option<i32>,
}

#[derive(serde::Deserialize, surrealdb::types::SurrealValue)]
#[surreal(crate = "surrealdb::types")]
struct ZoneRow {
    id: surrealdb::types::RecordId,
    name: String,
    data_source_type: Option<String>,
    data_source_config: String,
}

/// Configuration for a Tempest weather station.
#[derive(serde::Deserialize)]
pub struct TempestConfig {
    /// The station's unique identifier.
    pub station_id: String,
    /// API token for accessing the station data.
    pub token: String,
}

/// Configuration for an AC Infinity device.
#[derive(serde::Deserialize)]
pub struct AcInfinityConfig {
    /// User email for AC Infinity login.
    pub email: String,
    /// User password for AC Infinity login.
    pub password: String,
    /// The device's unique identifier.
    pub device_id: String,
    /// The hardware port on the device to read from.
    #[serde(default = "default_port")]
    pub port: u32,
}

fn default_port() -> u32 {
    1
}

/// Configuration for an open weather API integration.
#[derive(serde::Deserialize)]
pub struct WeatherApiConfig {
    /// Latitude coordinate.
    pub latitude: f64,
    /// Longitude coordinate.
    pub longitude: f64,
}
