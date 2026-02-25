use clap::{Parser, Subcommand};

use crate::auth::hash_password;
use crate::db::db;
use surrealdb::types::SurrealValue;

/// Command-line arguments for the OrchidTracker server application.
#[derive(Parser)]
#[command(name = "orchid-tracker", about = "OrchidTracker web server")]
pub struct Cli {
    /// The subcommand to execute.
    #[command(subcommand)]
    pub command: Option<Command>,
}

/// Available CLI subcommands.
#[derive(Subcommand)]
pub enum Command {
    /// Reset a user's password
    ResetPassword {
        /// The username to reset
        #[arg(short, long)]
        username: String,
        /// The new password
        #[arg(short, long)]
        password: String,
    },
    /// Re-run AI analysis on all plants for a user
    ReprocessPlants {
        /// Username whose plants to reprocess
        #[arg(short, long)]
        user: String,
        /// Plants per batch before pausing (default: 5)
        #[arg(long, default_value = "5")]
        batch_size: usize,
        /// Seconds to wait between batches (default: 30)
        #[arg(long, default_value = "30")]
        delay_secs: u64,
        /// Print what would be done without calling the AI
        #[arg(long)]
        dry_run: bool,
    },
}

/// Executes the reset-password subcommand, hashing and updating the user's password.
pub async fn run_reset_password(username: &str, password: &str) -> Result<(), Box<dyn std::error::Error>> {
    let hash = hash_password(password)?;

    let mut response = db()
        .query("UPDATE user SET password_hash = $hash WHERE username = $username")
        .bind(("hash", hash))
        .bind(("username", username.to_owned()))
        .await?;

    let errors = response.take_errors();
    if !errors.is_empty() {
        let err_msg = errors.into_values().map(|e| e.to_string()).collect::<Vec<_>>().join("; ");
        return Err(format!("Database error: {}", err_msg).into());
    }

    // Check that a row was actually updated
    #[derive(serde::Deserialize, surrealdb::types::SurrealValue)]
    #[surreal(crate = "surrealdb::types")]
    struct UpdatedRow {
        #[allow(dead_code)]
        username: String,
    }

    let rows: Vec<UpdatedRow> = response.take(0)?;
    if rows.is_empty() {
        return Err(format!("No user found with username '{}'", username).into());
    }

    tracing::info!("Password reset successfully for user '{}'", username);
    Ok(())
}

