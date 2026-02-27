use leptos::prelude::*;

/// **What is it?**
/// A server function that retrieves the user's preferred temperature unit ("C" or "F").
///
/// **Why does it exist?**
/// It exists to ensure that climate data is displayed according to the individual user's regional preferences rather than a forced default.
///
/// **How should it be used?**
/// Call this from the frontend upon application load or when rendering settings panels to populate the current temperature unit state.
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

/// **What is it?**
/// A server function that saves the user's preferred temperature unit to the database.
///
/// **Why does it exist?**
/// It exists so users can persistently configure whether they see temperatures in Celsius or Fahrenheit across all their devices.
///
/// **How should it be used?**
/// Call this when the user changes their temperature unit preference in the settings UI.
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

    let errors = resp.take_errors();
    if !errors.is_empty() {
        let err_msg = errors.into_values().map(|e| e.to_string()).collect::<Vec<_>>().join("; ");
        return Err(internal_error("Save preference query error", err_msg));
    }

    // If no row existed, create one
    let updated: Vec<serde_json::Value> = resp.take(0).unwrap_or_default();
    if updated.is_empty() {
        db()
            .query("CREATE user_preference SET owner = $owner, temp_unit = $unit")
            .bind(("owner", owner))
            .bind(("unit", unit.to_string()))
            .await
            .map_err(|e| internal_error("Create preference query failed", e))?;
    }

    Ok(())
}

/// **What is it?**
/// A server function that retrieves the user's preferred hemisphere ("N" or "S").
///
/// **Why does it exist?**
/// It exists because seasonal context (e.g., when winter or summer occurs) is critical for determining correct orchid care and climate alerts.
///
/// **How should it be used?**
/// Fetch this value when initializing the user session to drive hemisphere-dependent calculations or display correct seasonal information.
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

/// **What is it?**
/// A server function that saves the user's hemisphere preference.
///
/// **Why does it exist?**
/// It allows users to manually specify their geographic hemisphere, correcting or setting the baseline for seasonal alerts and plant care cycles.
///
/// **How should it be used?**
/// Call this from the settings form whenever the user updates their location preferences.
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

    let errors = resp.take_errors();
    if !errors.is_empty() {
        let err_msg = errors.into_values().map(|e| e.to_string()).collect::<Vec<_>>().join("; ");
        return Err(internal_error("Save hemisphere query error", err_msg));
    }

    // If no row existed, create one
    let updated: Vec<serde_json::Value> = resp.take(0).unwrap_or_default();
    if updated.is_empty() {
        db()
            .query("CREATE user_preference SET owner = $owner, hemisphere = $hemi")
            .bind(("owner", owner))
            .bind(("hemi", hemisphere.to_string()))
            .await
            .map_err(|e| internal_error("Create hemisphere preference query failed", e))?;
    }

    Ok(())
}

/// **What is it?**
/// A server function that checks if the user's orchid collection is marked as public.
///
/// **Why does it exist?**
/// It exists to determine the privacy level of a collection, controlling whether unauthenticated users or a public gallery link can access the user's orchids.
///
/// **How should it be used?**
/// Query this to show the current visibility status in the user's settings or to conditionally enable sharing features on the frontend.
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

/// **What is it?**
/// A server function that updates whether the user's collection should be public or private.
///
/// **Why does it exist?**
/// It provides the means for users to toggle their collection's privacy status on the backend, granting or revoking public viewing access.
///
/// **How should it be used?**
/// Call this function when the user clicks a "Make Public" or "Make Private" toggle in their account settings.
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

    let errors = resp.take_errors();
    if !errors.is_empty() {
        let err_msg = errors.into_values().map(|e| e.to_string()).collect::<Vec<_>>().join("; ");
        return Err(internal_error("Save collection_public query error", err_msg));
    }

    // If no row existed, create one
    let updated: Vec<serde_json::Value> = resp.take(0).unwrap_or_default();
    if updated.is_empty() {
        db()
            .query("CREATE user_preference SET owner = $owner, collection_public = $public")
            .bind(("owner", owner))
            .bind(("public", public))
            .await
            .map_err(|e| internal_error("Create collection_public preference query failed", e))?;
    }

    Ok(())
}
