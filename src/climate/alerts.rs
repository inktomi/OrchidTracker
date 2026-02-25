use chrono::{DateTime, Utc};

/// A new alert to be stored (before it has an ID)
pub struct NewAlert {
    /// The ID of the user who owns the alert.
    pub owner: surrealdb::types::RecordId,
    /// The associated orchid, if any.
    pub orchid: Option<surrealdb::types::RecordId>,
    /// The associated zone, if any.
    pub zone: Option<surrealdb::types::RecordId>,
    /// The type of the alert (e.g. temperature_low).
    pub alert_type: String,
    /// The severity level (e.g. warning, critical).
    pub severity: String,
    /// Human-readable message explaining the alert.
    pub message: String,
}

/// An orchid with its structured climate requirements
pub struct OrchidRequirements {
    /// The orchid's record ID.
    pub id: surrealdb::types::RecordId,
    /// The ID of the owner.
    pub owner: surrealdb::types::RecordId,
    /// The orchid's name.
    pub name: String,
    /// The placement or zone of the orchid.
    pub placement: String,
    /// Target watering frequency in days.
    pub water_frequency_days: u32,
    /// When the orchid was last watered.
    pub last_watered_at: Option<DateTime<Utc>>,
    /// Minimum temperature requirement in Celsius.
    pub temp_min: Option<f64>,
    /// Maximum temperature requirement in Celsius.
    pub temp_max: Option<f64>,
    /// Minimum humidity requirement percentage.
    pub humidity_min: Option<f64>,
    /// Maximum humidity requirement percentage.
    pub humidity_max: Option<f64>,
}

/// A latest reading for a zone
pub struct ZoneReading {
    /// The name of the zone.
    pub zone_name: String,
    /// The unique record ID of the zone.
    pub zone_id: surrealdb::types::RecordId,
    /// The recorded temperature in Celsius.
    pub temperature: f64,
    /// The recorded relative humidity.
    pub humidity: f64,
}

/// Check all orchids against their zone readings and watering schedules.
/// Returns new alerts that should be created.
pub fn check_alerts(
    orchids: &[OrchidRequirements],
    readings: &[ZoneReading],
) -> Vec<NewAlert> {
    let mut alerts = Vec::new();

    for orchid in orchids {
        let reading = readings.iter().find(|r| r.zone_name == orchid.placement);

        if let Some(reading) = reading {
            // Temperature checks
            if let Some(temp_min) = orchid.temp_min {
                let diff = temp_min - reading.temperature;
                if diff > 0.0 {
                    let severity = if diff > 5.0 { "critical" } else { "warning" };
                    alerts.push(NewAlert {
                        owner: orchid.owner.clone(),
                        orchid: Some(orchid.id.clone()),
                        zone: Some(reading.zone_id.clone()),
                        alert_type: "temperature_low".into(),
                        severity: severity.into(),
                        message: format!(
                            "{}: Temperature {:.1}C is below minimum {:.1}C",
                            orchid.name, reading.temperature, temp_min
                        ),
                    });
                }
            }

            if let Some(temp_max) = orchid.temp_max {
                let diff = reading.temperature - temp_max;
                if diff > 0.0 {
                    let severity = if diff > 5.0 { "critical" } else { "warning" };
                    alerts.push(NewAlert {
                        owner: orchid.owner.clone(),
                        orchid: Some(orchid.id.clone()),
                        zone: Some(reading.zone_id.clone()),
                        alert_type: "temperature_high".into(),
                        severity: severity.into(),
                        message: format!(
                            "{}: Temperature {:.1}C is above maximum {:.1}C",
                            orchid.name, reading.temperature, temp_max
                        ),
                    });
                }
            }

            // Humidity checks
            if let Some(hum_min) = orchid.humidity_min {
                let diff = hum_min - reading.humidity;
                if diff > 0.0 {
                    let severity = if diff > 15.0 { "critical" } else { "warning" };
                    alerts.push(NewAlert {
                        owner: orchid.owner.clone(),
                        orchid: Some(orchid.id.clone()),
                        zone: Some(reading.zone_id.clone()),
                        alert_type: "humidity_low".into(),
                        severity: severity.into(),
                        message: format!(
                            "{}: Humidity {:.0}% is below minimum {:.0}%",
                            orchid.name, reading.humidity, hum_min
                        ),
                    });
                }
            }

            if let Some(hum_max) = orchid.humidity_max {
                let diff = reading.humidity - hum_max;
                if diff > 0.0 {
                    let severity = if diff > 15.0 { "critical" } else { "warning" };
                    alerts.push(NewAlert {
                        owner: orchid.owner.clone(),
                        orchid: Some(orchid.id.clone()),
                        zone: Some(reading.zone_id.clone()),
                        alert_type: "humidity_high".into(),
                        severity: severity.into(),
                        message: format!(
                            "{}: Humidity {:.0}% is above maximum {:.0}%",
                            orchid.name, reading.humidity, hum_max
                        ),
                    });
                }
            }
        }

        // Watering overdue check
        if let Some(last_watered) = orchid.last_watered_at {
            let days_since = (Utc::now() - last_watered).num_days();
            if days_since > orchid.water_frequency_days as i64 {
                let overdue = days_since - orchid.water_frequency_days as i64;
                alerts.push(NewAlert {
                    owner: orchid.owner.clone(),
                    orchid: Some(orchid.id.clone()),
                    zone: None,
                    alert_type: "watering_overdue".into(),
                    severity: "info".into(),
                    message: format!(
                        "{}: Watering overdue by {} days",
                        orchid.name, overdue
                    ),
                });
            }
        }
    }

    alerts
}

