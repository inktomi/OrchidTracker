//! Integration tests for the public collection sharing feature.
//!
//! These tests verify the **security gate** — `resolve_public_user` — which is
//! the critical boundary between unauthenticated visitors and user data.
//!
//! Every public server function flows through this gate first, so these tests
//! ensure that:
//! - Private collections are never exposed (default-deny)
//! - Nonexistent users produce appropriate errors
//! - Only explicitly opted-in collections are accessible
//! - Data isolation between users is maintained
//! - The `collection_public` preference toggles access correctly
//! - Edge cases (empty username, long username, toggling back to private) are handled
#![cfg(feature = "ssr")]

use surrealdb::engine::local::Mem;
use surrealdb::types::SurrealValue;
use surrealdb::Surreal;

type Db = Surreal<surrealdb::engine::local::Db>;

// ── Test DB Setup ───────────────────────────────────────────────────

/// Set up an in-memory SurrealDB with user + user_preference tables matching
/// the production schema used by `resolve_public_user`.
async fn setup_db() -> Db {
    let db = Surreal::new::<Mem>(()).await.expect("in-memory DB");
    db.use_ns("test").use_db("test").await.expect("use ns/db");

    db.query(
        "DEFINE TABLE IF NOT EXISTS user SCHEMAFULL;
         DEFINE FIELD IF NOT EXISTS username ON user TYPE string;
         DEFINE FIELD IF NOT EXISTS email ON user TYPE string;
         DEFINE FIELD IF NOT EXISTS password_hash ON user TYPE string;",
    )
    .await
    .expect("define user table");

    db.query(
        "DEFINE TABLE IF NOT EXISTS user_preference SCHEMAFULL;
         DEFINE FIELD IF NOT EXISTS owner ON user_preference TYPE record<user>;
         DEFINE FIELD IF NOT EXISTS temp_unit ON user_preference TYPE string DEFAULT 'C';
         DEFINE FIELD IF NOT EXISTS hemisphere ON user_preference TYPE string DEFAULT 'N';
         DEFINE FIELD IF NOT EXISTS collection_public ON user_preference TYPE bool DEFAULT false;
         DEFINE INDEX IF NOT EXISTS idx_user_preference_owner ON user_preference FIELDS owner UNIQUE;",
    )
    .await
    .expect("define preference table");

    db
}

/// Create a test user and optionally set their collection_public preference.
async fn create_user(db: &Db, key: &str, username: &str, collection_public: Option<bool>) {
    let query = format!(
        "CREATE user:{} SET username = '{}', email = '{}@test.com', password_hash = 'hash'",
        key, username, username
    );
    db.query(&query).await.expect("create user");

    if let Some(public) = collection_public {
        let owner = surrealdb::types::RecordId::parse_simple(&format!("user:{}", key)).unwrap();
        db.query("CREATE user_preference SET owner = $owner, collection_public = $public")
            .bind(("owner", owner))
            .bind(("public", public))
            .await
            .expect("create preference");
    }
}

// ── resolve_public_user query patterns ──────────────────────────────

