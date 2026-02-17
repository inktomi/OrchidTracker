use leptos::prelude::*;
use crate::orchid::{Orchid, LogEntry};

#[server]
pub async fn get_orchids() -> Result<Vec<Orchid>, ServerFnError> {
    use crate::auth::require_auth;
    use crate::db::db;

    let user_id = require_auth().await?;

    let orchids: Vec<Orchid> = db()
        .query("SELECT * FROM orchid WHERE owner = $owner ORDER BY created_at DESC")
        .bind(("owner", user_id))
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?
        .take(0)
        .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

    Ok(orchids)
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
) -> Result<Orchid, ServerFnError> {
    use crate::auth::require_auth;
    use crate::db::db;

    let user_id = require_auth().await?;

    let orchid: Option<Orchid> = db()
        .query(
            "CREATE orchid SET \
             owner = $owner, name = $name, species = $species, \
             water_frequency_days = $water_freq, light_requirement = $light_req, \
             notes = $notes, placement = $placement, light_lux = $light_lux, \
             temperature_range = $temp_range, conservation_status = $conservation \
             RETURN *"
        )
        .bind(("owner", user_id))
        .bind(("name", name))
        .bind(("species", species))
        .bind(("water_freq", water_frequency_days as i64))
        .bind(("light_req", light_requirement))
        .bind(("notes", notes))
        .bind(("placement", placement))
        .bind(("light_lux", light_lux))
        .bind(("temp_range", temperature_range))
        .bind(("conservation", conservation_status))
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?
        .take(0)
        .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

    orchid.ok_or_else(|| ServerFnError::new("Failed to create orchid"))
}

#[server]
pub async fn update_orchid(orchid: Orchid) -> Result<Orchid, ServerFnError> {
    use crate::auth::require_auth;
    use crate::db::db;

    let user_id = require_auth().await?;

    let light_req_str = orchid.light_requirement.to_string();
    let placement_str = orchid.placement.to_string();

    let updated: Option<Orchid> = db()
        .query(
            "UPDATE $id SET \
             name = $name, species = $species, \
             water_frequency_days = $water_freq, light_requirement = $light_req, \
             notes = $notes, placement = $placement, light_lux = $light_lux, \
             temperature_range = $temp_range, conservation_status = $conservation, \
             updated_at = time::now() \
             WHERE owner = $owner \
             RETURN *"
        )
        .bind(("id", orchid.id))
        .bind(("owner", user_id))
        .bind(("name", orchid.name))
        .bind(("species", orchid.species))
        .bind(("water_freq", orchid.water_frequency_days as i64))
        .bind(("light_req", light_req_str))
        .bind(("notes", orchid.notes))
        .bind(("placement", placement_str))
        .bind(("light_lux", orchid.light_lux))
        .bind(("temp_range", orchid.temperature_range))
        .bind(("conservation", orchid.conservation_status))
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?
        .take(0)
        .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

    updated.ok_or_else(|| ServerFnError::new("Orchid not found or not owned by you"))
}

#[server]
pub async fn delete_orchid(id: String) -> Result<(), ServerFnError> {
    use crate::auth::require_auth;
    use crate::db::db;

    let user_id = require_auth().await?;

    db()
        .query("DELETE $id WHERE owner = $owner")
        .bind(("id", id))
        .bind(("owner", user_id))
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

    Ok(())
}

#[server]
pub async fn add_log_entry(
    orchid_id: String,
    note: String,
    image_filename: Option<String>,
) -> Result<LogEntry, ServerFnError> {
    use crate::auth::require_auth;
    use crate::db::db;

    let user_id = require_auth().await?;

    let entry: Option<LogEntry> = db()
        .query(
            "CREATE log_entry SET \
             orchid = $orchid_id, owner = $owner, \
             note = $note, image_filename = $image_filename \
             RETURN *"
        )
        .bind(("orchid_id", orchid_id))
        .bind(("owner", user_id))
        .bind(("note", note))
        .bind(("image_filename", image_filename))
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?
        .take(0)
        .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

    entry.ok_or_else(|| ServerFnError::new("Failed to create log entry"))
}

#[server]
pub async fn get_log_entries(orchid_id: String) -> Result<Vec<LogEntry>, ServerFnError> {
    use crate::auth::require_auth;
    use crate::db::db;

    let user_id = require_auth().await?;

    let entries: Vec<LogEntry> = db()
        .query("SELECT * FROM log_entry WHERE orchid = $orchid_id AND owner = $owner ORDER BY timestamp DESC")
        .bind(("orchid_id", orchid_id))
        .bind(("owner", user_id))
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?
        .take(0)
        .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

    Ok(entries)
}
