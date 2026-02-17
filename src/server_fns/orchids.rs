use leptos::prelude::*;
use crate::orchid::{Orchid, LogEntry};

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

    validate_orchid_fields(&name, &species, &notes, water_frequency_days, &light_requirement, &placement, &light_lux, &temperature_range, &conservation_status)?;

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

    let light_req_str = orchid.light_requirement.to_string();
    let placement_str = orchid.placement.to_string();

    validate_orchid_fields(&orchid.name, &orchid.species, &orchid.notes, orchid.water_frequency_days, &light_req_str, &placement_str, &orchid.light_lux, &orchid.temperature_range, &orchid.conservation_status)?;

    let user_id = require_auth().await?;

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

    if note.len() > 5000 {
        return Err(ServerFnError::new("Note must be at most 5000 characters"));
    }
    if let Some(ref filename) = image_filename {
        validate_filename(filename)?;
    }

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
