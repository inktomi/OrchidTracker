use leptos::prelude::*;
use crate::orchid::GrowingZone;

/// Parse the "table:key" user_id string into a SurrealDB RecordId
#[cfg(feature = "ssr")]
fn parse_owner(user_id: &str) -> Result<surrealdb::types::RecordId, ServerFnError> {
    use crate::error::internal_error;
    surrealdb::types::RecordId::parse_simple(user_id)
        .map_err(|e| internal_error("Owner ID parse failed", e))
}

/// SSR-only struct matching SurrealDB's record shape (id is a RecordId, not a String).
#[cfg(feature = "ssr")]
pub(crate) mod ssr_types {
    use surrealdb::types::SurrealValue;
    use crate::orchid::{GrowingZone, LightRequirement, LocationType};
    use crate::server_fns::auth::record_id_to_string;

    #[derive(serde::Deserialize, SurrealValue)]
    #[surreal(crate = "surrealdb::types")]
    pub struct GrowingZoneDbRow {
        pub id: surrealdb::types::RecordId,
        pub name: String,
        /// Stored as plain string in DB; SurrealValue untagged enum can't round-trip
        pub light_level: String,
        pub location_type: String,
        #[surreal(default)]
        pub temperature_range: String,
        #[surreal(default)]
        pub humidity: String,
        #[surreal(default)]
        pub description: String,
        #[surreal(default)]
        pub sort_order: i32,
        #[surreal(default)]
        pub data_source_type: Option<String>,
        #[surreal(default)]
        pub data_source_config: String,
        #[surreal(default)]
        pub hardware_device: Option<surrealdb::types::RecordId>,
        #[surreal(default)]
        pub hardware_port: Option<i32>,
    }

    impl GrowingZoneDbRow {
        pub fn into_growing_zone(self) -> GrowingZone {
            GrowingZone {
                id: record_id_to_string(&self.id),
                name: self.name,
                light_level: parse_light_level(&self.light_level),
                location_type: parse_location_type(&self.location_type),
                temperature_range: self.temperature_range,
                humidity: self.humidity,
                description: self.description,
                sort_order: self.sort_order,
                data_source_type: self.data_source_type,
                data_source_config: crate::crypto::decrypt_or_raw(&self.data_source_config),
                hardware_device_id: self.hardware_device.as_ref().map(record_id_to_string),
                hardware_port: self.hardware_port,
            }
        }
    }

    pub fn parse_light_level(s: &str) -> LightRequirement {
        match s {
            "Low" => LightRequirement::Low,
            "High" => LightRequirement::High,
            _ => LightRequirement::Medium,
        }
    }

    pub fn parse_location_type(s: &str) -> LocationType {
        match s {
            "Outdoor" => LocationType::Outdoor,
            _ => LocationType::Indoor,
        }
    }
}

#[cfg(feature = "ssr")]
use ssr_types::*;

#[server]
#[tracing::instrument(level = "info", skip_all)]
pub async fn get_zones() -> Result<Vec<GrowingZone>, ServerFnError> {
    use crate::auth::require_auth;
    use crate::db::db;
    use crate::error::internal_error;

    let user_id = require_auth().await?;
    let owner = parse_owner(&user_id)?;

    let mut response = db()
        .query("SELECT * FROM growing_zone WHERE owner = $owner ORDER BY sort_order ASC")
        .bind(("owner", owner))
        .await
        .map_err(|e| internal_error("Get zones query failed", e))?;

    let errors = response.take_errors();
    if !errors.is_empty() {
        let err_msg = errors.into_values().map(|e| e.to_string()).collect::<Vec<_>>().join("; ");
        return Err(internal_error("Get zones query error", err_msg));
    }

    let db_rows: Vec<GrowingZoneDbRow> = response.take(0)
        .map_err(|e| internal_error("Get zones parse failed", e))?;

    Ok(db_rows.into_iter().map(|r| r.into_growing_zone()).collect())
}

