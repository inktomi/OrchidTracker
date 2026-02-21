use leptos::prelude::*;
use crate::orchid::HardwareDevice;

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
    use crate::orchid::HardwareDevice;
    use crate::server_fns::auth::record_id_to_string;

    #[derive(serde::Deserialize, SurrealValue)]
    #[surreal(crate = "surrealdb::types")]
    pub struct HardwareDeviceDbRow {
        pub id: surrealdb::types::RecordId,
        pub name: String,
        pub device_type: String,
        #[surreal(default)]
        pub config: String,
    }

    impl HardwareDeviceDbRow {
        pub fn into_hardware_device(self) -> HardwareDevice {
            HardwareDevice {
                id: record_id_to_string(&self.id),
                name: self.name,
                device_type: self.device_type,
                config: crate::crypto::decrypt_or_raw(&self.config),
            }
        }
    }
}

#[cfg(feature = "ssr")]
use ssr_types::*;

/// List all hardware devices for the current user.
#[server]
pub async fn get_devices() -> Result<Vec<HardwareDevice>, ServerFnError> {
    use crate::auth::require_auth;
    use crate::db::db;
    use crate::error::internal_error;

    let user_id = require_auth().await?;
    let owner = parse_owner(&user_id)?;

    let mut response = db()
        .query("SELECT * FROM hardware_device WHERE owner = $owner ORDER BY created_at ASC")
        .bind(("owner", owner))
        .await
        .map_err(|e| internal_error("Get devices query failed", e))?;

    let errors = response.take_errors();
    if !errors.is_empty() {
        let err_msg = errors.into_values().map(|e| e.to_string()).collect::<Vec<_>>().join("; ");
        return Err(internal_error("Get devices query error", err_msg));
    }

    let rows: Vec<HardwareDeviceDbRow> = response.take(0)
        .map_err(|e| internal_error("Get devices parse failed", e))?;

    Ok(rows.into_iter().map(|r| r.into_hardware_device()).collect())
}

/// Create a new hardware device.
#[server]
pub async fn create_device(
    name: String,
    device_type: String,
    config_json: String,
) -> Result<HardwareDevice, ServerFnError> {
    use crate::auth::require_auth;
    use crate::db::db;
    use crate::error::internal_error;
    use crate::crypto::encrypt;

    if name.is_empty() || name.len() > 100 {
        return Err(ServerFnError::new("Device name must be 1-100 characters"));
    }
    if !["tempest", "ac_infinity"].contains(&device_type.as_str()) {
        return Err(ServerFnError::new("Device type must be 'tempest' or 'ac_infinity'"));
    }

    let user_id = require_auth().await?;
    let owner = parse_owner(&user_id)?;

    let stored_config = if config_json.is_empty() {
        config_json
    } else {
        encrypt(&config_json).map_err(|e| internal_error("Encrypt config failed", e))?
    };

    let mut response = db()
        .query(
            "CREATE hardware_device SET \
             owner = $owner, name = $name, device_type = $device_type, \
             config = $config \
             RETURN *"
        )
        .bind(("owner", owner))
        .bind(("name", name))
        .bind(("device_type", device_type))
        .bind(("config", stored_config))
        .await
        .map_err(|e| internal_error("Create device query failed", e))?;

    let errors = response.take_errors();
    if !errors.is_empty() {
        let err_msg = errors.into_values().map(|e| e.to_string()).collect::<Vec<_>>().join("; ");
        return Err(internal_error("Create device query error", err_msg));
    }

    let row: Option<HardwareDeviceDbRow> = response.take(0)
        .map_err(|e| internal_error("Create device parse failed", e))?;

    row.map(|r| r.into_hardware_device())
        .ok_or_else(|| ServerFnError::new("Failed to create device"))
}

/// Update an existing hardware device's name and/or config.
/// Device type is immutable after creation.
#[server]
pub async fn update_device(
    device_id: String,
    name: String,
    config_json: String,
) -> Result<HardwareDevice, ServerFnError> {
    use crate::auth::require_auth;
    use crate::db::db;
    use crate::error::internal_error;
    use crate::crypto::encrypt;

    if name.is_empty() || name.len() > 100 {
        return Err(ServerFnError::new("Device name must be 1-100 characters"));
    }

    let user_id = require_auth().await?;
    let owner = parse_owner(&user_id)?;
    let dev_id = surrealdb::types::RecordId::parse_simple(&device_id)
        .map_err(|e| internal_error("Device ID parse failed", e))?;

    let stored_config = if config_json.is_empty() {
        config_json
    } else {
        encrypt(&config_json).map_err(|e| internal_error("Encrypt config failed", e))?
    };

    let mut response = db()
        .query(
            "UPDATE $id SET name = $name, config = $config \
             WHERE owner = $owner \
             RETURN *"
        )
        .bind(("id", dev_id))
        .bind(("owner", owner))
        .bind(("name", name))
        .bind(("config", stored_config))
        .await
        .map_err(|e| internal_error("Update device query failed", e))?;

    let errors = response.take_errors();
    if !errors.is_empty() {
        let err_msg = errors.into_values().map(|e| e.to_string()).collect::<Vec<_>>().join("; ");
        return Err(internal_error("Update device query error", err_msg));
    }

    let row: Option<HardwareDeviceDbRow> = response.take(0)
        .map_err(|e| internal_error("Update device parse failed", e))?;

    row.map(|r| r.into_hardware_device())
        .ok_or_else(|| ServerFnError::new("Device not found or not owned by you"))
}

