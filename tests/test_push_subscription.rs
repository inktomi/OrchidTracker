//! Integration tests for push subscription persistence using in-memory SurrealDB.
//!
//! Validates the same query patterns used in `src/server_fns/alerts.rs`.
//! Key regression: `$auth` is a SurrealDB system variable — bind params must not collide.
#![cfg(feature = "ssr")]

use surrealdb::engine::local::Mem;
use surrealdb::Surreal;
use surrealdb::types::SurrealValue;

type Db = Surreal<surrealdb::engine::local::Db>;

async fn setup_db() -> Db {
    let db = Surreal::new::<Mem>(()).await.expect("in-memory DB");
    db.use_ns("test").use_db("test").await.expect("use ns/db");

    db.query(
        "DEFINE TABLE IF NOT EXISTS user SCHEMAFULL;
         DEFINE FIELD IF NOT EXISTS username ON user TYPE string;",
    )
    .await
    .expect("define user table");

    db.query(
        "DEFINE TABLE IF NOT EXISTS push_subscription SCHEMAFULL;
         DEFINE FIELD IF NOT EXISTS owner ON push_subscription TYPE record<user>;
         DEFINE FIELD IF NOT EXISTS endpoint ON push_subscription TYPE string;
         DEFINE FIELD IF NOT EXISTS p256dh ON push_subscription TYPE string;
         DEFINE FIELD IF NOT EXISTS auth ON push_subscription TYPE string;
         DEFINE FIELD IF NOT EXISTS created_at ON push_subscription TYPE datetime DEFAULT time::now();
         DEFINE INDEX IF NOT EXISTS idx_push_owner ON push_subscription FIELDS owner;",
    )
    .await
    .expect("define push_subscription table");

    db.query("CREATE user:testuser SET username = 'alice'")
        .await
        .expect("create test user");

    db
}

fn test_owner() -> surrealdb::types::RecordId {
    surrealdb::types::RecordId::parse_simple("user:testuser").expect("parse owner")
}

/// Same query pattern as subscribe_push server function.
/// Uses $sub_auth (NOT $auth which is a SurrealDB system variable).
async fn query_subscribe_push(db: &Db, owner: surrealdb::types::RecordId, endpoint: &str, p256dh: &str, auth: &str) -> Result<(), String> {
    // Delete existing
    let mut del_resp = db
        .query("DELETE push_subscription WHERE owner = $owner AND endpoint = $endpoint")
        .bind(("owner", owner.clone()))
        .bind(("endpoint", endpoint.to_string()))
        .await
        .map_err(|e| format!("delete transport: {e}"))?;

    let del_errors = del_resp.take_errors();
    if !del_errors.is_empty() {
        let msg = del_errors.into_values().map(|e| e.to_string()).collect::<Vec<_>>().join("; ");
        return Err(format!("delete statement: {msg}"));
    }

    // Create with $sub_auth to avoid $auth system variable collision
    let mut create_resp = db
        .query("CREATE push_subscription SET owner = $owner, endpoint = $endpoint, p256dh = $p256dh, auth = $sub_auth")
        .bind(("owner", owner))
        .bind(("endpoint", endpoint.to_string()))
        .bind(("p256dh", p256dh.to_string()))
        .bind(("sub_auth", auth.to_string()))
        .await
        .map_err(|e| format!("create transport: {e}"))?;

    let create_errors = create_resp.take_errors();
    if !create_errors.is_empty() {
        let msg = create_errors.into_values().map(|e| e.to_string()).collect::<Vec<_>>().join("; ");
        return Err(format!("create statement: {msg}"));
    }

    Ok(())
}

/// Query count of subscriptions for an owner.
async fn query_push_count(db: &Db, owner: surrealdb::types::RecordId) -> i64 {
    #[derive(serde::Deserialize, SurrealValue)]
    #[surreal(crate = "surrealdb::types")]
    struct CountRow { count: i64 }

    let mut resp = db
        .query("SELECT count() AS count FROM push_subscription WHERE owner = $owner GROUP ALL")
        .bind(("owner", owner))
        .await
        .expect("count query");

    let _ = resp.take_errors();
    let row: Option<CountRow> = resp.take(0).unwrap_or(None);
    row.map(|r| r.count).unwrap_or(0)
}

