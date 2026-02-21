use serde::{Deserialize, Serialize};
use std::fmt;
use chrono::{DateTime, Datelike, Utc};

#[cfg(feature = "ssr")]
use surrealdb::types::SurrealValue;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(surrealdb::types::SurrealValue))]
#[cfg_attr(feature = "ssr", surreal(crate = "surrealdb::types", untagged))]
pub enum LocationType {
    Indoor,
    Outdoor,
}

impl fmt::Display for LocationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LocationType::Indoor => write!(f, "Indoor"),
            LocationType::Outdoor => write!(f, "Outdoor"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(surrealdb::types::SurrealValue))]
#[cfg_attr(feature = "ssr", surreal(crate = "surrealdb::types", untagged))]
pub enum FitCategory {
    #[serde(rename = "Good Fit")]
    GoodFit,
    #[serde(rename = "Bad Fit")]
    BadFit,
    #[serde(rename = "Caution Fit")]
    CautionFit,
}

impl fmt::Display for FitCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FitCategory::GoodFit => write!(f, "Good Fit"),
            FitCategory::BadFit => write!(f, "Bad Fit"),
            FitCategory::CautionFit => write!(f, "Caution Fit"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(surrealdb::types::SurrealValue))]
#[cfg_attr(feature = "ssr", surreal(crate = "surrealdb::types", untagged))]
pub enum LightRequirement {
    #[serde(alias = "low", alias = "Low Light")]
    Low,
    #[serde(alias = "medium", alias = "Medium Light")]
    Medium,
    #[serde(alias = "high", alias = "High Light")]
    High,
}

impl LightRequirement {
    /// Returns the DB-compatible short key: "Low", "Medium", "High".
    /// Use this when sending values to SurrealDB or serializing for storage.
    /// For UI display, use `Display` which returns "Low Light", etc.
    pub fn as_str(&self) -> &'static str {
        match self {
            LightRequirement::Low => "Low",
            LightRequirement::Medium => "Medium",
            LightRequirement::High => "High",
        }
    }
}

impl fmt::Display for LightRequirement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LightRequirement::Low => write!(f, "Low Light"),
            LightRequirement::Medium => write!(f, "Medium Light"),
            LightRequirement::High => write!(f, "High Light"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(surrealdb::types::SurrealValue))]
#[cfg_attr(feature = "ssr", surreal(crate = "surrealdb::types"))]
pub struct GrowingZone {
    pub id: String,
    pub name: String,
    pub light_level: LightRequirement,
    pub location_type: LocationType,
    #[serde(default)]
    #[cfg_attr(feature = "ssr", surreal(default))]
    pub temperature_range: String,
    #[serde(default)]
    #[cfg_attr(feature = "ssr", surreal(default))]
    pub humidity: String,
    #[serde(default)]
    #[cfg_attr(feature = "ssr", surreal(default))]
    pub description: String,
    #[serde(default)]
    #[cfg_attr(feature = "ssr", surreal(default))]
    pub sort_order: i32,
    #[serde(default)]
    #[cfg_attr(feature = "ssr", surreal(default))]
    pub data_source_type: Option<String>,
    #[serde(default)]
    #[cfg_attr(feature = "ssr", surreal(default))]
    pub data_source_config: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(surrealdb::types::SurrealValue))]
#[cfg_attr(feature = "ssr", surreal(crate = "surrealdb::types"))]
pub struct ClimateReading {
    pub id: String,
    pub zone_id: String,
    pub zone_name: String,
    pub temperature: f64,
    pub humidity: f64,
    #[serde(default)]
    #[cfg_attr(feature = "ssr", surreal(default))]
    pub vpd: Option<f64>,
    #[serde(default)]
    #[cfg_attr(feature = "ssr", surreal(default))]
    pub source: Option<String>,
    pub recorded_at: DateTime<Utc>,
}

/// Check if an orchid's placement zone provides compatible light for its requirements.
/// Returns true if compatible or if the zone is unknown.
pub fn check_zone_compatibility(placement: &str, light_req: &LightRequirement, zones: &[GrowingZone]) -> bool {
    zones.iter()
        .find(|z| z.name == placement)
        .map(|z| z.light_level == *light_req)
        .unwrap_or(true)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(surrealdb::types::SurrealValue))]
#[cfg_attr(feature = "ssr", surreal(crate = "surrealdb::types"))]
pub struct LogEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub note: String,
    #[serde(default)]
    #[cfg_attr(feature = "ssr", surreal(default))]
    pub image_filename: Option<String>,
    #[serde(default)]
    #[cfg_attr(feature = "ssr", surreal(default))]
    pub event_type: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(surrealdb::types::SurrealValue))]
