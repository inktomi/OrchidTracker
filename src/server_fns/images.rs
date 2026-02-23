// Image upload is handled via a custom Axum handler (not a Leptos server function)
// because multipart form data requires direct access to the Axum extractors.
// See main.rs for the route registration.

#[cfg(feature = "ssr")]
pub mod handlers {
    use axum::{
        extract::{DefaultBodyLimit, Multipart},
        http::StatusCode,
        response::Json,
    };
    use serde_json::json;
    use std::path::PathBuf;

    /// Returns an Axum Router layer that overrides the default body limit for
    /// the upload route, allowing uploads up to 15MB (matching the tower-http
    /// RequestBodyLimitLayer). Without this, Axum's DefaultBodyLimit of 2MB
    /// rejects photos from modern phone cameras before the handler runs.
    pub fn upload_router() -> axum::Router<leptos::prelude::LeptosOptions> {
        axum::Router::new()
            .route("/api/images/upload", axum::routing::post(upload_image))
            .layer(DefaultBodyLimit::max(15 * 1024 * 1024))
    }

    pub async fn upload_image(
        session: tower_sessions::Session,
        mut multipart: Multipart,
    ) -> Result<Json<serde_json::Value>, StatusCode> {
        use crate::config::config;

        // Require authentication
        let user_id: String = session.get("user_id").await
            .map_err(|e| {
                tracing::error!("Session read error: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?
            .ok_or(StatusCode::UNAUTHORIZED)?;

        while let Some(field) = multipart.next_field().await.map_err(|e| {
            tracing::error!("Multipart field read error: {}", e);
            StatusCode::BAD_REQUEST
        })? {
            let name = field.name().unwrap_or("").to_string();
            if name != "image" {
                continue;
            }

            let data = field.bytes().await.map_err(|e| {
                tracing::error!("Field bytes read error: {}", e);
                StatusCode::BAD_REQUEST
            })?;

            tracing::info!("Image upload: {} bytes from user {}", data.len(), user_id);

            // Validate size (10MB max)
            if data.len() > 10 * 1024 * 1024 {
                tracing::warn!("Image too large: {} bytes", data.len());
                return Err(StatusCode::PAYLOAD_TOO_LARGE);
            }

            // Validate magic bytes for JPEG/PNG/WebP
            let is_jpeg = data.starts_with(&[0xFF, 0xD8, 0xFF]);
            let is_png = data.starts_with(&[0x89, 0x50, 0x4E, 0x47]);
            let is_webp = data.len() > 12
                && data.starts_with(b"RIFF")
                && &data[8..12] == b"WEBP";
            if !is_jpeg && !is_png && !is_webp {
                tracing::warn!(
                    "Unsupported image format (magic bytes: {:02X?})",
                    &data[..data.len().min(4)]
                );
                return Err(StatusCode::UNSUPPORTED_MEDIA_TYPE);
            }

            let ext = if is_jpeg { "jpg" } else if is_png { "png" } else { "webp" };
            let filename = format!("{}.{}", uuid::Uuid::new_v4(), ext);

            // Store in per-user subdirectory
            let storage_path = PathBuf::from(&config().image_storage_path).join(&user_id);
            tokio::fs::create_dir_all(&storage_path).await.map_err(|e| {
                tracing::error!("Failed to create image directory {:?}: {}", storage_path, e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            let file_path = storage_path.join(&filename);
            tokio::fs::write(&file_path, &data).await.map_err(|e| {
                tracing::error!("Failed to write image {:?}: {}", file_path, e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            // Return path relative to storage root (user_id/filename)
            let relative_path = format!("{}/{}", user_id, filename);
            return Ok(Json(json!({ "filename": relative_path })));
        }

        tracing::warn!("No 'image' field found in multipart upload");
        Err(StatusCode::BAD_REQUEST)
    }
}
