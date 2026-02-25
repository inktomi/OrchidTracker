/// Tempest weather station integration.
pub mod tempest;
/// AC Infinity environment controller integration.
pub mod ac_infinity;
/// Periodic climate polling tasks.
pub mod poller;
/// Open-Meteo API integration.
pub mod open_meteo;
/// Habitat weather polling tasks.
pub mod habitat_poller;
/// Climate alerts checking and management.
pub mod alerts;
/// Seasonal alerts checking and management.
pub mod seasonal_alerts;

/// A raw climate reading from any data source, before storage.
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

/// Calculate VPD (Vapor Pressure Deficit) from temperature and humidity
/// using the August-Roche-Magnus formula.
pub fn calculate_vpd(temp_c: f64, humidity_pct: f64) -> f64 {
    let saturation_pressure = 0.6108 * ((17.27 * temp_c) / (temp_c + 237.3)).exp();
    let actual_pressure = saturation_pressure * (humidity_pct / 100.0);
    saturation_pressure - actual_pressure
}
