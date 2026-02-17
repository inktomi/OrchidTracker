use crate::config::AppConfig;
use crate::error::AppError;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::types::SurrealValue;
use surrealdb::Surreal;
use std::sync::LazyLock;

static DB: LazyLock<Surreal<Client>> = LazyLock::new(Surreal::init);

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

    tracing::info!("Namespace/DB selected, running test query...");

    // Verify the connection works with a simple query
    DB.query("RETURN 1")
        .await
        .map_err(|e| AppError::Database(format!("Test query failed: {}", e)))?;

    tracing::info!("Test query succeeded — DB connection verified");
    Ok(())
}

pub fn db() -> &'static Surreal<Client> {
    &DB
}

/// Run all pending migrations from the migrations/ directory
pub async fn run_migrations() -> Result<(), AppError> {
    let db = db();

    tracing::info!("Starting migration check...");

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
        tracing::info!("Checking migration: {}", name);

        // Check if already applied
        let applied: Option<MigrationRecord> = db
            .query("SELECT * FROM migration WHERE name = $name LIMIT 1")
            .bind(("name", name.clone()))
            .await
            .map_err(|e| AppError::Database(format!("Migration check failed: {}", e)))?
            .take(0)
            .map_err(|e| AppError::Database(format!("Migration check failed: {}", e)))?;

        if applied.is_some() {
            tracing::info!("Migration {} already applied, skipping", name);
            continue;
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
    }

    Ok(())
}

#[derive(serde::Deserialize, surrealdb::types::SurrealValue)]
#[surreal(crate = "surrealdb::types")]
struct MigrationRecord {
    #[allow(dead_code)]
    name: String,
}
