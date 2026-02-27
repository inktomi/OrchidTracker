use serde::{Deserialize, Serialize};
use std::fmt;

/// Room type for indoor estimation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum RoomType {
    /// A kitchen area, typically warmer and slightly more humid.
    Kitchen,
    /// A bathroom, characterized by high humidity fluctuations.
    Bathroom,
    /// A living room, usually maintaining average indoor conditions.
    LivingRoom,
    /// A bedroom, typically stable with average conditions.
    Bedroom,
    /// A sunroom, experiencing higher temperatures and light levels.
    Sunroom,
    /// An office space, often with average, stable conditions.
    Office,
    /// A garage, typically cooler and subject to outdoor swings.
    Garage,
    /// Any other undefined room type.
    Other,
}

impl fmt::Display for RoomType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RoomType::Kitchen => write!(f, "Kitchen"),
            RoomType::Bathroom => write!(f, "Bathroom"),
            RoomType::LivingRoom => write!(f, "Living Room"),
            RoomType::Bedroom => write!(f, "Bedroom"),
            RoomType::Sunroom => write!(f, "Sunroom"),
            RoomType::Office => write!(f, "Office"),
            RoomType::Garage => write!(f, "Garage"),
            RoomType::Other => write!(f, "Other"),
        }
    }
}

/// Cardinal window direction.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum WindowDirection {
    /// North-facing window, providing low, indirect light (Northern Hemisphere).
    North,
    /// South-facing window, providing high, direct light (Northern Hemisphere).
    South,
    /// East-facing window, providing gentle morning sunlight.
    East,
    /// West-facing window, providing intense afternoon sunlight.
    West,
}

impl fmt::Display for WindowDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WindowDirection::North => write!(f, "North"),
            WindowDirection::South => write!(f, "South"),
            WindowDirection::East => write!(f, "East"),
            WindowDirection::West => write!(f, "West"),
        }
    }
}

/// Qualitative air description.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum AirDescription {
    /// Very dry air, typically below 30% humidity.
    VeryDry,
    /// Average indoor humidity, typically 30-50%.
    Average,
    /// Humid air, typically above 50% humidity.
    Humid,
}

impl fmt::Display for AirDescription {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AirDescription::VeryDry => write!(f, "Very Dry"),
            AirDescription::Average => write!(f, "Average"),
            AirDescription::Humid => write!(f, "Humid"),
        }
    }
}

/// Humidity booster methods.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum HumidityBooster {
    /// Use of an electric humidifier to actively increase moisture.
    Humidifier,
    /// Manually spraying water on or around the plants.
    RegularMisting,
    /// Placing pots on a tray filled with pebbles and water.
    PebbleTray,
    /// Clustering plants together to create a microclimate.
    GroupedPlants,
}

impl fmt::Display for HumidityBooster {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HumidityBooster::Humidifier => write!(f, "Humidifier"),
            HumidityBooster::RegularMisting => write!(f, "Regular Misting"),
            HumidityBooster::PebbleTray => write!(f, "Pebble Tray"),
            HumidityBooster::GroupedPlants => write!(f, "Grouped Plants"),
        }
    }
}

/// All wizard answers for indoor estimation.
#[derive(Clone, Debug)]
pub struct IndoorEstimationInput {
    /// The type of room where the orchid is located.
    pub room_type: RoomType,
    /// The baseline thermostat temperature setting in Celsius.
    pub thermostat_c: f64,
    /// Whether the room has a window providing natural light.
    pub has_window: bool,
    /// The primary direction the window faces, if applicable.
    pub window_direction: Option<WindowDirection>,
    /// Whether artificial grow lights are used.
    pub has_grow_lights: bool,
    /// A qualitative description of the room's baseline humidity.
    pub air_description: AirDescription,
    /// Any active methods used to increase local humidity.
    pub humidity_boosters: Vec<HumidityBooster>,
}

/// Result of the estimation algorithm.
#[derive(Clone, Debug, PartialEq)]
pub struct EstimationResult {
    /// The estimated low temperature in Celsius.
    pub temperature_low_c: f64,
    /// The estimated high temperature in Celsius.
    pub temperature_high_c: f64,
    /// The estimated average relative humidity percentage.
    pub humidity_pct: f64,
}

