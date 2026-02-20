#![allow(clippy::too_many_arguments)]

use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use crate::orchid::{Orchid, LogEntry};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AddLogEntryResponse {
    pub entry: LogEntry,
    pub is_first_bloom: bool,
}

#[cfg(feature = "ssr")]
fn parse_record_id(id: &str) -> Result<surrealdb::types::RecordId, ServerFnError> {
    use crate::error::internal_error;
    surrealdb::types::RecordId::parse_simple(id)
        .map_err(|e| internal_error("Record ID parse failed", e))
}

#[cfg(feature = "ssr")]
mod ssr_types {
    use surrealdb::types::SurrealValue;
    use crate::orchid::{Orchid, LogEntry, LightRequirement};
    use crate::server_fns::auth::record_id_to_string;

    #[derive(serde::Deserialize, SurrealValue)]
    #[surreal(crate = "surrealdb::types")]
    pub struct OrchidDbRow {
        pub id: surrealdb::types::RecordId,
        pub name: String,
        pub species: String,
        pub water_frequency_days: u32,
        /// Stored as plain string in DB; SurrealValue untagged enum can't round-trip
        pub light_requirement: String,
        pub notes: String,
        pub placement: String,
        pub light_lux: String,
        pub temperature_range: String,
        #[surreal(default)]
        pub conservation_status: Option<String>,
        #[surreal(default)]
        pub native_region: Option<String>,
        #[surreal(default)]
        pub native_latitude: Option<f64>,
        #[surreal(default)]
        pub native_longitude: Option<f64>,
        #[surreal(default)]
        pub last_watered_at: Option<chrono::DateTime<chrono::Utc>>,
        #[surreal(default)]
        pub temp_min: Option<f64>,
        #[surreal(default)]
        pub temp_max: Option<f64>,
        #[surreal(default)]
        pub humidity_min: Option<f64>,
        #[surreal(default)]
        pub humidity_max: Option<f64>,
        #[surreal(default)]
        pub first_bloom_at: Option<chrono::DateTime<chrono::Utc>>,
        #[surreal(default)]
        pub last_fertilized_at: Option<chrono::DateTime<chrono::Utc>>,
        #[surreal(default)]
        pub fertilize_frequency_days: Option<u32>,
        #[surreal(default)]
        pub fertilizer_type: Option<String>,
        #[surreal(default)]
        pub last_repotted_at: Option<chrono::DateTime<chrono::Utc>>,
        #[surreal(default)]
        pub pot_medium: Option<String>,
        #[surreal(default)]
        pub pot_size: Option<String>,
        #[surreal(default)]
        pub rest_start_month: Option<u32>,
        #[surreal(default)]
        pub rest_end_month: Option<u32>,
        #[surreal(default)]
        pub bloom_start_month: Option<u32>,
        #[surreal(default)]
        pub bloom_end_month: Option<u32>,
        #[surreal(default)]
        pub rest_water_multiplier: Option<f64>,
        #[surreal(default)]
        pub rest_fertilizer_multiplier: Option<f64>,
        #[surreal(default)]
        pub active_water_multiplier: Option<f64>,
        #[surreal(default)]
        pub active_fertilizer_multiplier: Option<f64>,
    }

    #[derive(serde::Deserialize, SurrealValue, Clone)]
    #[surreal(crate = "surrealdb::types")]
    pub struct LogEntryDbRow {
        pub id: surrealdb::types::RecordId,
        pub timestamp: chrono::DateTime<chrono::Utc>,
        pub note: String,
        #[surreal(default)]
        pub image_filename: Option<String>,
        #[surreal(default)]
        pub event_type: Option<String>,
    }

