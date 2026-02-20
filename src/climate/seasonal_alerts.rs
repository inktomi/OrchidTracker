use chrono::{Datelike, Utc};
use super::alerts::NewAlert;
use crate::orchid::Hemisphere;

/// Check all orchids with seasonal data for upcoming transitions.
/// Called daily from the background task.
pub async fn check_seasonal_alerts() {
    use crate::db::db;
    use surrealdb::types::SurrealValue;

    #[derive(serde::Deserialize, SurrealValue)]
    #[surreal(crate = "surrealdb::types")]
    struct SeasonalOrchidRow {
        id: surrealdb::types::RecordId,
        owner: surrealdb::types::RecordId,
        name: String,
        #[surreal(default)]
        rest_start_month: Option<u32>,
        #[surreal(default)]
        rest_end_month: Option<u32>,
        #[surreal(default)]
        bloom_start_month: Option<u32>,
        #[surreal(default)]
        bloom_end_month: Option<u32>,
    }

    #[derive(serde::Deserialize, SurrealValue)]
    #[surreal(crate = "surrealdb::types")]
    struct PrefRow {
        owner: surrealdb::types::RecordId,
        #[surreal(default)]
        hemisphere: String,
    }

    // 1. Fetch all orchids with seasonal data
    let mut orchid_resp = match db()
        .query("SELECT id, owner, name, rest_start_month, rest_end_month, bloom_start_month, bloom_end_month FROM orchid WHERE rest_start_month IS NOT NULL OR bloom_start_month IS NOT NULL")
        .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("Seasonal alert check: failed to query orchids: {}", e);
            return;
        }
    };
    let _ = orchid_resp.take_errors();
    let orchid_rows: Vec<SeasonalOrchidRow> = orchid_resp.take(0).unwrap_or_default();

    if orchid_rows.is_empty() {
        return;
    }

    // 2. Fetch hemisphere preferences for all owners
    let mut pref_resp = match db()
        .query("SELECT owner, hemisphere FROM user_preference")
        .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("Seasonal alert check: failed to query prefs: {}", e);
            return;
        }
    };
    let _ = pref_resp.take_errors();
    let pref_rows: Vec<PrefRow> = pref_resp.take(0).unwrap_or_default();

    let get_hemisphere = |owner: &surrealdb::types::RecordId| -> Hemisphere {
        pref_rows.iter()
            .find(|p| p.owner == *owner)
            .map(|p| Hemisphere::from_code(&p.hemisphere))
            .unwrap_or(Hemisphere::Northern)
    };

    let now_month = Utc::now().month();
    let next_month = if now_month == 12 { 1 } else { now_month + 1 };

    let mut alerts: Vec<NewAlert> = Vec::new();

    for orchid in &orchid_rows {
        let hemi = get_hemisphere(&orchid.owner);

        // Check rest period transitions
        if let Some(rs) = orchid.rest_start_month {
            let adjusted = hemi.adjust_month(rs);
            if adjusted == now_month || adjusted == next_month {
                let when = if adjusted == now_month { "this month" } else { "next month" };
                alerts.push(NewAlert {
                    owner: orchid.owner.clone(),
                    orchid: Some(orchid.id.clone()),
                    zone: None,
                    alert_type: "seasonal_rest_start".into(),
                    severity: "info".into(),
                    message: format!("{}: Rest period begins {}", orchid.name, when),
                });
            }
        }
        if let Some(re) = orchid.rest_end_month {
            let adjusted = hemi.adjust_month(re);
            let end_next = if adjusted == 12 { 1 } else { adjusted + 1 };
            if end_next == now_month || end_next == next_month {
                let when = if end_next == now_month { "this month" } else { "next month" };
                alerts.push(NewAlert {
                    owner: orchid.owner.clone(),
                    orchid: Some(orchid.id.clone()),
                    zone: None,
                    alert_type: "seasonal_rest_end".into(),
                    severity: "info".into(),
                    message: format!("{}: Rest period ends {}", orchid.name, when),
                });
            }
        }

        // Check bloom transitions
        if let Some(bs) = orchid.bloom_start_month {
            let adjusted = hemi.adjust_month(bs);
            if adjusted == now_month || adjusted == next_month {
                let when = if adjusted == now_month { "this month" } else { "next month" };
                alerts.push(NewAlert {
                    owner: orchid.owner.clone(),
                    orchid: Some(orchid.id.clone()),
                    zone: None,
                    alert_type: "seasonal_bloom_start".into(),
                    severity: "info".into(),
                    message: format!("{}: Bloom season begins {}", orchid.name, when),
                });
            }
        }
        if let Some(be) = orchid.bloom_end_month {
            let adjusted = hemi.adjust_month(be);
            let end_next = if adjusted == 12 { 1 } else { adjusted + 1 };
            if end_next == now_month || end_next == next_month {
                let when = if end_next == now_month { "this month" } else { "next month" };
                alerts.push(NewAlert {
                    owner: orchid.owner.clone(),
                    orchid: Some(orchid.id.clone()),
                    zone: None,
                    alert_type: "seasonal_bloom_end".into(),
                    severity: "info".into(),
                    message: format!("{}: Bloom season ends {}", orchid.name, when),
                });
            }
        }
    }

    if alerts.is_empty() {
        return;
    }

    tracing::info!("Seasonal alert check: {} alerts generated", alerts.len());

    // 3. Store alerts with dedup
    for alert in &alerts {
        // Skip if identical unacknowledged alert from last 24h
        let mut dup_check = match db()
            .query(
                "SELECT count() FROM alert WHERE owner = $owner AND alert_type = $atype AND message = $msg AND acknowledged_at IS NULL AND created_at > time::now() - 24h GROUP ALL"
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
    }
}