/// Executes the reprocess-plants subcommand, running AI analysis on a user's orchids.
pub async fn run_reprocess_plants(
    username: &str,
    batch_size: usize,
    delay_secs: u64,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::server_fns::orchids::ssr_types::OrchidDbRow;
    use crate::server_fns::scanner::analyze_species_core;

    // Look up user by username
    #[derive(serde::Deserialize, SurrealValue)]
    #[surreal(crate = "surrealdb::types")]
    struct UserRow {
        id: surrealdb::types::RecordId,
    }

    let mut user_resp = db()
        .query("SELECT id FROM user WHERE username = $username")
        .bind(("username", username.to_owned()))
        .await?;

    let _ = user_resp.take_errors();
    let users: Vec<UserRow> = user_resp.take(0)?;
    let user = users.into_iter().next()
        .ok_or_else(|| format!("No user found with username '{}'", username))?;
    let owner = user.id.clone();

    tracing::info!("Found user '{}' ({:?})", username, owner);

    // Query all orchids for this user
    let mut orchid_resp = db()
        .query("SELECT * FROM orchid WHERE owner = $owner ORDER BY name ASC")
        .bind(("owner", owner.clone()))
        .await?;

    let _ = orchid_resp.take_errors();
    let orchid_rows: Vec<OrchidDbRow> = orchid_resp.take(0)?;
    let orchids: Vec<_> = orchid_rows.into_iter().map(|r| r.into_orchid()).collect();

    tracing::info!("Found {} orchids for user '{}'", orchids.len(), username);

    // Build climate summary from zones + latest readings
    let climate_summary = build_climate_summary_for_owner(&owner).await;
    let zone_names = get_zone_names_for_owner(&owner).await;

    // Collect existing species names
    let existing_species: Vec<String> = orchids.iter()
        .filter(|o| !o.species.is_empty())
        .map(|o| o.species.clone())
        .collect();

    let mut succeeded = 0u32;
    let mut failed = 0u32;
    let mut skipped = 0u32;

    for (i, orchid) in orchids.iter().enumerate() {
        if orchid.species.trim().is_empty() {
            tracing::info!("Skipping '{}' (no species name)", orchid.name);
            skipped += 1;
            continue;
        }

        if dry_run {
            tracing::info!("[DRY RUN] Would reprocess: '{}' ({})", orchid.name, orchid.species);
            skipped += 1;
            continue;
        }

        tracing::info!("[{}/{}] Analyzing '{}' ({})", i + 1, orchids.len(), orchid.name, orchid.species);

        match analyze_species_core(&orchid.species, &climate_summary, &zone_names, &existing_species).await {
            Ok(result) => {
                if let Err(e) = update_orchid_ai_fields(&orchid.id, &result).await {
                    tracing::warn!("Failed to update '{}': {}", orchid.name, e);
                    failed += 1;
                } else {
                    tracing::info!("Updated '{}': temp={}-{}C, humidity={}-{}%",
                        orchid.name,
                        result.temp_min.map_or("?".into(), |v| format!("{:.0}", v)),
                        result.temp_max.map_or("?".into(), |v| format!("{:.0}", v)),
                        result.humidity_min.map_or("?".into(), |v| format!("{:.0}", v)),
                        result.humidity_max.map_or("?".into(), |v| format!("{:.0}", v)),
                    );
                    succeeded += 1;
                }
            }
            Err(e) => {
                tracing::warn!("AI analysis failed for '{}': {}", orchid.name, e);
                failed += 1;
            }
        }

        // Batch delay
        if (i + 1) % batch_size == 0 && i + 1 < orchids.len() {
            tracing::info!("Batch complete, waiting {} seconds...", delay_secs);
            tokio::time::sleep(std::time::Duration::from_secs(delay_secs)).await;
        }
    }

    tracing::info!(
        "Reprocessing complete: {} succeeded, {} failed, {} skipped",
        succeeded, failed, skipped
    );

    Ok(())
}

/// Build a climate summary string from DB zone readings (no auth context needed).
async fn build_climate_summary_for_owner(owner: &surrealdb::types::RecordId) -> String {
    #[derive(serde::Deserialize, SurrealValue)]
    #[surreal(crate = "surrealdb::types")]
    struct ZoneRow {
        id: surrealdb::types::RecordId,
        name: String,
    }

    #[derive(serde::Deserialize, SurrealValue)]
    #[surreal(crate = "surrealdb::types")]
    struct ReadingRow {
        #[allow(dead_code)]
        id: surrealdb::types::RecordId,
        temperature: f64,
        humidity: f64,
        #[surreal(default)]
        vpd: Option<f64>,
    }

    let mut zone_resp = match db()
        .query("SELECT id, name FROM growing_zone WHERE owner = $owner")
        .bind(("owner", owner.clone()))
        .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("Failed to query zones: {}", e);
            return "No live climate data available".to_string();
        }
    };

    let _ = zone_resp.take_errors();
    let zones: Vec<ZoneRow> = zone_resp.take(0).unwrap_or_default();

    if zones.is_empty() {
        return "No live climate data available".to_string();
    }

    let mut parts = Vec::new();
    for zone in &zones {
        let mut resp = match db()
            .query("SELECT * FROM climate_reading WHERE zone = $zone_id ORDER BY recorded_at DESC LIMIT 1")
            .bind(("zone_id", zone.id.clone()))
            .await
        {
            Ok(r) => r,
            Err(_) => continue,
        };
        let _ = resp.take_errors();
        let reading: Option<ReadingRow> = resp.take(0).unwrap_or(None);
        if let Some(r) = reading {
            let vpd_str = r.vpd.map(|v| format!(", {:.2} kPa VPD", v)).unwrap_or_default();
            parts.push(format!("{}: {:.1}C, {:.1}% Humidity{}", zone.name, r.temperature, r.humidity, vpd_str));
        }
    }

    if parts.is_empty() {
        "No live climate data available".to_string()
    } else {
        parts.join(" | ")
    }
}