    impl OrchidDbRow {
        pub fn into_orchid(self) -> Orchid {
            let light_requirement = match self.light_requirement.as_str() {
                "Low" => LightRequirement::Low,
                "High" => LightRequirement::High,
                _ => LightRequirement::Medium,
            };
            Orchid {
                id: record_id_to_string(&self.id),
                name: self.name,
                species: self.species,
                water_frequency_days: self.water_frequency_days,
                light_requirement,
                notes: self.notes,
                placement: self.placement,
                light_lux: self.light_lux,
                temperature_range: self.temperature_range,
                conservation_status: self.conservation_status,
                native_region: self.native_region,
                native_latitude: self.native_latitude,
                native_longitude: self.native_longitude,
                last_watered_at: self.last_watered_at,
                temp_min: self.temp_min,
                temp_max: self.temp_max,
                humidity_min: self.humidity_min,
                humidity_max: self.humidity_max,
                first_bloom_at: self.first_bloom_at,
                last_fertilized_at: self.last_fertilized_at,
                fertilize_frequency_days: self.fertilize_frequency_days,
                fertilizer_type: self.fertilizer_type,
                last_repotted_at: self.last_repotted_at,
                pot_medium: self.pot_medium,
                pot_size: self.pot_size,
                rest_start_month: self.rest_start_month,
                rest_end_month: self.rest_end_month,
                bloom_start_month: self.bloom_start_month,
                bloom_end_month: self.bloom_end_month,
                rest_water_multiplier: self.rest_water_multiplier,
                rest_fertilizer_multiplier: self.rest_fertilizer_multiplier,
                active_water_multiplier: self.active_water_multiplier,
                active_fertilizer_multiplier: self.active_fertilizer_multiplier,
            }
        }
    }

    impl LogEntryDbRow {
        pub fn into_log_entry(self) -> LogEntry {
            LogEntry {
                id: record_id_to_string(&self.id),
                timestamp: self.timestamp,
                note: self.note,
                image_filename: self.image_filename,
                event_type: self.event_type,
            }
        }
    }
}

#[cfg(feature = "ssr")]
use ssr_types::*;

/// Normalize light requirement strings to DB-compatible values.
/// Handles aliases like "Medium Light" -> "Medium", "low" -> "Low", etc.
/// Defaults to "Medium" for unrecognized input.
#[cfg(feature = "ssr")]
fn normalize_light_requirement(input: &str) -> String {
    match input.trim() {
        "Low" | "low" | "Low Light" => "Low".to_string(),
        "Medium" | "medium" | "Medium Light" => "Medium".to_string(),
        "High" | "high" | "High Light" => "High".to_string(),
        _ => "Medium".to_string(),
    }
}

#[cfg(feature = "ssr")]
fn validate_orchid_fields(
    name: &str,
    species: &str,
    notes: &str,
    water_frequency_days: u32,
    light_requirement: &str,
    placement: &str,
    light_lux: &str,
    temperature_range: &str,
    conservation_status: &Option<String>,
) -> Result<(), ServerFnError> {
    if name.is_empty() || name.len() > 200 {
        return Err(ServerFnError::new("Name must be 1-200 characters"));
    }
    if species.is_empty() || species.len() > 200 {
        return Err(ServerFnError::new("Species must be 1-200 characters"));
    }
    if notes.len() > 5000 {
        return Err(ServerFnError::new("Notes must be at most 5000 characters"));
    }
    if water_frequency_days < 1 || water_frequency_days > 365 {
        return Err(ServerFnError::new("Water frequency must be 1-365 days"));
    }
    if light_requirement.len() > 100 {
        return Err(ServerFnError::new("Light requirement must be at most 100 characters"));
    }
    if placement.len() > 100 {
        return Err(ServerFnError::new("Placement must be at most 100 characters"));
    }
    if light_lux.len() > 100 {
        return Err(ServerFnError::new("Light lux must be at most 100 characters"));
    }
    if temperature_range.len() > 100 {
        return Err(ServerFnError::new("Temperature range must be at most 100 characters"));
    }
    if let Some(cs) = conservation_status {
        if cs.len() > 200 {
            return Err(ServerFnError::new("Conservation status must be at most 200 characters"));
        }
    }
    Ok(())
}

#[cfg(feature = "ssr")]
fn validate_filename(filename: &str) -> Result<(), ServerFnError> {
    // Block path traversal
    if filename.contains("..") || filename.contains('\\') || filename.starts_with('/') {
        return Err(ServerFnError::new("Invalid image filename"));
    }
    // Allow only alphanumeric, hyphens, underscores, dots, and forward slashes (for user_id/file.ext)
    let valid = filename.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '/');
    if !valid {
        return Err(ServerFnError::new("Invalid image filename"));
    }
    // At most one dot (before extension) in the final path component
    let basename = filename.rsplit('/').next().unwrap_or(filename);
    let dot_count = basename.chars().filter(|&c| c == '.').count();
    if dot_count > 1 {
        return Err(ServerFnError::new("Invalid image filename"));
    }
    Ok(())
}

