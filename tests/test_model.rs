use orchid_tracker::orchid::{Orchid, LightRequirement, FitCategory};

#[test]
fn test_orchid_json_serialization() {
    let orchid = Orchid {
        id: "orchid:test123".into(),
        name: "Test Name".into(),
        species: "Test Species".into(),
        water_frequency_days: 10,
        light_requirement: LightRequirement::Low,
        notes: "Test Note".into(),
        placement: "High Light Area".to_string(),
        light_lux: "500".into(),
        temperature_range: "15-25C".into(),
        conservation_status: Some("Endangered".into()),
        native_region: None,
        native_latitude: None,
        native_longitude: None,
        history: Vec::new(),
    };

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
fn test_fit_category_serialization() {
    let good = FitCategory::GoodFit;
    let json = serde_json::to_string(&good).expect("Failed to serialize");
    assert_eq!(json, "\"Good Fit\"");
    let d: FitCategory = serde_json::from_str(&json).expect("Failed to deserialize");
    assert_eq!(good, d);
}