#[server]
#[tracing::instrument(level = "info", skip_all)]
pub async fn create_zone(
    name: String,
    light_level: String,
    location_type: String,
    temperature_range: String,
    humidity: String,
    description: String,
    sort_order: i32,
) -> Result<GrowingZone, ServerFnError> {
    use crate::auth::require_auth;
    use crate::db::db;
    use crate::error::internal_error;

    if name.is_empty() || name.len() > 100 {
        return Err(ServerFnError::new("Zone name must be 1-100 characters"));
    }
    if !["Low", "Medium", "High"].contains(&light_level.as_str()) {
        return Err(ServerFnError::new("Light level must be Low, Medium, or High"));
    }
    if !["Indoor", "Outdoor"].contains(&location_type.as_str()) {
        return Err(ServerFnError::new("Location type must be Indoor or Outdoor"));
    }
    if temperature_range.len() > 100 {
        return Err(ServerFnError::new("Temperature range must be at most 100 characters"));
    }
    if humidity.len() > 100 {
        return Err(ServerFnError::new("Humidity must be at most 100 characters"));
    }
    if description.len() > 500 {
        return Err(ServerFnError::new("Description must be at most 500 characters"));
    }

    let user_id = require_auth().await?;
    let owner = parse_owner(&user_id)?;

    let mut response = db()
        .query(
            "CREATE growing_zone SET \
             owner = $owner, name = $name, light_level = $light_level, \
             location_type = $location_type, temperature_range = $temp_range, \
             humidity = $humidity, description = $description, sort_order = $sort_order \
             RETURN *"
        )
        .bind(("owner", owner))
        .bind(("name", name))
        .bind(("light_level", light_level))
        .bind(("location_type", location_type))
        .bind(("temp_range", temperature_range))
        .bind(("humidity", humidity))
        .bind(("description", description))
        .bind(("sort_order", sort_order as i64))
        .await
        .map_err(|e| internal_error("Create zone query failed", e))?;

    let errors = response.take_errors();
    if !errors.is_empty() {
        let err_msg = errors.into_values().map(|e| e.to_string()).collect::<Vec<_>>().join("; ");
        return Err(internal_error("Create zone query error", err_msg));
    }

    let db_row: Option<GrowingZoneDbRow> = response.take(0)
        .map_err(|e| internal_error("Create zone parse failed", e))?;

    db_row.map(|r| r.into_growing_zone())
        .ok_or_else(|| ServerFnError::new("Failed to create zone"))
}

#[server]
#[tracing::instrument(level = "info", skip_all)]
pub async fn update_zone(zone: GrowingZone) -> Result<GrowingZone, ServerFnError> {
    use crate::auth::require_auth;
    use crate::db::db;
    use crate::error::internal_error;

    if zone.name.is_empty() || zone.name.len() > 100 {
        return Err(ServerFnError::new("Zone name must be 1-100 characters"));
    }

    let user_id = require_auth().await?;
    let owner = parse_owner(&user_id)?;
    let zone_id = surrealdb::types::RecordId::parse_simple(&zone.id)
        .map_err(|e| internal_error("Zone ID parse failed", e))?;

    let light_level_str = match zone.light_level {
        crate::orchid::LightRequirement::Low => "Low",
        crate::orchid::LightRequirement::Medium => "Medium",
        crate::orchid::LightRequirement::High => "High",
    };
    let location_type_str = match zone.location_type {
        crate::orchid::LocationType::Indoor => "Indoor",
        crate::orchid::LocationType::Outdoor => "Outdoor",
    };

    let mut response = db()
        .query(
            "UPDATE $id SET \
             name = $name, light_level = $light_level, \
             location_type = $location_type, temperature_range = $temp_range, \
             humidity = $humidity, description = $description, sort_order = $sort_order \
             WHERE owner = $owner \
             RETURN *"
        )
        .bind(("id", zone_id))
        .bind(("owner", owner))
        .bind(("name", zone.name))
        .bind(("light_level", light_level_str.to_string()))
        .bind(("location_type", location_type_str.to_string()))
        .bind(("temp_range", zone.temperature_range))
        .bind(("humidity", zone.humidity))
        .bind(("description", zone.description))
        .bind(("sort_order", zone.sort_order as i64))
        .await
        .map_err(|e| internal_error("Update zone query failed", e))?;

    let errors = response.take_errors();
    if !errors.is_empty() {
        let err_msg = errors.into_values().map(|e| e.to_string()).collect::<Vec<_>>().join("; ");
        return Err(internal_error("Update zone query error", err_msg));
    }

    let updated: Option<GrowingZoneDbRow> = response.take(0)
        .map_err(|e| internal_error("Update zone parse failed", e))?;

    updated.map(|r| r.into_growing_zone())
        .ok_or_else(|| ServerFnError::new("Zone not found or not owned by you"))
}

