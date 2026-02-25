use leptos::prelude::*;
use crate::orchid::{Orchid, GrowingZone, ClimateReading, LogEntry};

/// Resolve a username to a user_id, verifying that their collection is public.
/// Returns the user_id string (e.g. "user:abc123") or an error.
#[cfg(feature = "ssr")]
async fn resolve_public_user(username: &str) -> Result<String, ServerFnError> {
    use crate::db::db;
    use crate::error::internal_error;
    use crate::server_fns::auth::record_id_to_string;
    use surrealdb::types::SurrealValue;

    if username.is_empty() || username.len() > 50 {
        return Err(ServerFnError::new("User not found"));
    }

    // Look up user by username and get their public preference in one query
    #[derive(serde::Deserialize, SurrealValue)]
    #[surreal(crate = "surrealdb::types")]
    struct UserRow {
        id: surrealdb::types::RecordId,
        is_public: Option<bool>,
    }

    let mut resp = db()
        .query("
            SELECT 
                id, 
                (SELECT VALUE collection_public FROM user_preference WHERE owner = $parent.id LIMIT 1)[0] AS is_public 
            FROM user 
            WHERE username = $uname 
            LIMIT 1
        ")
        .bind(("uname", username.to_string()))
        .await
        .map_err(|e| internal_error("Public user lookup failed", e))?;

    let _ = resp.take_errors();
    let user_row: Option<UserRow> = resp.take(0).unwrap_or(None);
    let user_row = user_row.ok_or_else(|| ServerFnError::new("User not found"))?;
    
    if !user_row.is_public.unwrap_or(false) {
        return Err(ServerFnError::new("This collection is private"));
    }

    Ok(record_id_to_string(&user_row.id))
}

/// **What is it?**
/// A server function that retrieves all orchids for a given username, provided their collection is marked as public.
///
/// **Why does it exist?**
/// It exists to allow unauthenticated guests to view a user's plant gallery, while strictly enforcing privacy settings at the database layer.
///
/// **How should it be used?**
/// Call this from the public gallery route (`/public/:username`) to load the grid of orchids.
#[server]
#[tracing::instrument(level = "info", skip_all)]
pub async fn get_public_orchids(
    /// The username of the user whose collection to view.
    username: String
) -> Result<Vec<Orchid>, ServerFnError> {
    use crate::db::db;
    use crate::error::internal_error;
    use crate::server_fns::climate::parse_owner;
    use crate::server_fns::orchids::ssr_types::OrchidDbRow;

    let user_id = resolve_public_user(&username).await?;
    let owner = parse_owner(&user_id)?;

    let mut response = db()
        .query("SELECT * FROM orchid WHERE owner = $owner ORDER BY created_at DESC")
        .bind(("owner", owner))
        .await
        .map_err(|e| internal_error("Public get orchids query failed", e))?;

    let errors = response.take_errors();
    if !errors.is_empty() {
        let err_msg = errors.into_values().map(|e| e.to_string()).collect::<Vec<_>>().join("; ");
        return Err(internal_error("Public get orchids query error", err_msg));
    }

    let db_rows: Vec<OrchidDbRow> = response.take(0)
        .map_err(|e| internal_error("Public get orchids parse failed", e))?;

    Ok(db_rows.into_iter().map(|r| r.into_orchid()).collect())
}

/// **What is it?**
/// A server function that retrieves the growing zones for a given user, provided their collection is public.
///
/// **Why does it exist?**
/// It exists so that public viewers can understand the context of where the orchids are grown (e.g., "Living Room" vs "Greenhouse") without exposing private data.
///
/// **How should it be used?**
/// Fetch this alongside public orchids to properly render placement filters or zone labels on the public gallery.
#[server]
#[tracing::instrument(level = "info", skip_all)]
pub async fn get_public_zones(
    /// The username of the user whose collection to view.
    username: String
) -> Result<Vec<GrowingZone>, ServerFnError> {
    use crate::db::db;
    use crate::error::internal_error;
    use crate::server_fns::climate::parse_owner;
    use crate::server_fns::zones::ssr_types::GrowingZoneDbRow;

    let user_id = resolve_public_user(&username).await?;
    let owner = parse_owner(&user_id)?;

    let mut response = db()
        .query("SELECT * FROM growing_zone WHERE owner = $owner ORDER BY sort_order ASC")
        .bind(("owner", owner))
        .await
        .map_err(|e| internal_error("Public get zones query failed", e))?;

    let errors = response.take_errors();
    if !errors.is_empty() {
        let err_msg = errors.into_values().map(|e| e.to_string()).collect::<Vec<_>>().join("; ");
        return Err(internal_error("Public get zones query error", err_msg));
    }

    let db_rows: Vec<GrowingZoneDbRow> = response.take(0)
        .map_err(|e| internal_error("Public get zones parse failed", e))?;

    Ok(db_rows.into_iter().map(|r| r.into_growing_zone()).collect())
}

/// **What is it?**
/// A server function that retrieves the most recent climate reading for each of a public user's configured growing zones.
///
/// **Why does it exist?**
/// It exists to show visitors the current environmental conditions (temperature, humidity) that a public user's orchids are experiencing right now.
///
/// **How should it be used?**
/// Call this from the public gallery page to populate live weather widgets or zone status banners for visitors.
#[server]
#[tracing::instrument(level = "info", skip_all)]
pub async fn get_public_climate_readings(
    /// The username of the user whose collection to view.
    username: String
) -> Result<Vec<ClimateReading>, ServerFnError> {
    use crate::db::db;
    use crate::error::internal_error;
    use crate::server_fns::climate::parse_owner;
    use crate::server_fns::climate::ssr_types::{ZoneIdRow, ReadingDbRow};

    let user_id = resolve_public_user(&username).await?;
    let owner = parse_owner(&user_id)?;

    let mut zone_resp = db()
        .query("SELECT id, name FROM growing_zone WHERE owner = $owner")
        .bind(("owner", owner))
        .await
        .map_err(|e| internal_error("Public get climate zones query failed", e))?;

    let errors = zone_resp.take_errors();
    if !errors.is_empty() {
        let err_msg = errors.into_values().map(|e| e.to_string()).collect::<Vec<_>>().join("; ");
        return Err(internal_error("Public get climate zones query error", err_msg));
    }

    let zones: Vec<ZoneIdRow> = zone_resp.take(0)
        .map_err(|e| internal_error("Public get climate zones parse failed", e))?;

    let mut readings = Vec::new();
    let mut set = tokio::task::JoinSet::new();

    for zone in &zones {
        let zone_id = zone.id.clone();
        set.spawn(async move {
            let mut resp = db()
                .query(
                    "SELECT * FROM climate_reading WHERE zone = $zone_id ORDER BY recorded_at DESC LIMIT 1"
                )
                .bind(("zone_id", zone_id))
                .await
                .map_err(|e| internal_error("Public get reading query failed", e))?;

            let _ = resp.take_errors();
            let reading: Option<ReadingDbRow> = resp.take(0).unwrap_or(None);
            Ok::<_, ServerFnError>(reading)
        });
    }

    while let Some(res) = set.join_next().await {
        let reading = res.map_err(|e| internal_error("Join error", e))??;
        if let Some(row) = reading {
            readings.push(row.into_climate_reading());
        }
    }

    Ok(readings)
}

/// **What is it?**
/// A server function that retrieves the log entries (care history, blooming events) for a specific orchid in a public collection.
///
/// **Why does it exist?**
/// It exists to let visitors dive into the specific care history of a plant they find interesting on a public gallery without being authenticated.
///
/// **How should it be used?**
/// Query this from the public orchid details modal or dedicated page when a guest clicks on an individual plant card.
#[server]
#[tracing::instrument(level = "info", skip_all)]
pub async fn get_public_log_entries(
    /// The username of the user whose collection to view.
    username: String, 
    /// The unique identifier of the orchid.
    orchid_id: String
) -> Result<Vec<LogEntry>, ServerFnError> {
    use crate::db::db;
    use crate::error::internal_error;
    use crate::server_fns::climate::parse_owner;
    use crate::server_fns::orchids::ssr_types::LogEntryDbRow;

    let user_id = resolve_public_user(&username).await?;
    let owner = parse_owner(&user_id)?;
    let orchid_record = surrealdb::types::RecordId::parse_simple(&orchid_id)
        .map_err(|e| internal_error("Orchid ID parse failed", e))?;

    let mut response = db()
        .query("SELECT * FROM log_entry WHERE orchid = $orchid_id AND owner = $owner ORDER BY timestamp DESC")
        .bind(("orchid_id", orchid_record))
        .bind(("owner", owner))
        .await
        .map_err(|e| internal_error("Public get log entries query failed", e))?;

    let errors = response.take_errors();
    if !errors.is_empty() {
        let err_msg = errors.into_values().map(|e| e.to_string()).collect::<Vec<_>>().join("; ");
        return Err(internal_error("Public get log entries query error", err_msg));
    }

    let db_rows: Vec<LogEntryDbRow> = response.take(0)
        .map_err(|e| internal_error("Public get log entries parse failed", e))?;

    Ok(db_rows.into_iter().map(|r| r.into_log_entry()).collect())
}

/// **What is it?**
/// A server function that gets the preferred hemisphere ("N" or "S") for a public user.
///
/// **Why does it exist?**
/// It exists because displaying accurate season-dependent care information (like resting months) to a public visitor requires knowing where the grower is located geographically.
///
/// **How should it be used?**
/// Call this from a public gallery route to properly interpret bloom or rest months based on the owner's hemisphere rather than the viewer's location.
#[server]
#[tracing::instrument(level = "info", skip_all)]
pub async fn get_public_hemisphere(
    /// The username of the user.
    username: String
) -> Result<String, ServerFnError> {
    use crate::db::db;
    use crate::error::internal_error;
    use crate::server_fns::climate::parse_owner;
    use surrealdb::types::SurrealValue;

    let user_id = resolve_public_user(&username).await?;
    let owner = parse_owner(&user_id)?;

    #[derive(serde::Deserialize, SurrealValue)]
    #[surreal(crate = "surrealdb::types")]
    struct PrefRow {
        hemisphere: String,
    }

    let mut resp = db()
        .query("SELECT hemisphere FROM user_preference WHERE owner = $owner LIMIT 1")
        .bind(("owner", owner))
        .await
        .map_err(|e| internal_error("Public get hemisphere query failed", e))?;

    let _ = resp.take_errors();
    let row: Option<PrefRow> = resp.take(0).unwrap_or(None);
    Ok(row.map(|r| r.hemisphere).unwrap_or_else(|| "N".to_string()))
}

/// **What is it?**
/// A server function that gets the preferred temperature unit ("C" or "F") for a public user.
///
/// **Why does it exist?**
/// It exists to render temperature readings and alerts on the public gallery in the unit format that the collection owner prefers, preserving their intent.
///
/// **How should it be used?**
/// Call this when hydrating the public gallery page to format climate graphs or temperature alerts correctly.
#[server]
#[tracing::instrument(level = "info", skip_all)]
pub async fn get_public_temp_unit(
    /// The username of the user.
    username: String
) -> Result<String, ServerFnError> {
    use crate::db::db;
    use crate::error::internal_error;
    use crate::server_fns::climate::parse_owner;
    use surrealdb::types::SurrealValue;

    let user_id = resolve_public_user(&username).await?;
    let owner = parse_owner(&user_id)?;

    #[derive(serde::Deserialize, SurrealValue)]
    #[surreal(crate = "surrealdb::types")]
    struct PrefRow {
        temp_unit: String,
    }

    let mut resp = db()
        .query("SELECT temp_unit FROM user_preference WHERE owner = $owner LIMIT 1")
        .bind(("owner", owner))
        .await
        .map_err(|e| internal_error("Public get temp_unit query failed", e))?;

    let _ = resp.take_errors();
    let row: Option<PrefRow> = resp.take(0).unwrap_or(None);
    Ok(row.map(|r| r.temp_unit).unwrap_or_else(|| "C".to_string()))
}
