use orchid_tracker::components::scanner::AnalysisResult;
use orchid_tracker::orchid::{FitCategory, LightRequirement};

// ── Helper ──────────────────────────────────────────────────────────

fn full_analysis_result() -> AnalysisResult {
    AnalysisResult {
        species_name: "Phalaenopsis bellina".into(),
        fit_category: FitCategory::GoodFit,
        reason: "Warm-growing species that thrives in your conditions.".into(),
        already_owned: false,
        water_freq: 7,
        light_req: LightRequirement::Medium,
        temp_range: "18-30C".into(),
        placement_suggestion: "Living Room Window".into(),
        conservation_status: Some("CITES II".into()),
        native_region: Some("Borneo and Peninsular Malaysia".into()),
        native_latitude: Some(4.5),
        native_longitude: Some(114.7),
        temp_min: Some(18.0),
        temp_max: Some(30.0),
        humidity_min: Some(60.0),
        humidity_max: Some(85.0),
        rest_start_month: None,
        rest_end_month: None,
        bloom_start_month: Some(4),
        bloom_end_month: Some(8),
        rest_water_multiplier: None,
        rest_fertilizer_multiplier: None,
        active_water_multiplier: Some(1.0),
        active_fertilizer_multiplier: Some(1.0),
    }
}

// ── Serde Roundtrip ─────────────────────────────────────────────────

#[test]
fn test_analysis_result_serde_roundtrip() {
    let original = full_analysis_result();
    let json = serde_json::to_string(&original).unwrap();
    let deserialized: AnalysisResult = serde_json::from_str(&json).unwrap();
    assert_eq!(original, deserialized);
}

#[test]
fn test_analysis_result_all_fit_categories_roundtrip() {
    for category in [FitCategory::GoodFit, FitCategory::BadFit, FitCategory::CautionFit] {
        let mut result = full_analysis_result();
        result.fit_category = category.clone();
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: AnalysisResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.fit_category, category);
    }
}

#[test]
fn test_analysis_result_all_light_requirements_roundtrip() {
    for light in [LightRequirement::High, LightRequirement::Medium, LightRequirement::Low] {
        let mut result = full_analysis_result();
        result.light_req = light.clone();
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: AnalysisResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.light_req, light);
    }
}

// ── Backward Compatibility ──────────────────────────────────────────

#[test]
fn test_analysis_result_minimal_json_without_optional_fields() {
    // Simulates an AI response with only the core required fields
    let json = r#"{
        "species_name": "Dendrobium nobile",
        "fit_category": "Caution Fit",
        "reason": "Needs a cool rest period.",
        "already_owned": true,
        "water_freq": 5,
        "light_req": "High",
        "temp_range": "10-25C",
        "placement_suggestion": "Sunroom"
    }"#;
    let result: AnalysisResult = serde_json::from_str(json).unwrap();
    assert_eq!(result.species_name, "Dendrobium nobile");
    assert_eq!(result.fit_category, FitCategory::CautionFit);
    assert!(result.already_owned);
    assert_eq!(result.water_freq, 5);
    assert_eq!(result.light_req, LightRequirement::High);
    assert_eq!(result.conservation_status, None);
    assert_eq!(result.native_region, None);
    assert_eq!(result.native_latitude, None);
    assert_eq!(result.temp_min, None);
    assert_eq!(result.humidity_min, None);
    assert_eq!(result.rest_start_month, None);
    assert_eq!(result.bloom_start_month, None);
    assert_eq!(result.rest_water_multiplier, None);
    assert_eq!(result.active_water_multiplier, None);
}

#[test]
fn test_analysis_result_conservation_status_null() {
    let json = r#"{
        "species_name": "Phalaenopsis equestris",
        "fit_category": "Good Fit",
        "reason": "Easy grower.",
        "already_owned": false,
        "water_freq": 7,
        "light_req": "Medium",
        "temp_range": "18-30C",
        "placement_suggestion": "Kitchen",
        "conservation_status": null
    }"#;
    let result: AnalysisResult = serde_json::from_str(json).unwrap();
    assert_eq!(result.conservation_status, None);
}

// ── Realistic AI Response ───────────────────────────────────────────