#[server]
#[tracing::instrument(level = "info", skip_all)]
pub async fn delete_zone(id: String) -> Result<(), ServerFnError> {
    use crate::auth::require_auth;
    use crate::db::db;
    use crate::error::internal_error;

    let user_id = require_auth().await?;
    let owner = parse_owner(&user_id)?;
    let zone_id = surrealdb::types::RecordId::parse_simple(&id)
        .map_err(|e| internal_error("Zone ID parse failed", e))?;

    db()
        .query("DELETE $id WHERE owner = $owner")
        .bind(("id", zone_id))
        .bind(("owner", owner))
        .await
        .map_err(|e| internal_error("Delete zone query failed", e))?;

    Ok(())
}

#[server]
#[tracing::instrument(level = "info", skip_all)]
pub async fn migrate_legacy_placements() -> Result<bool, ServerFnError> {
    use crate::auth::require_auth;
    use crate::db::db;
    use crate::error::internal_error;
    use crate::orchid::Orchid;

    let user_id = require_auth().await?;
    let owner = parse_owner(&user_id)?;

    // Check if user already has zones
    let mut response = db()
        .query("SELECT count() as total FROM growing_zone WHERE owner = $owner GROUP ALL")
        .bind(("owner", owner.clone()))
        .await
        .map_err(|e| internal_error("Check zones count failed", e))?;

    let errors = response.take_errors();
    if !errors.is_empty() {
        // Table might not exist yet, treat as 0 zones
        tracing::warn!("Zone count query errors (may be first run): {:?}", errors);
    } else {
        let count_row: Option<CountRow> = response.take(0)
            .map_err(|e| internal_error("Zone count parse failed", e))?;
        if let Some(row) = count_row {
            if row.total > 0 {
                return Ok(false); // Already has zones, skip migration
            }
        }
    }

    // Check if user has any orchids to migrate from
    let mut response = db()
        .query("SELECT * FROM orchid WHERE owner = $owner")
        .bind(("owner", owner.clone()))
        .await
        .map_err(|e| internal_error("Get orchids for migration failed", e))?;

    let _ = response.take_errors();
    let orchids: Vec<Orchid> = response.take(0).unwrap_or_default();

    if orchids.is_empty() {
        return Ok(false); // No orchids to migrate
    }

    // Collect unique placement values
    let mut placements: Vec<String> = orchids.iter().map(|o| o.placement.clone()).collect();
    placements.sort();
    placements.dedup();

    // Map legacy placement values to zones
    let legacy_map: Vec<(&str, &str, &str, &str, i32)> = vec![
        ("Low", "Low Light Area", "Low", "Indoor", 3),
        ("Medium", "Medium Light Area", "Medium", "Indoor", 2),
        ("High", "High Light Area", "High", "Indoor", 1),
        ("Patio", "Patio", "Medium", "Outdoor", 4),
        ("OutdoorRack", "Outdoor Rack", "High", "Outdoor", 5),
    ];

    for (old_val, new_name, light, location, order) in &legacy_map {
        if placements.iter().any(|p| p == *old_val) {
            let _ = db()
                .query(
                    "CREATE growing_zone SET \
                     owner = $owner, name = $name, light_level = $light_level, \
                     location_type = $location_type, temperature_range = '', \
                     humidity = '', description = '', sort_order = $sort_order"
                )
                .bind(("owner", owner.clone()))
                .bind(("name", new_name.to_string()))
                .bind(("light_level", light.to_string()))
                .bind(("location_type", location.to_string()))
                .bind(("sort_order", *order as i64))
                .await;

            // Update orchid placement strings
            let _ = db()
                .query("UPDATE orchid SET placement = $new_name WHERE owner = $owner AND placement = $old_val")
                .bind(("owner", owner.clone()))
                .bind(("new_name", new_name.to_string()))
                .bind(("old_val", old_val.to_string()))
                .await;
        }
    }

    Ok(true)
}

#[cfg(feature = "ssr")]
mod count_row_impl {
    use surrealdb::types::SurrealValue;

    #[derive(serde::Deserialize, surrealdb::types::SurrealValue)]
    #[surreal(crate = "surrealdb::types")]
    pub(super) struct CountRow {
        #[allow(dead_code)]
        pub total: i64,
    }
}
#[cfg(feature = "ssr")]
use count_row_impl::CountRow;