#[server]
pub async fn get_orchids() -> Result<Vec<Orchid>, ServerFnError> {
    use crate::auth::require_auth;
    use crate::db::db;
    use crate::error::internal_error;

    let user_id = require_auth().await?;
    let owner = parse_record_id(&user_id)?;

    let mut response = db()
        .query("SELECT * FROM orchid WHERE owner = $owner ORDER BY created_at DESC")
        .bind(("owner", owner))
        .await
        .map_err(|e| internal_error("Get orchids query failed", e))?;

    let errors = response.take_errors();
    if !errors.is_empty() {
        let err_msg = errors.into_values().map(|e| e.to_string()).collect::<Vec<_>>().join("; ");
        return Err(internal_error("Get orchids query error", err_msg));
    }

    let db_rows: Vec<OrchidDbRow> = response.take(0)
        .map_err(|e| internal_error("Get orchids parse failed", e))?;

    Ok(db_rows.into_iter().map(|r| r.into_orchid()).collect())
}

#[server]
pub async fn create_orchid(
    name: String,
    species: String,
    water_frequency_days: u32,
    light_requirement: String,
    notes: String,
    placement: String,
    light_lux: String,
    temperature_range: String,
    conservation_status: Option<String>,
    native_region: Option<String>,
    native_latitude: Option<f64>,
    native_longitude: Option<f64>,
    temp_min: Option<f64>,
    temp_max: Option<f64>,
    humidity_min: Option<f64>,
    humidity_max: Option<f64>,
    fertilize_frequency_days: Option<u32>,
    fertilizer_type: Option<String>,
    pot_medium: Option<String>,
    pot_size: Option<String>,
    rest_start_month: Option<u32>,
    rest_end_month: Option<u32>,
    bloom_start_month: Option<u32>,
    bloom_end_month: Option<u32>,
    rest_water_multiplier: Option<f64>,
    rest_fertilizer_multiplier: Option<f64>,
    active_water_multiplier: Option<f64>,
    active_fertilizer_multiplier: Option<f64>,
) -> Result<Orchid, ServerFnError> {
    use crate::auth::require_auth;
    use crate::db::db;
    use crate::error::internal_error;

    let light_requirement = normalize_light_requirement(&light_requirement);

    validate_orchid_fields(&name, &species, &notes, water_frequency_days, &light_requirement, &placement, &light_lux, &temperature_range, &conservation_status)?;

    let user_id = require_auth().await?;
    let owner = parse_record_id(&user_id)?;

    let mut response = db()
        .query(
            "CREATE orchid SET \
             owner = $owner, name = $name, species = $species, \
             water_frequency_days = $water_freq, light_requirement = $light_req, \
             notes = $notes, placement = $placement, light_lux = $light_lux, \
             temperature_range = $temp_range, conservation_status = $conservation, \
             native_region = $native_region, native_latitude = $native_lat, \
             native_longitude = $native_lon, \
             temp_min = $temp_min, temp_max = $temp_max, \
             humidity_min = $humidity_min, humidity_max = $humidity_max, \
             fertilize_frequency_days = $fert_freq, fertilizer_type = $fert_type, \
             pot_medium = $pot_medium, pot_size = $pot_size, \
             rest_start_month = $rest_start, rest_end_month = $rest_end, \
             bloom_start_month = $bloom_start, bloom_end_month = $bloom_end, \
             rest_water_multiplier = $rest_water_mult, rest_fertilizer_multiplier = $rest_fert_mult, \
             active_water_multiplier = $active_water_mult, active_fertilizer_multiplier = $active_fert_mult \
             RETURN *"
        )
        .bind(("owner", owner))
        .bind(("name", name))
        .bind(("species", species))
        .bind(("water_freq", water_frequency_days as i64))
        .bind(("light_req", light_requirement))
        .bind(("notes", notes))
        .bind(("placement", placement))
        .bind(("light_lux", light_lux))
        .bind(("temp_range", temperature_range))
        .bind(("conservation", conservation_status))
        .bind(("native_region", native_region))
        .bind(("native_lat", native_latitude))
        .bind(("native_lon", native_longitude))
        .bind(("temp_min", temp_min))
        .bind(("temp_max", temp_max))
        .bind(("humidity_min", humidity_min))
        .bind(("humidity_max", humidity_max))
        .bind(("fert_freq", fertilize_frequency_days.map(|v| v as i64)))
        .bind(("fert_type", fertilizer_type))
        .bind(("pot_medium", pot_medium))
        .bind(("pot_size", pot_size))
        .bind(("rest_start", rest_start_month.map(|v| v as i64)))
        .bind(("rest_end", rest_end_month.map(|v| v as i64)))
        .bind(("bloom_start", bloom_start_month.map(|v| v as i64)))
        .bind(("bloom_end", bloom_end_month.map(|v| v as i64)))
        .bind(("rest_water_mult", rest_water_multiplier))
        .bind(("rest_fert_mult", rest_fertilizer_multiplier))
        .bind(("active_water_mult", active_water_multiplier))
        .bind(("active_fert_mult", active_fertilizer_multiplier))
        .await
        .map_err(|e| internal_error("Create orchid query failed", e))?;

    let errors = response.take_errors();
    if !errors.is_empty() {
        let err_msg = errors.into_values().map(|e| e.to_string()).collect::<Vec<_>>().join("; ");
        return Err(internal_error("Create orchid query error", err_msg));
    }

    let db_row: Option<OrchidDbRow> = response.take(0)
        .map_err(|e| internal_error("Create orchid parse failed", e))?;

    db_row.map(|r| r.into_orchid())
        .ok_or_else(|| ServerFnError::new("Failed to create orchid"))
}