/// Get zone names for a user (no auth context needed).
async fn get_zone_names_for_owner(owner: &surrealdb::types::RecordId) -> Vec<String> {
    #[derive(serde::Deserialize, SurrealValue)]
    #[surreal(crate = "surrealdb::types")]
    struct NameRow {
        name: String,
    }

    let mut resp = match db()
        .query("SELECT name FROM growing_zone WHERE owner = $owner ORDER BY sort_order ASC")
        .bind(("owner", owner.clone()))
        .await
    {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    let _ = resp.take_errors();
    let rows: Vec<NameRow> = resp.take(0).unwrap_or_default();
    rows.into_iter().map(|r| r.name).collect()
}

/// Update only AI-derived fields on an orchid record, preserving user-set fields.
async fn update_orchid_ai_fields(
    orchid_id: &str,
    result: &crate::components::scanner::AnalysisResult,
) -> Result<(), Box<dyn std::error::Error>> {
    let record = surrealdb::types::RecordId::parse_simple(orchid_id)
        .map_err(|e| format!("Parse orchid ID failed: {}", e))?;

    let light_req = result.light_req.as_str().to_string();

    let mut response = db()
        .query(
            "UPDATE $id SET \
                water_frequency_days = $water_freq, \
                light_requirement = $light_req, \
                temperature_range = $temp_range, \
                temp_min = $temp_min, \
                temp_max = $temp_max, \
                humidity_min = $humidity_min, \
                humidity_max = $humidity_max, \
                conservation_status = $conservation_status, \
                native_region = $native_region, \
                native_latitude = $native_latitude, \
                native_longitude = $native_longitude, \
                rest_start_month = $rest_start_month, \
                rest_end_month = $rest_end_month, \
                bloom_start_month = $bloom_start_month, \
                bloom_end_month = $bloom_end_month, \
                rest_water_multiplier = $rest_water_multiplier, \
                rest_fertilizer_multiplier = $rest_fertilizer_multiplier, \
                active_water_multiplier = $active_water_multiplier, \
                active_fertilizer_multiplier = $active_fertilizer_multiplier"
        )
        .bind(("id", record))
        .bind(("water_freq", result.water_freq))
        .bind(("light_req", light_req))
        .bind(("temp_range", result.temp_range.clone()))
        .bind(("temp_min", result.temp_min))
        .bind(("temp_max", result.temp_max))
        .bind(("humidity_min", result.humidity_min))
        .bind(("humidity_max", result.humidity_max))
        .bind(("conservation_status", result.conservation_status.clone()))
        .bind(("native_region", result.native_region.clone()))
        .bind(("native_latitude", result.native_latitude))
        .bind(("native_longitude", result.native_longitude))
        .bind(("rest_start_month", result.rest_start_month))
        .bind(("rest_end_month", result.rest_end_month))
        .bind(("bloom_start_month", result.bloom_start_month))
        .bind(("bloom_end_month", result.bloom_end_month))
        .bind(("rest_water_multiplier", result.rest_water_multiplier))
        .bind(("rest_fertilizer_multiplier", result.rest_fertilizer_multiplier))
        .bind(("active_water_multiplier", result.active_water_multiplier))
        .bind(("active_fertilizer_multiplier", result.active_fertilizer_multiplier))
        .await?;

    let errors = response.take_errors();
    if !errors.is_empty() {
        let err_msg = errors.into_values().map(|e| e.to_string()).collect::<Vec<_>>().join("; ");
        return Err(format!("DB update error: {}", err_msg).into());
    }

    Ok(())
}
