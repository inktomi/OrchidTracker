use crate::config::config;

/// A push subscription row from the database
pub struct PushSubscriptionRow {
    pub endpoint: String,
    pub p256dh: String,
    pub auth: String,
}

/// Send a Web Push notification to a single subscriber.
pub async fn send_push(
    subscription: &PushSubscriptionRow,
    title: &str,
    body: &str,
) -> Result<(), crate::error::AppError> {
    use web_push::*;

    let cfg = config();

    if cfg.vapid_private_key.is_empty() {
        tracing::error!("VAPID private key not configured");
        return Err(crate::error::AppError::Network(
            "VAPID private key not configured".into(),
        ));
    }

    tracing::info!(
        endpoint = %subscription.endpoint,
        title = %title,
        "Sending push notification"
    );

    let subscription_info = SubscriptionInfo::new(
        &subscription.endpoint,
        &subscription.p256dh,
        &subscription.auth,
    );

    let payload = serde_json::json!({
        "title": title,
        "body": body,
        "url": "/"
    });

    let mut builder = WebPushMessageBuilder::new(&subscription_info);
    let payload_bytes = payload.to_string().into_bytes();
    builder.set_payload(ContentEncoding::Aes128Gcm, &payload_bytes);

    let sig_builder = VapidSignatureBuilder::from_base64(
        &cfg.vapid_private_key,
        URL_SAFE_NO_PAD,
        &subscription_info,
    )
    .map_err(|e| {
        tracing::error!("VAPID key error: {}", e);
        crate::error::AppError::Network(format!("VAPID key error: {}", e))
    })?;

    let sig = sig_builder
        .build()
        .map_err(|e| {
            tracing::error!("VAPID sign error: {}", e);
            crate::error::AppError::Network(format!("VAPID sign error: {}", e))
        })?;

    builder.set_vapid_signature(sig);

    let message = builder
        .build()
        .map_err(|e| {
            tracing::error!("Push message build error: {}", e);
            crate::error::AppError::Network(format!("Push message build error: {}", e))
        })?;

    let client = IsahcWebPushClient::new()
        .map_err(|e| {
            tracing::error!("Push client error: {}", e);
            crate::error::AppError::Network(format!("Push client error: {}", e))
        })?;

    client
        .send(message)
        .await
        .map_err(|e| {
            tracing::error!(endpoint = %subscription.endpoint, "Push send failed: {}", e);
            crate::error::AppError::Network(format!("Push send error: {}", e))
        })?;

    tracing::info!(endpoint = %subscription.endpoint, "Push notification sent successfully");
    Ok(())
}
