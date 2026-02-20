use leptos::prelude::*;

#[server]
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

#[server]
pub async fn save_temp_unit(unit: String) -> Result<(), ServerFnError> {
    use crate::auth::require_auth;
    use crate::db::db;
    use crate::error::internal_error;

    let user_id = require_auth().await?;
    let owner = surrealdb::types::RecordId::parse_simple(&user_id)
        .map_err(|e| internal_error("Owner ID parse failed", e))?;

    // Validate
    let unit = if unit == "F" { "F" } else { "C" };

    // Upsert: delete then create (SurrealDB UPSERT with schemafull can be tricky)
    let _ = db()
        .query("DELETE user_preference WHERE owner = $owner")
        .bind(("owner", owner.clone()))
        .await;

    db()
        .query("CREATE user_preference SET owner = $owner, temp_unit = $unit")
        .bind(("owner", owner))
        .bind(("unit", unit.to_string()))
        .await
        .map_err(|e| internal_error("Save preference query failed", e))?;

    Ok(())
}
