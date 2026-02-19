pub mod tempest;
pub mod ac_infinity;
pub mod poller;
pub mod open_meteo;
pub mod habitat_poller;
pub mod alerts;

/// A raw climate reading from any data source, before storage.
pub struct RawReading {
    pub temperature_c: f64,
    pub humidity_pct: f64,
    pub vpd_kpa: Option<f64>,
}

/// Calculate VPD (Vapor Pressure Deficit) from temperature and humidity
/// using the August-Roche-Magnus formula.
pub fn calculate_vpd(temp_c: f64, humidity_pct: f64) -> f64 {
    let saturation_pressure = 0.6108 * ((17.27 * temp_c) / (temp_c + 237.3)).exp();
    let actual_pressure = saturation_pressure * (humidity_pct / 100.0);
    saturation_pressure - actual_pressure
}
