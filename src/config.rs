use std::sync::OnceLock;

static CONFIG: OnceLock<AppConfig> = OnceLock::new();

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub surreal_url: String,
    pub surreal_ns: String,
    pub surreal_db: String,
    pub surreal_user: String,
    pub surreal_pass: String,
    pub image_storage_path: String,
    pub gemini_api_key: String,
    pub gemini_model: String,
    pub session_secret: String,
    pub site_addr: String,
    pub reload_port: u32,
}

impl AppConfig {
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
            session_secret: std::env::var("SESSION_SECRET").unwrap_or_else(|_| "change-me-in-production-must-be-at-least-64-chars-long-for-security-purposes-ok".into()),
            site_addr: std::env::var("LEPTOS_SITE_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".into()),
            reload_port: std::env::var("LEPTOS_RELOAD_PORT").unwrap_or_else(|_| "3001".into()).parse::<u32>().unwrap_or(3001),
        }
    }
}

pub fn init_config() {
    CONFIG.set(AppConfig::from_env()).expect("Config already initialized");
}

pub fn config() -> &'static AppConfig {
    CONFIG.get().expect("Config not initialized — call init_config() first")
}