#[cfg_attr(feature = "ssr", surreal(crate = "surrealdb::types"))]
pub struct Orchid {
    pub id: String,
    pub name: String,
    pub species: String,
    pub water_frequency_days: u32,
    pub light_requirement: LightRequirement,
    pub notes: String,
    pub placement: String,
    pub light_lux: String,
    pub temperature_range: String,
    #[serde(default)]
    #[cfg_attr(feature = "ssr", surreal(default))]
    pub conservation_status: Option<String>,
    #[serde(default)]
    #[cfg_attr(feature = "ssr", surreal(default))]
    pub native_region: Option<String>,
    #[serde(default)]
    #[cfg_attr(feature = "ssr", surreal(default))]
    pub native_latitude: Option<f64>,
    #[serde(default)]
    #[cfg_attr(feature = "ssr", surreal(default))]
    pub native_longitude: Option<f64>,
    #[serde(default)]
    #[cfg_attr(feature = "ssr", surreal(default))]
    pub last_watered_at: Option<DateTime<Utc>>,
    #[serde(default)]
    #[cfg_attr(feature = "ssr", surreal(default))]
    pub temp_min: Option<f64>,
    #[serde(default)]
    #[cfg_attr(feature = "ssr", surreal(default))]
    pub temp_max: Option<f64>,
    #[serde(default)]
    #[cfg_attr(feature = "ssr", surreal(default))]
    pub humidity_min: Option<f64>,
    #[serde(default)]
    #[cfg_attr(feature = "ssr", surreal(default))]
    pub humidity_max: Option<f64>,
    #[serde(default)]
    #[cfg_attr(feature = "ssr", surreal(default))]
    pub first_bloom_at: Option<DateTime<Utc>>,
    #[serde(default)]
    #[cfg_attr(feature = "ssr", surreal(default))]
    pub last_fertilized_at: Option<DateTime<Utc>>,
    #[serde(default)]
    #[cfg_attr(feature = "ssr", surreal(default))]
    pub fertilize_frequency_days: Option<u32>,
    #[serde(default)]
    #[cfg_attr(feature = "ssr", surreal(default))]
    pub fertilizer_type: Option<String>,
    #[serde(default)]
    #[cfg_attr(feature = "ssr", surreal(default))]
    pub last_repotted_at: Option<DateTime<Utc>>,
    #[serde(default)]
    #[cfg_attr(feature = "ssr", surreal(default))]
    pub pot_medium: Option<String>,
    #[serde(default)]
    #[cfg_attr(feature = "ssr", surreal(default))]
    pub pot_size: Option<String>,
    // Seasonal care fields
    #[serde(default)]
    #[cfg_attr(feature = "ssr", surreal(default))]
    pub rest_start_month: Option<u32>,
    #[serde(default)]
    #[cfg_attr(feature = "ssr", surreal(default))]
    pub rest_end_month: Option<u32>,
    #[serde(default)]
    #[cfg_attr(feature = "ssr", surreal(default))]
    pub bloom_start_month: Option<u32>,
    #[serde(default)]
    #[cfg_attr(feature = "ssr", surreal(default))]
    pub bloom_end_month: Option<u32>,
    #[serde(default)]
    #[cfg_attr(feature = "ssr", surreal(default))]
    pub rest_water_multiplier: Option<f64>,
    #[serde(default)]
    #[cfg_attr(feature = "ssr", surreal(default))]
    pub rest_fertilizer_multiplier: Option<f64>,
    #[serde(default)]
    #[cfg_attr(feature = "ssr", surreal(default))]
    pub active_water_multiplier: Option<f64>,
    #[serde(default)]
    #[cfg_attr(feature = "ssr", surreal(default))]
    pub active_fertilizer_multiplier: Option<f64>,
}

impl Orchid {
    /// Days since last watered, or None if never watered.
    pub fn days_since_watered(&self) -> Option<i64> {
        self.last_watered_at.map(|dt| (Utc::now() - dt).num_days())
    }

    /// True if watering is overdue based on water_frequency_days.
    pub fn is_overdue(&self) -> bool {
        self.days_since_watered()
            .map(|days| days > self.water_frequency_days as i64)
            .unwrap_or(false)
    }

    /// Days until watering is due. Negative = overdue. None if never watered.
    pub fn days_until_due(&self) -> Option<i64> {
        self.days_since_watered()
            .map(|days| self.water_frequency_days as i64 - days)
    }

    /// Days since last fertilized, or None if never fertilized.
    pub fn days_since_fertilized(&self) -> Option<i64> {
        self.last_fertilized_at.map(|dt| (Utc::now() - dt).num_days())
    }

    /// Days until fertilizing is due. None if no schedule set.
    pub fn fertilize_days_until_due(&self) -> Option<i64> {
        self.fertilize_frequency_days.and_then(|freq| {
            self.days_since_fertilized()
                .map(|days| freq as i64 - days)
        })
    }

    /// Days since last repotted, or None if never repotted.
    pub fn days_since_repotted(&self) -> Option<i64> {
        self.last_repotted_at.map(|dt| (Utc::now() - dt).num_days())
    }

    /// Whether this orchid has seasonal data configured.
    pub fn has_seasonal_data(&self) -> bool {
        self.rest_start_month.is_some() || self.bloom_start_month.is_some()
    }

    /// Determine the current seasonal phase for the given hemisphere.
    pub fn current_phase(&self, hemisphere: &Hemisphere) -> SeasonalPhase {
        let now_month = Utc::now().month();

        // Check bloom season first (most specific)
        if let (Some(bs), Some(be)) = (self.bloom_start_month, self.bloom_end_month) {
            let start = hemisphere.adjust_month(bs);
            let end = hemisphere.adjust_month(be);
            if month_in_range(now_month, start, end) {
                return SeasonalPhase::Blooming;
            }
        }

        // Check rest period
        if let (Some(rs), Some(re)) = (self.rest_start_month, self.rest_end_month) {
            let start = hemisphere.adjust_month(rs);
            let end = hemisphere.adjust_month(re);
            if month_in_range(now_month, start, end) {
                return SeasonalPhase::Rest;
            }
        }

        // If seasonal data exists but we're not in rest or bloom, we're active
        if self.has_seasonal_data() {
            return SeasonalPhase::Active;
        }

        SeasonalPhase::Unknown
    }

