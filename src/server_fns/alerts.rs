use leptos::prelude::*;
use crate::orchid::Alert;

/// **What is it?**
/// A server function that retrieves the VAPID public key from the backend configuration.
///
/// **Why does it exist?**
/// It exists because the browser's Push API requires a VAPID public key to authenticate the application and securely register for push notifications.
///
/// **How should it be used?**
/// Call this from the frontend when initializing or toggling push notifications to get the key needed for the `pushManager.subscribe()` call.
#[server]
#[tracing::instrument(level = "info", skip_all)]
pub async fn get_vapid_public_key() -> Result<String, ServerFnError> {
    use crate::auth::require_auth;
    use crate::config::config;

    require_auth().await?;
    Ok(config().vapid_public_key.clone())
}

/// **What is it?**
/// A server function that saves the current user's browser push subscription details to the database.
///
/// **Why does it exist?**
/// It exists to securely map a user's device endpoint and cryptographic keys to their account, allowing the backend to route push notifications to them.
///
/// **How should it be used?**
/// Pass the `endpoint`, `p256dh` public key, and `auth` secret returned by the browser's `pushManager.subscribe()` into this function to enable notifications on that device.
#[server]
#[tracing::instrument(level = "info", skip_all)]
pub async fn subscribe_push(
    /// The push notification endpoint URL.
    endpoint: String,
    /// The P-256 elliptic curve Diffie-Hellman public key.
    p256dh: String,
    /// The authentication secret for the subscription.
    auth: String,
) -> Result<(), ServerFnError> {
    use crate::auth::require_auth;
    use crate::db::db;
    use crate::error::internal_error;

    let user_id = require_auth().await?;
    let owner = surrealdb::types::RecordId::parse_simple(&user_id)
        .map_err(|e| internal_error("Owner ID parse failed", e))?;

    // Upsert: delete existing subscriptions for this user + endpoint, then create
    let mut del_resp = db()
        .query("DELETE push_subscription WHERE owner = $owner AND endpoint = $endpoint")
        .bind(("owner", owner.clone()))
        .bind(("endpoint", endpoint.clone()))
        .await
        .map_err(|e| internal_error("Delete push sub query failed", e))?;

    let del_errors = del_resp.take_errors();
    if !del_errors.is_empty() {
        let msg = del_errors.into_values().map(|e| e.to_string()).collect::<Vec<_>>().join("; ");
        return Err(internal_error("Delete push sub statement error", msg));
    }

    // Note: bind param is "sub_auth" not "auth" — $auth is a SurrealDB system variable
    let mut create_resp = db()
        .query(
            "CREATE push_subscription SET owner = $owner, endpoint = $endpoint, p256dh = $p256dh, auth = $sub_auth"
        )
        .bind(("owner", owner))
        .bind(("endpoint", endpoint))
        .bind(("p256dh", p256dh))
        .bind(("sub_auth", auth))
        .await
        .map_err(|e| internal_error("Subscribe push query failed", e))?;

    let create_errors = create_resp.take_errors();
    if !create_errors.is_empty() {
        let msg = create_errors.into_values().map(|e| e.to_string()).collect::<Vec<_>>().join("; ");
        return Err(internal_error("Subscribe push statement error", msg));
    }

    tracing::info!(user = %user_id, "Push subscription stored successfully");
    Ok(())
}

/// **What is it?**
/// A server function that removes all push notification subscriptions for the current authenticated user.
///
/// **Why does it exist?**
/// It exists to provide an opt-out mechanism for users who no longer wish to receive notifications, ensuring their subscription keys are deleted from the database.
///
/// **How should it be used?**
/// Call this from the settings UI when a user turns off the push notifications toggle.
#[server]
#[tracing::instrument(level = "info", skip_all)]
pub async fn unsubscribe_push() -> Result<(), ServerFnError> {
    use crate::auth::require_auth;
    use crate::db::db;
    use crate::error::internal_error;

    let user_id = require_auth().await?;
    let owner = surrealdb::types::RecordId::parse_simple(&user_id)
        .map_err(|e| internal_error("Owner ID parse failed", e))?;

    db()
        .query("DELETE push_subscription WHERE owner = $owner")
        .bind(("owner", owner))
        .await
        .map_err(|e| internal_error("Unsubscribe push query failed", e))?;

    Ok(())
}

