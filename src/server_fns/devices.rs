use leptos::prelude::*;
use crate::orchid::HardwareDevice;

/// **What is it?**
/// A utility function that parses the "table:key" user_id string into a SurrealDB RecordId.
///
/// **Why does it exist?**
/// It exists to standardize error handling across the backend when extracting the authenticated user's ID for database constraints.
///
/// **How should it be used?**
/// Call this inside server functions after `require_auth` to obtain the `RecordId` needed for the `owner` field in database queries.
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

/// **What is it?**
/// A server function that retrieves a list of all hardware devices configured by the current user.
///
/// **Why does it exist?**
/// It exists so the user can view and manage their list of shared hardware (like AC Infinity controllers) across their different zones.
///
/// **How should it be used?**
/// Call this from the Hardware Settings or Devices page to populate the list of available integrations.
#[server]
#[tracing::instrument(level = "info", skip_all)]
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

/// **What is it?**
/// A server function that registers a new hardware device in the database.
///
/// **Why does it exist?**
/// It exists to securely encrypt and store credentials or connection strings for IoT devices (like Tempest weather stations), so the backend can automatically poll them later.
///
/// **How should it be used?**
/// Call this when the user submits the "Add Device" form with their API keys or local IP configurations.
#[server]
#[tracing::instrument(level = "info", skip_all)]
pub async fn create_device(
    /// The user-defined name for the device.
    name: String,
    /// The type of the device (e.g., "tempest", "ac_infinity").
    device_type: String,
    /// The JSON configuration string for the device.
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

/// **What is it?**
/// A server function that updates an existing hardware device's name or configuration.
///
/// **Why does it exist?**
/// It exists to allow users to rotate their API keys, update IP addresses, or rename a device without having to delete and recreate it (which would break zone links).
///
/// **How should it be used?**
/// Call this from the "Edit Device" form. Note that the device type cannot be changed after creation.
#[server]
#[tracing::instrument(level = "info", skip_all)]
pub async fn update_device(
    /// The unique identifier of the device.
    device_id: String,
    /// The new name for the device.
    name: String,
    /// The new JSON configuration string.
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

/// **What is it?**
/// A server function that deletes a hardware device and removes its references from any associated zones.
///
/// **Why does it exist?**
/// It exists to allow a user to remove a piece of hardware they no longer own or use, while maintaining referential integrity by safely unlinking it from their growing zones.
///
/// **How should it be used?**
/// Call this when a user clicks "Delete" on a hardware device card.
#[server]
#[tracing::instrument(level = "info", skip_all)]
pub async fn delete_device(
    /// The unique identifier of the device to delete.
    device_id: String
) -> Result<(), ServerFnError> {
    use crate::auth::require_auth;
    use crate::db::db;
    use crate::error::internal_error;

    let user_id = require_auth().await?;
    let owner = parse_owner(&user_id)?;
    let dev_id = surrealdb::types::RecordId::parse_simple(&device_id)
        .map_err(|e| internal_error("Device ID parse failed", e))?;

    // Unlink zones and delete device atomically
    let mut response = db()
        .query(
            "BEGIN TRANSACTION; \
             UPDATE growing_zone SET hardware_device = NONE, hardware_port = NONE \
                 WHERE hardware_device = $dev; \
             DELETE $dev WHERE owner = $owner; \
             COMMIT TRANSACTION;"
        )
        .bind(("dev", dev_id))
        .bind(("owner", owner))
        .await
        .map_err(|e| internal_error("Delete device query failed", e))?;

    let errors = response.take_errors();
    if !errors.is_empty() {
        let err_msg = errors.into_values().map(|e| e.to_string()).collect::<Vec<_>>().join("; ");
        return Err(internal_error("Delete device query error", err_msg));
    }

    Ok(())
}

/// **What is it?**
/// A server function that verifies a set of device credentials by performing a live test fetch against the hardware.
///
/// **Why does it exist?**
/// It exists to give users immediate feedback on whether their API keys, IP addresses, or passwords are correct *before* they save a broken device configuration.
///
/// **How should it be used?**
/// Call this when the user clicks a "Test Connection" button in the device configuration form.
#[server]
#[tracing::instrument(level = "info", skip_all)]
pub async fn test_device(
    /// The type of the device to test.
    device_type: String,
    /// The JSON configuration string for the device.
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

/// **What is it?**
/// A server function that assigns a growing zone to pull its climate data from a specific shared hardware device.
///
/// **Why does it exist?**
/// It exists to establish the relationship between a physical location (zone) and a sensor (device), instructing the backend poller where to route the gathered data.
///
/// **How should it be used?**
/// Call this when a user selects a device from the dropdown menu in a zone's settings panel.
#[server]
#[tracing::instrument(level = "info", skip_all)]
pub async fn link_zone_to_device(
    /// The unique identifier of the zone.
    zone_id: String,
    /// The unique identifier of the hardware device.
    device_id: String,
    /// The specific port or sensor on the device, if applicable.
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

/// **What is it?**
/// A server function that removes the hardware linkage from a growing zone.
///
/// **Why does it exist?**
/// It exists to stop the background poller from associating data from a specific sensor with this zone, effectively returning the zone to a "manual data" or "no data" state.
///
/// **How should it be used?**
/// Call this when the user clicks "Unlink" or selects "None" for the data source in a zone's settings.
#[server]
#[tracing::instrument(level = "info", skip_all)]
pub async fn unlink_zone_from_device(
    /// The unique identifier of the zone.
    zone_id: String
) -> Result<(), ServerFnError> {
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
