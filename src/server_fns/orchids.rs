#![allow(clippy::too_many_arguments)]

use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use crate::orchid::{Orchid, LogEntry};

/// **What is it?**
/// The struct representing the response when successfully adding a log entry for an orchid.
///
/// **Why does it exist?**
/// It exists to return both the newly created log entry and additional context (like whether this was the first bloom) so the frontend can update its state immediately.
///
/// **How should it be used?**
/// Parse this struct on the frontend after calling the `add_log_entry` server function to update the UI without needing a full reload.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AddLogEntryResponse {
    /// The newly created log entry.
    pub entry: LogEntry,
    /// Indicates if this was the first bloom recorded for the orchid.
    pub is_first_bloom: bool,
}

#[cfg(feature = "ssr")]
fn parse_record_id(id: &str) -> Result<surrealdb::types::RecordId, ServerFnError> {
    use crate::error::internal_error;
    surrealdb::types::RecordId::parse_simple(id)
        .map_err(|e| internal_error("Record ID parse failed", e))
}

#[cfg(feature = "ssr")]
pub(crate) mod ssr_types {
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
        pub pot_type: Option<String>,
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
        #[surreal(default)]
        pub par_ppfd: Option<f64>,
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
                pot_medium: self.pot_medium.and_then(|s| serde_json::from_str(&format!("\"{}\"", s)).ok()),
                pot_size: self.pot_size.and_then(|s| serde_json::from_str(&format!("\"{}\"", s)).ok()),
                pot_type: self.pot_type.and_then(|s| serde_json::from_str(&format!("\"{}\"", s)).ok()),
                rest_start_month: self.rest_start_month,
                rest_end_month: self.rest_end_month,
                bloom_start_month: self.bloom_start_month,
                bloom_end_month: self.bloom_end_month,
                rest_water_multiplier: self.rest_water_multiplier,
                rest_fertilizer_multiplier: self.rest_fertilizer_multiplier,
                active_water_multiplier: self.active_water_multiplier,
                active_fertilizer_multiplier: self.active_fertilizer_multiplier,
                par_ppfd: self.par_ppfd,
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

/// Serialize an enum to its serde variant name for DB storage as a plain string.
/// Uses serde_json serialization to get the canonical name (e.g., "SphagnumMoss" not "Sphagnum Moss").
#[cfg(feature = "ssr")]
fn enum_to_db_string<T: serde::Serialize>(val: &T) -> String {
    serde_json::to_string(val)
        .unwrap_or_default()
        .trim_matches('"')
        .to_string()
}

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
    if !(1..=365).contains(&water_frequency_days) {
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
    if let Some(cs) = conservation_status
        && cs.len() > 200
    {
        return Err(ServerFnError::new("Conservation status must be at most 200 characters"));
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

/// **What is it?**
/// A server function that retrieves the entire collection of orchids owned by the currently authenticated user.
///
/// **Why does it exist?**
/// It exists to securely query the database, ensuring users only see their own plants, and to serialize the resulting rows into frontend-compatible `Orchid` structs.
///
/// **How should it be used?**
/// Call this from the main dashboard or collection view to load and display the user's plants.
#[server]
#[tracing::instrument(level = "info", skip_all)]
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

/// **What is it?**
/// A server function that validates and creates a new orchid record in the database.
///
/// **Why does it exist?**
/// It exists to handle the complex validation and backend insertion logic needed to persist a new plant, ensuring it's safely associated with the authenticated user.
///
/// **How should it be used?**
/// Call this from the "Add Orchid" form in the user interface, passing in all the collected plant details as arguments.
#[server]
#[tracing::instrument(level = "info", skip_all)]
pub async fn create_orchid(
    /// The name given to the orchid.
    name: String,
    /// The species or hybrid name of the orchid.
    species: String,
    /// The number of days between waterings.
    water_frequency_days: u32,
    /// The light requirement for the orchid.
    light_requirement: String,
    /// Any additional notes about the orchid.
    notes: String,
    /// Where the orchid is placed.
    placement: String,
    /// The optimal light level in lux.
    light_lux: String,
    /// The optimal temperature range for the orchid.
    temperature_range: String,
    /// The conservation status in the wild.
    conservation_status: Option<String>,
    /// The native geographic region of the orchid.
    native_region: Option<String>,
    /// The native latitude coordinate.
    native_latitude: Option<f64>,
    /// The native longitude coordinate.
    native_longitude: Option<f64>,
    /// The minimum tolerated temperature.
    temp_min: Option<f64>,
    /// The maximum tolerated temperature.
    temp_max: Option<f64>,
    /// The minimum required humidity percentage.
    humidity_min: Option<f64>,
    /// The maximum tolerated humidity percentage.
    humidity_max: Option<f64>,
    /// The number of days between fertilizer applications.
    fertilize_frequency_days: Option<u32>,
    /// The type of fertilizer to use.
    fertilizer_type: Option<String>,
    /// The potting medium used for the orchid.
    pot_medium: Option<crate::orchid::PotMedium>,
    /// The size of the pot the orchid is in.
    pot_size: Option<crate::orchid::PotSize>,
    /// The type of pot the orchid is in.
    pot_type: Option<crate::orchid::PotType>,
    /// The starting month of the resting period.
    rest_start_month: Option<u32>,
    /// The ending month of the resting period.
    rest_end_month: Option<u32>,
    /// The typical starting month for blooming.
    bloom_start_month: Option<u32>,
    /// The typical ending month for blooming.
    bloom_end_month: Option<u32>,
    /// The watering multiplier during the resting period.
    rest_water_multiplier: Option<f64>,
    /// The fertilizer multiplier during the resting period.
    rest_fertilizer_multiplier: Option<f64>,
    /// The watering multiplier during active growth.
    active_water_multiplier: Option<f64>,
    /// The fertilizer multiplier during active growth.
    active_fertilizer_multiplier: Option<f64>,
    /// Measured PAR (PPFD) in µmol/m²/s.
    par_ppfd: Option<f64>,
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
             pot_medium = $pot_medium, pot_size = $pot_size, pot_type = $pot_type, \
             rest_start_month = $rest_start, rest_end_month = $rest_end, \
             bloom_start_month = $bloom_start, bloom_end_month = $bloom_end, \
             rest_water_multiplier = $rest_water_mult, rest_fertilizer_multiplier = $rest_fert_mult, \
             active_water_multiplier = $active_water_mult, active_fertilizer_multiplier = $active_fert_mult, \
             par_ppfd = $par_ppfd \
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
        .bind(("pot_medium", pot_medium.map(|v| enum_to_db_string(&v))))
        .bind(("pot_size", pot_size.map(|v| enum_to_db_string(&v))))
        .bind(("pot_type", pot_type.map(|v| enum_to_db_string(&v))))
        .bind(("rest_start", rest_start_month.map(|v| v as i64)))
        .bind(("rest_end", rest_end_month.map(|v| v as i64)))
        .bind(("bloom_start", bloom_start_month.map(|v| v as i64)))
        .bind(("bloom_end", bloom_end_month.map(|v| v as i64)))
        .bind(("rest_water_mult", rest_water_multiplier))
        .bind(("rest_fert_mult", rest_fertilizer_multiplier))
        .bind(("active_water_mult", active_water_multiplier))
        .bind(("active_fert_mult", active_fertilizer_multiplier))
        .bind(("par_ppfd", par_ppfd))
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

/// **What is it?**
/// A server function that applies changes to an existing orchid record.
///
/// **Why does it exist?**
/// It exists to allow users to modify details about a plant (like its name or placement) after it has been created, persisting those changes in the backend.
///
/// **How should it be used?**
/// Call this from the "Edit Orchid" modal, passing the fully updated `Orchid` struct containing the user's modifications.
#[server]
#[tracing::instrument(level = "info", skip_all)]
pub async fn update_orchid(
    /// The fully populated Orchid struct containing the updated data.
    orchid: Orchid
) -> Result<Orchid, ServerFnError> {
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
             pot_medium = $pot_medium, pot_size = $pot_size, pot_type = $pot_type, \
             rest_start_month = $rest_start, rest_end_month = $rest_end, \
             bloom_start_month = $bloom_start, bloom_end_month = $bloom_end, \
             rest_water_multiplier = $rest_water_mult, rest_fertilizer_multiplier = $rest_fert_mult, \
             active_water_multiplier = $active_water_mult, active_fertilizer_multiplier = $active_fert_mult, \
             par_ppfd = $par_ppfd, \
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
        .bind(("pot_medium", orchid.pot_medium.map(|v| enum_to_db_string(&v))))
        .bind(("pot_size", orchid.pot_size.map(|v| enum_to_db_string(&v))))
        .bind(("pot_type", orchid.pot_type.map(|v| enum_to_db_string(&v))))
        .bind(("rest_start", orchid.rest_start_month.map(|v| v as i64)))
        .bind(("rest_end", orchid.rest_end_month.map(|v| v as i64)))
        .bind(("bloom_start", orchid.bloom_start_month.map(|v| v as i64)))
        .bind(("bloom_end", orchid.bloom_end_month.map(|v| v as i64)))
        .bind(("rest_water_mult", orchid.rest_water_multiplier))
        .bind(("rest_fert_mult", orchid.rest_fertilizer_multiplier))
        .bind(("active_water_mult", orchid.active_water_multiplier))
        .bind(("active_fert_mult", orchid.active_fertilizer_multiplier))
        .bind(("par_ppfd", orchid.par_ppfd))
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

/// **What is it?**
/// A server function that deletes a specific orchid from the user's collection.
///
/// **Why does it exist?**
/// It exists to securely handle the removal of a plant record from the database, ensuring that only the verified owner can delete it.
///
/// **How should it be used?**
/// Call this from a confirmation dialog when the user clicks the "Delete" button on an orchid card.
#[server]
#[tracing::instrument(level = "info", skip_all)]
pub async fn delete_orchid(
    /// The unique identifier of the orchid to delete.
    id: String
) -> Result<(), ServerFnError> {
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

/// **What is it?**
/// A server function that creates a new log entry for a specific orchid, such as a watering or repotting event.
///
/// **Why does it exist?**
/// It exists to allow users to maintain a detailed history of care actions, and to automatically trigger side effects (like updating `last_watered_at`).
///
/// **How should it be used?**
/// Call this from the "Add Entry" timeline UI, specifying the plant, the type of action performed, and any optional notes or images.
#[server]
#[tracing::instrument(level = "info", skip_all)]
pub async fn add_log_entry(
    /// The unique identifier of the orchid.
    orchid_id: String,
    /// The note or description of the event.
    note: String,
    /// An optional image filename associated with the entry.
    image_filename: Option<String>,
    /// The type of event (e.g., "Watered", "Fertilized").
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
    if let Some(ref et) = event_type
        && !allowed_event_types.contains(&et.as_str())
    {
        return Err(ServerFnError::new("Invalid event type"));
    }

    let user_id = require_auth().await?;
    let orchid_record = parse_record_id(&orchid_id)?;
    let owner = parse_record_id(&user_id)?;

    // Create log entry + update care timestamps atomically
    // The WHERE clause with $event_type comparison makes non-matching UPDATEs no-ops
    let mut response = db()
        .query(
            "BEGIN TRANSACTION; \
             CREATE log_entry SET \
                 orchid = $orchid_id, owner = $owner, \
                 note = $note, image_filename = $image_filename, \
                 event_type = $event_type \
                 RETURN *; \
             UPDATE $orchid_id SET last_watered_at = time::now() WHERE owner = $owner AND $event_type = 'Watered'; \
             UPDATE $orchid_id SET last_fertilized_at = time::now() WHERE owner = $owner AND $event_type = 'Fertilized'; \
             UPDATE $orchid_id SET last_repotted_at = time::now() WHERE owner = $owner AND $event_type = 'Repotted'; \
             COMMIT TRANSACTION;"
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

    // Index 1 = CREATE log_entry result (index 0 = BEGIN)
    let db_row: Option<LogEntryDbRow> = response.take(1)
        .map_err(|e| internal_error("Add log entry parse failed", e))?;

    let entry = db_row.map(|r| r.into_log_entry())
        .ok_or_else(|| ServerFnError::new("Failed to create log entry"))?;

    // Check for first bloom (separate query — reads data created in the transaction above)
    let mut is_first_bloom = false;
    if event_type.as_deref() == Some("Flowering") {
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

        let bloom_errors = bloom_resp.take_errors();
        if !bloom_errors.is_empty() {
            let err_msg = bloom_errors.into_values().map(|e| e.to_string()).collect::<Vec<_>>().join("; ");
            return Err(internal_error("Check bloom query error", err_msg));
        }

        let bloom_count: Option<serde_json::Value> = bloom_resp.take(0)
            .unwrap_or(None);

        let count = bloom_count
            .and_then(|v| v.get("cnt").and_then(|c| c.as_i64()))
            .unwrap_or(0);

        // count == 1 means the entry we just created is the only one
        if count <= 1 {
            is_first_bloom = true;
            db()
                .query(
                    "UPDATE $orchid_id SET first_bloom_at = time::now() \
                     WHERE owner = $owner"
                )
                .bind(("orchid_id", orchid_record))
                .bind(("owner", owner))
                .await
                .map_err(|e| internal_error("Set first bloom query failed", e))?;
        }
    }

    Ok(AddLogEntryResponse { entry, is_first_bloom })
}

/// **What is it?**
/// A server function that retrieves all log entries for a specific orchid in the database.
///
/// **Why does it exist?**
/// It exists to securely query the historical timeline of care events (watering, repotting, blooming) associated with a single plant owned by the current user.
///
/// **How should it be used?**
/// Call this from the "Orchid Details" modal to load the timeline view of the plant's history.
#[server]
#[tracing::instrument(level = "info", skip_all)]
pub async fn get_log_entries(
    /// The unique identifier of the orchid.
    orchid_id: String
) -> Result<Vec<LogEntry>, ServerFnError> {
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

/// **What is it?**
/// A server function that marks a specific orchid as having just been watered.
///
/// **Why does it exist?**
/// It exists as a convenience endpoint to quickly update the `last_watered_at` timestamp and automatically create a corresponding log entry without requiring the user to fill out a full form.
///
/// **How should it be used?**
/// Call this from a "Water Now" button in the collection grid or detailed view.
#[server]
#[tracing::instrument(level = "info", skip_all, fields(orchid_id = %orchid_id))]
pub async fn mark_watered(
    /// The unique identifier of the orchid.
    orchid_id: String
) -> Result<Orchid, ServerFnError> {
    use crate::auth::require_auth;
    use crate::db::db;
    use crate::error::internal_error;

    let user_id = require_auth().await?;
    tracing::info!(orchid_id = %orchid_id, user_id = %user_id, "mark_watered called");
    let oid = parse_record_id(&orchid_id)?;
    let owner = parse_record_id(&user_id)?;

    // Update orchid + create log entry atomically
    let mut response = db()
        .query(
            "BEGIN TRANSACTION; \
             UPDATE $id SET last_watered_at = time::now() WHERE owner = $owner RETURN *; \
             CREATE log_entry SET orchid = $id, owner = $owner, note = 'Watered', event_type = 'Watered'; \
             COMMIT TRANSACTION;"
        )
        .bind(("id", oid))
        .bind(("owner", owner))
        .await
        .map_err(|e| internal_error("Mark watered query failed", e))?;

    let errors = response.take_errors();
    if !errors.is_empty() {
        let err_msg = errors.into_values().map(|e| e.to_string()).collect::<Vec<_>>().join("; ");
        return Err(internal_error("Mark watered query error", err_msg));
    }

    // Index 1 = UPDATE result (index 0 = BEGIN)
    let db_row: Option<OrchidDbRow> = response.take(1)
        .map_err(|e| internal_error("Mark watered parse failed", e))?;

    let orchid = db_row.map(|r| r.into_orchid())
        .ok_or_else(|| ServerFnError::new("Orchid not found or not owned by you"))?;

    Ok(orchid)
}

/// **What is it?**
/// A server function that marks multiple orchids as having just been watered.
///
/// **Why does it exist?**
/// It provides a bulk action endpoint to update the `last_watered_at` timestamp for a group of plants in a single request.
///
/// **How should it be used?**
/// Call this from a "Water All" button in the Today tasks view or collection view.
#[server]
#[tracing::instrument(level = "info", skip_all, fields(count = orchid_ids.len()))]
pub async fn mark_watered_batch(
    /// The unique identifiers of the orchids to water.
    orchid_ids: Vec<String>
) -> Result<Vec<Orchid>, ServerFnError> {
    use crate::auth::require_auth;
    use crate::db::db;
    use crate::error::internal_error;

    if orchid_ids.is_empty() {
        return Ok(vec![]);
    }

    let user_id = require_auth().await?;
    tracing::info!(user_id = %user_id, count = %orchid_ids.len(), "mark_watered_batch called");
    let owner = parse_record_id(&user_id)?;

    let mut oids = Vec::new();
    for id in &orchid_ids {
        oids.push(parse_record_id(id)?);
    }

    // Update orchids + create log entries atomically
    let mut response = db()
        .query(
            "BEGIN TRANSACTION; \
             UPDATE $ids SET last_watered_at = time::now() WHERE owner = $owner RETURN *; \
             FOR $oid IN $ids { \
                 CREATE log_entry SET orchid = $oid, owner = $owner, note = 'Watered', event_type = 'Watered'; \
             }; \
             COMMIT TRANSACTION;"
        )
        .bind(("ids", oids))
        .bind(("owner", owner))
        .await
        .map_err(|e| internal_error("Mark watered batch query failed", e))?;

    let errors = response.take_errors();
    if !errors.is_empty() {
        let err_msg = errors.into_values().map(|e| e.to_string()).collect::<Vec<_>>().join("; ");
        return Err(internal_error("Mark watered batch query error", err_msg));
    }

    // Index 1 = UPDATE result (index 0 = BEGIN)
    let db_rows: Vec<OrchidDbRow> = response.take(1)
        .map_err(|e| internal_error("Mark watered batch parse failed", e))?;

    let orchids: Vec<Orchid> = db_rows.into_iter().map(|r| r.into_orchid()).collect();

    Ok(orchids)
}

/// **What is it?**
/// A server function that marks a specific orchid as having just been fertilized.
///
/// **Why does it exist?**
/// It exists as a quick-action endpoint to update the `last_fertilized_at` timestamp and automatically append a "Fertilized" event to the plant's timeline.
///
/// **How should it be used?**
/// Call this from a "Fertilize Now" button when a user indicates they have applied nutrients to an orchid.
#[server]
#[tracing::instrument(level = "info", skip_all)]
pub async fn mark_fertilized(
    /// The unique identifier of the orchid.
    orchid_id: String
) -> Result<Orchid, ServerFnError> {
    use crate::auth::require_auth;
    use crate::db::db;
    use crate::error::internal_error;

    let user_id = require_auth().await?;
    let oid = parse_record_id(&orchid_id)?;
    let owner = parse_record_id(&user_id)?;

    // Update orchid + create log entry atomically
    let mut response = db()
        .query(
            "BEGIN TRANSACTION; \
             UPDATE $id SET last_fertilized_at = time::now() WHERE owner = $owner RETURN *; \
             CREATE log_entry SET orchid = $id, owner = $owner, note = 'Fertilized', event_type = 'Fertilized'; \
             COMMIT TRANSACTION;"
        )
        .bind(("id", oid))
        .bind(("owner", owner))
        .await
        .map_err(|e| internal_error("Mark fertilized query failed", e))?;

    let errors = response.take_errors();
    if !errors.is_empty() {
        let err_msg = errors.into_values().map(|e| e.to_string()).collect::<Vec<_>>().join("; ");
        return Err(internal_error("Mark fertilized query error", err_msg));
    }

    // Index 1 = UPDATE result (index 0 = BEGIN)
    let db_row: Option<OrchidDbRow> = response.take(1)
        .map_err(|e| internal_error("Mark fertilized parse failed", e))?;

    let orchid = db_row.map(|r| r.into_orchid())
        .ok_or_else(|| ServerFnError::new("Orchid not found or not owned by you"))?;

    Ok(orchid)
}

/// **What is it?**
/// A server function that marks a specific orchid as having just been repotted.
///
/// **Why does it exist?**
/// It exists to update the `last_repotted_at` timestamp and automatically track the new pot size or medium within the plant's history.
///
/// **How should it be used?**
/// Call this from the "Repotting form" after a user supplies the updated medium and size information.
#[server]
#[tracing::instrument(level = "info", skip_all)]
pub async fn mark_repotted(
    /// The unique identifier of the orchid.
    orchid_id: String, 
    /// The new potting medium used.
    pot_medium: Option<String>, 
    /// The new pot size used.
    pot_size: Option<String>
) -> Result<Orchid, ServerFnError> {
    use crate::auth::require_auth;
    use crate::db::db;
    use crate::error::internal_error;

    let user_id = require_auth().await?;
    let oid = parse_record_id(&orchid_id)?;
    let owner = parse_record_id(&user_id)?;

    // Update orchid + create log entry atomically
    let mut response = db()
        .query(
            "BEGIN TRANSACTION; \
             UPDATE $id SET last_repotted_at = time::now(), pot_medium = $pot_medium, pot_size = $pot_size WHERE owner = $owner RETURN *; \
             CREATE log_entry SET orchid = $id, owner = $owner, note = 'Repotted', event_type = 'Repotted'; \
             COMMIT TRANSACTION;"
        )
        .bind(("id", oid))
        .bind(("owner", owner))
        .bind(("pot_medium", pot_medium))
        .bind(("pot_size", pot_size))
        .await
        .map_err(|e| internal_error("Mark repotted query failed", e))?;

    let errors = response.take_errors();
    if !errors.is_empty() {
        let err_msg = errors.into_values().map(|e| e.to_string()).collect::<Vec<_>>().join("; ");
        return Err(internal_error("Mark repotted query error", err_msg));
    }

    // Index 1 = UPDATE result (index 0 = BEGIN)
    let db_row: Option<OrchidDbRow> = response.take(1)
        .map_err(|e| internal_error("Mark repotted parse failed", e))?;

    let orchid = db_row.map(|r| r.into_orchid())
        .ok_or_else(|| ServerFnError::new("Orchid not found or not owned by you"))?;

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