#[server]
pub async fn update_orchid(orchid: Orchid) -> Result<Orchid, ServerFnError> {
    use crate::auth::require_auth;
    use crate::db::db;
    use crate::error::internal_error;

    let light_req_str = orchid.light_requirement.as_str();
    let placement_str = orchid.placement.clone();

    validate_orchid_fields(&orchid.name, &orchid.species, &orchid.notes, orchid.water_frequency_days, light_req_str, &placement_str, &orchid.light_lux, &orchid.temperature_range, &orchid.conservation_status)?;

    let user_id = require_auth().await?;
    let orchid_id = parse_record_id(&orchid.id)?;
    let owner = parse_record_id(&user_id)?;

    let mut response = db()
        .query(
            "UPDATE $id SET \
             name = $name, species = $species, \
             water_frequency_days = $water_freq, light_requirement = $light_req, \
             notes = $notes, placement = $placement, light_lux = $light_lux, \
             temperature_range = $temp_range, conservation_status = $conservation, \
             native_region = $native_region, native_latitude = $native_lat, \
             native_longitude = $native_lon, \
             temp_min = $temp_min, temp_max = $temp_max, \
             humidity_min = $humidity_min, humidity_max = $humidity_max, \
             fertilize_frequency_days = $fert_freq, fertilizer_type = $fert_type, \
             pot_medium = $pot_medium, pot_size = $pot_size, \
             rest_start_month = $rest_start, rest_end_month = $rest_end, \
             bloom_start_month = $bloom_start, bloom_end_month = $bloom_end, \
             rest_water_multiplier = $rest_water_mult, rest_fertilizer_multiplier = $rest_fert_mult, \
             active_water_multiplier = $active_water_mult, active_fertilizer_multiplier = $active_fert_mult, \
             updated_at = time::now() \
             WHERE owner = $owner \
             RETURN *"
        )
        .bind(("id", orchid_id))
        .bind(("owner", owner))
        .bind(("name", orchid.name))
        .bind(("species", orchid.species))
        .bind(("water_freq", orchid.water_frequency_days as i64))
        .bind(("light_req", light_req_str.to_string()))
        .bind(("notes", orchid.notes))
        .bind(("placement", placement_str))
        .bind(("light_lux", orchid.light_lux))
        .bind(("temp_range", orchid.temperature_range))
        .bind(("conservation", orchid.conservation_status))
        .bind(("native_region", orchid.native_region))
        .bind(("native_lat", orchid.native_latitude))
        .bind(("native_lon", orchid.native_longitude))
        .bind(("temp_min", orchid.temp_min))
        .bind(("temp_max", orchid.temp_max))
        .bind(("humidity_min", orchid.humidity_min))
        .bind(("humidity_max", orchid.humidity_max))
        .bind(("fert_freq", orchid.fertilize_frequency_days.map(|v| v as i64)))
        .bind(("fert_type", orchid.fertilizer_type))
        .bind(("pot_medium", orchid.pot_medium))
        .bind(("pot_size", orchid.pot_size))
        .bind(("rest_start", orchid.rest_start_month.map(|v| v as i64)))
        .bind(("rest_end", orchid.rest_end_month.map(|v| v as i64)))
        .bind(("bloom_start", orchid.bloom_start_month.map(|v| v as i64)))
        .bind(("bloom_end", orchid.bloom_end_month.map(|v| v as i64)))
        .bind(("rest_water_mult", orchid.rest_water_multiplier))
        .bind(("rest_fert_mult", orchid.rest_fertilizer_multiplier))
        .bind(("active_water_mult", orchid.active_water_multiplier))
        .bind(("active_fert_mult", orchid.active_fertilizer_multiplier))
        .await
        .map_err(|e| internal_error("Update orchid query failed", e))?;

    let errors = response.take_errors();
    if !errors.is_empty() {
        let err_msg = errors.into_values().map(|e| e.to_string()).collect::<Vec<_>>().join("; ");
        return Err(internal_error("Update orchid query error", err_msg));
    }

    let updated: Option<OrchidDbRow> = response.take(0)
        .map_err(|e| internal_error("Update orchid parse failed", e))?;

    updated.map(|r| r.into_orchid())
        .ok_or_else(|| ServerFnError::new("Orchid not found or not owned by you"))
}

