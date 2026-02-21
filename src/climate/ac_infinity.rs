use std::collections::HashMap;
use super::RawReading;
use crate::error::AppError;

/// Fetch a climate reading from an AC Infinity controller via their cloud API.
///
/// This is a reverse-engineered API — the endpoints and field names may change.
/// Step 1: Login to get a token
/// Step 2: Fetch device list with the token
/// Step 3: Find the target device/port and extract sensor readings
pub async fn fetch_ac_infinity_reading(
    client: &reqwest::Client,
    email: &str,
    password: &str,
    device_id: &str,
    port: u32,
) -> Result<RawReading, AppError> {
    // Step 1: Login
    // Note: "appPasswordl" is intentional — the API has a typo in the field name
    let login_body = serde_json::json!({
        "appEmail": email,
        "appPasswordl": password,
    });

    let login_resp = client
        .post("http://www.acinfinityserver.com/api/user/appUserLogin")
        .json(&login_body)
        .send()
        .await
        .map_err(|e| AppError::Network(format!("AC Infinity login request failed: {}", e)))?;

    let login_json: serde_json::Value = login_resp
        .json()
        .await
        .map_err(|e| AppError::Serialization(format!("AC Infinity login parse error: {}", e)))?;

    let token = login_json
        .get("data")
        .and_then(|d| d.get("appId"))
        .and_then(|t| t.as_str())
        .ok_or_else(|| AppError::Auth("AC Infinity login failed: no token in response".into()))?;

    // Step 2: Fetch device list
    let devices_resp = client
        .post("http://www.acinfinityserver.com/api/user/devInfoListAll")
        .header("token", token)
        .json(&serde_json::json!({}))
        .send()
        .await
        .map_err(|e| AppError::Network(format!("AC Infinity device list request failed: {}", e)))?;

    let devices_json: serde_json::Value = devices_resp
        .json()
        .await
        .map_err(|e| {
            AppError::Serialization(format!("AC Infinity device list parse error: {}", e))
        })?;

    // Step 3: Find device and port
    let devices = devices_json
        .get("data")
        .and_then(|d| d.as_array())
        .ok_or_else(|| AppError::Serialization("No device data in AC Infinity response".into()))?;

    let device = devices
        .iter()
        .find(|d| {
            d.get("devId")
                .and_then(|id| id.as_str())
                .is_some_and(|id| id == device_id)
        })
        .ok_or_else(|| {
            AppError::Validation(format!("Device '{}' not found in AC Infinity account", device_id))
        })?;

    let ports = device
        .get("ports")
        .and_then(|p| p.as_array())
        .ok_or_else(|| AppError::Serialization("No ports on AC Infinity device".into()))?;

    let port_data = ports
        .iter()
        .find(|p| {
            p.get("portId")
                .and_then(|id| id.as_u64())
                .is_some_and(|id| id == port as u64)
        })
        .or_else(|| ports.first())
        .ok_or_else(|| {
            AppError::Validation(format!("Port {} not found on device '{}'", port, device_id))
        })?;

    // Extract sensor values — AC Infinity returns values * 100
    let temp_f_raw = port_data
        .get("temperatureF")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let humidity_raw = port_data
        .get("humidity")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let vpd_raw = port_data
        .get("vpdnums")
        .and_then(|v| v.as_f64());

    let temp_f = temp_f_raw / 100.0;
    let temp_c = (temp_f - 32.0) * 5.0 / 9.0;
    let humidity = humidity_raw / 100.0;
    let vpd = vpd_raw.map(|v| v / 100.0);

    Ok(RawReading {
        temperature_c: temp_c,
        humidity_pct: humidity,
        vpd_kpa: vpd,
    })
}

/// Fetch readings from ALL ports on an AC Infinity controller in a single API call.
/// Returns a map of port_id -> RawReading for every port that has sensor data.
pub async fn fetch_ac_infinity_all_ports(
    client: &reqwest::Client,
    email: &str,
    password: &str,
    device_id: &str,
) -> Result<HashMap<u32, RawReading>, AppError> {
    // Step 1: Login (same as single-port)
    let login_body = serde_json::json!({
        "appEmail": email,
        "appPasswordl": password,
    });

    let login_resp = client
        .post("http://www.acinfinityserver.com/api/user/appUserLogin")
        .json(&login_body)
        .send()
        .await
        .map_err(|e| AppError::Network(format!("AC Infinity login request failed: {}", e)))?;

    let login_json: serde_json::Value = login_resp
        .json()
        .await
        .map_err(|e| AppError::Serialization(format!("AC Infinity login parse error: {}", e)))?;

    let token = login_json
        .get("data")
        .and_then(|d| d.get("appId"))
        .and_then(|t| t.as_str())
        .ok_or_else(|| AppError::Auth("AC Infinity login failed: no token in response".into()))?;

    // Step 2: Fetch device list
    let devices_resp = client
        .post("http://www.acinfinityserver.com/api/user/devInfoListAll")
        .header("token", token)
        .json(&serde_json::json!({}))
        .send()
        .await
        .map_err(|e| AppError::Network(format!("AC Infinity device list request failed: {}", e)))?;

    let devices_json: serde_json::Value = devices_resp
        .json()
        .await
        .map_err(|e| {
            AppError::Serialization(format!("AC Infinity device list parse error: {}", e))
        })?;

    // Step 3: Find device
    let devices = devices_json
        .get("data")
        .and_then(|d| d.as_array())
        .ok_or_else(|| AppError::Serialization("No device data in AC Infinity response".into()))?;

    let device = devices
        .iter()
        .find(|d| {
            d.get("devId")
                .and_then(|id| id.as_str())
                .is_some_and(|id| id == device_id)
        })
        .ok_or_else(|| {
            AppError::Validation(format!("Device '{}' not found in AC Infinity account", device_id))
        })?;

    let ports = device
        .get("ports")
        .and_then(|p| p.as_array())
        .ok_or_else(|| AppError::Serialization("No ports on AC Infinity device".into()))?;

    // Step 4: Iterate ALL ports and build readings map
    let mut readings = HashMap::new();

    for port_data in ports {
        let port_id = match port_data.get("portId").and_then(|id| id.as_u64()) {
            Some(id) => id as u32,
            None => continue,
        };

        let temp_f_raw = port_data
            .get("temperatureF")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let humidity_raw = port_data
            .get("humidity")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let vpd_raw = port_data
            .get("vpdnums")
            .and_then(|v| v.as_f64());

        let temp_f = temp_f_raw / 100.0;
        let temp_c = (temp_f - 32.0) * 5.0 / 9.0;
        let humidity = humidity_raw / 100.0;
        let vpd = vpd_raw.map(|v| v / 100.0);

        readings.insert(port_id, RawReading {
            temperature_c: temp_c,
            humidity_pct: humidity,
            vpd_kpa: vpd,
        });
    }

    Ok(readings)
}
