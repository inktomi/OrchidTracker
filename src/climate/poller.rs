use crate::db::db;
use surrealdb::types::SurrealValue;
use super::{tempest, ac_infinity};

/// Poll all zones that have a configured data source, fetch readings, and store them.
/// Called periodically by the background task in main.rs.
pub async fn poll_all_zones() {
    let db = db();
    let client = reqwest::Client::new();

    // Query all zones with a data source configured
    let mut response = match db
        .query("SELECT id, name, data_source_type, data_source_config FROM growing_zone WHERE data_source_type IS NOT NULL")
        .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("Climate poll: failed to query zones: {}", e);
            return;
        }
    };

    let errors = response.take_errors();
    if !errors.is_empty() {
        tracing::warn!("Climate poll: zone query errors: {:?}", errors);
        return;
    }

    let zones: Vec<ZoneRow> = match response.take(0) {
        Ok(z) => z,
        Err(e) => {
            tracing::warn!("Climate poll: failed to parse zones: {}", e);
            return;
        }
    };

    if zones.is_empty() {
        tracing::debug!("Climate poll: no zones with data sources configured");
        return;
    }

    tracing::info!("Climate poll: polling {} zones", zones.len());

    for zone in &zones {
        let zone_id = &zone.id;
        let zone_name = &zone.name;
        let source_type = match &zone.data_source_type {
            Some(t) => t.as_str(),
            None => continue,
        };
        let config_str = &zone.data_source_config;

        let reading = match source_type {
            "tempest" => {
                let config: TempestConfig = match serde_json::from_str(config_str) {
                    Ok(c) => c,
                    Err(e) => {
                        tracing::warn!("Climate poll: bad tempest config for zone '{}': {}", zone_name, e);
                        continue;
                    }
                };
                tempest::fetch_tempest_reading(&client, &config.station_id, &config.token).await
            }
            "ac_infinity" => {
                let config: AcInfinityConfig = match serde_json::from_str(config_str) {
                    Ok(c) => c,
                    Err(e) => {
                        tracing::warn!("Climate poll: bad ac_infinity config for zone '{}': {}", zone_name, e);
                        continue;
                    }
                };
                ac_infinity::fetch_ac_infinity_reading(
                    &client,
                    &config.email,
                    &config.password,
                    &config.device_id,
                    config.port,
                )
                .await
            }
            other => {
                tracing::warn!("Climate poll: unknown data source type '{}' for zone '{}'", other, zone_name);
                continue;
            }
        };

        match reading {
            Ok(raw) => {
                if let Err(e) = db
                    .query(
                        "CREATE climate_reading SET \
                         zone = $zone_id, zone_name = $zone_name, \
                         temperature = $temp, humidity = $humidity, \
                         vpd = $vpd, recorded_at = time::now()",
                    )
                    .bind(("zone_id", zone_id.clone()))
                    .bind(("zone_name", zone_name.clone()))
                    .bind(("temp", raw.temperature_c))
                    .bind(("humidity", raw.humidity_pct))
                    .bind(("vpd", raw.vpd_kpa))
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
            Err(e) => {
                tracing::warn!("Climate poll: failed to fetch reading for zone '{}': {}", zone_name, e);
            }
        }
    }

    // Prune readings older than 30 days
    if let Err(e) = db
        .query("DELETE climate_reading WHERE recorded_at < time::now() - 30d")
        .await
    {
        tracing::warn!("Climate poll: failed to prune old readings: {}", e);
    }

    tracing::info!("Climate poll completed");
}

// Internal structs for zone query result and config parsing

#[derive(serde::Deserialize, surrealdb::types::SurrealValue)]
#[surreal(crate = "surrealdb::types")]
struct ZoneRow {
    id: surrealdb::types::RecordId,
    name: String,
    data_source_type: Option<String>,
    data_source_config: String,
}

#[derive(serde::Deserialize)]
pub struct TempestConfig {
    pub station_id: String,
    pub token: String,
}

#[derive(serde::Deserialize)]
pub struct AcInfinityConfig {
    pub email: String,
    pub password: String,
    pub device_id: String,
    #[serde(default = "default_port")]
    pub port: u32,
}

fn default_port() -> u32 {
    1
}
