use leptos::prelude::*;

/// Get the user's preferred temperature unit ("C" or "F").
#[server]
#[tracing::instrument(level = "info", skip_all)]
pub async fn get_temp_unit() -> Result<String, ServerFnError> {
    use crate::auth::require_auth;
    use crate::db::db;
    use crate::error::internal_error;
    use surrealdb::types::SurrealValue;

    let user_id = require_auth().await?;
    let owner = surrealdb::types::RecordId::parse_simple(&user_id)
        .map_err(|e| internal_error("Owner ID parse failed", e))?;

    #[derive(serde::Deserialize, SurrealValue)]
    #[surreal(crate = "surrealdb::types")]
    struct PrefRow {
        temp_unit: String,
    }

    let mut resp = db()
        .query("SELECT temp_unit FROM user_preference WHERE owner = $owner LIMIT 1")
        .bind(("owner", owner))
        .await
        .map_err(|e| internal_error("Get preference query failed", e))?;

    let _ = resp.take_errors();
    let row: Option<PrefRow> = resp.take(0).unwrap_or(None);
    Ok(row.map(|r| r.temp_unit).unwrap_or_else(|| "C".to_string()))
}

/// Save the user's preferred temperature unit.
#[server]
#[tracing::instrument(level = "info", skip_all)]
pub async fn save_temp_unit(
    /// The temperature unit ("C" or "F").
    unit: String
) -> Result<(), ServerFnError> {
    use crate::auth::require_auth;
    use crate::db::db;
    use crate::error::internal_error;

    let user_id = require_auth().await?;
    let owner = surrealdb::types::RecordId::parse_simple(&user_id)
        .map_err(|e| internal_error("Owner ID parse failed", e))?;

    // Validate
    let unit = if unit == "F" { "F" } else { "C" };

    // Update existing preference row (preserves other fields)
    let mut resp = db()
        .query("UPDATE user_preference SET temp_unit = $unit WHERE owner = $owner")
        .bind(("owner", owner.clone()))
        .bind(("unit", unit.to_string()))
        .await
        .map_err(|e| internal_error("Save preference query failed", e))?;

    let _ = resp.take_errors();

    // If no row existed, create one
    let updated: Vec<serde_json::Value> = resp.take(0).unwrap_or_default();
    if updated.is_empty() {
        let _ = db()
            .query("CREATE user_preference SET owner = $owner, temp_unit = $unit")
            .bind(("owner", owner))
            .bind(("unit", unit.to_string()))
            .await;
    }

    Ok(())
}

/// Get the user's preferred hemisphere ("N" or "S").
#[server]
#[tracing::instrument(level = "info", skip_all)]
pub async fn get_hemisphere() -> Result<String, ServerFnError> {
    use crate::auth::require_auth;
    use crate::db::db;
    use crate::error::internal_error;
    use surrealdb::types::SurrealValue;

    let user_id = require_auth().await?;
    let owner = surrealdb::types::RecordId::parse_simple(&user_id)
        .map_err(|e| internal_error("Owner ID parse failed", e))?;

    #[derive(serde::Deserialize, SurrealValue)]
    #[surreal(crate = "surrealdb::types")]
    struct PrefRow {
        hemisphere: String,
    }

    let mut resp = db()
        .query("SELECT hemisphere FROM user_preference WHERE owner = $owner LIMIT 1")
        .bind(("owner", owner))
        .await
        .map_err(|e| internal_error("Get hemisphere query failed", e))?;

    let _ = resp.take_errors();
    let row: Option<PrefRow> = resp.take(0).unwrap_or(None);
    Ok(row.map(|r| r.hemisphere).unwrap_or_else(|| "N".to_string()))
}

/// Save the user's preferred hemisphere.
#[server]
#[tracing::instrument(level = "info", skip_all)]
pub async fn save_hemisphere(
    /// The hemisphere ("N" or "S").
    hemisphere: String
) -> Result<(), ServerFnError> {
    use crate::auth::require_auth;
    use crate::db::db;
    use crate::error::internal_error;

    let user_id = require_auth().await?;
    let owner = surrealdb::types::RecordId::parse_simple(&user_id)
        .map_err(|e| internal_error("Owner ID parse failed", e))?;

    let hemisphere = if hemisphere == "S" { "S" } else { "N" };

    // Update existing preference row (preserves temp_unit and other fields)
    let mut resp = db()
        .query("UPDATE user_preference SET hemisphere = $hemi WHERE owner = $owner")
        .bind(("owner", owner.clone()))
        .bind(("hemi", hemisphere.to_string()))
        .await
        .map_err(|e| internal_error("Save hemisphere query failed", e))?;

    let _ = resp.take_errors();

    // If no row existed, create one
    let updated: Vec<serde_json::Value> = resp.take(0).unwrap_or_default();
    if updated.is_empty() {
        let _ = db()
            .query("CREATE user_preference SET owner = $owner, hemisphere = $hemi")
            .bind(("owner", owner))
            .bind(("hemi", hemisphere.to_string()))
            .await;
    }

    Ok(())
}

/// Check if the user's collection is marked as public.
#[server]
#[tracing::instrument(level = "info", skip_all)]
pub async fn get_collection_public() -> Result<bool, ServerFnError> {
    use crate::auth::require_auth;
    use crate::db::db;
    use crate::error::internal_error;
    use surrealdb::types::SurrealValue;

    let user_id = require_auth().await?;
    let owner = surrealdb::types::RecordId::parse_simple(&user_id)
        .map_err(|e| internal_error("Owner ID parse failed", e))?;

    #[derive(serde::Deserialize, SurrealValue)]
    #[surreal(crate = "surrealdb::types")]
    struct PrefRow {
        #[surreal(default)]
        collection_public: bool,
    }

    let mut resp = db()
        .query("SELECT collection_public FROM user_preference WHERE owner = $owner LIMIT 1")
        .bind(("owner", owner))
        .await
        .map_err(|e| internal_error("Get collection_public query failed", e))?;

    let _ = resp.take_errors();
    let row: Option<PrefRow> = resp.take(0).unwrap_or(None);
    Ok(row.map(|r| r.collection_public).unwrap_or(false))
}

/// Set whether the user's collection should be public.
#[server]
#[tracing::instrument(level = "info", skip_all)]
pub async fn save_collection_public(
    /// True if public, false if private.
    public: bool
) -> Result<(), ServerFnError> {
    use crate::auth::require_auth;
    use crate::db::db;
    use crate::error::internal_error;

    let user_id = require_auth().await?;
    let owner = surrealdb::types::RecordId::parse_simple(&user_id)
        .map_err(|e| internal_error("Owner ID parse failed", e))?;

    let mut resp = db()
        .query("UPDATE user_preference SET collection_public = $public WHERE owner = $owner")
        .bind(("owner", owner.clone()))
        .bind(("public", public))
        .await
        .map_err(|e| internal_error("Save collection_public query failed", e))?;

    let _ = resp.take_errors();

    // If no row existed, create one
    let updated: Vec<serde_json::Value> = resp.take(0).unwrap_or_default();
    if updated.is_empty() {
        let _ = db()
            .query("CREATE user_preference SET owner = $owner, collection_public = $public")
            .bind(("owner", owner))
            .bind(("public", public))
            .await;
    }

    Ok(())
}
