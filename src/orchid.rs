use serde::{Deserialize, Serialize};
use std::fmt;
use chrono::{DateTime, Utc};

#[cfg(feature = "ssr")]
use surrealdb::types::SurrealValue;

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
#[cfg_attr(feature = "ssr", surreal(crate = "surrealdb::types", untagged))]
pub enum Placement {
    Low,
    Medium,
    High,
    Patio,
    OutdoorRack,
}

impl fmt::Display for Placement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Placement::Low => write!(f, "Low Light Area"),
            Placement::Medium => write!(f, "Medium Light Area"),
            Placement::High => write!(f, "High Light Area"),
            Placement::Patio => write!(f, "Patio (Outdoors)"),
            Placement::OutdoorRack => write!(f, "Outdoor Rack"),
        }
    }
}

impl Placement {
    pub fn is_compatible_with(&self, req: &LightRequirement) -> bool {
        matches!(
            (self, req),
            (Placement::Low, LightRequirement::Low)
                | (Placement::Medium, LightRequirement::Medium)
                | (Placement::High, LightRequirement::High)
                | (Placement::Patio, LightRequirement::Medium)
                | (Placement::Patio, LightRequirement::High)
                | (Placement::OutdoorRack, LightRequirement::High)
        )
    }
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
    pub placement: Placement,
    pub light_lux: String,
    pub temperature_range: String,
    #[serde(default)]
    #[cfg_attr(feature = "ssr", surreal(default))]
    pub conservation_status: Option<String>,
    #[serde(default)]
    #[cfg_attr(feature = "ssr", surreal(default))]
    pub history: Vec<LogEntry>,
}

impl Orchid {
    pub fn suggested_placement(&self) -> Placement {
        match self.light_requirement {
            LightRequirement::Low => Placement::Low,
            LightRequirement::Medium => Placement::Medium,
            LightRequirement::High => Placement::High,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_placement_compatibility() {
        assert!(Placement::Low.is_compatible_with(&LightRequirement::Low));
        assert!(!Placement::Low.is_compatible_with(&LightRequirement::High));

        assert!(Placement::Patio.is_compatible_with(&LightRequirement::Medium));
        assert!(Placement::Patio.is_compatible_with(&LightRequirement::High));
        assert!(!Placement::Patio.is_compatible_with(&LightRequirement::Low));

        assert!(Placement::OutdoorRack.is_compatible_with(&LightRequirement::High));
        assert!(!Placement::OutdoorRack.is_compatible_with(&LightRequirement::Low));
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
            placement: Placement::Medium,
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
    fn test_suggested_placement() {
        let orchid = Orchid {
            id: "test:1".into(),
            name: "Test".into(),
            species: "Test".into(),
            water_frequency_days: 7,
            light_requirement: LightRequirement::High,
            notes: String::new(),
            placement: Placement::Low,
            light_lux: String::new(),
            temperature_range: String::new(),
            conservation_status: None,
            history: Vec::new(),
        };
        assert_eq!(orchid.suggested_placement(), Placement::High);
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
