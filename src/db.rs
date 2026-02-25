use crate::config::AppConfig;
use crate::error::AppError;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::types::SurrealValue;
use surrealdb::Surreal;
use std::sync::LazyLock;

static DB: LazyLock<Surreal<Client>> = LazyLock::new(Surreal::init);

/// What is it? An asynchronous initialization routine for the application's SurrealDB connection.
/// Why does it exist? It manages the early-boot setup sequence, including resolving connection details, authenticating the root user, selecting the namespace/db, and automatically applying schema migrations before traffic is accepted.
/// How should it be used? Call this exactly once during the server startup phase (e.g., in `main.rs`) before binding the Axum router. If it fails, the application should panic and exit.
pub async fn init_db(config: &AppConfig) -> Result<(), AppError> {
    tracing::info!("Connecting to SurrealDB at {}", config.surreal_url);

    DB.connect::<Ws>(&config.surreal_url)
        .await
        .map_err(|e| AppError::Database(format!("Connection failed: {}", e)))?;

    tracing::info!("WebSocket connected, signing in...");

    DB.signin(Root {
        username: config.surreal_user.clone(),
        password: config.surreal_pass.clone(),
    })
    .await
    .map_err(|e| AppError::Database(format!("Auth failed: {}", e)))?;

    tracing::info!("Signed in, selecting namespace/db: {}/{}", config.surreal_ns, config.surreal_db);

    DB.use_ns(&config.surreal_ns)
        .use_db(&config.surreal_db)
        .await
        .map_err(|e| AppError::Database(format!("Namespace/DB selection failed: {}", e)))?;

    tracing::info!("DB connected and configured");

    // Run migrations
    run_migrations().await?;

    Ok(())
}

/// What is it? An accessor function for the global, lazily-initialized SurrealDB client.
/// Why does it exist? It provides a thread-safe, static reference to the database connection pool, eliminating the need to pass connection clones manually through every function layer or framework context.
/// How should it be used? Call `crate::db::db()` inside server functions or background tasks to obtain the client, then chain `.query()` or `.create()` methods to interact with SurrealDB.
pub fn db() -> &'static Surreal<Client> {
    &DB
}

/// What is it? An asynchronous utility that scans and executes `.surql` schema and data definition files.
/// Why does it exist? It ensures the SurrealDB schema (tables, fields, events, and indexes) stays synchronized with the codebase structure and prevents older schema versions from causing runtime errors.
/// How should it be used? It is called automatically by `init_db()` during startup. It reads files from the local `migrations/` directory, checks a `migration` tracking table to skip previously applied files, and runs new files sequentially.
pub async fn run_migrations() -> Result<(), AppError> {
    let db = db();

    // Read migration files
    let mut entries: Vec<_> = std::fs::read_dir("migrations")
        .map_err(|e| AppError::Database(format!("Can't read migrations dir: {}", e)))?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path().extension().is_some_and(|ext| ext == "surql")
        })
        .collect();

    entries.sort_by_key(|e| e.file_name());
    tracing::info!("Found {} migration files", entries.len());

    for entry in entries {
        let name = entry.file_name().to_string_lossy().to_string();

        // Check if already applied — use .check() to surface real SurrealDB errors
        // instead of the misleading "Connection uninitialised" from .take()
        let mut response = db
            .query("SELECT * FROM migration WHERE name = $name LIMIT 1")
            .bind(("name", name.clone()))
            .await
            .map_err(|e| AppError::Database(format!("Migration query failed: {}", e)))?;

        // Extract any query-level errors (SurrealDB returns errors per-statement)
        let errors = response.take_errors();
        if !errors.is_empty() {
            tracing::warn!("Migration check query returned errors: {:?}", errors);
            // On first run, the migration table doesn't exist yet — treat errors as "not applied"
            tracing::info!("Treating migration {} as not yet applied (table may not exist)", name);
        } else {
            let applied: Option<MigrationRecord> = response.take(0)
                .map_err(|e| AppError::Database(format!("Migration deserialize failed: {}", e)))?;

            if applied.is_some() {
                tracing::info!("Migration {} already applied, skipping", name);
                continue;
            }
        }

        // Read and execute
        let sql = std::fs::read_to_string(entry.path())
            .map_err(|e| AppError::Database(format!("Can't read migration {}: {}", name, e)))?;

        tracing::info!("Applying migration: {}", name);
        db.query(&sql)
            .await
            .map_err(|e| AppError::Database(format!("Migration {} failed: {}", name, e)))?;

        // Record it
        db.query("CREATE migration SET name = $name")
            .bind(("name", name.clone()))
            .await
            .map_err(|e| AppError::Database(format!("Failed to record migration {}: {}", name, e)))?;

        tracing::info!("Migration {} applied successfully", name);
    }

    Ok(())
}

#[derive(serde::Deserialize, surrealdb::types::SurrealValue)]
#[surreal(crate = "surrealdb::types")]
struct MigrationRecord {
    #[allow(dead_code)]
    name: String,
}