#[server]
pub async fn delete_orchid(id: String) -> Result<(), ServerFnError> {
    use crate::auth::require_auth;
    use crate::db::db;
    use crate::error::internal_error;

    let user_id = require_auth().await?;
    let orchid_id = parse_record_id(&id)?;
    let owner = parse_record_id(&user_id)?;

    db()
        .query("DELETE $id WHERE owner = $owner")
        .bind(("id", orchid_id))
        .bind(("owner", owner))
        .await
        .map_err(|e| internal_error("Delete orchid query failed", e))?;

    Ok(())
}

#[server]
pub async fn add_log_entry(
    orchid_id: String,
    note: String,
    image_filename: Option<String>,
    event_type: Option<String>,
) -> Result<AddLogEntryResponse, ServerFnError> {
    use crate::auth::require_auth;
    use crate::db::db;
    use crate::error::internal_error;

    if note.len() > 5000 {
        return Err(ServerFnError::new("Note must be at most 5000 characters"));
    }
    if let Some(ref filename) = image_filename {
        validate_filename(filename)?;
    }

    // Validate event_type against allowed values
    let allowed_event_types = [
        "Flowering", "NewGrowth", "Repotted", "Fertilized",
        "PestTreatment", "Purchased", "Watered", "Note",
    ];
    if let Some(ref et) = event_type {
        if !allowed_event_types.contains(&et.as_str()) {
            return Err(ServerFnError::new("Invalid event type"));
        }
    }

    let user_id = require_auth().await?;
    let orchid_record = parse_record_id(&orchid_id)?;
    let owner = parse_record_id(&user_id)?;

    let mut response = db()
        .query(
            "CREATE log_entry SET \
             orchid = $orchid_id, owner = $owner, \
             note = $note, image_filename = $image_filename, \
             event_type = $event_type \
             RETURN *"
        )
        .bind(("orchid_id", orchid_record.clone()))
        .bind(("owner", owner.clone()))
        .bind(("note", note))
        .bind(("image_filename", image_filename))
        .bind(("event_type", event_type.clone()))
        .await
        .map_err(|e| internal_error("Add log entry query failed", e))?;

    let errors = response.take_errors();
    if !errors.is_empty() {
        let err_msg = errors.into_values().map(|e| e.to_string()).collect::<Vec<_>>().join("; ");
        return Err(internal_error("Add log entry query error", err_msg));
    }

    let db_row: Option<LogEntryDbRow> = response.take(0)
        .map_err(|e| internal_error("Add log entry parse failed", e))?;

    let entry = db_row.map(|r| r.into_log_entry())
        .ok_or_else(|| ServerFnError::new("Failed to create log entry"))?;

    // Check for first bloom
    let mut is_first_bloom = false;
    if event_type.as_deref() == Some("Flowering") {
        // Check if any prior flowering entries exist
        let mut bloom_resp = db()
            .query(
                "SELECT count() AS cnt FROM log_entry \
                 WHERE orchid = $orchid_id AND owner = $owner \
                 AND event_type = 'Flowering' \
                 GROUP ALL"
            )
            .bind(("orchid_id", orchid_record.clone()))
            .bind(("owner", owner.clone()))
            .await
            .map_err(|e| internal_error("Check bloom query failed", e))?;

        let _ = bloom_resp.take_errors();
        let bloom_count: Option<serde_json::Value> = bloom_resp.take(0)
            .unwrap_or(None);

        let count = bloom_count
            .and_then(|v| v.get("cnt").and_then(|c| c.as_i64()))
            .unwrap_or(0);

        // count == 1 means the entry we just created is the only one
        if count <= 1 {
            is_first_bloom = true;
            let _ = db()
                .query(
                    "UPDATE $orchid_id SET first_bloom_at = time::now() \
                     WHERE owner = $owner"
                )
                .bind(("orchid_id", orchid_record))
                .bind(("owner", owner))
                .await;
        }
    }

    Ok(AddLogEntryResponse { entry, is_first_bloom })
}

