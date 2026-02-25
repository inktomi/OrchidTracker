use std::sync::OnceLock;

static CONFIG: OnceLock<AppConfig> = OnceLock::new();

/// Application configuration loaded from environment variables.
#[derive(Clone, Debug)]
pub struct AppConfig {
    /// URL of the SurrealDB server.
    pub surreal_url: String,
    /// The SurrealDB namespace.
    pub surreal_ns: String,
    /// The SurrealDB database name.
    pub surreal_db: String,
    /// Username for SurrealDB.
    pub surreal_user: String,
    /// Password for SurrealDB.
    pub surreal_pass: String,
    /// Directory path for storing uploaded images.
    pub image_storage_path: String,
    /// API key for Google Gemini.
    pub gemini_api_key: String,
    /// The Google Gemini model to use.
    pub gemini_model: String,
    /// API key for Anthropic Claude.
    pub claude_api_key: String,
    /// The Anthropic Claude model to use.
    pub claude_model: String,
    /// Secret key used for session encryption.
    pub session_secret: String,
    /// Address to bind the Leptos server to.
    pub site_addr: String,
    /// Port used for Leptos hot reloading.
    pub reload_port: u32,
    /// VAPID private key for web push notifications.
    pub vapid_private_key: String,
    /// VAPID public key for web push notifications.
    pub vapid_public_key: String,
    /// Contact information (email/URL) for VAPID.
    pub vapid_contact: String,
}

impl AppConfig {
    /// Reads configuration values from the environment.
    pub fn from_env() -> Self {
        Self {
            surreal_url: std::env::var("SURREAL_URL").unwrap_or_else(|_| "ws://127.0.0.1:8000".into()),
            surreal_ns: std::env::var("SURREAL_NS").unwrap_or_else(|_| "orchidtracker".into()),
            surreal_db: std::env::var("SURREAL_DB").unwrap_or_else(|_| "orchidtracker".into()),
            surreal_user: std::env::var("SURREAL_USER").unwrap_or_else(|_| "root".into()),
            surreal_pass: std::env::var("SURREAL_PASS").unwrap_or_else(|_| "root".into()),
            image_storage_path: std::env::var("IMAGE_STORAGE_PATH").unwrap_or_else(|_| "./data/images".into()),
            gemini_api_key: std::env::var("GEMINI_API_KEY").unwrap_or_default(),
            gemini_model: std::env::var("GEMINI_MODEL").unwrap_or_else(|_| "gemini-2.0-flash".into()),
            claude_api_key: std::env::var("CLAUDE_API_KEY").unwrap_or_default(),
            claude_model: std::env::var("CLAUDE_MODEL").unwrap_or_else(|_| "claude-sonnet-4-20250514".into()),
            session_secret: std::env::var("SESSION_SECRET").unwrap_or_else(|_| "change-me-in-production-must-be-at-least-64-chars-long-for-security-purposes-ok".into()),
            site_addr: std::env::var("LEPTOS_SITE_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".into()),
            reload_port: std::env::var("LEPTOS_RELOAD_PORT").unwrap_or_else(|_| "3001".into()).parse::<u32>().unwrap_or(3001),
            vapid_private_key: std::env::var("VAPID_PRIVATE_KEY").unwrap_or_default(),
            vapid_public_key: std::env::var("VAPID_PUBLIC_KEY").unwrap_or_default(),
            vapid_contact: std::env::var("VAPID_CONTACT").unwrap_or_else(|_| "mailto:admin@example.com".into()),
        }
    }
}

/// Initializes the global configuration instance.
pub fn init_config() {
    CONFIG
        .set(AppConfig::from_env())
        .expect("Config already initialized");
}

/// Returns a reference to the global configuration.
pub fn config() -> &'static AppConfig {
    CONFIG
        .get()
        .expect("Config not initialized — call init_config() first")
}