/// Convert Fahrenheit to Celsius.
pub fn f_to_c(f: f64) -> f64 {
    (f - 32.0) * 5.0 / 9.0
}

/// Convert Celsius to Fahrenheit.
pub fn c_to_f(c: f64) -> f64 {
    (c * 9.0 / 5.0) + 32.0
}

/// Calculate VPD (Vapor Pressure Deficit) from temperature and humidity
/// using the August-Roche-Magnus formula.
pub fn calculate_vpd(temp_c: f64, humidity_pct: f64) -> f64 {
    let saturation_pressure = 0.6108 * ((17.27 * temp_c) / (temp_c + 237.3)).exp();
    let actual_pressure = saturation_pressure * (humidity_pct / 100.0);
    saturation_pressure - actual_pressure
}

/// Estimate indoor climate conditions from wizard answers.
pub fn estimate_indoor(input: &IndoorEstimationInput) -> EstimationResult {
    // ── Temperature ──
    let mut temp_adj = 0.0_f64;

    // Room type adjustments
    match input.room_type {
        RoomType::Kitchen => temp_adj += 1.0,
        RoomType::Bathroom => temp_adj += 2.0,
        RoomType::Sunroom => temp_adj += 3.0,
        RoomType::Garage => temp_adj -= 4.0,
        _ => {}
    }

    // Window direction
    if input.has_window
        && matches!(
            input.window_direction,
            Some(WindowDirection::South | WindowDirection::West)
        )
    {
        temp_adj += 1.0;
    }

    // Grow lights
    if input.has_grow_lights {
        temp_adj += 1.0;
    }

    let base_temp = input.thermostat_c + temp_adj;
    let temperature_low_c = base_temp - 2.0;
    let temperature_high_c = base_temp + 2.0;

    // ── Humidity ──
    let mut humidity: f64 = match input.air_description {
        AirDescription::VeryDry => 25.0,
        AirDescription::Average => 40.0,
        AirDescription::Humid => 55.0,
    };

    // Room type humidity adjustments
    match input.room_type {
        RoomType::Bathroom => humidity += 15.0,
        RoomType::Kitchen => humidity += 5.0,
        _ => {}
    }

    // Humidity boosters
    for booster in &input.humidity_boosters {
        match booster {
            HumidityBooster::Humidifier => humidity += 15.0,
            HumidityBooster::RegularMisting => humidity += 5.0,
            HumidityBooster::PebbleTray => humidity += 5.0,
            HumidityBooster::GroupedPlants => humidity += 3.0,
        }
    }

    // Clamp humidity to realistic range
    humidity = humidity.clamp(15.0, 95.0);

    EstimationResult {
        temperature_low_c,
        temperature_high_c,
        humidity_pct: humidity,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_input() -> IndoorEstimationInput {
        IndoorEstimationInput {
            room_type: RoomType::LivingRoom,
            thermostat_c: 22.0,
            has_window: false,
            window_direction: None,
            has_grow_lights: false,
            air_description: AirDescription::Average,
            humidity_boosters: vec![],
        }
    }

    // ── Unit conversion tests ──

    #[test]
    fn test_f_to_c() {
        assert!((f_to_c(32.0) - 0.0).abs() < 0.01);
        assert!((f_to_c(212.0) - 100.0).abs() < 0.01);
        assert!((f_to_c(72.0) - 22.22).abs() < 0.01);
    }

    #[test]
    fn test_c_to_f() {
        assert!((c_to_f(0.0) - 32.0).abs() < 0.01);
        assert!((c_to_f(100.0) - 212.0).abs() < 0.01);
        assert!((c_to_f(22.0) - 71.6).abs() < 0.01);
    }

    #[test]
    fn test_roundtrip_conversion() {
        let temps = [0.0, 20.0, 37.0, -10.0, 100.0];
        for t in temps {
            assert!(
                (f_to_c(c_to_f(t)) - t).abs() < 0.001,
                "Roundtrip failed for {}C",
                t
            );
        }
    }

    // ── VPD calculation tests ──

    #[test]
    fn test_calculate_vpd_typical_orchid_conditions() {
        // At 22°C / 60% RH, VPD ~1.06 kPa (good for orchids: 0.8-1.2 range)
        let vpd = calculate_vpd(22.0, 60.0);
        assert!(
            vpd > 0.9 && vpd < 1.2,
            "Expected VPD ~1.06 at 22C/60%, got {:.3}",
            vpd
        );
    }

    #[test]
    fn test_calculate_vpd_100_percent_humidity() {
        // At 100% RH, VPD should be ~0
        let vpd = calculate_vpd(25.0, 100.0);
        assert!(
            vpd.abs() < 0.01,
            "Expected VPD ~0 at 100% RH, got {:.3}",
            vpd
        );
    }

    #[test]
    fn test_calculate_vpd_very_dry() {
        // At 25°C / 10% RH, VPD should be quite high
        let vpd = calculate_vpd(25.0, 10.0);
        assert!(vpd > 2.5, "Expected high VPD at 10% RH, got {:.3}", vpd);
    }

    #[test]
    fn test_calculate_vpd_zero_humidity() {
        // At 0% humidity, VPD equals saturation pressure
        let vpd = calculate_vpd(20.0, 0.0);
        let exponent: f64 = (17.27 * 20.0) / (20.0 + 237.3);
        let expected_svp = 0.6108 * exponent.exp();
        assert!((vpd - expected_svp).abs() < 0.001);
    }

    // ── Room type temperature adjustments ──

    #[test]
    fn test_living_room_baseline() {
        let input = base_input();
        let result = estimate_indoor(&input);
        assert!((result.temperature_low_c - 20.0).abs() < 0.01);
        assert!((result.temperature_high_c - 24.0).abs() < 0.01);
    }

    #[test]
    fn test_kitchen_adds_1c() {
        let mut input = base_input();
        input.room_type = RoomType::Kitchen;
        let result = estimate_indoor(&input);
        assert!((result.temperature_low_c - 21.0).abs() < 0.01);
        assert!((result.temperature_high_c - 25.0).abs() < 0.01);
    }

    #[test]
    fn test_bathroom_adds_2c() {
        let mut input = base_input();
        input.room_type = RoomType::Bathroom;
        let result = estimate_indoor(&input);
        assert!((result.temperature_low_c - 22.0).abs() < 0.01);
        assert!((result.temperature_high_c - 26.0).abs() < 0.01);
    }

    #[test]
    fn test_sunroom_adds_3c() {
        let mut input = base_input();
        input.room_type = RoomType::Sunroom;
        let result = estimate_indoor(&input);
        assert!((result.temperature_low_c - 23.0).abs() < 0.01);
        assert!((result.temperature_high_c - 27.0).abs() < 0.01);
    }

    #[test]
    fn test_garage_subtracts_4c() {
        let mut input = base_input();
        input.room_type = RoomType::Garage;
        let result = estimate_indoor(&input);
        assert!((result.temperature_low_c - 16.0).abs() < 0.01);
        assert!((result.temperature_high_c - 20.0).abs() < 0.01);
    }

    #[test]
    fn test_office_no_temp_adjustment() {
        let mut input = base_input();
        input.room_type = RoomType::Office;
        let result = estimate_indoor(&input);
        assert!((result.temperature_low_c - 20.0).abs() < 0.01);
        assert!((result.temperature_high_c - 24.0).abs() < 0.01);
    }

    // ── Window direction ──

    #[test]
    fn test_south_window_adds_1c() {
        let mut input = base_input();
        input.has_window = true;
        input.window_direction = Some(WindowDirection::South);
        let result = estimate_indoor(&input);
        assert!((result.temperature_low_c - 21.0).abs() < 0.01);
    }

    #[test]
    fn test_west_window_adds_1c() {
        let mut input = base_input();
        input.has_window = true;
        input.window_direction = Some(WindowDirection::West);
        let result = estimate_indoor(&input);
        assert!((result.temperature_low_c - 21.0).abs() < 0.01);
    }

    #[test]
    fn test_north_window_no_adjustment() {
        let mut input = base_input();
        input.has_window = true;
        input.window_direction = Some(WindowDirection::North);
        let result = estimate_indoor(&input);
        assert!((result.temperature_low_c - 20.0).abs() < 0.01);
    }

    #[test]
    fn test_no_window_ignores_direction() {
        let mut input = base_input();
        input.has_window = false;
        input.window_direction = Some(WindowDirection::South);
        let result = estimate_indoor(&input);
        assert!((result.temperature_low_c - 20.0).abs() < 0.01);
    }

    // ── Grow lights ──

    #[test]
    fn test_grow_lights_add_1c() {
        let mut input = base_input();
        input.has_grow_lights = true;
        let result = estimate_indoor(&input);
        assert!((result.temperature_low_c - 21.0).abs() < 0.01);
    }

    // ── Air description humidity ──

    #[test]
    fn test_very_dry_humidity() {
        let mut input = base_input();
        input.air_description = AirDescription::VeryDry;
        let result = estimate_indoor(&input);
        assert!((result.humidity_pct - 25.0).abs() < 0.01);
    }

    #[test]
    fn test_average_humidity() {
        let input = base_input();
        let result = estimate_indoor(&input);
        assert!((result.humidity_pct - 40.0).abs() < 0.01);
    }

    #[test]
    fn test_humid_humidity() {
        let mut input = base_input();
        input.air_description = AirDescription::Humid;
        let result = estimate_indoor(&input);
        assert!((result.humidity_pct - 55.0).abs() < 0.01);
    }

    // ── Room type humidity adjustments ──

    #[test]
    fn test_bathroom_adds_15pct_humidity() {
        let mut input = base_input();
        input.room_type = RoomType::Bathroom;
        let result = estimate_indoor(&input);
        assert!((result.humidity_pct - 55.0).abs() < 0.01);
    }

    #[test]
    fn test_kitchen_adds_5pct_humidity() {
        let mut input = base_input();
        input.room_type = RoomType::Kitchen;
        let result = estimate_indoor(&input);
        assert!((result.humidity_pct - 45.0).abs() < 0.01);
    }

    // ── Humidity boosters ──

    #[test]
    fn test_humidifier_adds_15pct() {
        let mut input = base_input();
        input.humidity_boosters = vec![HumidityBooster::Humidifier];
        let result = estimate_indoor(&input);
        assert!((result.humidity_pct - 55.0).abs() < 0.01);
    }

    #[test]
    fn test_misting_adds_5pct() {
        let mut input = base_input();
        input.humidity_boosters = vec![HumidityBooster::RegularMisting];
        let result = estimate_indoor(&input);
        assert!((result.humidity_pct - 45.0).abs() < 0.01);
    }

    #[test]
    fn test_pebble_tray_adds_5pct() {
        let mut input = base_input();
        input.humidity_boosters = vec![HumidityBooster::PebbleTray];
        let result = estimate_indoor(&input);
        assert!((result.humidity_pct - 45.0).abs() < 0.01);
    }

    #[test]
    fn test_grouped_plants_adds_3pct() {
        let mut input = base_input();
        input.humidity_boosters = vec![HumidityBooster::GroupedPlants];
        let result = estimate_indoor(&input);
        assert!((result.humidity_pct - 43.0).abs() < 0.01);
    }

    // ── Stacking ──

    #[test]
    fn test_all_boosters_stack() {
        let mut input = base_input();
        input.humidity_boosters = vec![
            HumidityBooster::Humidifier,
            HumidityBooster::RegularMisting,
            HumidityBooster::PebbleTray,
            HumidityBooster::GroupedPlants,
        ];
        // 40 + 15 + 5 + 5 + 3 = 68
        let result = estimate_indoor(&input);
        assert!((result.humidity_pct - 68.0).abs() < 0.01);
    }

    #[test]
    fn test_bathroom_humid_with_all_boosters_clamped() {
        let mut input = base_input();
        input.room_type = RoomType::Bathroom;
        input.air_description = AirDescription::Humid;
        input.humidity_boosters = vec![
            HumidityBooster::Humidifier,
            HumidityBooster::RegularMisting,
            HumidityBooster::PebbleTray,
            HumidityBooster::GroupedPlants,
        ];
        // 55 + 15 + 15 + 5 + 5 + 3 = 98 → clamped to 95
        let result = estimate_indoor(&input);
        assert!((result.humidity_pct - 95.0).abs() < 0.01);
    }

    #[test]
    fn test_humidity_floor_clamp() {
        // VeryDry with no boosters = 25%, which is above the floor.
        // No way to go below 15% with current inputs, but verify clamp works.
        let mut input = base_input();
        input.air_description = AirDescription::VeryDry;
        let result = estimate_indoor(&input);
        assert!(result.humidity_pct >= 15.0);
    }

    // ── Combined adjustments ──

    #[test]
    fn test_sunroom_south_window_grow_lights_all_stack() {
        let mut input = base_input();
        input.room_type = RoomType::Sunroom;
        input.has_window = true;
        input.window_direction = Some(WindowDirection::South);
        input.has_grow_lights = true;
        // Temp: 22 + 3 (sunroom) + 1 (south) + 1 (lights) = 27
        let result = estimate_indoor(&input);
        assert!((result.temperature_low_c - 25.0).abs() < 0.01);
        assert!((result.temperature_high_c - 29.0).abs() < 0.01);
    }

    // ── Display trait coverage ──

    #[test]
    fn test_room_type_display() {
        assert_eq!(RoomType::Kitchen.to_string(), "Kitchen");
        assert_eq!(RoomType::LivingRoom.to_string(), "Living Room");
        assert_eq!(RoomType::Garage.to_string(), "Garage");
    }

    #[test]
    fn test_window_direction_display() {
        assert_eq!(WindowDirection::North.to_string(), "North");
        assert_eq!(WindowDirection::South.to_string(), "South");
    }

    #[test]
    fn test_air_description_display() {
        assert_eq!(AirDescription::VeryDry.to_string(), "Very Dry");
        assert_eq!(AirDescription::Average.to_string(), "Average");
    }

    #[test]
    fn test_humidity_booster_display() {
        assert_eq!(HumidityBooster::Humidifier.to_string(), "Humidifier");
        assert_eq!(HumidityBooster::PebbleTray.to_string(), "Pebble Tray");
    }
}

/// Rough volumetric estimate (in ml) for standard pot sizes.
pub fn get_pot_volume_ml(size: &crate::orchid::PotSize) -> f64 {
    use crate::orchid::PotSize;
    match size {
        PotSize::Small => 150.0,   // ~2 inch
        PotSize::Medium => 500.0,  // ~4 inch
        PotSize::Large => 1500.0,  // ~6 inch
        PotSize::Unknown => 500.0, // default to medium
    }
}

/// The percentage of total pot volume that the medium can hold as water.
/// Represents the Water Holding Capacity (WHC).
pub fn get_medium_whc(medium: &crate::orchid::PotMedium) -> f64 {
    use crate::orchid::PotMedium;
    match medium {
        PotMedium::Bark => 0.25,         // Drains fast, holds ~25%
        PotMedium::SphagnumMoss => 0.75, // Acts like a sponge, holds ~75%
        PotMedium::Leca => 0.40,         // Semi-hydro, holds ~40%
        PotMedium::Inorganic => 0.15,    // Lava rock/pumice, holds very little
        PotMedium::Unknown => 0.30,      // Default to standard bark-like behavior
    }
}

/// A multiplier representing the porosity and aeration of the pot.
/// > 1.0 means it evaporates faster than a standard solid plastic pot.
pub fn get_pot_type_porosity_modifier(pot_type: &crate::orchid::PotType) -> f64 {
    use crate::orchid::PotType;
    match pot_type {
        PotType::Solid => 1.0,   // Standard plastic/glazed ceramic
        PotType::Slotted => 1.4, // Net pots, air flow through sides
        PotType::Clay => 1.9,    // Terra cotta wicks and evaporates heavily
        PotType::Mounted => 5.0, // No pot, extreme evaporation
        PotType::Unknown => 1.0,
    }
}

/// A multiplier representing how fast the plant consumes water based on light levels.
pub fn get_light_consumption_modifier(light: &crate::orchid::LightRequirement) -> f64 {
    use crate::orchid::LightRequirement;
    match light {
        LightRequirement::Low => 0.8,
        LightRequirement::Medium => 1.0,
        LightRequirement::High => 1.3,
    }
}

/// Basic physics constants for the estimation model.
pub const VPD_BASELINE: f64 = 1.19; // 22C / 55% RH

/// Algorithm B: Scientifically calculate the baseline watering days.
///
/// `Inputs:`
/// - `pot_size`: Determines total maximum volume.
/// - `pot_medium`: Determines how much of that volume is actually water (WHC).
/// - `pot_type`: Determines the aeration/porosity multiplier.
/// - `light_req`: Modifies plant consumption rate.
/// - `home_vpd_kpa`: The evaporative demand of the environment.
pub fn calculate_algorithmic_base_days(
    pot_size: &crate::orchid::PotSize,
    pot_medium: &crate::orchid::PotMedium,
    pot_type: &crate::orchid::PotType,
    light_req: &crate::orchid::LightRequirement,
    home_vpd_kpa: f64,
) -> u32 {
    // Prevent division by zero or extreme outliers
    let safe_vpd = home_vpd_kpa.clamp(0.2, 4.0);

    // 1. Total Water Volume (ml)
    let volume_ml = get_pot_volume_ml(pot_size);
    let whc = get_medium_whc(pot_medium);
    let total_water_ml = volume_ml * whc;

    // 2. Base Evaporation Rate
    // Through trial and error, 18ml/day at 1.19 kPa accurately produces ~7 days
    // for a Medium pot with Bark (125ml WHC).
    let baseline_evaporation_rate = 18.0 * (safe_vpd / VPD_BASELINE);

    // 3. Apply Modifiers
    let porosity_mod = get_pot_type_porosity_modifier(pot_type);
    let consumption_mod = get_light_consumption_modifier(light_req);

    let daily_water_loss = baseline_evaporation_rate * porosity_mod * consumption_mod;

    // 4. Calculate Days
    let raw_days = total_water_ml / daily_water_loss;

    // We clamp the result between 1 and 30 days to keep things reasonable
    (raw_days.round() as u32).clamp(1, 30)
}

/// A scientific recommendation about the potting setup.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SuitabilityRecommendation {
    /// The medium determined to be the best fit.
    pub suggested_medium: crate::orchid::PotMedium,
    /// The pot type determined to be the best fit.
    pub suggested_pot_type: crate::orchid::PotType,
    /// Detailed text describing why this setup is recommended.
    pub scientific_reasoning: String,
}