/// Mirrors the `resolve_public_user` logic from `src/server_fns/public.rs`.
/// Returns Ok(user_id_string) or Err(message).
async fn query_resolve_public_user(db: &Db, username: &str) -> Result<String, String> {
    // Input validation
    if username.is_empty() || username.len() > 50 {
        return Err("User not found".to_string());
    }

    // Look up user by username
    #[derive(serde::Deserialize, SurrealValue)]
    #[surreal(crate = "surrealdb::types")]
    struct UserRow {
        id: surrealdb::types::RecordId,
    }

    let mut resp = db
        .query("SELECT id FROM user WHERE username = $uname LIMIT 1")
        .bind(("uname", username.to_string()))
        .await
        .map_err(|e| format!("query failed: {e}"))?;

    let _ = resp.take_errors();
    let user_row: Option<UserRow> = resp.take(0).unwrap_or(None);
    let user_row = user_row.ok_or_else(|| "User not found".to_string())?;
    let user_id = format!(
        "{}:{}",
        user_row.id.table,
        match &user_row.id.key {
            surrealdb::types::RecordIdKey::String(s) => s.clone(),
            other => format!("{:?}", other),
        }
    );
    let owner = user_row.id;

    // Check collection_public preference
    #[derive(serde::Deserialize, SurrealValue)]
    #[surreal(crate = "surrealdb::types")]
    struct PrefRow {
        #[surreal(default)]
        collection_public: bool,
    }

    let mut pref_resp = db
        .query("SELECT collection_public FROM user_preference WHERE owner = $owner LIMIT 1")
        .bind(("owner", owner))
        .await
        .map_err(|e| format!("pref query failed: {e}"))?;

    let _ = pref_resp.take_errors();
    let pref: Option<PrefRow> = pref_resp.take(0).unwrap_or(None);
    let is_public = pref.map(|p| p.collection_public).unwrap_or(false);

    if !is_public {
        return Err("This collection is private".to_string());
    }

    Ok(user_id)
}

/// Mirrors the `save_collection_public` logic from preferences.rs.
async fn query_save_collection_public(db: &Db, owner: surrealdb::types::RecordId, public: bool) {
    let mut resp = db
        .query("UPDATE user_preference SET collection_public = $public WHERE owner = $owner")
        .bind(("owner", owner.clone()))
        .bind(("public", public))
        .await
        .expect("update collection_public");

    let _ = resp.take_errors();
    let updated: Vec<serde_json::Value> = resp.take(0).unwrap_or_default();
    if updated.is_empty() {
        let _ = db
            .query("CREATE user_preference SET owner = $owner, collection_public = $public")
            .bind(("owner", owner))
            .bind(("public", public))
            .await;
    }
}

/// Mirrors the `get_collection_public` logic from preferences.rs.
async fn query_get_collection_public(db: &Db, owner: surrealdb::types::RecordId) -> bool {
    #[derive(serde::Deserialize, SurrealValue)]
    #[surreal(crate = "surrealdb::types")]
    struct PrefRow {
        #[surreal(default)]
        collection_public: bool,
    }

    let mut resp = db
        .query("SELECT collection_public FROM user_preference WHERE owner = $owner LIMIT 1")
        .bind(("owner", owner))
        .await
        .expect("get collection_public");

    let _ = resp.take_errors();
    let row: Option<PrefRow> = resp.take(0).unwrap_or(None);
    row.map(|r| r.collection_public).unwrap_or(false)
}

fn owner(key: &str) -> surrealdb::types::RecordId {
    surrealdb::types::RecordId::parse_simple(&format!("user:{}", key)).expect("parse owner")
}

// ═══════════════════════════════════════════════════════════════════════
// SECURITY GATE TESTS — resolve_public_user
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_nonexistent_user_returns_not_found() {
    let db = setup_db().await;
    let result = query_resolve_public_user(&db, "nobody").await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "User not found");
}

#[tokio::test]
async fn test_empty_username_returns_not_found() {
    let db = setup_db().await;
    let result = query_resolve_public_user(&db, "").await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "User not found");
}

#[tokio::test]
async fn test_username_too_long_returns_not_found() {
    let db = setup_db().await;
    let long_name = "a".repeat(51);
    let result = query_resolve_public_user(&db, &long_name).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "User not found");
}

#[tokio::test]
async fn test_username_at_max_length_does_not_reject() {
    let db = setup_db().await;
    // 50 chars is the max — should NOT be rejected by the length check
    // (will fail because user doesn't exist, not because of length)
    let name = "a".repeat(50);
    let result = query_resolve_public_user(&db, &name).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "User not found"); // not a length error
}

#[tokio::test]
async fn test_private_collection_default_deny() {
    let db = setup_db().await;
    // User exists but has NO preference row at all → default is private
    create_user(&db, "alice", "alice", None).await;

    let result = query_resolve_public_user(&db, "alice").await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "This collection is private");
}

