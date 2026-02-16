use orchid_tracker::orchid::{Orchid, LightRequirement, Placement};
use serde_json;

#[test]
fn test_orchid_json_serialization() {
    let orchid = Orchid::new(
        123,
        "Test Name".to_string(),
        "Test Species".to_string(),
        10,
        LightRequirement::Low,
        "Test Note".to_string(),
        Placement::High,
        "500".to_string(),
        "15-25C".to_string(),
        Some("Endangered".to_string()),
    );

    // Serialize
    let json = serde_json::to_string(&orchid).expect("Failed to serialize");
    
    // Deserialize
    let deserialized: Orchid = serde_json::from_str(&json).expect("Failed to deserialize");
    
    assert_eq!(orchid.id, deserialized.id);
    assert_eq!(orchid.name, deserialized.name);
    assert_eq!(orchid.light_requirement, deserialized.light_requirement);
    assert_eq!(orchid.placement, deserialized.placement);
    assert_eq!(orchid.conservation_status, deserialized.conservation_status);
}

#[test]
fn test_placement_serialization() {
    let p = Placement::OutdoorRack;
    let json = serde_json::to_string(&p).expect("Failed to serialize");
    // Enum serialization usually defaults to variant name string or object depending on config.
    // Placement derives Serialize/Deserialize so it should be unit variant string if no content.
    // Let's verify format if needed, but roundtrip is most important.
    
    let d: Placement = serde_json::from_str(&json).expect("Failed to deserialize");
    assert_eq!(p, d);
}
