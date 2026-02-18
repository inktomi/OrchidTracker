#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use axum::http::HeaderValue;
    use clap::Parser;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use orchid_tracker::app::App;
    use orchid_tracker::cli::{Cli, Command};
    use tower_http::services::ServeDir;
    use tower_http::limit::RequestBodyLimitLayer;
    use tower_http::set_header::SetResponseHeaderLayer;
    use tower_sessions::{SessionManagerLayer, Expiry};
    use orchid_tracker::session_store::SurrealSessionStore;
    use tower_governor::GovernorLayer;
    use tower_governor::governor::GovernorConfigBuilder;
    use time::Duration;

    // Load .env file
    let _ = dotenvy::dotenv();

    // Init tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Parse CLI args
    let cli = Cli::parse();

    // Init config
    orchid_tracker::config::init_config();
    let cfg = orchid_tracker::config::config();

    // Init SurrealDB (also runs migrations)
    orchid_tracker::db::init_db(cfg).await.expect("Failed to connect to SurrealDB");

    tracing::info!("SurrealDB connected and migrations applied");

    // Handle CLI subcommands
    if let Some(Command::ResetPassword { username, password }) = cli.command {
        match orchid_tracker::cli::run_reset_password(&username, &password).await {
            Ok(()) => std::process::exit(0),
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }

    // Session layer (SurrealDB-backed for persistence across restarts)
    let session_store = SurrealSessionStore;
    let session_layer = SessionManagerLayer::new(session_store.clone())
        .with_expiry(Expiry::OnInactivity(Duration::days(7)))
        .with_same_site(tower_sessions::cookie::SameSite::Strict)
        .with_http_only(true)
        .with_secure(true);

    // Rate limiting: 100 requests/sec per IP
    let governor_conf = GovernorConfigBuilder::default()
        .per_second(100)
        .burst_size(100)
        .finish()
        .expect("Failed to build rate limiter config");
    let governor_limiter = governor_conf.limiter().clone();
    let governor_layer = GovernorLayer::new(governor_conf);

    // Leptos config
    let site_addr: std::net::SocketAddr = cfg.site_addr.parse()
        .expect("Invalid SITE_ADDR format (expected e.g. 0.0.0.0:3000)");
    let leptos_options = LeptosOptions::builder()
        .output_name("orchid-tracker")
        .site_root("target/site")
        .site_pkg_dir("pkg")
        .site_addr(site_addr)
        .reload_port(cfg.reload_port)
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
        // Security headers
        .layer(SetResponseHeaderLayer::overriding(
            axum::http::header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            axum::http::header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            axum::http::header::REFERRER_POLICY,
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            axum::http::header::X_XSS_PROTECTION,
            HeaderValue::from_static("0"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            axum::http::HeaderName::from_static("permissions-policy"),
            HeaderValue::from_static("camera=(), microphone=(), geolocation=()"),
        ))
        // Request body size limit (15MB)
        .layer(RequestBodyLimitLayer::new(15 * 1024 * 1024))
        // Rate limiting
        .layer(governor_layer)
        .with_state(leptos_options);

    // Spawn background task to periodically clean up rate limiter + expired sessions
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
            governor_limiter.retain_recent();
            session_store.cleanup_expired().await;
        }
    });

    // Spawn climate data polling task (every 30 minutes)
    tokio::spawn(async move {
        // Initial delay to let the server fully start
        tokio::time::sleep(std::time::Duration::from_secs(30)).await;
        loop {
            orchid_tracker::climate::poller::poll_all_zones().await;
            tokio::time::sleep(std::time::Duration::from_secs(30 * 60)).await;
        }
    });

    let listener = tokio::net::TcpListener::bind(&cfg.site_addr).await.unwrap();
    tracing::info!("Listening on http://{}", cfg.site_addr);
    axum::serve(listener, app.into_make_service_with_connect_info::<std::net::SocketAddr>()).await.unwrap();
}

#[cfg(feature = "ssr")]
fn shell_fn(options: leptos::prelude::LeptosOptions) -> impl leptos::IntoView {
    orchid_tracker::app::shell(options)
}

#[cfg(not(feature = "ssr"))]
pub fn main() {}