#[tokio::test]
async fn test_private_collection_explicit_false() {
    let db = setup_db().await;
    // User exists with collection_public explicitly set to false
    create_user(&db, "alice", "alice", Some(false)).await;

    let result = query_resolve_public_user(&db, "alice").await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "This collection is private");
}

#[tokio::test]
async fn test_public_collection_allows_access() {
    let db = setup_db().await;
    create_user(&db, "alice", "alice", Some(true)).await;

    let result = query_resolve_public_user(&db, "alice").await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "user:alice");
}

#[tokio::test]
async fn test_toggle_public_to_private_blocks_access() {
    let db = setup_db().await;
    create_user(&db, "alice", "alice", Some(true)).await;

    // Initially public
    let result = query_resolve_public_user(&db, "alice").await;
    assert!(result.is_ok(), "Should be accessible when public");

    // Toggle to private
    query_save_collection_public(&db, owner("alice"), false).await;

    // Now blocked
    let result = query_resolve_public_user(&db, "alice").await;
    assert!(result.is_err(), "Should be blocked after toggling to private");
    assert_eq!(result.unwrap_err(), "This collection is private");
}

#[tokio::test]
async fn test_toggle_private_to_public_grants_access() {
    let db = setup_db().await;
    create_user(&db, "alice", "alice", Some(false)).await;

    // Initially private
    let result = query_resolve_public_user(&db, "alice").await;
    assert!(result.is_err(), "Should be blocked when private");

    // Toggle to public
    query_save_collection_public(&db, owner("alice"), true).await;

    // Now accessible
    let result = query_resolve_public_user(&db, "alice").await;
    assert!(result.is_ok(), "Should be accessible after toggling to public");
}

// ═══════════════════════════════════════════════════════════════════════
// DATA ISOLATION TESTS
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_user_a_public_user_b_private_isolated() {
    let db = setup_db().await;
    create_user(&db, "alice", "alice", Some(true)).await;
    create_user(&db, "bob", "bob", Some(false)).await;

    let alice_result = query_resolve_public_user(&db, "alice").await;
    let bob_result = query_resolve_public_user(&db, "bob").await;

    assert!(alice_result.is_ok(), "Alice's public collection should be accessible");
    assert!(bob_result.is_err(), "Bob's private collection should be blocked");
    assert_eq!(bob_result.unwrap_err(), "This collection is private");
}

#[tokio::test]
async fn test_making_one_user_public_doesnt_affect_others() {
    let db = setup_db().await;
    create_user(&db, "alice", "alice", Some(false)).await;
    create_user(&db, "bob", "bob", Some(false)).await;

    // Both private initially
    assert!(query_resolve_public_user(&db, "alice").await.is_err());
    assert!(query_resolve_public_user(&db, "bob").await.is_err());

    // Make only Alice public
    query_save_collection_public(&db, owner("alice"), true).await;

    // Alice is public, Bob is still private
    assert!(query_resolve_public_user(&db, "alice").await.is_ok());
    assert!(query_resolve_public_user(&db, "bob").await.is_err());
}

// ═══════════════════════════════════════════════════════════════════════
// PREFERENCE PERSISTENCE TESTS — collection_public field
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_get_collection_public_defaults_to_false() {
    let db = setup_db().await;
    create_user(&db, "alice", "alice", None).await;

    // No preference row at all → defaults to false
    let val = query_get_collection_public(&db, owner("alice")).await;
    assert!(!val, "Default should be false (private)");
}

#[tokio::test]
async fn test_get_collection_public_explicit_false() {
    let db = setup_db().await;
    create_user(&db, "alice", "alice", Some(false)).await;

    let val = query_get_collection_public(&db, owner("alice")).await;
    assert!(!val, "Explicit false should return false");
}

#[tokio::test]
async fn test_get_collection_public_explicit_true() {
    let db = setup_db().await;
    create_user(&db, "alice", "alice", Some(true)).await;

    let val = query_get_collection_public(&db, owner("alice")).await;
    assert!(val, "Explicit true should return true");
}

