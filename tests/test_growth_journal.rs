use orchid_tracker::orchid::{Orchid, LogEntry, LightRequirement};
use orchid_tracker::server_fns::orchids::AddLogEntryResponse;
use orchid_tracker::components::event_types::{get_event_info, EVENT_TYPES, ALLOWED_EVENT_TYPE_KEYS};
use chrono::Utc;

// ── AddLogEntryResponse tests ────────────────────────────────────────

#[test]
fn test_add_log_entry_response_serde_roundtrip() {
    let response = AddLogEntryResponse {
        entry: LogEntry {
            id: "log_entry:123".into(),
            timestamp: Utc::now(),
            note: "First flower!".into(),
            image_filename: Some("user1/photo.jpg".into()),
            event_type: Some("Flowering".into()),
        },
        is_first_bloom: true,
    };

    let json = serde_json::to_string(&response).unwrap();
    let deserialized: AddLogEntryResponse = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.entry.id, "log_entry:123");
    assert_eq!(deserialized.entry.note, "First flower!");
    assert_eq!(deserialized.entry.event_type, Some("Flowering".into()));
    assert_eq!(deserialized.entry.image_filename, Some("user1/photo.jpg".into()));
    assert!(deserialized.is_first_bloom);
}

#[test]
fn test_add_log_entry_response_not_first_bloom() {
    let response = AddLogEntryResponse {
        entry: LogEntry {
            id: "log_entry:456".into(),
            timestamp: Utc::now(),
            note: "Watered".into(),
            image_filename: None,
            event_type: Some("Watered".into()),
        },
        is_first_bloom: false,
    };

    let json = serde_json::to_string(&response).unwrap();
    let deserialized: AddLogEntryResponse = serde_json::from_str(&json).unwrap();

    assert!(!deserialized.is_first_bloom);
    assert_eq!(deserialized.entry.event_type, Some("Watered".into()));
}

// ── LogEntry backward compatibility ──────────────────────────────────

#[test]
fn test_log_entry_missing_optional_fields_deserializes() {
    // Simulates entries created before event_type/image_filename were added
    let json = r#"{
        "id": "log_entry:legacy",
        "timestamp": "2024-06-15T10:30:00Z",
        "note": "Legacy watering note"
    }"#;

    let entry: LogEntry = serde_json::from_str(json).unwrap();
    assert_eq!(entry.id, "log_entry:legacy");
    assert_eq!(entry.note, "Legacy watering note");
    assert_eq!(entry.event_type, None);
    assert_eq!(entry.image_filename, None);
}

#[test]
fn test_log_entry_with_all_fields() {
    let json = r#"{
        "id": "log_entry:full",
        "timestamp": "2025-03-01T14:00:00Z",
        "note": "Repotted into larger pot",
        "image_filename": "user42/abc123.jpg",
        "event_type": "Repotted"
    }"#;

    let entry: LogEntry = serde_json::from_str(json).unwrap();
    assert_eq!(entry.event_type, Some("Repotted".into()));
    assert_eq!(entry.image_filename, Some("user42/abc123.jpg".into()));
}

// ── Orchid first_bloom_at tests ──────────────────────────────────────

#[test]
fn test_orchid_first_bloom_at_roundtrip() {
    let now = Utc::now();
    let orchid = Orchid {
        id: "orchid:bloom".into(),
        name: "Blooming Orchid".into(),
        species: "Dendrobium".into(),
        water_frequency_days: 5,
        light_requirement: LightRequirement::High,
        notes: String::new(),
        placement: "South Window".into(),
        light_lux: String::new(),
        temperature_range: String::new(),
        conservation_status: None,
        native_region: None,
        native_latitude: None,
        native_longitude: None,
        last_watered_at: None,
        temp_min: None,
        temp_max: None,
        humidity_min: None,
        humidity_max: None,
        first_bloom_at: Some(now),
    };

    let json = serde_json::to_string(&orchid).unwrap();
    let deserialized: Orchid = serde_json::from_str(&json).unwrap();

    assert!(deserialized.first_bloom_at.is_some());
}

#[test]
fn test_orchid_without_first_bloom_at_backward_compat() {
    let json = r#"{
        "id": "orchid:old",
        "name": "Old Orchid",
        "species": "Phal",
        "water_frequency_days": 7,
        "light_requirement": "Medium",
        "notes": "",
        "placement": "Zone A",
        "light_lux": "",
        "temperature_range": ""
    }"#;

    let orchid: Orchid = serde_json::from_str(json).unwrap();
    assert_eq!(orchid.first_bloom_at, None);
    assert_eq!(orchid.name, "Old Orchid");
}

// ── Event type metadata consistency ──────────────────────────────────

#[test]
fn test_event_types_count() {
    assert_eq!(EVENT_TYPES.len(), 8, "Expected exactly 8 event types");
}

#[test]
fn test_allowed_keys_match_event_types() {
    // Every allowed key has metadata
    for key in ALLOWED_EVENT_TYPE_KEYS {
        assert!(
            get_event_info(key).is_some(),
            "Allowed key '{}' has no EventTypeInfo",
            key
        );
    }

    // Every metadata entry is an allowed key
    for et in EVENT_TYPES {
        assert!(
            ALLOWED_EVENT_TYPE_KEYS.contains(&et.key),
            "EventTypeInfo key '{}' is not in ALLOWED_EVENT_TYPE_KEYS",
            et.key
        );
    }
}

#[test]
fn test_milestone_event_types_have_info() {
    // Milestone types used in growth thread for special rendering
    let milestones = ["Flowering", "Purchased", "Repotted"];
    for key in milestones {
        let info = get_event_info(key);
        assert!(info.is_some(), "Milestone type '{}' missing from event types", key);
    }
}

#[test]
fn test_watered_event_type_exists() {
    // Watered is used by mark_watered server fn
    let info = get_event_info("Watered");
    assert!(info.is_some());
    assert_eq!(info.unwrap().label, "Watered");
}