/// Delete a hardware device and unlink all referencing zones.
#[server]
pub async fn delete_device(device_id: String) -> Result<(), ServerFnError> {
    use crate::auth::require_auth;
    use crate::db::db;
    use crate::error::internal_error;

    let user_id = require_auth().await?;
    let owner = parse_owner(&user_id)?;
    let dev_id = surrealdb::types::RecordId::parse_simple(&device_id)
        .map_err(|e| internal_error("Device ID parse failed", e))?;

    // Unlink all zones referencing this device
    let _ = db()
        .query(
            "UPDATE growing_zone SET hardware_device = NONE, hardware_port = NONE \
             WHERE hardware_device = $dev"
        )
        .bind(("dev", dev_id.clone()))
        .await;

    // Delete the device itself
    db()
        .query("DELETE $id WHERE owner = $owner")
        .bind(("id", dev_id))
        .bind(("owner", owner))
        .await
        .map_err(|e| internal_error("Delete device query failed", e))?;

    Ok(())
}

/// Test a device connection by attempting to fetch a reading.
#[server]
pub async fn test_device(
    device_type: String,
    config_json: String,
) -> Result<String, ServerFnError> {
    use crate::auth::require_auth;

    require_auth().await?;

    let client = reqwest::Client::new();

    match device_type.as_str() {
        "tempest" => {
            let config: crate::climate::poller::TempestConfig =
                serde_json::from_str(&config_json)
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
            let config: crate::climate::poller::AcInfinityConfig =
                serde_json::from_str(&config_json)
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
        _ => Err(ServerFnError::new(format!("Unknown device type: {}", device_type))),
    }
}

/// Link a zone to a shared hardware device.
/// Clears the zone's legacy data_source_type/data_source_config.
#[server]
pub async fn link_zone_to_device(
    zone_id: String,
    device_id: String,
    port: Option<i32>,
) -> Result<(), ServerFnError> {
    use crate::auth::require_auth;
    use crate::db::db;
    use crate::error::internal_error;

    let user_id = require_auth().await?;
    let owner = parse_owner(&user_id)?;
    let zone_record = surrealdb::types::RecordId::parse_simple(&zone_id)
        .map_err(|e| internal_error("Zone ID parse failed", e))?;
    let dev_record = surrealdb::types::RecordId::parse_simple(&device_id)
        .map_err(|e| internal_error("Device ID parse failed", e))?;

    let mut response = db()
        .query(
            "UPDATE $id SET \
             hardware_device = $dev, hardware_port = $port, \
             data_source_type = NONE, data_source_config = '' \
             WHERE owner = $owner \
             RETURN *"
        )
        .bind(("id", zone_record))
        .bind(("owner", owner))
        .bind(("dev", dev_record))
        .bind(("port", port))
        .await
        .map_err(|e| internal_error("Link zone to device query failed", e))?;

    let errors = response.take_errors();
    if !errors.is_empty() {
        let err_msg = errors.into_values().map(|e| e.to_string()).collect::<Vec<_>>().join("; ");
        return Err(internal_error("Link zone to device query error", err_msg));
    }

    Ok(())
}

/// Unlink a zone from its hardware device.
#[server]
pub async fn unlink_zone_from_device(zone_id: String) -> Result<(), ServerFnError> {
    use crate::auth::require_auth;
    use crate::db::db;
    use crate::error::internal_error;

    let user_id = require_auth().await?;
    let owner = parse_owner(&user_id)?;
    let zone_record = surrealdb::types::RecordId::parse_simple(&zone_id)
        .map_err(|e| internal_error("Zone ID parse failed", e))?;

    let mut response = db()
        .query(
            "UPDATE $id SET hardware_device = NONE, hardware_port = NONE \
             WHERE owner = $owner \
             RETURN *"
        )
        .bind(("id", zone_record))
        .bind(("owner", owner))
        .await
        .map_err(|e| internal_error("Unlink zone from device query failed", e))?;

    let errors = response.take_errors();
    if !errors.is_empty() {
        let err_msg = errors.into_values().map(|e| e.to_string()).collect::<Vec<_>>().join("; ");
        return Err(internal_error("Unlink zone from device query error", err_msg));
    }

    Ok(())
}