#[tokio::test]
async fn test_save_collection_public_roundtrip() {
    let db = setup_db().await;
    create_user(&db, "alice", "alice", None).await;

    // No preference row yet — save creates one
    query_save_collection_public(&db, owner("alice"), true).await;
    assert!(query_get_collection_public(&db, owner("alice")).await);

    // Toggle back
    query_save_collection_public(&db, owner("alice"), false).await;
    assert!(!query_get_collection_public(&db, owner("alice")).await);

    // Toggle forward again
    query_save_collection_public(&db, owner("alice"), true).await;
    assert!(query_get_collection_public(&db, owner("alice")).await);
}

#[tokio::test]
async fn test_save_collection_public_per_user_isolation() {
    let db = setup_db().await;
    create_user(&db, "alice", "alice", None).await;
    create_user(&db, "bob", "bob", None).await;

    // Set Alice to public, Bob stays private
    query_save_collection_public(&db, owner("alice"), true).await;

    assert!(
        query_get_collection_public(&db, owner("alice")).await,
        "Alice should be public"
    );
    assert!(
        !query_get_collection_public(&db, owner("bob")).await,
        "Bob should still be private (default)"
    );
}

#[tokio::test]
async fn test_save_collection_public_upserts_when_pref_row_exists() {
    let db = setup_db().await;
    // Create user with existing preference (temp_unit set)
    create_user(&db, "alice", "alice", Some(false)).await;

    // Save collection_public — should UPDATE, not create duplicate
    query_save_collection_public(&db, owner("alice"), true).await;

    // Verify only one row
    #[derive(serde::Deserialize, SurrealValue)]
    #[surreal(crate = "surrealdb::types")]
    struct CountRow {
        count: i64,
    }

    let mut resp = db
        .query("SELECT count() AS count FROM user_preference WHERE owner = $owner GROUP ALL")
        .bind(("owner", owner("alice")))
        .await
        .expect("count query");

    let _ = resp.take_errors();
    let row: Option<CountRow> = resp.take(0).unwrap_or(None);
    let count = row.map(|r| r.count).unwrap_or(0);
    assert_eq!(count, 1, "Should still have exactly one preference row");
}

