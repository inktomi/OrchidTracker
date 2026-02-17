#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use orchid_tracker::app::App;
    use tower_http::services::ServeDir;
    use tower_sessions::{MemoryStore, SessionManagerLayer, Expiry};
    use time::Duration;

    // Load .env file
    let _ = dotenvy::dotenv();

    // Init tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Init config
    orchid_tracker::config::init_config();
    let cfg = orchid_tracker::config::config();

    // Init SurrealDB
    orchid_tracker::db::init_db(cfg).await.expect("Failed to connect to SurrealDB");
    orchid_tracker::db::run_migrations().await.expect("Failed to run migrations");

    tracing::info!("SurrealDB connected and migrations applied");

    // Session layer
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_expiry(Expiry::OnInactivity(Duration::days(7)))
        .with_same_site(tower_sessions::cookie::SameSite::Strict)
        .with_http_only(true);

    // Leptos config
    let leptos_options = LeptosOptions::builder()
        .output_name("orchid-tracker")
        .site_root("target/site")
        .site_pkg_dir("pkg")
        .site_addr(std::net::SocketAddr::from(([0, 0, 0, 0], 3000)))
        .reload_port(3001)
        .build();
    let routes = generate_route_list(App);

    // Image serving
    let image_service = ServeDir::new(&cfg.image_storage_path);

    // Build router
    let app = Router::new()
        .route("/api/images/upload", axum::routing::post(orchid_tracker::server_fns::images::handlers::upload_image))
        .nest_service("/images", image_service)
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || {
                use orchid_tracker::app::shell;
                shell(leptos_options.clone())
            }
        })
        .fallback(leptos_axum::file_and_error_handler(shell_fn))
        .layer(session_layer)
        .with_state(leptos_options);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("Listening on http://0.0.0.0:3000");
    axum::serve(listener, app.into_make_service()).await.unwrap();
}

#[cfg(feature = "ssr")]
fn shell_fn(options: leptos::prelude::LeptosOptions) -> impl leptos::IntoView {
    orchid_tracker::app::shell(options)
}

#[cfg(not(feature = "ssr"))]
pub fn main() {}