/// **What is it?**
/// A server function that retrieves a list of active, unacknowledged alerts for the currently authenticated user.
///
/// **Why does it exist?**
/// It exists to keep the user informed about urgent issues requiring their attention, such as critical temperature drops or watering reminders.
///
/// **How should it be used?**
/// Query this function from an application-wide notification panel or polling loop to populate the user's current alerts view.
#[server]
#[tracing::instrument(level = "info", skip_all)]
pub async fn get_active_alerts() -> Result<Vec<Alert>, ServerFnError> {
    use crate::auth::require_auth;
    use crate::db::db;
    use crate::error::internal_error;
    use surrealdb::types::SurrealValue;

    let user_id = require_auth().await?;
    let owner = surrealdb::types::RecordId::parse_simple(&user_id)
        .map_err(|e| internal_error("Owner ID parse failed", e))?;

    #[derive(serde::Deserialize, SurrealValue)]
    #[surreal(crate = "surrealdb::types")]
    struct AlertDbRow {
        id: surrealdb::types::RecordId,
        alert_type: String,
        severity: String,
        message: String,
        created_at: chrono::DateTime<chrono::Utc>,
    }

    let mut response = db()
        .query(
            "SELECT id, alert_type, severity, message, created_at FROM alert WHERE owner = $owner AND acknowledged_at IS NULL ORDER BY created_at DESC LIMIT 20"
        )
        .bind(("owner", owner))
        .await
        .map_err(|e| internal_error("Get alerts query failed", e))?;

    let errors = response.take_errors();
    if !errors.is_empty() {
        let err_msg = errors.into_values().map(|e| e.to_string()).collect::<Vec<_>>().join("; ");
        return Err(internal_error("Get alerts query error", err_msg));
    }

    let rows: Vec<AlertDbRow> = response.take(0)
        .map_err(|e| internal_error("Get alerts parse failed", e))?;

    Ok(rows.into_iter().map(|r| {
        Alert {
            id: crate::server_fns::auth::record_id_to_string(&r.id),
            orchid_name: None,
            zone_name: None,
            alert_type: r.alert_type,
            severity: r.severity,
            message: r.message,
            created_at: r.created_at,
        }
    }).collect())
}

/// **What is it?**
/// A server function that checks if the current user has any active push notification subscriptions on record.
///
/// **Why does it exist?**
/// It exists to determine the initial state of the "Enable Push Notifications" toggle in the user interface.
///
/// **How should it be used?**
/// Fetch this boolean value when loading the user's notification settings to reflect whether their browser is currently subscribed.
#[server]
#[tracing::instrument(level = "info", skip_all)]
pub async fn has_push_subscription() -> Result<bool, ServerFnError> {
    use crate::auth::require_auth;
    use crate::db::db;
    use crate::error::internal_error;
    use surrealdb::types::SurrealValue;

    let user_id = require_auth().await?;
    let owner = surrealdb::types::RecordId::parse_simple(&user_id)
        .map_err(|e| internal_error("Owner ID parse failed", e))?;

    #[derive(serde::Deserialize, SurrealValue)]
    #[surreal(crate = "surrealdb::types")]
    struct CountRow {
        count: i64,
    }

    let mut resp = db()
        .query("SELECT count() FROM push_subscription WHERE owner = $owner GROUP ALL")
        .bind(("owner", owner))
        .await
        .map_err(|e| internal_error("Check push sub query failed", e))?;

    let _ = resp.take_errors();
    let row: Option<CountRow> = resp.take(0).unwrap_or(None);
    Ok(row.map(|r| r.count > 0).unwrap_or(false))
}