/// Read back the auth field to verify it stored correctly.
async fn query_push_auth(db: &Db, owner: surrealdb::types::RecordId) -> Option<String> {
    #[derive(serde::Deserialize, SurrealValue)]
    #[surreal(crate = "surrealdb::types")]
    struct AuthRow { auth: String }

    let mut resp = db
        .query("SELECT auth FROM push_subscription WHERE owner = $owner LIMIT 1")
        .bind(("owner", owner))
        .await
        .expect("auth query");

    let _ = resp.take_errors();
    let row: Option<AuthRow> = resp.take(0).unwrap_or(None);
    row.map(|r| r.auth)
}

#[tokio::test]
async fn test_subscribe_push_stores_subscription() {
    let db = setup_db().await;
    let owner = test_owner();

    query_subscribe_push(&db, owner.clone(), "https://push.example.com/abc", "p256dh_key", "auth_secret")
        .await
        .expect("subscribe should succeed");

    assert_eq!(query_push_count(&db, owner).await, 1);
}

#[tokio::test]
async fn test_subscribe_push_auth_value_stored_correctly() {
    let db = setup_db().await;
    let owner = test_owner();

    // This is the core regression test: $auth is a SurrealDB system variable.
    // Using $sub_auth instead ensures the bound value is stored, not the system variable.
    let auth_value = "my-push-auth-secret-123";
    query_subscribe_push(&db, owner.clone(), "https://push.example.com/abc", "p256dh_key", auth_value)
        .await
        .expect("subscribe should succeed");

    let stored = query_push_auth(&db, owner).await;
    assert_eq!(
        stored.as_deref(),
        Some(auth_value),
        "auth field must contain the bound value, not the SurrealDB $auth system variable"
    );
}

#[tokio::test]
async fn test_subscribe_push_upserts_same_endpoint() {
    let db = setup_db().await;
    let owner = test_owner();

    query_subscribe_push(&db, owner.clone(), "https://push.example.com/abc", "key1", "auth1")
        .await
        .expect("first subscribe");

    query_subscribe_push(&db, owner.clone(), "https://push.example.com/abc", "key2", "auth2")
        .await
        .expect("second subscribe (same endpoint)");

    assert_eq!(query_push_count(&db, owner.clone()).await, 1, "same endpoint should upsert, not duplicate");
    assert_eq!(query_push_auth(&db, owner).await.as_deref(), Some("auth2"));
}

#[tokio::test]
async fn test_subscribe_push_different_endpoints() {
    let db = setup_db().await;
    let owner = test_owner();

    query_subscribe_push(&db, owner.clone(), "https://push.example.com/abc", "key1", "auth1")
        .await
        .expect("first endpoint");

    query_subscribe_push(&db, owner.clone(), "https://push.example.com/xyz", "key2", "auth2")
        .await
        .expect("second endpoint");

    assert_eq!(query_push_count(&db, owner).await, 2, "different endpoints should create separate rows");
}

/// Regression test: using $auth (system variable) instead of $sub_auth fails.
/// SurrealDB rejects binding "auth" because $auth is a protected variable.
#[tokio::test]
async fn test_dollar_auth_is_protected_variable() {
    let db = setup_db().await;
    let owner = test_owner();

    // Try using $auth directly (the OLD buggy query) — SurrealDB rejects this
    let result = db
        .query("CREATE push_subscription SET owner = $owner, endpoint = $endpoint, p256dh = $p256dh, auth = $auth")
        .bind(("owner", owner))
        .bind(("endpoint", "https://push.example.com/test".to_string()))
        .bind(("p256dh", "test_key".to_string()))
        .bind(("auth", "test_auth_value".to_string()))
        .await;

    assert!(
        result.is_err(),
        "Binding 'auth' should fail — $auth is a protected SurrealDB variable"
    );

    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("protected variable"),
        "Error should mention protected variable, got: {err_msg}"
    );
}
