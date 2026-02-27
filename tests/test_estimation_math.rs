use orchid_tracker::estimation::*;
use orchid_tracker::orchid::{LightRequirement, PotMedium, PotSize, PotType};

// ── Algorithmic Base Days Tests ──

#[test]
fn test_calculate_algorithmic_base_days_standard_phalaenopsis() {
    // A standard Phalaenopsis in a Medium solid plastic pot with bark in a normal living room.
    let days = calculate_algorithmic_base_days(
        &PotSize::Medium, // 500ml
        &PotMedium::Bark, // 25% WHC -> 125ml water
        &PotType::Solid,  // 1.0x porosity
        &LightRequirement::Low, // 0.8x consumption
        VPD_BASELINE, // 1.19 kPa
    );
    // Evap: 18 * (1.19/1.19) * 1.0 * 0.8 = 14.4ml/day
    // Days: 125 / 14.4 = 8.68 -> ~9 days
    assert_eq!(days, 9);
}

#[test]
fn test_calculate_algorithmic_base_days_dry_environment() {
    // Same plant but in a very dry house (2.0 kPa)
    let days = calculate_algorithmic_base_days(
        &PotSize::Medium,
        &PotMedium::Bark,
        &PotType::Solid,
        &LightRequirement::Low,
        2.0,
    );
    // Should need watering much sooner
    assert!(days < 9);
    assert_eq!(days, 5); // 125 / (18 * (2.0/1.19) * 1.0 * 0.8) = ~5 days
}

#[test]
fn test_calculate_algorithmic_base_days_sphagnum_retains_longer() {
    // Change medium to Sphagnum Moss (75% WHC -> 375ml water)
    let days = calculate_algorithmic_base_days(
        &PotSize::Medium,
        &PotMedium::SphagnumMoss,
        &PotType::Solid,
        &LightRequirement::Low,
        VPD_BASELINE,
    );
    // Should hold water much longer than Bark
    assert!(days > 9);
    assert_eq!(days, 26); // 375 / 14.4 = 26.04 -> 26 days
}

#[test]
fn test_calculate_algorithmic_base_days_terra_cotta_dries_faster() {
    // Sphagnum but in a Terra Cotta pot (1.9x evaporation)
    let days = calculate_algorithmic_base_days(
        &PotSize::Medium,
        &PotMedium::SphagnumMoss,
        &PotType::Clay, // Terra cotta
        &LightRequirement::Low,
        VPD_BASELINE,
    );
    // The clay wicks the moisture away
    assert!(days < 26);
    assert_eq!(days, 14); // 375 / (14.4 * 1.9) = 13.7 -> 14 days
}

#[test]
fn test_calculate_algorithmic_base_days_mounted() {
    // Mounted (no pot, 5.0x porosity, 5% WHC - wait WHC logic gives Bark 25%, but porosity is 5x)
    let days = calculate_algorithmic_base_days(
        &PotSize::Medium, // Doesn't really matter for mounted, but say 500ml equivalent
        &PotMedium::Bark, // Bark slab
        &PotType::Mounted,
        &LightRequirement::High, // High light (Cattleya)
        VPD_BASELINE,
    );
    // Should definitely be 1 day (clamped)
    assert_eq!(days, 1);
}

// ── Suitability Recommendation Tests ──

#[test]
fn test_recommend_potting_setup_very_dry_home() {
    // Cloud forest orchid (0.5 kPa) in dry house (1.5 kPa)
    let rec = recommend_potting_setup(0.5, 1.5);
    assert_eq!(rec.suggested_medium, PotMedium::SphagnumMoss);
    assert_eq!(rec.suggested_pot_type, PotType::Solid);
    assert!(rec.scientific_reasoning.contains("significantly drier"));
}

#[test]
fn test_recommend_potting_setup_slightly_dry_home() {
    // Standard Cattleya (1.0 kPa) in average house (1.3 kPa)
    let rec = recommend_potting_setup(1.0, 1.3);
    assert_eq!(rec.suggested_medium, PotMedium::Bark);
    assert_eq!(rec.suggested_pot_type, PotType::Solid);
    assert!(rec.scientific_reasoning.contains("slightly drier"));
}

#[test]
fn test_recommend_potting_setup_perfect_match() {
    // Native (1.2 kPa) in matching house (1.2 kPa)
    let rec = recommend_potting_setup(1.2, 1.2);
    assert_eq!(rec.suggested_medium, PotMedium::Bark);
    assert_eq!(rec.suggested_pot_type, PotType::Slotted);
    assert!(rec.scientific_reasoning.contains("perfectly matches"));
}

#[test]
fn test_recommend_potting_setup_humid_home() {
    // Native (1.5 kPa) in very humid greenhouse (1.0 kPa)
    let rec = recommend_potting_setup(1.5, 1.0);
    assert_eq!(rec.suggested_medium, PotMedium::Bark);
    assert_eq!(rec.suggested_pot_type, PotType::Slotted);
    assert!(rec.scientific_reasoning.contains("more humid"));
}