#[server]
pub async fn get_log_entries(orchid_id: String) -> Result<Vec<LogEntry>, ServerFnError> {
    use crate::auth::require_auth;
    use crate::db::db;
    use crate::error::internal_error;

    let user_id = require_auth().await?;
    let orchid_record = parse_record_id(&orchid_id)?;
    let owner = parse_record_id(&user_id)?;

    let mut response = db()
        .query("SELECT * FROM log_entry WHERE orchid = $orchid_id AND owner = $owner ORDER BY timestamp DESC")
        .bind(("orchid_id", orchid_record))
        .bind(("owner", owner))
        .await
        .map_err(|e| internal_error("Get log entries query failed", e))?;

    let errors = response.take_errors();
    if !errors.is_empty() {
        let err_msg = errors.into_values().map(|e| e.to_string()).collect::<Vec<_>>().join("; ");
        return Err(internal_error("Get log entries query error", err_msg));
    }

    let db_rows: Vec<LogEntryDbRow> = response.take(0)
        .map_err(|e| internal_error("Get log entries parse failed", e))?;

    Ok(db_rows.into_iter().map(|r| r.into_log_entry()).collect())
}

#[server]
pub async fn mark_watered(orchid_id: String) -> Result<Orchid, ServerFnError> {
    use crate::auth::require_auth;
    use crate::db::db;
    use crate::error::internal_error;

    let user_id = require_auth().await?;
    let oid = parse_record_id(&orchid_id)?;
    let owner = parse_record_id(&user_id)?;

    // Update last_watered_at
    let mut response = db()
        .query(
            "UPDATE $id SET last_watered_at = time::now() WHERE owner = $owner RETURN *"
        )
        .bind(("id", oid.clone()))
        .bind(("owner", owner.clone()))
        .await
        .map_err(|e| internal_error("Mark watered query failed", e))?;

    let errors = response.take_errors();
    if !errors.is_empty() {
        let err_msg = errors.into_values().map(|e| e.to_string()).collect::<Vec<_>>().join("; ");
        return Err(internal_error("Mark watered query error", err_msg));
    }

    let db_row: Option<OrchidDbRow> = response.take(0)
        .map_err(|e| internal_error("Mark watered parse failed", e))?;

    let orchid = db_row.map(|r| r.into_orchid())
        .ok_or_else(|| ServerFnError::new("Orchid not found or not owned by you"))?;

    // Also create a log entry with event_type
    let _ = db()
        .query(
            "CREATE log_entry SET orchid = $orchid_id, owner = $owner, note = 'Watered', event_type = 'Watered'"
        )
        .bind(("orchid_id", oid))
        .bind(("owner", owner))
        .await;

    Ok(orchid)
}

