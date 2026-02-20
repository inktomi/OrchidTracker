//! Integration tests for user preference persistence using in-memory SurrealDB.
#![cfg(feature = "ssr")]

use surrealdb::engine::local::Mem;
use surrealdb::Surreal;

/// Set up an in-memory SurrealDB with user + user_preference tables.
async fn setup_db() -> Surreal<surrealdb::engine::local::Db> {
    let db = Surreal::new::<Mem>(()).await.expect("in-memory DB");
    db.use_ns("test").use_db("test").await.expect("use ns/db");

    // Minimal schema: user table + preferences table
    db.query(
        "DEFINE TABLE IF NOT EXISTS user SCHEMAFULL;
         DEFINE FIELD IF NOT EXISTS username ON user TYPE string;",
    )
    .await
    .expect("define user table");

    db.query(
        "DEFINE TABLE IF NOT EXISTS user_preference SCHEMAFULL;
         DEFINE FIELD IF NOT EXISTS owner ON user_preference TYPE record<user>;
         DEFINE FIELD IF NOT EXISTS temp_unit ON user_preference TYPE string DEFAULT 'C';
         DEFINE INDEX IF NOT EXISTS idx_user_preference_owner ON user_preference FIELDS owner UNIQUE;",
    )
    .await
    .expect("define preference table");

    // Create a test user
    db.query("CREATE user:testuser SET username = 'alice'")
        .await
        .expect("create test user");

    db
}

/// Same query logic as `get_temp_unit()` server function.
async fn query_get_temp_unit(
    db: &Surreal<surrealdb::engine::local::Db>,
    owner: surrealdb::types::RecordId,
) -> String {
    use surrealdb::types::SurrealValue;

    #[derive(serde::Deserialize, SurrealValue)]
    #[surreal(crate = "surrealdb::types")]
    struct PrefRow {
        temp_unit: String,
    }

    let mut resp = db
        .query("SELECT temp_unit FROM user_preference WHERE owner = $owner LIMIT 1")
        .bind(("owner", owner))
        .await
        .expect("get_temp_unit query");

    let _ = resp.take_errors();
    let row: Option<PrefRow> = resp.take(0).unwrap_or(None);
    row.map(|r| r.temp_unit).unwrap_or_else(|| "C".to_string())
}

/// Same query logic as `save_temp_unit()` server function.
async fn query_save_temp_unit(
    db: &Surreal<surrealdb::engine::local::Db>,
    owner: surrealdb::types::RecordId,
    unit: &str,
) {
    let unit = if unit == "F" { "F" } else { "C" };

    let _ = db
        .query("DELETE user_preference WHERE owner = $owner")
        .bind(("owner", owner.clone()))
        .await;

    db.query("CREATE user_preference SET owner = $owner, temp_unit = $unit")
        .bind(("owner", owner))
        .bind(("unit", unit.to_string()))
        .await
        .expect("save_temp_unit query");
}

fn test_owner() -> surrealdb::types::RecordId {
    surrealdb::types::RecordId::parse_simple("user:testuser")
        .expect("parse test owner")
}

#[tokio::test]
async fn test_get_temp_unit_returns_default_when_no_preference() {
    let db = setup_db().await;
    let unit = query_get_temp_unit(&db, test_owner()).await;
    assert_eq!(unit, "C", "Should default to Celsius when no preference saved");
}

#[tokio::test]
async fn test_save_and_get_temp_unit_fahrenheit() {
    let db = setup_db().await;
    let owner = test_owner();

    query_save_temp_unit(&db, owner.clone(), "F").await;
    let unit = query_get_temp_unit(&db, owner).await;
    assert_eq!(unit, "F", "Should return Fahrenheit after saving");
}

#[tokio::test]
async fn test_save_and_get_temp_unit_celsius() {
    let db = setup_db().await;
    let owner = test_owner();

    // Save F first, then switch back to C
    query_save_temp_unit(&db, owner.clone(), "F").await;
    query_save_temp_unit(&db, owner.clone(), "C").await;
    let unit = query_get_temp_unit(&db, owner).await;
    assert_eq!(unit, "C", "Should return Celsius after switching back");
}

#[tokio::test]
async fn test_save_temp_unit_validates_input() {
    let db = setup_db().await;
    let owner = test_owner();

    // Invalid unit should fall back to "C"
    query_save_temp_unit(&db, owner.clone(), "K").await;
    let unit = query_get_temp_unit(&db, owner).await;
    assert_eq!(unit, "C", "Invalid unit should be saved as Celsius");
}

#[tokio::test]
async fn test_preferences_are_per_user() {
    let db = setup_db().await;

    // Create a second user
    db.query("CREATE user:otheruser SET username = 'bob'")
        .await
        .expect("create second user");

    let alice = surrealdb::types::RecordId::parse_simple("user:testuser").unwrap();
    let bob = surrealdb::types::RecordId::parse_simple("user:otheruser").unwrap();

    // Alice sets F, Bob keeps default
    query_save_temp_unit(&db, alice.clone(), "F").await;

    let alice_unit = query_get_temp_unit(&db, alice).await;
    let bob_unit = query_get_temp_unit(&db, bob).await;

    assert_eq!(alice_unit, "F", "Alice should have Fahrenheit");
    assert_eq!(bob_unit, "C", "Bob should still have default Celsius");
}

#[tokio::test]
async fn test_save_temp_unit_upserts_not_duplicates() {
    use surrealdb::types::SurrealValue;

    let db = setup_db().await;
    let owner = test_owner();

    // Save multiple times
    query_save_temp_unit(&db, owner.clone(), "F").await;
    query_save_temp_unit(&db, owner.clone(), "C").await;
    query_save_temp_unit(&db, owner.clone(), "F").await;

    // Should have exactly one preference row
    #[derive(serde::Deserialize, SurrealValue)]
    #[surreal(crate = "surrealdb::types")]
    struct CountRow {
        count: i64,
    }

    let mut resp = db
        .query("SELECT count() AS count FROM user_preference WHERE owner = $owner GROUP ALL")
        .bind(("owner", owner))
        .await
        .expect("count query");

    let _ = resp.take_errors();
    let row: Option<CountRow> = resp.take(0).unwrap_or(None);
    let count = row.map(|r| r.count).unwrap_or(0);
    assert_eq!(count, 1, "Should have exactly one preference row after multiple saves");
}
