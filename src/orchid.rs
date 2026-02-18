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
    #[cfg_attr(feature = "ssr", surreal(default))]
    pub image_filename: Option<String>,
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
    pub history: Vec<LogEntry>,
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
            history: Vec::new(),
        };

        assert_eq!(orchid.name, "Test Orchid");
        assert_eq!(orchid.light_requirement, LightRequirement::Medium);
        assert_eq!(orchid.history.len(), 0);
        assert_eq!(orchid.conservation_status, Some("CITES II".into()));
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
}