#[test]
fn test_analysis_result_from_realistic_ai_response() {
    // Simulates a full JSON blob from the AI with all fields populated
    let json = r#"{
        "species_name": "Cattleya walkeriana",
        "fit_category": "Good Fit",
        "reason": "Brazilian species tolerant of intermediate conditions. Benefits from bright light and a slight winter dry rest.",
        "already_owned": false,
        "water_freq": 5,
        "light_req": "High",
        "temp_range": "14-30C",
        "temp_min": 14.0,
        "temp_max": 30.0,
        "humidity_min": 50.0,
        "humidity_max": 80.0,
        "placement_suggestion": "Sunroom South Window",
        "conservation_status": "CITES II",
        "native_region": "Cerrado biome of central Brazil",
        "native_latitude": -15.78,
        "native_longitude": -47.93,
        "rest_start_month": 6,
        "rest_end_month": 8,
        "bloom_start_month": 9,
        "bloom_end_month": 11,
        "rest_water_multiplier": 0.3,
        "rest_fertilizer_multiplier": 0.0,
        "active_water_multiplier": 1.0,
        "active_fertilizer_multiplier": 1.0
    }"#;
    let result: AnalysisResult = serde_json::from_str(json).unwrap();

    assert_eq!(result.species_name, "Cattleya walkeriana");
    assert_eq!(result.fit_category, FitCategory::GoodFit);
    assert!(!result.already_owned);
    assert_eq!(result.water_freq, 5);
    assert_eq!(result.light_req, LightRequirement::High);

    // Geo/native fields
    assert_eq!(result.native_region.as_deref(), Some("Cerrado biome of central Brazil"));
    assert!((result.native_latitude.unwrap() - (-15.78)).abs() < 0.01);
    assert!((result.native_longitude.unwrap() - (-47.93)).abs() < 0.01);

    // Climate ranges
    assert!((result.temp_min.unwrap() - 14.0).abs() < 0.01);
    assert!((result.temp_max.unwrap() - 30.0).abs() < 0.01);
    assert!((result.humidity_min.unwrap() - 50.0).abs() < 0.01);
    assert!((result.humidity_max.unwrap() - 80.0).abs() < 0.01);

    // Seasonal fields
    assert_eq!(result.rest_start_month, Some(6));
    assert_eq!(result.rest_end_month, Some(8));
    assert_eq!(result.bloom_start_month, Some(9));
    assert_eq!(result.bloom_end_month, Some(11));
    assert!((result.rest_water_multiplier.unwrap() - 0.3).abs() < 0.01);
    assert!((result.rest_fertilizer_multiplier.unwrap() - 0.0).abs() < 0.01);
    assert!((result.active_water_multiplier.unwrap() - 1.0).abs() < 0.01);
    assert!((result.active_fertilizer_multiplier.unwrap() - 1.0).abs() < 0.01);
}

// ── LightRequirement Aliases (AI can return various forms) ──────────

#[test]
fn test_analysis_result_light_req_lowercase_alias() {
    let json = r#"{
        "species_name": "Test",
        "fit_category": "Good Fit",
        "reason": "OK",
        "already_owned": false,
        "water_freq": 7,
        "light_req": "low",
        "temp_range": "18-25C",
        "placement_suggestion": "Shelf"
    }"#;
    let result: AnalysisResult = serde_json::from_str(json).unwrap();
    assert_eq!(result.light_req, LightRequirement::Low);
}

#[test]
fn test_analysis_result_light_req_display_alias() {
    let json = r#"{
        "species_name": "Test",
        "fit_category": "Bad Fit",
        "reason": "Too cold",
        "already_owned": false,
        "water_freq": 14,
        "light_req": "High Light",
        "temp_range": "20-35C",
        "placement_suggestion": "None"
    }"#;
    let result: AnalysisResult = serde_json::from_str(json).unwrap();
    assert_eq!(result.light_req, LightRequirement::High);
}

// ── Edge Cases ──────────────────────────────────────────────────────

#[test]
fn test_analysis_result_zero_water_freq() {
    let mut result = full_analysis_result();
    result.water_freq = 0;
    let json = serde_json::to_string(&result).unwrap();
    let deserialized: AnalysisResult = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.water_freq, 0);
}

#[test]
fn test_analysis_result_negative_coordinates() {
    let mut result = full_analysis_result();
    result.native_latitude = Some(-33.87);
    result.native_longitude = Some(-151.21);
    let json = serde_json::to_string(&result).unwrap();
    let deserialized: AnalysisResult = serde_json::from_str(&json).unwrap();
    assert!((deserialized.native_latitude.unwrap() - (-33.87)).abs() < 0.01);
    assert!((deserialized.native_longitude.unwrap() - (-151.21)).abs() < 0.01);
}

#[test]
fn test_analysis_result_multiplier_boundaries() {
    let mut result = full_analysis_result();
    result.rest_water_multiplier = Some(0.0);
    result.rest_fertilizer_multiplier = Some(0.0);
    result.active_water_multiplier = Some(1.0);
    result.active_fertilizer_multiplier = Some(1.0);
    let json = serde_json::to_string(&result).unwrap();
    let deserialized: AnalysisResult = serde_json::from_str(&json).unwrap();
    assert!((deserialized.rest_water_multiplier.unwrap() - 0.0).abs() < 0.01);
    assert!((deserialized.active_water_multiplier.unwrap() - 1.0).abs() < 0.01);
}

#[test]
fn test_analysis_result_all_seasonal_fields_none() {
    let mut result = full_analysis_result();
    result.rest_start_month = None;
    result.rest_end_month = None;
    result.bloom_start_month = None;
    result.bloom_end_month = None;
    result.rest_water_multiplier = None;
    result.rest_fertilizer_multiplier = None;
    result.active_water_multiplier = None;
    result.active_fertilizer_multiplier = None;
    let json = serde_json::to_string(&result).unwrap();
    let deserialized: AnalysisResult = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.rest_start_month, None);
    assert_eq!(deserialized.bloom_start_month, None);
    assert_eq!(deserialized.rest_water_multiplier, None);
    assert_eq!(deserialized.active_fertilizer_multiplier, None);
}

#[test]
fn test_analysis_result_empty_strings() {
    let mut result = full_analysis_result();
    result.species_name = String::new();
    result.reason = String::new();
    result.placement_suggestion = String::new();
    let json = serde_json::to_string(&result).unwrap();
    let deserialized: AnalysisResult = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.species_name, "");
    assert_eq!(deserialized.reason, "");
}