#[server]
pub async fn mark_fertilized(orchid_id: String) -> Result<Orchid, ServerFnError> {
    use crate::auth::require_auth;
    use crate::db::db;
    use crate::error::internal_error;

    let user_id = require_auth().await?;
    let oid = parse_record_id(&orchid_id)?;
    let owner = parse_record_id(&user_id)?;

    let mut response = db()
        .query(
            "UPDATE $id SET last_fertilized_at = time::now() WHERE owner = $owner RETURN *"
        )
        .bind(("id", oid.clone()))
        .bind(("owner", owner.clone()))
        .await
        .map_err(|e| internal_error("Mark fertilized query failed", e))?;

    let errors = response.take_errors();
    if !errors.is_empty() {
        let err_msg = errors.into_values().map(|e| e.to_string()).collect::<Vec<_>>().join("; ");
        return Err(internal_error("Mark fertilized query error", err_msg));
    }

    let db_row: Option<OrchidDbRow> = response.take(0)
        .map_err(|e| internal_error("Mark fertilized parse failed", e))?;

    let orchid = db_row.map(|r| r.into_orchid())
        .ok_or_else(|| ServerFnError::new("Orchid not found or not owned by you"))?;

    let _ = db()
        .query(
            "CREATE log_entry SET orchid = $orchid_id, owner = $owner, note = 'Fertilized', event_type = 'Fertilized'"
        )
        .bind(("orchid_id", oid))
        .bind(("owner", owner))
        .await;

    Ok(orchid)
}

#[server]
pub async fn mark_repotted(orchid_id: String, pot_medium: Option<String>, pot_size: Option<String>) -> Result<Orchid, ServerFnError> {
    use crate::auth::require_auth;
    use crate::db::db;
    use crate::error::internal_error;

    let user_id = require_auth().await?;
    let oid = parse_record_id(&orchid_id)?;
    let owner = parse_record_id(&user_id)?;

    let mut response = db()
        .query(
            "UPDATE $id SET last_repotted_at = time::now(), pot_medium = $pot_medium, pot_size = $pot_size WHERE owner = $owner RETURN *"
        )
        .bind(("id", oid.clone()))
        .bind(("owner", owner.clone()))
        .bind(("pot_medium", pot_medium))
        .bind(("pot_size", pot_size))
        .await
        .map_err(|e| internal_error("Mark repotted query failed", e))?;

    let errors = response.take_errors();
    if !errors.is_empty() {
        let err_msg = errors.into_values().map(|e| e.to_string()).collect::<Vec<_>>().join("; ");
        return Err(internal_error("Mark repotted query error", err_msg));
    }

    let db_row: Option<OrchidDbRow> = response.take(0)
        .map_err(|e| internal_error("Mark repotted parse failed", e))?;

    let orchid = db_row.map(|r| r.into_orchid())
        .ok_or_else(|| ServerFnError::new("Orchid not found or not owned by you"))?;

    let _ = db()
        .query(
            "CREATE log_entry SET orchid = $orchid_id, owner = $owner, note = 'Repotted', event_type = 'Repotted'"
        )
        .bind(("orchid_id", oid))
        .bind(("owner", owner))
        .await;

    Ok(orchid)
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "ssr")]
    use super::normalize_light_requirement;

    #[test]
    #[cfg(feature = "ssr")]
    fn test_normalize_light_requirement_canonical() {
        assert_eq!(normalize_light_requirement("Low"), "Low");
        assert_eq!(normalize_light_requirement("Medium"), "Medium");
        assert_eq!(normalize_light_requirement("High"), "High");
    }

    #[test]
    #[cfg(feature = "ssr")]
    fn test_normalize_light_requirement_display_aliases() {
        assert_eq!(normalize_light_requirement("Low Light"), "Low");
        assert_eq!(normalize_light_requirement("Medium Light"), "Medium");
        assert_eq!(normalize_light_requirement("High Light"), "High");
    }

    #[test]
    #[cfg(feature = "ssr")]
    fn test_normalize_light_requirement_lowercase() {
        assert_eq!(normalize_light_requirement("low"), "Low");
        assert_eq!(normalize_light_requirement("medium"), "Medium");
        assert_eq!(normalize_light_requirement("high"), "High");
    }

    #[test]
    #[cfg(feature = "ssr")]
    fn test_normalize_light_requirement_unknown_defaults_to_medium() {
        assert_eq!(normalize_light_requirement("Bright"), "Medium");
        assert_eq!(normalize_light_requirement(""), "Medium");
        assert_eq!(normalize_light_requirement("Full Sun"), "Medium");
    }

    #[test]
    #[cfg(feature = "ssr")]
    fn test_normalize_light_requirement_trims_whitespace() {
        assert_eq!(normalize_light_requirement("  Low  "), "Low");
        assert_eq!(normalize_light_requirement(" Medium Light "), "Medium");
    }
}
