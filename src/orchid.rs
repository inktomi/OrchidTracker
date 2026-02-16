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
}

impl fmt::Display for Placement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Placement::Low => write!(f, "Low Light Area"),
            Placement::Medium => write!(f, "Medium Light Area"),
            Placement::High => write!(f, "High Light Area"),
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
            id: js_sys::Date::now() as u64,
            timestamp: Utc::now(),
            note,
            image_data,
        };
        self.history.push(entry);
    }
}