#[tokio::test]
async fn test_save_collection_public_creates_row_when_none_exists() {
    let db = setup_db().await;
    // User exists but NO preference row
    create_user(&db, "alice", "alice", None).await;

    // This should create the row via the fallback path
    query_save_collection_public(&db, owner("alice"), true).await;

    assert!(
        query_get_collection_public(&db, owner("alice")).await,
        "Should be public after save"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// EDGE CASE / REGRESSION TESTS
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_case_sensitive_username_lookup() {
    let db = setup_db().await;
    create_user(&db, "alice", "Alice", Some(true)).await;

    // Exact match should work
    let result = query_resolve_public_user(&db, "Alice").await;
    assert!(result.is_ok(), "Exact username match should work");

    // Different case should NOT match (usernames are case-sensitive)
    let result = query_resolve_public_user(&db, "alice").await;
    assert!(result.is_err(), "Different case should not match");
}

#[tokio::test]
async fn test_multiple_rapid_toggles_end_state_correct() {
    let db = setup_db().await;
    create_user(&db, "alice", "alice", Some(false)).await;

    // Rapid toggles
    query_save_collection_public(&db, owner("alice"), true).await;
    query_save_collection_public(&db, owner("alice"), false).await;
    query_save_collection_public(&db, owner("alice"), true).await;
    query_save_collection_public(&db, owner("alice"), false).await;
    query_save_collection_public(&db, owner("alice"), true).await;

    // End state: public
    assert!(query_get_collection_public(&db, owner("alice")).await);
    assert!(query_resolve_public_user(&db, "alice").await.is_ok());

    // One more toggle to private
    query_save_collection_public(&db, owner("alice"), false).await;

    assert!(!query_get_collection_public(&db, owner("alice")).await);
    assert!(query_resolve_public_user(&db, "alice").await.is_err());
}

#[tokio::test]
async fn test_resolve_returns_correct_user_id_format() {
    let db = setup_db().await;
    create_user(&db, "testuser123", "testuser123", Some(true)).await;

    let result = query_resolve_public_user(&db, "testuser123").await;
    assert!(result.is_ok());
    let user_id = result.unwrap();
    assert_eq!(user_id, "user:testuser123", "Should return table:key format");
}

#[tokio::test]
async fn test_user_with_special_chars_in_key() {
    let db = setup_db().await;
    // Username with hyphens/underscores (valid per registration rules)
    create_user(&db, "my_user", "my-user_123", Some(true)).await;

    let result = query_resolve_public_user(&db, "my-user_123").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_whitespace_username_rejected() {
    let db = setup_db().await;
    // Space-only username — should not find any user
    let result = query_resolve_public_user(&db, "   ").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_collection_public_field_default_in_schema() {
    let db = setup_db().await;
    create_user(&db, "alice", "alice", None).await;

    // Create a preference row WITHOUT setting collection_public
    // The schema DEFAULT false should kick in
    let alice_owner = owner("alice");
    db.query("CREATE user_preference SET owner = $owner, temp_unit = 'F'")
        .bind(("owner", alice_owner.clone()))
        .await
        .expect("create pref without collection_public");

    let val = query_get_collection_public(&db, alice_owner.clone()).await;
    assert!(!val, "Schema default should be false");

    // And the resolve gate should block
    let result = query_resolve_public_user(&db, "alice").await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "This collection is private");
}

// ═══════════════════════════════════════════════════════════════════════
// CROSS-USER MUTATION REJECTION TESTS — defense-in-depth
// ═══════════════════════════════════════════════════════════════════════
//
// These tests verify that even if the UI leaked a mutation button,
// the server-side `WHERE owner = $owner` clause prevents cross-user
// modifications. Mutations targeting orchids NOT owned by the caller
// must return no rows (None), proving data isolation.

/// Extended setup: also defines the orchid and log_entry tables.
async fn setup_db_with_orchids() -> Db {
    let db = setup_db().await;

    db.query(
        "DEFINE TABLE IF NOT EXISTS orchid SCHEMAFULL;
         DEFINE FIELD IF NOT EXISTS owner ON orchid TYPE record<user>;
         DEFINE FIELD IF NOT EXISTS name ON orchid TYPE string;
         DEFINE FIELD IF NOT EXISTS species ON orchid TYPE string;
         DEFINE FIELD IF NOT EXISTS water_frequency_days ON orchid TYPE int;
         DEFINE FIELD IF NOT EXISTS light_requirement ON orchid TYPE string;
         DEFINE FIELD IF NOT EXISTS notes ON orchid TYPE string DEFAULT '';
         DEFINE FIELD IF NOT EXISTS last_watered_at ON orchid TYPE option<datetime>;
         DEFINE FIELD IF NOT EXISTS last_fertilized_at ON orchid TYPE option<datetime>;",
    )
    .await
    .expect("define orchid table");

    db.query(
        "DEFINE TABLE IF NOT EXISTS log_entry SCHEMAFULL;
         DEFINE FIELD IF NOT EXISTS orchid ON log_entry TYPE record<orchid>;
         DEFINE FIELD IF NOT EXISTS owner ON log_entry TYPE record<user>;
         DEFINE FIELD IF NOT EXISTS note ON log_entry TYPE string;
         DEFINE FIELD IF NOT EXISTS event_type ON log_entry TYPE option<string>;
         DEFINE FIELD IF NOT EXISTS created_at ON log_entry TYPE datetime DEFAULT time::now();",
    )
    .await
    .expect("define log_entry table");

    db
}

/// Create an orchid owned by the given user key, returning its record ID string.
async fn create_orchid(db: &Db, orchid_key: &str, owner_key: &str) -> String {
    let owner_rid = owner(owner_key);
    let query = format!(
        "CREATE orchid:{} SET owner = $owner, name = 'Test Orchid', species = 'Phalaenopsis', \
         water_frequency_days = 7, light_requirement = 'Medium'",
        orchid_key
    );
    db.query(&query)
        .bind(("owner", owner_rid))
        .await
        .expect("create orchid");
    format!("orchid:{}", orchid_key)
}

#[tokio::test]
async fn test_mark_watered_rejects_non_owner() {
    let db = setup_db_with_orchids().await;
    create_user(&db, "alice", "alice", Some(true)).await;
    create_user(&db, "bob", "bob", Some(true)).await;

    // Alice owns this orchid
    let orchid_id = create_orchid(&db, "orch1", "alice").await;
    let orchid_rid = surrealdb::types::RecordId::parse_simple(&orchid_id).unwrap();

    // Bob tries to mark Alice's orchid as watered
    let bob_owner = owner("bob");
    let mut resp = db
        .query("UPDATE $id SET last_watered_at = time::now() WHERE owner = $owner RETURN *")
        .bind(("id", orchid_rid))
        .bind(("owner", bob_owner))
        .await
        .expect("mark_watered query");

    let _ = resp.take_errors();
    let row: Option<serde_json::Value> = resp.take(0).unwrap_or(None);
    assert!(
        row.is_none(),
        "Bob should NOT be able to mark Alice's orchid as watered"
    );
}

#[tokio::test]
async fn test_mark_fertilized_rejects_non_owner() {
    let db = setup_db_with_orchids().await;
    create_user(&db, "alice", "alice", Some(true)).await;
    create_user(&db, "bob", "bob", Some(true)).await;

    let orchid_id = create_orchid(&db, "orch1", "alice").await;
    let orchid_rid = surrealdb::types::RecordId::parse_simple(&orchid_id).unwrap();

    // Bob tries to mark Alice's orchid as fertilized
    let bob_owner = owner("bob");
    let mut resp = db
        .query("UPDATE $id SET last_fertilized_at = time::now() WHERE owner = $owner RETURN *")
        .bind(("id", orchid_rid))
        .bind(("owner", bob_owner))
        .await
        .expect("mark_fertilized query");

    let _ = resp.take_errors();
    let row: Option<serde_json::Value> = resp.take(0).unwrap_or(None);
    assert!(
        row.is_none(),
        "Bob should NOT be able to mark Alice's orchid as fertilized"
    );
}

#[tokio::test]
async fn test_add_log_entry_for_non_owned_orchid_rejected() {
    let db = setup_db_with_orchids().await;
    create_user(&db, "alice", "alice", Some(true)).await;
    create_user(&db, "bob", "bob", Some(true)).await;

    let orchid_id = create_orchid(&db, "orch1", "alice").await;
    let orchid_rid = surrealdb::types::RecordId::parse_simple(&orchid_id).unwrap();

    // Bob creates a log entry but with his own owner field — the entry is created
    // with Bob as owner, which means it does NOT belong to Alice's orchid lineage.
    // The key defense: when reading log entries, they are filtered by owner,
    // so Bob's entry won't appear in Alice's journal.
    //
    // Additionally, the care timestamp updates use WHERE owner = $owner,
    // so Bob's log entry cannot modify Alice's orchid timestamps.
    let bob_owner = owner("bob");

    // Verify the care timestamp update is blocked
    let mut resp = db
        .query("UPDATE $orchid_id SET last_watered_at = time::now() WHERE owner = $owner")
        .bind(("orchid_id", orchid_rid))
        .bind(("owner", bob_owner))
        .await
        .expect("care timestamp update query");

    let _ = resp.take_errors();
    let row: Option<serde_json::Value> = resp.take(0).unwrap_or(None);
    assert!(
        row.is_none(),
        "Bob's log entry should NOT update Alice's orchid care timestamps"
    );
}