/// Check alerts and store new ones + send push notifications.
/// Called after poll_all_zones().
pub async fn check_and_send_alerts() {
    use crate::db::db;
    use surrealdb::types::SurrealValue;

    #[derive(serde::Deserialize, SurrealValue)]
    #[surreal(crate = "surrealdb::types")]
    struct OrchidRow {
        id: surrealdb::types::RecordId,
        owner: surrealdb::types::RecordId,
        name: String,
        placement: String,
        water_frequency_days: u32,
        #[surreal(default)]
        last_watered_at: Option<DateTime<Utc>>,
        #[surreal(default)]
        temp_min: Option<f64>,
        #[surreal(default)]
        temp_max: Option<f64>,
        #[surreal(default)]
        humidity_min: Option<f64>,
        #[surreal(default)]
        humidity_max: Option<f64>,
    }

    #[derive(serde::Deserialize, SurrealValue)]
    #[surreal(crate = "surrealdb::types")]
    struct ReadingRow {
        zone: surrealdb::types::RecordId,
        zone_name: String,
        temperature: f64,
        humidity: f64,
    }

    #[derive(serde::Deserialize, SurrealValue)]
    #[surreal(crate = "surrealdb::types")]
    struct PushSubRow {
        owner: surrealdb::types::RecordId,
        endpoint: String,
        p256dh: String,
        auth: String,
    }

    // 1. Fetch all orchids with structured requirements
    let mut orchid_resp = match db()
        .query("SELECT id, owner, name, placement, water_frequency_days, last_watered_at, temp_min, temp_max, humidity_min, humidity_max FROM orchid WHERE temp_min IS NOT NULL OR temp_max IS NOT NULL OR humidity_min IS NOT NULL OR humidity_max IS NOT NULL OR last_watered_at IS NOT NULL")
        .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("Alert check: failed to query orchids: {}", e);
            return;
        }
    };
    let _ = orchid_resp.take_errors();
    let orchid_rows: Vec<OrchidRow> = orchid_resp.take(0).unwrap_or_default();

    if orchid_rows.is_empty() {
        return;
    }

    // 2. Get latest readings per zone (fetch recent, deduplicate by zone in Rust)
    let mut reading_resp = match db()
        .query("SELECT zone, zone_name, temperature, humidity, recorded_at FROM climate_reading WHERE recorded_at > time::now() - 2h ORDER BY recorded_at DESC")
        .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("Alert check: failed to query readings: {}", e);
            return;
        }
    };
    let _ = reading_resp.take_errors();
    let all_readings: Vec<ReadingRow> = reading_resp.take(0).unwrap_or_default();
    // Keep only the latest reading per zone (first occurrence since ordered DESC)
    let mut seen_zones = std::collections::HashSet::new();
    let reading_rows: Vec<ReadingRow> = all_readings
        .into_iter()
        .filter(|r| seen_zones.insert(format!("{:?}", r.zone)))
        .collect();

    let orchid_reqs: Vec<OrchidRequirements> = orchid_rows
        .into_iter()
        .map(|r| OrchidRequirements {
            id: r.id,
            owner: r.owner,
            name: r.name,
            placement: r.placement,
            water_frequency_days: r.water_frequency_days,
            last_watered_at: r.last_watered_at,
            temp_min: r.temp_min,
            temp_max: r.temp_max,
            humidity_min: r.humidity_min,
            humidity_max: r.humidity_max,
        })
        .collect();

    let zone_readings: Vec<ZoneReading> = reading_rows
        .into_iter()
        .map(|r| ZoneReading {
            zone_name: r.zone_name,
            zone_id: r.zone,
            temperature: r.temperature,
            humidity: r.humidity,
        })
        .collect();

    // 3. Check alerts
    let new_alerts = check_alerts(&orchid_reqs, &zone_readings);

    if new_alerts.is_empty() {
        return;
    }

    tracing::info!("Alert check: {} new alerts generated", new_alerts.len());

    // 4. Store alerts (with dedup: skip if identical unacknowledged alert from last 6h)
    for alert in &new_alerts {
        let mut dup_check = match db()
            .query(
                "SELECT count() FROM alert WHERE owner = $owner AND alert_type = $atype AND message = $msg AND acknowledged_at IS NULL AND created_at > time::now() - 6h GROUP ALL"
            )
            .bind(("owner", alert.owner.clone()))
            .bind(("atype", alert.alert_type.clone()))
            .bind(("msg", alert.message.clone()))
            .await
        {
            Ok(r) => r,
            Err(_) => continue,
        };
        let _ = dup_check.take_errors();

        #[derive(serde::Deserialize, SurrealValue)]
        #[surreal(crate = "surrealdb::types")]
        struct CountRow {
            count: i64,
        }

        let dup: Option<CountRow> = dup_check.take(0).unwrap_or(None);
        if dup.map(|c| c.count > 0).unwrap_or(false) {
            continue;
        }

        let _ = db()
            .query(
                "CREATE alert SET owner = $owner, orchid = $orchid, zone = $zone, alert_type = $atype, severity = $severity, message = $msg"
            )
            .bind(("owner", alert.owner.clone()))
            .bind(("orchid", alert.orchid.clone()))
            .bind(("zone", alert.zone.clone()))
            .bind(("atype", alert.alert_type.clone()))
            .bind(("severity", alert.severity.clone()))
            .bind(("msg", alert.message.clone()))
            .await;

        // 5. For critical/warning alerts, send push notifications
        if alert.severity == "critical" || alert.severity == "warning" {
            let mut sub_resp = match db()
                .query("SELECT owner, endpoint, p256dh, auth FROM push_subscription WHERE owner = $owner")
                .bind(("owner", alert.owner.clone()))
                .await
            {
                Ok(r) => r,
                Err(_) => continue,
            };
            let _ = sub_resp.take_errors();
            let subs: Vec<PushSubRow> = sub_resp.take(0).unwrap_or_default();

            for sub in subs {
                let push_sub = crate::push::PushSubscriptionRow {
                    endpoint: sub.endpoint,
                    p256dh: sub.p256dh,
                    auth: sub.auth,
                };
                let title = match alert.severity.as_str() {
                    "critical" => "Critical Alert",
                    _ => "Warning",
                };
                if let Err(e) = crate::push::send_push(&push_sub, title, &alert.message).await {
                    tracing::warn!("Push notification failed: {}", e);
                }
            }
        }
    }
}
