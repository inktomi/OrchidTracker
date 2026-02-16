use serde::{Deserialize, Serialize};
use std::fmt;
use chrono::{DateTime, Utc};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum LightRequirement {
    Low,
    Medium,
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
    pub image_data: Option<String>, // Base64 encoded image or URL (simplest for now, though limiting)
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
    pub fn new(
        id: u64,
        name: String,
        species: String,
        water_frequency_days: u32,
        light_requirement: LightRequirement,
        notes: String,
        placement: Placement,
        light_lux: String,
        temperature_range: String,
        conservation_status: Option<String>,
    ) -> Self {
        Orchid {
            id,
            name,
            species,
            water_frequency_days,
            light_requirement,
            notes,
            placement,
            light_lux,
            temperature_range,
            conservation_status,
            history: Vec::new(),
        }
    }
    
    pub fn suggested_placement(&self) -> Placement {
        match self.light_requirement {
            LightRequirement::Low => Placement::Low,
            LightRequirement::Medium => Placement::Medium,
            LightRequirement::High => Placement::High,
        }
    }

    pub fn add_log(&mut self, note: String, image_data: Option<String>) {
        let entry = LogEntry {
            id: Utc::now().timestamp_millis() as u64,
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
        let orchid = Orchid::new(
            1,
            "Test Orchid".to_string(),
            "Phalaenopsis".to_string(),
            7,
            LightRequirement::Medium,
            "Notes".to_string(),
            Placement::Medium,
            "1000".to_string(),
            "20-30C".to_string(),
            Some("CITES II".to_string()),
        );

        assert_eq!(orchid.name, "Test Orchid");
        assert_eq!(orchid.light_requirement, LightRequirement::Medium);
        assert_eq!(orchid.history.len(), 0);
        assert_eq!(orchid.conservation_status, Some("CITES II".to_string()));
    }

    #[test]
    fn test_add_log() {
        let mut orchid = Orchid::new(
            1, "Test".to_string(), "Test".to_string(), 7, 
            LightRequirement::High, "".to_string(), Placement::Low, 
            "".to_string(), "".to_string(), None
        );
        
        orchid.add_log("Watered".to_string(), None);
        assert_eq!(orchid.history.len(), 1);
        assert_eq!(orchid.history[0].note, "Watered");
        assert!(orchid.history[0].image_data.is_none());
        
        orchid.add_log("Photo".to_string(), Some("img.jpg".to_string()));
        assert_eq!(orchid.history.len(), 2);
        assert_eq!(orchid.history[1].image_data, Some("img.jpg".to_string()));
    }
    
    #[test]
    fn test_suggested_placement() {
        let orchid = Orchid::new(
            1, "Test".to_string(), "Test".to_string(), 7, 
            LightRequirement::High, "".to_string(), Placement::Low, 
            "".to_string(), "".to_string(), None
        );
        assert_eq!(orchid.suggested_placement(), Placement::High);
    }
}