    /// Get effective water frequency adjusted for current season.
    pub fn effective_water_frequency(&self, hemisphere: &Hemisphere) -> u32 {
        let base = self.water_frequency_days;
        let multiplier = match self.current_phase(hemisphere) {
            SeasonalPhase::Rest => self.rest_water_multiplier,
            SeasonalPhase::Active | SeasonalPhase::Blooming => self.active_water_multiplier,
            SeasonalPhase::Unknown => None,
        };
        match multiplier {
            Some(m) if m > 0.0 => ((base as f64 / m).round() as u32).max(1),
            _ => base,
        }
    }

    /// Get effective fertilizer frequency adjusted for current season.
    pub fn effective_fertilize_frequency(&self, hemisphere: &Hemisphere) -> Option<u32> {
        let base = self.fertilize_frequency_days?;
        let multiplier = match self.current_phase(hemisphere) {
            SeasonalPhase::Rest => self.rest_fertilizer_multiplier,
            SeasonalPhase::Active | SeasonalPhase::Blooming => self.active_fertilizer_multiplier,
            SeasonalPhase::Unknown => None,
        };
        match multiplier {
            Some(m) if m > 0.0 => Some(((base as f64 / m).round() as u32).max(1)),
            _ => Some(base),
        }
    }

    /// Get month name for display.
    pub fn month_name(month: u32) -> &'static str {
        match month {
            1 => "Jan", 2 => "Feb", 3 => "Mar", 4 => "Apr",
            5 => "May", 6 => "Jun", 7 => "Jul", 8 => "Aug",
            9 => "Sep", 10 => "Oct", 11 => "Nov", 12 => "Dec",
            _ => "???",
        }
    }

    /// Returns the next seasonal transition: (month, phase_name).
    /// Returns None if no seasonal data.
    pub fn next_transition(&self, hemisphere: &Hemisphere) -> Option<(u32, String)> {
        let now_month = Utc::now().month();
        let mut transitions = Vec::new();

        if let (Some(rs), Some(_re)) = (self.rest_start_month, self.rest_end_month) {
            transitions.push((hemisphere.adjust_month(rs), "Rest begins".to_string()));
        }
        if let (Some(_rs), Some(re)) = (self.rest_start_month, self.rest_end_month) {
            let end_adjusted = hemisphere.adjust_month(re);
            let active_month = if end_adjusted == 12 { 1 } else { end_adjusted + 1 };
            transitions.push((active_month, "Rest ends".to_string()));
        }
        if let (Some(bs), Some(_be)) = (self.bloom_start_month, self.bloom_end_month) {
            transitions.push((hemisphere.adjust_month(bs), "Bloom begins".to_string()));
        }
        if let (Some(_bs), Some(be)) = (self.bloom_start_month, self.bloom_end_month) {
            let end_adjusted = hemisphere.adjust_month(be);
            let after_month = if end_adjusted == 12 { 1 } else { end_adjusted + 1 };
            transitions.push((after_month, "Bloom ends".to_string()));
        }

        // Find the next transition after the current month
        transitions.sort_by_key(|(m, _)| {
            if *m > now_month { *m - now_month } else { *m + 12 - now_month }
        });

        transitions.into_iter().find(|(m, _)| *m != now_month).or(None)
    }
}

/// Hemisphere for seasonal calculations
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Hemisphere {
    Northern,
    Southern,
}

impl Hemisphere {
    pub fn from_code(code: &str) -> Self {
        match code {
            "S" => Hemisphere::Southern,
            _ => Hemisphere::Northern,
        }
    }

    pub fn code(&self) -> &str {
        match self {
            Hemisphere::Northern => "N",
            Hemisphere::Southern => "S",
        }
    }

    /// Adjust a month stored in Northern Hemisphere terms for the given hemisphere.
    pub fn adjust_month(&self, month: u32) -> u32 {
        match self {
            Hemisphere::Northern => month,
            Hemisphere::Southern => ((month + 5) % 12) + 1, // shift 6 months
        }
    }
}

/// The current seasonal phase of an orchid
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SeasonalPhase {
    Rest,
    Active,
    Blooming,
    Unknown,
}

impl fmt::Display for SeasonalPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SeasonalPhase::Rest => write!(f, "Rest"),
            SeasonalPhase::Active => write!(f, "Active"),
            SeasonalPhase::Blooming => write!(f, "Blooming"),
            SeasonalPhase::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Check if a given month falls within a range (handles wrap-around, e.g. Nov-Feb).
pub fn month_in_range(month: u32, start: u32, end: u32) -> bool {
    if start <= end {
        month >= start && month <= end
    } else {
        // Wraps around year-end, e.g. Nov(11) to Feb(2)
        month >= start || month <= end
    }
}

