use serde::{Deserialize, Serialize};
use std::fmt;
use chrono::{DateTime, Utc};

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
    }
}
