use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use chrono::{DateTime, Utc};

static ID_COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn generate_id() -> u64 {
    let ts = js_sys::Date::now() as u64;
    let seq = ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    ts * 1000 + (seq % 1000)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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
        match (self, req) {
            (Placement::Low, LightRequirement::Low) => true,
            (Placement::Medium, LightRequirement::Medium) => true,
            (Placement::High, LightRequirement::High) => true,
            (Placement::Patio, LightRequirement::Medium) | (Placement::Patio, LightRequirement::High) => true,
            (Placement::OutdoorRack, LightRequirement::High) => true,
            _ => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: u64,
    pub timestamp: DateTime<Utc>,
    pub note: String,
    pub image_data: Option<String>, // IndexedDB ID (numeric) or LFS filename
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Orchid {
    pub id: u64,
    pub name: String,
    pub species: String,
    pub water_frequency_days: u32,
    pub light_requirement: LightRequirement,
    pub notes: String,
    pub placement: Placement,
    pub light_lux: String,
    pub temperature_range: String,
    #[serde(default)]
    pub conservation_status: Option<String>,
    #[serde(default)]
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

    pub fn add_log(&mut self, note: String, image_data: Option<String>) {
        let entry = LogEntry {
            id: generate_id(),
            timestamp: Utc::now(),
            note,
            image_data,
        };
        self.history.push(entry);
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
            id: 1,
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
            id: 1,
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