/// An alert for condition drift or overdue watering
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    #[serde(default)]
    pub orchid_name: Option<String>,
    #[serde(default)]
    pub zone_name: Option<String>,
    pub alert_type: String,
    pub severity: String,
    pub message: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HabitatWeather {
    pub temperature: f64,
    pub humidity: f64,
    pub precipitation: f64,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HabitatWeatherSummary {
    pub period_type: String,
    pub period_start: DateTime<Utc>,
    pub avg_temperature: f64,
    pub min_temperature: f64,
    pub max_temperature: f64,
    pub avg_humidity: f64,
    pub total_precipitation: f64,
    pub sample_count: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_climate_reading_serde_with_source() {
        let reading = ClimateReading {
            id: "climate_reading:abc".into(),
            zone_id: "growing_zone:123".into(),
            zone_name: "Kitchen".into(),
            temperature: 22.5,
            humidity: 55.0,
            vpd: Some(0.85),
            source: Some("wizard".into()),
            recorded_at: Utc::now(),
        };

        let json = serde_json::to_string(&reading).unwrap();
        let deserialized: ClimateReading = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.source, Some("wizard".into()));
        assert_eq!(deserialized.zone_name, "Kitchen");
        assert!((deserialized.temperature - 22.5).abs() < 0.01);
    }

    #[test]
    fn test_climate_reading_serde_without_source() {
        // Backward compat: older readings without source field
        let json = r#"{"id":"cr:old","zone_id":"gz:1","zone_name":"Zone A","temperature":21.0,"humidity":50.0,"recorded_at":"2025-01-01T00:00:00Z"}"#;
        let reading: ClimateReading = serde_json::from_str(json).unwrap();

        assert_eq!(reading.source, None);
        assert_eq!(reading.vpd, None);
        assert_eq!(reading.zone_name, "Zone A");
    }

    #[test]
    fn test_zone_compatibility() {
        let zones = vec![
            GrowingZone {
                id: "1".into(),
                name: "Low Light Area".into(),
                light_level: LightRequirement::Low,
                location_type: LocationType::Indoor,
                temperature_range: String::new(),
                humidity: String::new(),
                description: String::new(),
                sort_order: 0,
                data_source_type: None,
                data_source_config: String::new(),
            },
            GrowingZone {
                id: "2".into(),
                name: "High Light Area".into(),
                light_level: LightRequirement::High,
                location_type: LocationType::Indoor,
                temperature_range: String::new(),
                humidity: String::new(),
                description: String::new(),
                sort_order: 1,
                data_source_type: None,
                data_source_config: String::new(),
            },
        ];

        assert!(check_zone_compatibility("Low Light Area", &LightRequirement::Low, &zones));
        assert!(!check_zone_compatibility("Low Light Area", &LightRequirement::High, &zones));
        assert!(check_zone_compatibility("High Light Area", &LightRequirement::High, &zones));
        // Unknown zone = don't flag
        assert!(check_zone_compatibility("Unknown Zone", &LightRequirement::High, &zones));
    }

    #[test]
    fn test_orchid_creation() {
        let orchid = Orchid {
            id: "test:1".into(),
            name: "Test Orchid".into(),
            species: "Phalaenopsis".into(),
            water_frequency_days: 7,
            light_requirement: LightRequirement::Medium,
            notes: "Notes".into(),
            placement: "Medium Light Area".to_string(),
            light_lux: "1000".into(),
            temperature_range: "20-30C".into(),
            conservation_status: Some("CITES II".into()),
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
        };

        assert_eq!(orchid.name, "Test Orchid");
        assert_eq!(orchid.light_requirement, LightRequirement::Medium);
        assert_eq!(orchid.conservation_status, Some("CITES II".into()));
    }

    #[test]
    fn test_watering_helpers_never_watered() {
        let orchid = Orchid {
            id: "test:1".into(),
            name: "Test".into(),
            species: "Test".into(),
            water_frequency_days: 7,
            light_requirement: LightRequirement::Medium,
            notes: String::new(),
            placement: String::new(),
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
        };
        assert_eq!(orchid.days_since_watered(), None);
        assert!(!orchid.is_overdue());
        assert_eq!(orchid.days_until_due(), None);
    }

    #[test]
    fn test_watering_helpers_recently_watered() {
        let orchid = Orchid {
            id: "test:1".into(),
            name: "Test".into(),
            species: "Test".into(),
            water_frequency_days: 7,
            light_requirement: LightRequirement::Medium,
            notes: String::new(),
            placement: String::new(),
            light_lux: String::new(),
            temperature_range: String::new(),
            conservation_status: None,
            native_region: None,
            native_latitude: None,
            native_longitude: None,
            last_watered_at: Some(Utc::now() - chrono::Duration::days(2)),
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
        };
        assert_eq!(orchid.days_since_watered(), Some(2));
        assert!(!orchid.is_overdue());
        assert_eq!(orchid.days_until_due(), Some(5));
    }

    #[test]
    fn test_watering_helpers_overdue() {
        let orchid = Orchid {
            id: "test:1".into(),
            name: "Test".into(),
            species: "Test".into(),
            water_frequency_days: 7,
            light_requirement: LightRequirement::Medium,
            notes: String::new(),
            placement: String::new(),
            light_lux: String::new(),
            temperature_range: String::new(),
            conservation_status: None,
            native_region: None,
            native_latitude: None,
            native_longitude: None,
            last_watered_at: Some(Utc::now() - chrono::Duration::days(10)),
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
        };
        assert_eq!(orchid.days_since_watered(), Some(10));
        assert!(orchid.is_overdue());
        assert_eq!(orchid.days_until_due(), Some(-3));
    }

    #[test]
    fn test_fit_category_serde() {
        let good: FitCategory = serde_json::from_str("\"Good Fit\"").unwrap();
        assert_eq!(good, FitCategory::GoodFit);
        let bad: FitCategory = serde_json::from_str("\"Bad Fit\"").unwrap();
        assert_eq!(bad, FitCategory::BadFit);
        let caution: FitCategory = serde_json::from_str("\"Caution Fit\"").unwrap();
        assert_eq!(caution, FitCategory::CautionFit);
    }

    #[test]
    fn test_light_requirement_as_str() {
        assert_eq!(LightRequirement::Low.as_str(), "Low");
        assert_eq!(LightRequirement::Medium.as_str(), "Medium");
        assert_eq!(LightRequirement::High.as_str(), "High");
    }

    #[test]
    fn test_light_requirement_display_vs_as_str() {
        // Display is for UI ("Low Light"), as_str is for DB ("Low")
        assert_eq!(LightRequirement::Low.to_string(), "Low Light");
        assert_eq!(LightRequirement::Low.as_str(), "Low");
        assert_eq!(LightRequirement::Medium.to_string(), "Medium Light");
        assert_eq!(LightRequirement::Medium.as_str(), "Medium");
        assert_eq!(LightRequirement::High.to_string(), "High Light");
        assert_eq!(LightRequirement::High.as_str(), "High");
    }

    #[test]
    fn test_light_requirement_aliases() {
        let low: LightRequirement = serde_json::from_str("\"low\"").unwrap();
        assert_eq!(low, LightRequirement::Low);
        let low2: LightRequirement = serde_json::from_str("\"Low Light\"").unwrap();
        assert_eq!(low2, LightRequirement::Low);
        let medium: LightRequirement = serde_json::from_str("\"Medium\"").unwrap();
        assert_eq!(medium, LightRequirement::Medium);
    }

    #[test]
    fn test_log_entry_serde_with_event_type() {
        let entry = LogEntry {
            id: "log_entry:abc".into(),
            timestamp: Utc::now(),
            note: "New spike emerging".into(),
            image_filename: Some("user1/photo.jpg".into()),
            event_type: Some("Flowering".into()),
        };

        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: LogEntry = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, entry.id);
        assert_eq!(deserialized.note, entry.note);
        assert_eq!(deserialized.image_filename, Some("user1/photo.jpg".into()));
        assert_eq!(deserialized.event_type, Some("Flowering".into()));
    }

    #[test]
    fn test_log_entry_serde_without_event_type() {
        // Backward compat: older entries have no event_type or image_filename
        let json = r#"{"id":"log_entry:old","timestamp":"2025-01-01T00:00:00Z","note":"Watered"}"#;
        let entry: LogEntry = serde_json::from_str(json).unwrap();

        assert_eq!(entry.id, "log_entry:old");
        assert_eq!(entry.note, "Watered");
        assert_eq!(entry.event_type, None);
        assert_eq!(entry.image_filename, None);
    }

    #[test]
    fn test_orchid_serde_with_first_bloom_at() {
        let now = Utc::now();
        let orchid = Orchid {
            id: "orchid:bloom1".into(),
            name: "Blooming Beauty".into(),
            species: "Cattleya".into(),
            water_frequency_days: 5,
            light_requirement: LightRequirement::High,
            notes: String::new(),
            placement: "South Window".into(),
            light_lux: "8000".into(),
            temperature_range: "18-30C".into(),
            conservation_status: None,
            native_region: Some("Brazil".into()),
            native_latitude: Some(-15.78),
            native_longitude: Some(-47.93),
            last_watered_at: Some(now),
            temp_min: Some(18.0),
            temp_max: Some(30.0),
            humidity_min: Some(60.0),
            humidity_max: Some(80.0),
            first_bloom_at: Some(now),
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
        };

        let json = serde_json::to_string(&orchid).unwrap();
        let deserialized: Orchid = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.first_bloom_at.is_some(), true);
        assert_eq!(deserialized.name, "Blooming Beauty");
        assert_eq!(deserialized.native_region, Some("Brazil".into()));
    }

    #[test]
    fn test_orchid_serde_without_first_bloom_at() {
        // Backward compat: older orchids have no first_bloom_at
        let json = r#"{"id":"orchid:old","name":"Old","species":"Phal","water_frequency_days":7,"light_requirement":"Medium","notes":"","placement":"Zone A","light_lux":"","temperature_range":""}"#;
        let orchid: Orchid = serde_json::from_str(json).unwrap();

        assert_eq!(orchid.first_bloom_at, None);
        assert_eq!(orchid.name, "Old");
        // Seasonal fields should default to None
        assert_eq!(orchid.rest_start_month, None);
        assert_eq!(orchid.bloom_start_month, None);
    }

    #[test]
    fn test_month_in_range_normal() {
        assert!(month_in_range(3, 1, 5));
        assert!(!month_in_range(7, 1, 5));
    }

    #[test]
    fn test_month_in_range_wrap_around() {
        // Nov to Feb
        assert!(month_in_range(12, 11, 2));
        assert!(month_in_range(1, 11, 2));
        assert!(month_in_range(11, 11, 2));
        assert!(!month_in_range(5, 11, 2));
    }

    #[test]
    fn test_hemisphere_adjust_month() {
        let n = Hemisphere::Northern;
        let s = Hemisphere::Southern;
        assert_eq!(n.adjust_month(1), 1);
        assert_eq!(s.adjust_month(1), 7); // Jan NH → Jul SH
        assert_eq!(s.adjust_month(7), 1); // Jul NH → Jan SH
        assert_eq!(s.adjust_month(11), 5); // Nov NH → May SH
    }

    #[test]
    fn test_has_seasonal_data() {
        let mut orchid = Orchid {
            id: "test:1".into(),
            name: "Test".into(),
            species: "Test".into(),
            water_frequency_days: 7,
            light_requirement: LightRequirement::Medium,
            notes: String::new(),
            placement: String::new(),
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
        };
        assert!(!orchid.has_seasonal_data());
        orchid.rest_start_month = Some(11);
        assert!(orchid.has_seasonal_data());
    }

    /// Helper to create a minimal orchid with seasonal fields for testing.
    fn seasonal_orchid(
        water_freq: u32,
        fert_freq: Option<u32>,
        rest: Option<(u32, u32)>,
        bloom: Option<(u32, u32)>,
        rest_water_mult: Option<f64>,
        rest_fert_mult: Option<f64>,
        active_water_mult: Option<f64>,
        active_fert_mult: Option<f64>,
    ) -> Orchid {
        Orchid {
            id: "test:seasonal".into(),
            name: "Seasonal Test".into(),
            species: "Dendrobium nobile".into(),
            water_frequency_days: water_freq,
            light_requirement: LightRequirement::Medium,
            notes: String::new(),
            placement: String::new(),
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
            fertilize_frequency_days: fert_freq,
            fertilizer_type: None,
            last_repotted_at: None,
            pot_medium: None,
            pot_size: None,
            rest_start_month: rest.map(|(s, _)| s),
            rest_end_month: rest.map(|(_, e)| e),
            bloom_start_month: bloom.map(|(s, _)| s),
            bloom_end_month: bloom.map(|(_, e)| e),
            rest_water_multiplier: rest_water_mult,
            rest_fertilizer_multiplier: rest_fert_mult,
            active_water_multiplier: active_water_mult,
            active_fertilizer_multiplier: active_fert_mult,
        }
    }

    // ── Hemisphere enum tests ────────────────────────────────────────

    #[test]
    fn test_hemisphere_from_code() {
        assert_eq!(Hemisphere::from_code("N"), Hemisphere::Northern);
        assert_eq!(Hemisphere::from_code("S"), Hemisphere::Southern);
        assert_eq!(Hemisphere::from_code(""), Hemisphere::Northern); // default
        assert_eq!(Hemisphere::from_code("X"), Hemisphere::Northern); // unknown → default
    }

    #[test]
    fn test_hemisphere_code_roundtrip() {
        assert_eq!(Hemisphere::from_code(Hemisphere::Northern.code()), Hemisphere::Northern);
        assert_eq!(Hemisphere::from_code(Hemisphere::Southern.code()), Hemisphere::Southern);
    }

    #[test]
    fn test_hemisphere_adjust_all_months_southern() {
        let s = Hemisphere::Southern;
        // NH month → SH month: shift +6, wrap at 12
        // Jan(1)→Jul(7), Feb(2)→Aug(8), ..., Jun(6)→Dec(12), Jul(7)→Jan(1), ..., Dec(12)→Jun(6)
        let expected = [7, 8, 9, 10, 11, 12, 1, 2, 3, 4, 5, 6];
        for (nh_month, &sh_expected) in (1..=12u32).zip(expected.iter()) {
            assert_eq!(s.adjust_month(nh_month), sh_expected,
                "NH month {} should map to SH month {}", nh_month, sh_expected);
        }
    }

    // ── SeasonalPhase Display ────────────────────────────────────────

    #[test]
    fn test_seasonal_phase_display() {
        assert_eq!(SeasonalPhase::Rest.to_string(), "Rest");
        assert_eq!(SeasonalPhase::Active.to_string(), "Active");
        assert_eq!(SeasonalPhase::Blooming.to_string(), "Blooming");
        assert_eq!(SeasonalPhase::Unknown.to_string(), "Unknown");
    }

    // ── month_in_range edge cases ────────────────────────────────────

    #[test]
    fn test_month_in_range_single_month() {
        // start == end: only that month is in range
        assert!(month_in_range(6, 6, 6));
        assert!(!month_in_range(5, 6, 6));
        assert!(!month_in_range(7, 6, 6));
    }

    #[test]
    fn test_month_in_range_full_year() {
        // Jan to Dec: all months in range
        for m in 1..=12 {
            assert!(month_in_range(m, 1, 12));
        }
    }

    #[test]
    fn test_month_in_range_wrap_boundaries() {
        // Dec to Jan (wraps): only Dec and Jan
        assert!(month_in_range(12, 12, 1));
        assert!(month_in_range(1, 12, 1));
        assert!(!month_in_range(6, 12, 1));
        assert!(!month_in_range(11, 12, 1));
    }

    // ── current_phase tests ──────────────────────────────────────────

    #[test]
    fn test_current_phase_unknown_without_seasonal_data() {
        let orchid = seasonal_orchid(7, None, None, None, None, None, None, None);
        assert_eq!(orchid.current_phase(&Hemisphere::Northern), SeasonalPhase::Unknown);
    }

    #[test]
    fn test_current_phase_rest_period() {
        // Create an orchid with rest covering all 12 months to guarantee we hit it
        let orchid = seasonal_orchid(7, None, Some((1, 12)), None, None, None, None, None);
        assert_eq!(orchid.current_phase(&Hemisphere::Northern), SeasonalPhase::Rest);
    }

    #[test]
    fn test_current_phase_bloom_takes_priority_over_rest() {
        // Bloom covering all months AND rest covering all months — bloom wins
        let orchid = seasonal_orchid(7, None, Some((1, 12)), Some((1, 12)), None, None, None, None);
        assert_eq!(orchid.current_phase(&Hemisphere::Northern), SeasonalPhase::Blooming);
    }

    #[test]
    fn test_current_phase_active_when_outside_rest_and_bloom() {
        // Rest only in a month that's not current (use month 0 trick: set a single far-away month)
        // We can't know the current month at test time, but we can test the logic:
        // If rest is a single month that may/may not match, we test the active fallback instead
        // Use a month that's exactly 6 months away from now — very unlikely to match
        let now_month = Utc::now().month();
        let far_month = ((now_month + 5) % 12) + 1; // 6 months from now
        let orchid = seasonal_orchid(7, None, Some((far_month, far_month)), None, None, None, None, None);
        let phase = orchid.current_phase(&Hemisphere::Northern);
        // Should be either Active (if now != far_month) or Rest (if now == far_month)
        assert!(phase == SeasonalPhase::Active || phase == SeasonalPhase::Rest);
        // With has_seasonal_data true, it should never be Unknown
        assert_ne!(phase, SeasonalPhase::Unknown);
    }

    #[test]
    fn test_current_phase_southern_hemisphere_shifts() {
        // Rest Nov-Feb in NH terms. In SH, that shifts to May-Aug.
        let now_month = Utc::now().month();
        let orchid = seasonal_orchid(7, None, Some((11, 2)), None, None, None, None, None);

        let nh_phase = orchid.current_phase(&Hemisphere::Northern);
        let sh_phase = orchid.current_phase(&Hemisphere::Southern);

        // NH rest = Nov(11)-Feb(2), SH rest = May(5)-Aug(8)
        let in_nh_rest = month_in_range(now_month, 11, 2);
        let in_sh_rest = month_in_range(now_month, 5, 8);

        if in_nh_rest {
            assert_eq!(nh_phase, SeasonalPhase::Rest);
        } else {
            assert_eq!(nh_phase, SeasonalPhase::Active);
        }
        if in_sh_rest {
            assert_eq!(sh_phase, SeasonalPhase::Rest);
        } else {
            assert_eq!(sh_phase, SeasonalPhase::Active);
        }
    }

    // ── effective_water_frequency tests ──────────────────────────────

    #[test]
    fn test_effective_water_frequency_no_seasonal_data() {
        let orchid = seasonal_orchid(7, None, None, None, None, None, None, None);
        // No seasonal data → base frequency unchanged
        assert_eq!(orchid.effective_water_frequency(&Hemisphere::Northern), 7);
    }

    #[test]
    fn test_effective_water_frequency_no_multiplier() {
        // Has rest data but no multiplier → base frequency unchanged
        let orchid = seasonal_orchid(7, None, Some((1, 12)), None, None, None, None, None);
        assert_eq!(orchid.effective_water_frequency(&Hemisphere::Northern), 7);
    }

    #[test]
    fn test_effective_water_frequency_rest_multiplier() {
        // Rest covering all months, rest_water_multiplier = 0.5
        // With base 10 days and 0.5 multiplier: 10 / 0.5 = 20 days (less frequent)
        let orchid = seasonal_orchid(10, None, Some((1, 12)), None, Some(0.5), None, None, None);
        assert_eq!(orchid.effective_water_frequency(&Hemisphere::Northern), 20);
    }

    #[test]
    fn test_effective_water_frequency_rest_multiplier_very_low() {
        // rest_water_multiplier = 0.1 → 7 / 0.1 = 70 days
        let orchid = seasonal_orchid(7, None, Some((1, 12)), None, Some(0.1), None, None, None);
        assert_eq!(orchid.effective_water_frequency(&Hemisphere::Northern), 70);
    }

    #[test]
    fn test_effective_water_frequency_active_multiplier() {
        // No rest, but has seasonal data (bloom covers all months, treated as Blooming)
        // active_water_multiplier = 1.5 → 10 / 1.5 ≈ 7
        let orchid = seasonal_orchid(10, None, None, Some((1, 12)), None, None, Some(1.5), None);
        assert_eq!(orchid.effective_water_frequency(&Hemisphere::Northern), 7);
    }

    #[test]
    fn test_effective_water_frequency_minimum_one_day() {
        // Even with very high multiplier, should never go below 1
        let orchid = seasonal_orchid(1, None, None, Some((1, 12)), None, None, Some(100.0), None);
        assert_eq!(orchid.effective_water_frequency(&Hemisphere::Northern), 1);
    }

    // ── effective_fertilize_frequency tests ──────────────────────────

    #[test]
    fn test_effective_fertilize_frequency_no_schedule() {
        let orchid = seasonal_orchid(7, None, None, None, None, None, None, None);
        assert_eq!(orchid.effective_fertilize_frequency(&Hemisphere::Northern), None);
    }

    #[test]
    fn test_effective_fertilize_frequency_no_multiplier() {
        let orchid = seasonal_orchid(7, Some(14), Some((1, 12)), None, None, None, None, None);
        // Has rest period but no fertilizer multiplier → base unchanged
        assert_eq!(orchid.effective_fertilize_frequency(&Hemisphere::Northern), Some(14));
    }

    #[test]
    fn test_effective_fertilize_frequency_rest_stop() {
        // rest_fertilizer_multiplier = 0.0 → should still return base (division by 0 guarded)
        let orchid = seasonal_orchid(7, Some(14), Some((1, 12)), None, None, Some(0.0), None, None);
        // multiplier 0.0 is not > 0.0, so falls through to base
        assert_eq!(orchid.effective_fertilize_frequency(&Hemisphere::Northern), Some(14));
    }

    #[test]
    fn test_effective_fertilize_frequency_rest_reduced() {
        // rest_fertilizer_multiplier = 0.25 → 14 / 0.25 = 56 days (much less frequent)
        let orchid = seasonal_orchid(7, Some(14), Some((1, 12)), None, None, Some(0.25), None, None);
        assert_eq!(orchid.effective_fertilize_frequency(&Hemisphere::Northern), Some(56));
    }

    // ── next_transition tests ────────────────────────────────────────

    #[test]
    fn test_next_transition_no_seasonal_data() {
        let orchid = seasonal_orchid(7, None, None, None, None, None, None, None);
        assert_eq!(orchid.next_transition(&Hemisphere::Northern), None);
    }

    #[test]
    fn test_next_transition_returns_something_with_rest_data() {
        let orchid = seasonal_orchid(7, None, Some((11, 2)), None, None, None, None, None);
        let transition = orchid.next_transition(&Hemisphere::Northern);
        // Should return Some with a month (1-12) and a label
        assert!(transition.is_some());
        let (month, label) = transition.unwrap();
        assert!(month >= 1 && month <= 12);
        assert!(!label.is_empty());
    }

    #[test]
    fn test_next_transition_includes_bloom_transitions() {
        let orchid = seasonal_orchid(7, None, None, Some((3, 5)), None, None, None, None);
        let transition = orchid.next_transition(&Hemisphere::Northern);
        assert!(transition.is_some());
        let (_, label) = transition.unwrap();
        assert!(label.contains("Bloom"), "Label should mention bloom: {}", label);
    }

    #[test]
    fn test_next_transition_with_both_rest_and_bloom() {
        // Both rest (Nov-Feb) and bloom (Mar-May)
        let orchid = seasonal_orchid(7, None, Some((11, 2)), Some((3, 5)), None, None, None, None);
        let transition = orchid.next_transition(&Hemisphere::Northern);
        // Should return the nearest future transition
        assert!(transition.is_some());
    }

    // ── Orchid::month_name tests ─────────────────────────────────────

    #[test]
    fn test_month_name_all_valid() {
        let names = ["Jan", "Feb", "Mar", "Apr", "May", "Jun",
                     "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
        for (i, expected) in names.iter().enumerate() {
            assert_eq!(Orchid::month_name((i + 1) as u32), *expected);
        }
    }

    #[test]
    fn test_month_name_invalid() {
        assert_eq!(Orchid::month_name(0), "???");
        assert_eq!(Orchid::month_name(13), "???");
    }

    // ── Seasonal serde round-trip ────────────────────────────────────

    #[test]
    fn test_seasonal_fields_serde_roundtrip() {
        let orchid = seasonal_orchid(
            7, Some(14),
            Some((11, 2)), Some((3, 5)),
            Some(0.3), Some(0.0),
            Some(1.0), Some(1.5),
        );

        let json = serde_json::to_string(&orchid).unwrap();
        let deserialized: Orchid = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.rest_start_month, Some(11));
        assert_eq!(deserialized.rest_end_month, Some(2));
        assert_eq!(deserialized.bloom_start_month, Some(3));
        assert_eq!(deserialized.bloom_end_month, Some(5));
        assert_eq!(deserialized.rest_water_multiplier, Some(0.3));
        assert_eq!(deserialized.rest_fertilizer_multiplier, Some(0.0));
        assert_eq!(deserialized.active_water_multiplier, Some(1.0));
        assert_eq!(deserialized.active_fertilizer_multiplier, Some(1.5));
    }

    #[test]
    fn test_seasonal_fields_backward_compat() {
        // JSON without any seasonal fields should deserialize with all None
        let json = r#"{"id":"orchid:old","name":"Old","species":"Phal","water_frequency_days":7,"light_requirement":"Medium","notes":"","placement":"Zone A","light_lux":"","temperature_range":""}"#;
        let orchid: Orchid = serde_json::from_str(json).unwrap();

        assert_eq!(orchid.rest_start_month, None);
        assert_eq!(orchid.rest_end_month, None);
        assert_eq!(orchid.bloom_start_month, None);
        assert_eq!(orchid.bloom_end_month, None);
        assert_eq!(orchid.rest_water_multiplier, None);
        assert_eq!(orchid.rest_fertilizer_multiplier, None);
        assert_eq!(orchid.active_water_multiplier, None);
        assert_eq!(orchid.active_fertilizer_multiplier, None);
    }

    // ── Hemisphere serde round-trip ──────────────────────────────────

    #[test]
    fn test_hemisphere_serde_roundtrip() {
        let hemi = Hemisphere::Southern;
        let json = serde_json::to_string(&hemi).unwrap();
        let deserialized: Hemisphere = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, Hemisphere::Southern);
    }

    #[test]
    fn test_seasonal_phase_serde_roundtrip() {
        for phase in [SeasonalPhase::Rest, SeasonalPhase::Active, SeasonalPhase::Blooming, SeasonalPhase::Unknown] {
            let json = serde_json::to_string(&phase).unwrap();
            let deserialized: SeasonalPhase = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, phase);
        }
    }
}
