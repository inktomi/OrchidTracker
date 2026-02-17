// Image upload is handled via a custom Axum handler (not a Leptos server function)
// because multipart form data requires direct access to the Axum extractors.
// See main.rs for the route registration.

#[cfg(feature = "ssr")]
pub mod handlers {
    use axum::{
        extract::Multipart,
        http::StatusCode,
        response::Json,
    };
    use serde_json::json;
    use std::path::PathBuf;

    pub async fn upload_image(
        session: tower_sessions::Session,
        mut multipart: Multipart,
    ) -> Result<Json<serde_json::Value>, StatusCode> {
        use crate::config::config;

        // Require authentication
        let user_id: String = session.get("user_id").await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::UNAUTHORIZED)?;

        while let Some(field) = multipart.next_field().await.map_err(|_| StatusCode::BAD_REQUEST)? {
            let name = field.name().unwrap_or("").to_string();
            if name != "image" {
                continue;
            }

            let data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;

            // Validate size (10MB max)
            if data.len() > 10 * 1024 * 1024 {
                return Err(StatusCode::PAYLOAD_TOO_LARGE);
            }

            // Validate magic bytes for JPEG/PNG
            let is_jpeg = data.starts_with(&[0xFF, 0xD8, 0xFF]);
            let is_png = data.starts_with(&[0x89, 0x50, 0x4E, 0x47]);
            if !is_jpeg && !is_png {
                return Err(StatusCode::UNSUPPORTED_MEDIA_TYPE);
            }

            let ext = if is_jpeg { "jpg" } else { "png" };
            let filename = format!("{}.{}", uuid::Uuid::new_v4(), ext);

            // Store in per-user subdirectory
            let storage_path = PathBuf::from(&config().image_storage_path).join(&user_id);
            tokio::fs::create_dir_all(&storage_path).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            let file_path = storage_path.join(&filename);
            tokio::fs::write(&file_path, &data).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            // Return path relative to storage root (user_id/filename)
            let relative_path = format!("{}/{}", user_id, filename);
            return Ok(Json(json!({ "filename": relative_path })));
        }

        Err(StatusCode::BAD_REQUEST)
    }
}
