use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// A structured client-side telemetry event.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClientEvent {
    /// Log level: "error", "warn", "info", "debug"
    pub level: String,
    /// Human-readable message
    pub message: String,
    /// Dot-separated source path (e.g. "orchid_detail.on_edit_save")
    pub source: String,
    /// JSON-encoded context data (key-value pairs for structured fields)
    pub context: String,
}

/// Proxy endpoint: accepts a client-side telemetry event and logs it via tracing
/// so it flows into Axiom alongside server traces.
#[server]
pub async fn log_client_event(
    /// The structured telemetry event from the client.
    event: ClientEvent,
) -> Result<(), ServerFnError> {
    match event.level.as_str() {
        "error" => tracing::error!(
            source = %event.source,
            context = %event.context,
            origin = "client",
            "{}",
            event.message
        ),
        "warn" => tracing::warn!(
            source = %event.source,
            context = %event.context,
            origin = "client",
            "{}",
            event.message
        ),
        "info" => tracing::info!(
            source = %event.source,
            context = %event.context,
            origin = "client",
            "{}",
            event.message
        ),
        _ => tracing::debug!(
            source = %event.source,
            context = %event.context,
            origin = "client",
            "{}",
            event.message
        ),
    }
    Ok(())
}

/// Fire-and-forget helper to send a telemetry event from the client.
/// Spawns an async task so the caller is never blocked.
#[cfg(feature = "hydrate")]
pub fn emit(level: &str, source: &str, message: &str, context: &[(&str, &str)]) {
    let event = ClientEvent {
        level: level.to_string(),
        message: message.to_string(),
        source: source.to_string(),
        context: serde_json::to_string(
            &context.iter().map(|(k, v)| (*k, *v)).collect::<std::collections::HashMap<_, _>>(),
        )
        .unwrap_or_default(),
    };
    leptos::task::spawn_local(async move {
        let _ = log_client_event(event).await;
    });
}

/// Convenience: emit an info-level telemetry event from the client.
#[cfg(feature = "hydrate")]
pub fn emit_info(source: &str, message: &str, context: &[(&str, &str)]) {
    emit("info", source, message, context);
}

/// Convenience: emit a warn-level telemetry event from the client.
#[cfg(feature = "hydrate")]
pub fn emit_warn(source: &str, message: &str, context: &[(&str, &str)]) {
    emit("warn", source, message, context);
}

/// Convenience: emit an error-level telemetry event from the client.
#[cfg(feature = "hydrate")]
pub fn emit_error(source: &str, message: &str, context: &[(&str, &str)]) {
    emit("error", source, message, context);
}
