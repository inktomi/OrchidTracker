use crate::orchid::{LightRequirement, Orchid};
use crate::watering::{ClimateSnapshot, DataQuality};

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
        pot_type: None,
        rest_start_month: None,
        rest_end_month: None,
        bloom_start_month: None,
        bloom_end_month: None,
        rest_water_multiplier: None,
        rest_fertilizer_multiplier: None,
        active_water_multiplier: None,
        active_fertilizer_multiplier: None,
        par_ppfd: None,
    }
}

/// Orchid with care tracking fields populated (fertilizer, pot info).
pub fn test_orchid_with_care() -> Orchid {
    Orchid {
        fertilizer_type: Some("MSU".to_string()),
        fertilize_frequency_days: Some(14),
        pot_medium: Some(crate::orchid::PotMedium::Bark),
        pot_size: Some(crate::orchid::PotSize::Medium),
        pot_type: Some(crate::orchid::PotType::Solid),
        ..test_orchid()
    }
}

/// Orchid mounted on slab/cork — no pot medium or pot size.
pub fn test_orchid_mounted() -> Orchid {
    Orchid {
        pot_type: Some(crate::orchid::PotType::Mounted),
        pot_medium: None,
        pot_size: None,
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

/// Standard indoor climate snapshot at reference conditions (22°C, 55% RH).
pub fn test_climate_snapshot() -> ClimateSnapshot {
    ClimateSnapshot {
        zone_name: "Test Zone".into(),
        avg_temp_c: 22.0,
        avg_humidity_pct: 55.0,
        avg_vpd_kpa: crate::watering::REFERENCE_VPD_KPA,
        precipitation_48h_mm: None,
        newest_reading_at: chrono::Utc::now(),
        reading_count: 10,
        quality: DataQuality::Fresh,
        is_outdoor: false,
    }
}

/// Hot, dry indoor climate snapshot (30°C, 30% RH).
pub fn test_climate_snapshot_hot_dry() -> ClimateSnapshot {
    ClimateSnapshot {
        zone_name: "Hot Zone".into(),
        avg_temp_c: 30.0,
        avg_humidity_pct: 30.0,
        avg_vpd_kpa: 2.97,
        precipitation_48h_mm: None,
        newest_reading_at: chrono::Utc::now(),
        reading_count: 10,
        quality: DataQuality::Fresh,
        is_outdoor: false,
    }
}

/// Outdoor climate snapshot after heavy rain.
pub fn test_climate_snapshot_rainy() -> ClimateSnapshot {
    ClimateSnapshot {
        zone_name: "Patio".into(),
        avg_temp_c: 18.0,
        avg_humidity_pct: 85.0,
        avg_vpd_kpa: 0.31,
        precipitation_48h_mm: Some(25.0),
        newest_reading_at: chrono::Utc::now(),
        reading_count: 48,
        quality: DataQuality::Fresh,
        is_outdoor: true,
    }
}
