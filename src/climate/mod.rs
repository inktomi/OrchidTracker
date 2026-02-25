//! **What is it?**
//! The root module for climate data ingestion, processing, and alerts in OrchidTracker.
//!
//! **Why does it exist?**
//! It exists to unify various environmental data sources (sensors, APIs) and run background tasks to monitor habitat health.
//!
//! **How should it be used?**
//! Import specific submodules to configure polling jobs or use the calculation functions to derive climate metrics.

/// **What is it?**
/// A module for Tempest weather station integration.
///
/// **Why does it exist?**
/// It exists to fetch real-time and historical weather data from a local or remote Tempest device.
///
/// **How should it be used?**
/// Use the functions in this module to schedule updates or query specific metrics from a configured Tempest station.
pub mod tempest;
/// **What is it?**
/// A module for AC Infinity environment controller integration.
///
/// **Why does it exist?**
/// It exists to interface with AC Infinity hardware, reading sensor data and managing environmental controls like fans.
///
/// **How should it be used?**
/// Call these functions to parse device states or poll the AC Infinity API for indoor climate data.
pub mod ac_infinity;
/// **What is it?**
/// A module containing periodic climate polling tasks.
///
/// **Why does it exist?**
/// It exists to run background loops that continuously gather data from registered sensors and external APIs.
///
/// **How should it be used?**
/// Spawn these polling tasks during server initialization to keep the system's climate data up to date.
pub mod poller;
/// **What is it?**
/// A module for Open-Meteo API integration.
///
/// **Why does it exist?**
/// It exists to provide free, local weather forecasts and historical climate data without needing a physical sensor.
///
/// **How should it be used?**
/// Call its functions to fetch temperature, precipitation, or other metrics based on geographical coordinates.
pub mod open_meteo;
/// **What is it?**
/// A module containing habitat weather polling tasks.
///
/// **Why does it exist?**
/// It exists to orchestrate the retrieval of weather data specifically tailored to user-defined outdoor habitats.
///
/// **How should it be used?**
/// Run these tasks in the background to periodically update the environmental conditions of outdoor orchid zones.
pub mod habitat_poller;
/// **What is it?**
/// A module for climate alerts checking and management.
///
/// **Why does it exist?**
/// It exists to analyze incoming climate data against user-defined thresholds and trigger warnings if conditions become dangerous.
///
/// **How should it be used?**
/// Hook these routines into the data ingestion pipeline to automatically generate notifications for out-of-bounds readings.
pub mod alerts;
/// **What is it?**
/// A module for seasonal alerts checking and management.
///
/// **Why does it exist?**
/// It exists to proactively warn users about impending seasonal changes, such as the first frost or extreme summer heat.
///
/// **How should it be used?**
/// Run these checks periodically using forecast data to alert users days in advance of significant seasonal shifts.
pub mod seasonal_alerts;

/// **What is it?**
/// A structure representing a raw climate reading from any data source, before storage.
///
/// **Why does it exist?**
/// It exists to provide a common, standardized format for environmental data regardless of whether it came from a sensor or an API.
///
/// **How should it be used?**
/// Instantiate this struct when parsing incoming data, then pass it to database functions to persist the reading.
pub struct RawReading {
    /// Temperature in Celsius.
    pub temperature_c: f64,
    /// Relative humidity percentage.
    pub humidity_pct: f64,
    /// Vapor Pressure Deficit in kilopascals.
    pub vpd_kpa: Option<f64>,
    /// Precipitation in millimeters.
    pub precipitation_mm: Option<f64>,
}

/// **What is it?**
/// A function that calculates VPD (Vapor Pressure Deficit) from temperature and humidity.
///
/// **Why does it exist?**
/// It exists because VPD is a crucial metric for plant transpiration, but sensors often only report raw temperature and humidity.
///
/// **How should it be used?**
/// Call this function with a temperature in Celsius and a relative humidity percentage to compute the VPD in kilopascals using the August-Roche-Magnus formula.
pub fn calculate_vpd(temp_c: f64, humidity_pct: f64) -> f64 {
    let saturation_pressure = 0.6108 * ((17.27 * temp_c) / (temp_c + 237.3)).exp();
    let actual_pressure = saturation_pressure * (humidity_pct / 100.0);
    saturation_pressure - actual_pressure
}
