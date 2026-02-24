use crate::orchid::{LightRequirement, Orchid};

/// Minimal Orchid with defaults — suitable for most component tests.
pub fn test_orchid() -> Orchid {
    Orchid {
        id: "test:1".into(),
        name: "Test Orchid".into(),
        species: "Phalaenopsis".into(),
        water_frequency_days: 7,
        light_requirement: LightRequirement::Medium,
        notes: String::new(),
        placement: "Medium Light Area".to_string(),
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
        first_bloom_at: None,
        last_fertilized_at: None,
        fertilize_frequency_days: None,
        fertilizer_type: None,
        last_repotted_at: None,
        pot_medium: None,
        pot_size: None,
        rest_start_month: None,
        rest_end_month: None,
        bloom_start_month: None,
        bloom_end_month: None,
        rest_water_multiplier: None,
        rest_fertilizer_multiplier: None,
        active_water_multiplier: None,
        active_fertilizer_multiplier: None,
    }
}

/// Orchid with care tracking fields populated (fertilizer, pot info).
pub fn test_orchid_with_care() -> Orchid {
    Orchid {
        fertilizer_type: Some("MSU".to_string()),
        fertilize_frequency_days: Some(14),
        pot_medium: Some("Bark".to_string()),
        pot_size: Some("4 inch".to_string()),
        ..test_orchid()
    }
}

/// Orchid with seasonal care data (rest/bloom months + multipliers).
pub fn test_orchid_seasonal() -> Orchid {
    Orchid {
        rest_start_month: Some(11),
        rest_end_month: Some(2),
        bloom_start_month: Some(3),
        bloom_end_month: Some(5),
        rest_water_multiplier: Some(0.5),
        rest_fertilizer_multiplier: Some(0.0),
        active_water_multiplier: Some(1.0),
        active_fertilizer_multiplier: Some(1.0),
        ..test_orchid_with_care()
    }
}