/// **What is it?**
/// A server function that triggers a test push notification to all endpoints subscribed by the current user.
///
/// **Why does it exist?**
/// It exists to provide an immediate way for users to confirm their browser and operating system settings are correctly configured to receive alerts from OrchidTracker.
///
/// **How should it be used?**
/// Call this from the frontend when the user clicks a "Send Test Notification" button in the settings panel.
#[server]
#[tracing::instrument(level = "info", skip_all)]
pub async fn send_test_push() -> Result<String, ServerFnError> {
    use crate::auth::require_auth;
    use crate::db::db;
    use crate::error::internal_error;
    use surrealdb::types::SurrealValue;

    let user_id = require_auth().await?;
    let owner = surrealdb::types::RecordId::parse_simple(&user_id)
        .map_err(|e| internal_error("Owner ID parse failed", e))?;

    #[derive(serde::Deserialize, SurrealValue)]
    #[surreal(crate = "surrealdb::types")]
    struct PushSubRow {
        endpoint: String,
        p256dh: String,
        auth: String,
    }

    let mut resp = db()
        .query("SELECT endpoint, p256dh, auth FROM push_subscription WHERE owner = $owner")
        .bind(("owner", owner))
        .await
        .map_err(|e| internal_error("Query push subs failed", e))?;

    let _ = resp.take_errors();
    let subs: Vec<PushSubRow> = resp.take(0).unwrap_or_default();

    tracing::info!(user = %user_id, count = subs.len(), "send_test_push: found subscriptions");

    if subs.is_empty() {
        return Ok("No push subscriptions found. Try toggling notifications off and on.".into());
    }

    let mut sent = 0;
    let mut errors = Vec::new();
    for sub in &subs {
        tracing::info!(
            endpoint = %sub.endpoint,
            p256dh_len = sub.p256dh.len(),
            auth_len = sub.auth.len(),
            "send_test_push: sending to subscription"
        );
        let push_sub = crate::push::PushSubscriptionRow {
            endpoint: sub.endpoint.clone(),
            p256dh: sub.p256dh.clone(),
            auth: sub.auth.clone(),
        };
        match crate::push::send_push(
            &push_sub,
            "Test Notification",
            "Push notifications are working! You'll receive alerts for watering and climate conditions.",
        ).await {
            Ok(()) => sent += 1,
            Err(e) => errors.push(e.to_string()),
        }
    }

    if sent > 0 && errors.is_empty() {
        Ok(format!("Sent to {} subscription(s)", sent))
    } else if sent > 0 {
        Ok(format!("Sent to {}/{} subscription(s). Errors: {}", sent, subs.len(), errors.join(", ")))
    } else {
        Err(ServerFnError::new(format!("Failed to send: {}", errors.join(", "))))
    }
}

/// **What is it?**
/// A server function that marks a specific alert as acknowledged by the current user.
///
/// **Why does it exist?**
/// It exists to clear an alert from the active list, indicating that the user has seen and resolved or accepted the notification.
///
/// **How should it be used?**
/// Call this function when the user clicks the "Acknowledge" or "X" button on an active alert component.
#[server]
#[tracing::instrument(level = "info", skip_all)]
pub async fn acknowledge_alert(
    /// The unique identifier of the alert to acknowledge.
    alert_id: String
) -> Result<(), ServerFnError> {
    use crate::auth::require_auth;
    use crate::db::db;
    use crate::error::internal_error;

    let user_id = require_auth().await?;
    let owner = surrealdb::types::RecordId::parse_simple(&user_id)
        .map_err(|e| internal_error("Owner ID parse failed", e))?;
    let aid = surrealdb::types::RecordId::parse_simple(&alert_id)
        .map_err(|e| internal_error("Alert ID parse failed", e))?;

    db()
        .query("UPDATE $id SET acknowledged_at = time::now() WHERE owner = $owner")
        .bind(("id", aid))
        .bind(("owner", owner))
        .await
        .map_err(|e| internal_error("Acknowledge alert query failed", e))?;

    Ok(())
}