/// Algorithm A: "Suitability Fit" & Medium Recommendation.
/// Compares the home environment's VPD to the plant's native VPD.
pub fn recommend_potting_setup(native_vpd: f64, home_vpd: f64) -> SuitabilityRecommendation {
    let gradient = home_vpd - native_vpd;
    use crate::orchid::{PotMedium, PotType};

    if gradient > 0.6 {
        SuitabilityRecommendation {
            suggested_medium: PotMedium::SphagnumMoss,
            suggested_pot_type: PotType::Solid,
            scientific_reasoning: format!(
                "Your environment ({:.1} kPa) is significantly drier and evaporates water much faster than this plant's native habitat ({:.1} kPa). We highly recommend potting in Sphagnum Moss inside a Solid pot to artificially retain moisture.",
                home_vpd, native_vpd
            ),
        }
    } else if gradient > 0.2 {
        SuitabilityRecommendation {
            suggested_medium: PotMedium::Bark,
            suggested_pot_type: PotType::Solid,
            scientific_reasoning: format!(
                "Your environment ({:.1} kPa) is slightly drier than the native habitat ({:.1} kPa). A standard Bark mix in a Solid plastic pot will provide a healthy balance of aeration and moisture retention.",
                home_vpd, native_vpd
            ),
        }
    } else if gradient < -0.2 {
        SuitabilityRecommendation {
            suggested_medium: PotMedium::Bark,
            suggested_pot_type: PotType::Slotted,
            scientific_reasoning: format!(
                "Your environment ({:.1} kPa) is more humid and evaporates less water than the native habitat ({:.1} kPa). We recommend coarse Bark in a Slotted pot or Terra Cotta to increase aeration and prevent root rot.",
                home_vpd, native_vpd
            ),
        }
    } else {
        SuitabilityRecommendation {
            suggested_medium: PotMedium::Bark,
            suggested_pot_type: PotType::Slotted,
            scientific_reasoning: format!(
                "Your environment ({:.1} kPa) perfectly matches the evaporative demand of the native habitat ({:.1} kPa). A standard Bark mix in a Slotted pot will mimic natural tree-branch conditions beautifully.",
                home_vpd, native_vpd
            ),
        }
    }
}
