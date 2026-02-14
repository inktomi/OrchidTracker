use serde::{Deserialize, Serialize};
use std::fmt;

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
        }
    }
    
    pub fn suggested_placement(&self) -> Placement {
        match self.light_requirement {
            LightRequirement::Low => Placement::Low,
            LightRequirement::Medium => Placement::Medium,
            LightRequirement::High => Placement::High,
        }
    }
}
