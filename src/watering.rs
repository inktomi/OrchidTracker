//! Climate-aware dynamic watering algorithm.
//!
//! Uses VPD (Vapor Pressure Deficit) as the primary driver to adjust the
//! static watering interval based on actual environmental conditions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::orchid::{ClimateReading, LightRequirement};

// ── Reference Conditions ────────────────────────────────────────────
// What `water_frequency_days` assumes: standard indoor environment.

/// The baseline temperature in Celsius assumed by default watering frequencies.
pub const REFERENCE_TEMP_C: f64 = 22.0;
/// The baseline relative humidity percentage assumed by default watering frequencies.
pub const REFERENCE_HUMIDITY_PCT: f64 = 55.0;
/// Reference VPD calculated from 22°C / 55% RH ≈ 1.19 kPa
pub const REFERENCE_VPD_KPA: f64 = 1.19;

// ── Types ───────────────────────────────────────────────────────────

/// Quality of the climate data used for adjustment.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DataQuality {
    /// Most recent reading < 6 hours old.
    Fresh,
    /// Most recent reading 6–48 hours old.
    Stale,
    /// No readings or readings > 48 hours old.
    Unavailable,
}

/// A snapshot of recent climate conditions for a zone.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ClimateSnapshot {
    /// The name of the growing zone.
    pub zone_name: String,
    /// The average temperature in Celsius.
    pub avg_temp_c: f64,
    /// The average relative humidity percentage.
    pub avg_humidity_pct: f64,
    /// The average vapor pressure deficit in kilopascals.
    pub avg_vpd_kpa: f64,
    /// Total precipitation in the last 48 hours (mm). None if indoor or no data.
    pub precipitation_48h_mm: Option<f64>,
    /// Timestamp of the most recent reading included in this snapshot.
    pub newest_reading_at: DateTime<Utc>,
    /// Number of readings averaged into this snapshot.
    pub reading_count: usize,
    /// The overall reliability and recency of the data.
    pub quality: DataQuality,
    /// True if the zone represents an outdoor location.
    pub is_outdoor: bool,
}

/// Result of the climate-adjusted watering calculation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WateringEstimate {
    /// The climate-adjusted watering interval in days.
    pub adjusted_days: u32,
    /// The base (seasonal) interval that was adjusted.
    pub base_days: u32,
    /// Quality of climate data used.
    pub quality: DataQuality,
    /// Whether climate adjustment was actually applied (vs. fallback).
    pub climate_active: bool,
    /// Individual factor values for UI display.
    pub factors: Option<FactorBreakdown>,
}

/// Breakdown of individual adjustment factors.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FactorBreakdown {
    /// Multiplier derived from Vapor Pressure Deficit.
    pub vpd_factor: f64,
    /// Multiplier derived from low temperatures.
    pub cold_stress_factor: f64,
    /// Multiplier based on potting medium water retention.
    pub medium_factor: f64,
    /// Multiplier based on the plant's light requirements.
    pub light_factor: f64,
    /// Multiplier based on recent outdoor precipitation.
    pub rain_factor: f64,
}

// ── Factor Functions ────────────────────────────────────────────────

/// VPD factor: primary driver of evaporative demand.
/// High VPD (hot/dry) → factor < 1.0 → water sooner.
/// Low VPD (cool/humid) → factor > 1.0 → water later.
pub fn vpd_factor(avg_vpd_kpa: f64) -> f64 {
    if avg_vpd_kpa <= 0.0 {
        return 2.5; // Near-zero VPD: extremely humid, max extension
    }
    (REFERENCE_VPD_KPA / avg_vpd_kpa).clamp(0.4, 2.5)
}

/// Cold stress factor: plants slow water uptake in cold conditions.
/// ≥ 18°C → 1.0, 10–18°C → linear 1.0–1.8, ≤ 10°C → 1.8
pub fn cold_stress_factor(avg_temp_c: f64) -> f64 {
    if avg_temp_c >= 18.0 {
        1.0
    } else if avg_temp_c <= 10.0 {
        1.8
    } else {
        // Linear interpolation: 18°C→1.0, 10°C→1.8
        1.0 + (18.0 - avg_temp_c) * (0.8 / 8.0)
    }
}

/// Medium factor: substrate water retention rates.
pub fn medium_factor(pot_medium: Option<&crate::orchid::PotMedium>) -> f64 {
    match pot_medium {
        Some(m) => match m {
            crate::orchid::PotMedium::Bark => 0.85,
            crate::orchid::PotMedium::SphagnumMoss => 1.3,
            crate::orchid::PotMedium::Leca => 1.4,
            crate::orchid::PotMedium::Inorganic => 1.0, // Baseline for inorganic mix
            crate::orchid::PotMedium::Unknown => 1.0,
        },
        None => 1.0,
    }
}

/// Light factor: higher light drives more transpiration.
pub fn light_factor(light_req: &LightRequirement) -> f64 {
    match light_req {
        LightRequirement::High => 0.85,
        LightRequirement::Medium => 1.0,
        LightRequirement::Low => 1.15,
    }
}

/// Piecewise linear interpolation with clamping at extremes.
/// `points` must be sorted by x in ascending order with at least 2 entries.
pub fn piecewise_linear(x: f64, points: &[(f64, f64)]) -> f64 {
    if x <= points[0].0 {
        return points[0].1;
    }
    let last = points.len() - 1;
    if x >= points[last].0 {
        return points[last].1;
    }
    for window in points.windows(2) {
        let (x0, y0) = window[0];
        let (x1, y1) = window[1];
        if x >= x0 && x <= x1 {
            let t = (x - x0) / (x1 - x0);
            return y0 + t * (y1 - y0);
        }
    }
    points[last].1
}

/// Light factor from measured PAR (PPFD, µmol/m²/s).
/// More light → lower factor → water sooner (inverse relationship).
pub fn light_factor_par(ppfd: f64) -> f64 {
    const POINTS: &[(f64, f64)] = &[
        (50.0, 1.20),
        (100.0, 1.10),
        (200.0, 1.00),
        (400.0, 0.85),
        (800.0, 0.70),
    ];
    piecewise_linear(ppfd, POINTS)
}

/// Rain factor: recent precipitation reduces watering need (outdoor only).
/// For indoor zones, always returns 1.0.
pub fn rain_factor(precipitation_48h_mm: Option<f64>, is_outdoor: bool) -> f64 {
    if !is_outdoor {
        return 1.0;
    }
    match precipitation_48h_mm {
        Some(mm) if mm > 30.0 => 2.5,
        Some(mm) if mm > 15.0 => 2.0,
        Some(mm) if mm > 5.0 => 1.6,
        Some(mm) if mm > 1.0 => 1.3,
        _ => 1.0,
    }
}

// ── Main Algorithm ──────────────────────────────────────────────────

/// Compute the climate-adjusted watering frequency.
///
/// `base_days` should already include seasonal adjustment (from
/// `effective_water_frequency()`). When `climate` is `None` or data
/// quality is `Unavailable`, falls back to the base value.
pub fn climate_adjusted_frequency(
    base_days: u32,
    climate: Option<&ClimateSnapshot>,
    pot_medium: Option<&crate::orchid::PotMedium>,
    light_req: &LightRequirement,
    par_ppfd: Option<f64>,
) -> WateringEstimate {
    let Some(snapshot) = climate else {
        return WateringEstimate {
            adjusted_days: base_days,
            base_days,
            quality: DataQuality::Unavailable,
            climate_active: false,
            factors: None,
        };
    };

    if snapshot.quality == DataQuality::Unavailable {
        return WateringEstimate {
            adjusted_days: base_days,
            base_days,
            quality: DataQuality::Unavailable,
            climate_active: false,
            factors: None,
        };
    }

    let vf = vpd_factor(snapshot.avg_vpd_kpa);
    let csf = cold_stress_factor(snapshot.avg_temp_c);
    let mf = medium_factor(pot_medium);
    let lf = match par_ppfd {
        Some(ppfd) => light_factor_par(ppfd),
        None => light_factor(light_req),
    };
    let rf = rain_factor(snapshot.precipitation_48h_mm, snapshot.is_outdoor);

    let combined = base_days as f64 * vf * csf * mf * lf * rf;
    let max_days = base_days * 3;
    let adjusted = (combined.round() as u32).clamp(1, max_days);

    WateringEstimate {
        adjusted_days: adjusted,
        base_days,
        quality: snapshot.quality.clone(),
        climate_active: true,
        factors: Some(FactorBreakdown {
            vpd_factor: vf,
            cold_stress_factor: csf,
            medium_factor: mf,
            light_factor: lf,
            rain_factor: rf,
        }),
    }
}

// ── VPD Calculation ──────────────────────────────────────────────────

/// Calculate VPD (Vapor Pressure Deficit) from temperature and humidity
/// using the August-Roche-Magnus formula. Duplicated from climate::calculate_vpd
/// to keep watering.rs usable without the `ssr` feature.
pub fn calculate_vpd(temp_c: f64, humidity_pct: f64) -> f64 {
    let saturation_pressure = 0.6108 * ((17.27 * temp_c) / (temp_c + 237.3)).exp();
    let actual_pressure = saturation_pressure * (humidity_pct / 100.0);
    saturation_pressure - actual_pressure
}

// ── ClimateSnapshot Builder ─────────────────────────────────────────

impl ClimateSnapshot {
    /// Build a snapshot from a slice of recent readings for a zone.
    ///
    /// Computes averages for temperature, humidity, and VPD, sums precipitation,
    /// and determines data quality from the most recent reading timestamp.
    pub fn from_readings(
        zone_name: &str,
        readings: &[ClimateReading],
        is_outdoor: bool,
    ) -> Option<Self> {
        if readings.is_empty() {
            return None;
        }

        let newest = readings
            .iter()
            .map(|r| r.recorded_at)
            .max()
            .unwrap_or_else(Utc::now);

        let quality = data_quality_from_age(newest);

        let count = readings.len() as f64;
        let avg_temp = readings.iter().map(|r| r.temperature).sum::<f64>() / count;
        let avg_hum = readings.iter().map(|r| r.humidity).sum::<f64>() / count;
        let avg_vpd = if readings.iter().any(|r| r.vpd.is_some()) {
            let vpd_readings: Vec<f64> = readings.iter().filter_map(|r| r.vpd).collect();
            if vpd_readings.is_empty() {
                calculate_vpd(avg_temp, avg_hum)
            } else {
                vpd_readings.iter().sum::<f64>() / vpd_readings.len() as f64
            }
        } else {
            calculate_vpd(avg_temp, avg_hum)
        };

        let precip_sum: Option<f64> = if is_outdoor {
            let precip_values: Vec<f64> = readings.iter().filter_map(|r| r.precipitation).collect();
            if precip_values.is_empty() {
                None
            } else {
                Some(precip_values.iter().sum())
            }
        } else {
            None
        };

        Some(ClimateSnapshot {
            zone_name: zone_name.to_string(),
            avg_temp_c: avg_temp,
            avg_humidity_pct: avg_hum,
            avg_vpd_kpa: avg_vpd,
            precipitation_48h_mm: precip_sum,
            newest_reading_at: newest,
            reading_count: readings.len(),
            quality,
            is_outdoor,
        })
    }
}

/// Determine data quality from the age of the newest reading.
fn data_quality_from_age(newest: DateTime<Utc>) -> DataQuality {
    let age_hours = (Utc::now() - newest).num_hours();
    if age_hours < 6 {
        DataQuality::Fresh
    } else if age_hours <= 48 {
        DataQuality::Stale
    } else {
        DataQuality::Unavailable
    }
}

// ── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod tests {
    use super::*;
    use crate::orchid::LightRequirement;

    // ── vpd_factor tests ────────────────────────────────────────────

    #[test]
    fn test_vpd_factor_at_reference() {
        let f = vpd_factor(REFERENCE_VPD_KPA);
        assert!(
            (f - 1.0).abs() < 0.01,
            "At reference VPD, factor should be ~1.0, got {f}"
        );
    }

    #[test]
    fn test_vpd_factor_high_vpd_hot_dry() {
        // VPD = 2.0 kPa (hot, dry) → factor < 1.0 (water sooner)
        let f = vpd_factor(2.0);
        assert!(f < 1.0, "High VPD should yield factor < 1.0, got {f}");
        // 1.19 / 2.0 = 0.595
        assert!((f - 0.595).abs() < 0.01);
    }

    #[test]
    fn test_vpd_factor_low_vpd_cool_humid() {
        // VPD = 0.3 kPa (cool, humid) → factor > 1.0 (water later)
        let f = vpd_factor(0.3);
        assert!(f > 1.0, "Low VPD should yield factor > 1.0, got {f}");
    }

    #[test]
    fn test_vpd_factor_zero_clamped() {
        let f = vpd_factor(0.0);
        assert!(
            (f - 2.5).abs() < 0.01,
            "Zero VPD should clamp to 2.5, got {f}"
        );
    }

    #[test]
    fn test_vpd_factor_negative_clamped() {
        let f = vpd_factor(-1.0);
        assert!(
            (f - 2.5).abs() < 0.01,
            "Negative VPD should clamp to 2.5, got {f}"
        );
    }

    #[test]
    fn test_vpd_factor_very_high_clamped() {
        // VPD = 5.0 → 0.95/5.0 = 0.19, should clamp at 0.4
        let f = vpd_factor(5.0);
        assert!(
            (f - 0.4).abs() < 0.01,
            "Very high VPD should clamp at 0.4, got {f}"
        );
    }

    #[test]
    fn test_vpd_factor_very_low_clamped() {
        // VPD = 0.1 → 0.95/0.1 = 9.5, should clamp at 2.5
        let f = vpd_factor(0.1);
        assert!(
            (f - 2.5).abs() < 0.01,
            "Very low VPD should clamp at 2.5, got {f}"
        );
    }

    // ── cold_stress_factor tests ────────────────────────────────────

    #[test]
    fn test_cold_stress_warm() {
        assert!((cold_stress_factor(25.0) - 1.0).abs() < 0.01);
        assert!((cold_stress_factor(18.0) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_cold_stress_cold() {
        assert!((cold_stress_factor(10.0) - 1.8).abs() < 0.01);
        assert!((cold_stress_factor(5.0) - 1.8).abs() < 0.01);
    }

    #[test]
    fn test_cold_stress_midrange() {
        // 14°C is midpoint between 10 and 18 → factor = 1.4
        let f = cold_stress_factor(14.0);
        assert!(
            (f - 1.4).abs() < 0.01,
            "14°C should give factor ~1.4, got {f}"
        );
    }

    #[test]
    fn test_cold_stress_boundary_values() {
        // Just below 18°C
        let f = cold_stress_factor(17.9);
        assert!(f > 1.0 && f < 1.05);
        // Just above 10°C
        let f = cold_stress_factor(10.1);
        assert!(f > 1.75 && f < 1.8);
    }

    // ── medium_factor tests ─────────────────────────────────────────

    #[test]
    fn test_medium_factor_mounted() {
        assert!((medium_factor(Some(&crate::orchid::PotMedium::Unknown)) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_medium_factor_bark() {
        assert!((medium_factor(Some(&crate::orchid::PotMedium::Bark)) - 0.85).abs() < 0.01);
    }

    #[test]
    fn test_medium_factor_sphagnum() {
        assert!((medium_factor(Some(&crate::orchid::PotMedium::SphagnumMoss)) - 1.3).abs() < 0.01);
    }

    #[test]
    fn test_medium_factor_semi_hydro() {
        assert!((medium_factor(Some(&crate::orchid::PotMedium::Leca)) - 1.4).abs() < 0.01);
    }

    #[test]
    fn test_medium_factor_unknown() {
        assert!((medium_factor(Some(&crate::orchid::PotMedium::Inorganic)) - 1.0).abs() < 0.01);
        assert!((medium_factor(None) - 1.0).abs() < 0.01);
    }

    // ── light_factor tests ──────────────────────────────────────────

    #[test]
    fn test_light_factor_all_levels() {
        assert!((light_factor(&LightRequirement::High) - 0.85).abs() < 0.01);
        assert!((light_factor(&LightRequirement::Medium) - 1.0).abs() < 0.01);
        assert!((light_factor(&LightRequirement::Low) - 1.15).abs() < 0.01);
    }

    // ── rain_factor tests ───────────────────────────────────────────

    #[test]
    fn test_rain_factor_indoor_always_one() {
        assert!((rain_factor(Some(50.0), false) - 1.0).abs() < 0.01);
        assert!((rain_factor(None, false) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_rain_factor_outdoor_none() {
        assert!((rain_factor(None, true) - 1.0).abs() < 0.01);
        assert!((rain_factor(Some(0.0), true) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_rain_factor_outdoor_light() {
        assert!((rain_factor(Some(3.0), true) - 1.3).abs() < 0.01);
    }

    #[test]
    fn test_rain_factor_outdoor_moderate() {
        assert!((rain_factor(Some(10.0), true) - 1.6).abs() < 0.01);
    }

    #[test]
    fn test_rain_factor_outdoor_heavy() {
        assert!((rain_factor(Some(20.0), true) - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_rain_factor_outdoor_soaking() {
        assert!((rain_factor(Some(40.0), true) - 2.5).abs() < 0.01);
    }

    #[test]
    fn test_rain_factor_boundary_values() {
        // At exactly 1.0 mm → still 1.0 (must be > 1.0)
        assert!((rain_factor(Some(1.0), true) - 1.0).abs() < 0.01);
        // Just above 1.0
        assert!((rain_factor(Some(1.1), true) - 1.3).abs() < 0.01);
        // Exactly 5.0 → still 1.3 (must be > 5.0)
        assert!((rain_factor(Some(5.0), true) - 1.3).abs() < 0.01);
        // Just above 5.0
        assert!((rain_factor(Some(5.1), true) - 1.6).abs() < 0.01);
    }

    // ── climate_adjusted_frequency tests ────────────────────────────

    fn test_snapshot(temp: f64, hum: f64, vpd: f64) -> ClimateSnapshot {
        ClimateSnapshot {
            zone_name: "Test Zone".into(),
            avg_temp_c: temp,
            avg_humidity_pct: hum,
            avg_vpd_kpa: vpd,
            precipitation_48h_mm: None,
            newest_reading_at: Utc::now(),
            reading_count: 10,
            quality: DataQuality::Fresh,
            is_outdoor: false,
        }
    }

    #[test]
    fn test_adjusted_no_climate_data() {
        let est = climate_adjusted_frequency(7, None, None, &LightRequirement::Medium, None);
        assert_eq!(est.adjusted_days, 7);
        assert!(!est.climate_active);
        assert_eq!(est.quality, DataQuality::Unavailable);
        assert!(est.factors.is_none());
    }

    #[test]
    fn test_adjusted_unavailable_quality() {
        let mut snap = test_snapshot(22.0, 55.0, REFERENCE_VPD_KPA);
        snap.quality = DataQuality::Unavailable;
        let est = climate_adjusted_frequency(7, Some(&snap), None, &LightRequirement::Medium, None);
        assert_eq!(est.adjusted_days, 7);
        assert!(!est.climate_active);
    }

    #[test]
    fn test_adjusted_at_reference_conditions() {
        // Reference conditions: all factors ~1.0 → adjusted ≈ base
        let snap = test_snapshot(REFERENCE_TEMP_C, REFERENCE_HUMIDITY_PCT, REFERENCE_VPD_KPA);
        let est = climate_adjusted_frequency(7, Some(&snap), None, &LightRequirement::Medium, None);
        assert!(est.climate_active);
        assert_eq!(
            est.adjusted_days, 7,
            "At reference conditions, adjusted should equal base"
        );
    }

    #[test]
    fn test_adjusted_hot_dry_waters_sooner() {
        // Hot dry: VPD = 2.0 → vpd_factor ≈ 0.475
        let snap = test_snapshot(30.0, 30.0, 2.0);
        let est = climate_adjusted_frequency(10, Some(&snap), None, &LightRequirement::Medium, None);
        assert!(est.climate_active);
        assert!(
            est.adjusted_days < 10,
            "Hot/dry should reduce interval, got {}",
            est.adjusted_days
        );
    }

    #[test]
    fn test_adjusted_cool_humid_waters_later() {
        // Cool humid: VPD = 0.3, temp = 15°C
        let snap = test_snapshot(15.0, 80.0, 0.3);
        let est = climate_adjusted_frequency(7, Some(&snap), None, &LightRequirement::Medium, None);
        assert!(est.climate_active);
        assert!(
            est.adjusted_days > 7,
            "Cool/humid should extend interval, got {}",
            est.adjusted_days
        );
    }

    #[test]
    fn test_adjusted_clamps_minimum_one() {
        // Extreme hot/dry: VPD = 4.0, high light, mounted (bark factor)
        let snap = test_snapshot(35.0, 20.0, 4.0);
        let est = climate_adjusted_frequency(
            2,
            Some(&snap),
            Some(&crate::orchid::PotMedium::Bark),
            &LightRequirement::High,
            None,
        );
        assert!(est.adjusted_days >= 1, "Should never go below 1 day");
    }

    #[test]
    fn test_adjusted_clamps_maximum_three_times_base() {
        // Extreme cool/humid: VPD = 0.05, cold, sphagnum, low light
        let snap = test_snapshot(8.0, 95.0, 0.05);
        let est = climate_adjusted_frequency(
            7,
            Some(&snap),
            Some(&crate::orchid::PotMedium::SphagnumMoss),
            &LightRequirement::Low,
            None,
        );
        assert!(
            est.adjusted_days <= 21,
            "Should not exceed 3x base (21), got {}",
            est.adjusted_days
        );
    }

    #[test]
    fn test_adjusted_rain_extends_outdoor() {
        let mut snap = test_snapshot(22.0, 55.0, REFERENCE_VPD_KPA);
        snap.is_outdoor = true;
        snap.precipitation_48h_mm = Some(25.0);
        let est = climate_adjusted_frequency(7, Some(&snap), None, &LightRequirement::Medium, None);
        assert!(
            est.adjusted_days > 7,
            "Rain should extend outdoor watering, got {}",
            est.adjusted_days
        );
        let factors = est.factors.unwrap();
        assert!((factors.rain_factor - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_adjusted_rain_ignored_indoor() {
        let mut snap = test_snapshot(22.0, 55.0, REFERENCE_VPD_KPA);
        snap.is_outdoor = false;
        snap.precipitation_48h_mm = Some(50.0);
        let est = climate_adjusted_frequency(7, Some(&snap), None, &LightRequirement::Medium, None);
        // Indoor: rain factor is 1.0, so adjusted should be near base
        assert_eq!(est.adjusted_days, 7);
    }

    #[test]
    fn test_adjusted_bark_medium_dries_faster() {
        let snap = test_snapshot(REFERENCE_TEMP_C, REFERENCE_HUMIDITY_PCT, REFERENCE_VPD_KPA);
        let est = climate_adjusted_frequency(
            10,
            Some(&snap),
            Some(&crate::orchid::PotMedium::Bark),
            &LightRequirement::Medium,
            None,
        );
        // bark factor = 0.85 → 10 * 0.85 = 8.5 → 9
        assert!(
            est.adjusted_days < 10,
            "Bark should dry faster, got {}",
            est.adjusted_days
        );
    }

    #[test]
    fn test_adjusted_sphagnum_retains_longer() {
        let snap = test_snapshot(REFERENCE_TEMP_C, REFERENCE_HUMIDITY_PCT, REFERENCE_VPD_KPA);
        let est = climate_adjusted_frequency(
            10,
            Some(&snap),
            Some(&crate::orchid::PotMedium::SphagnumMoss),
            &LightRequirement::Medium,
            None,
        );
        // sphagnum factor = 1.3 → 10 * 1.3 = 13
        assert!(
            est.adjusted_days > 10,
            "Sphagnum should retain longer, got {}",
            est.adjusted_days
        );
    }

    #[test]
    fn test_adjusted_stale_data_still_adjusts() {
        let mut snap = test_snapshot(30.0, 30.0, 2.0);
        snap.quality = DataQuality::Stale;
        let est = climate_adjusted_frequency(10, Some(&snap), None, &LightRequirement::Medium, None);
        assert!(est.climate_active);
        assert_eq!(est.quality, DataQuality::Stale);
    }

    #[test]
    fn test_adjusted_very_low_base_frequency() {
        let snap = test_snapshot(REFERENCE_TEMP_C, REFERENCE_HUMIDITY_PCT, REFERENCE_VPD_KPA);
        let est = climate_adjusted_frequency(1, Some(&snap), None, &LightRequirement::Medium, None);
        assert_eq!(est.adjusted_days, 1);
    }

    #[test]
    fn test_factor_breakdown_populated() {
        let snap = test_snapshot(20.0, 60.0, 0.8);
        let est = climate_adjusted_frequency(
            7,
            Some(&snap),
            Some(&crate::orchid::PotMedium::Bark),
            &LightRequirement::High,
            None,
        );
        assert!(est.factors.is_some());
        let factors = est.factors.unwrap();
        assert!(factors.vpd_factor > 0.0);
        assert!(factors.cold_stress_factor > 0.0);
        assert!(factors.medium_factor > 0.0);
        assert!(factors.light_factor > 0.0);
        assert!((factors.rain_factor - 1.0).abs() < 0.01); // indoor
    }

    // ── data_quality_from_age tests ─────────────────────────────────

    #[test]
    fn test_data_quality_fresh() {
        let recent = Utc::now() - chrono::Duration::hours(2);
        assert_eq!(data_quality_from_age(recent), DataQuality::Fresh);
    }

    #[test]
    fn test_data_quality_stale() {
        let old = Utc::now() - chrono::Duration::hours(12);
        assert_eq!(data_quality_from_age(old), DataQuality::Stale);
    }

    #[test]
    fn test_data_quality_unavailable() {
        let very_old = Utc::now() - chrono::Duration::hours(72);
        assert_eq!(data_quality_from_age(very_old), DataQuality::Unavailable);
    }

    #[test]
    fn test_data_quality_boundary_6h() {
        // Exactly 6 hours should be Stale (< 6 is Fresh, >= 6 is Stale)
        let at_boundary = Utc::now() - chrono::Duration::hours(6);
        assert_eq!(data_quality_from_age(at_boundary), DataQuality::Stale);
    }

    #[test]
    fn test_data_quality_boundary_48h() {
        // Exactly 48 hours should be Stale (<= 48)
        let at_boundary = Utc::now() - chrono::Duration::hours(48);
        assert_eq!(data_quality_from_age(at_boundary), DataQuality::Stale);
    }

    // ── ClimateSnapshot::from_readings tests ────────────────────────

    fn make_reading(
        temp: f64,
        hum: f64,
        vpd: Option<f64>,
        precip: Option<f64>,
        age_hours: i64,
    ) -> ClimateReading {
        ClimateReading {
            id: "cr:test".into(),
            zone_id: "gz:test".into(),
            zone_name: "Test".into(),
            temperature: temp,
            humidity: hum,
            vpd,
            precipitation: precip,
            source: Some("test".into()),
            recorded_at: Utc::now() - chrono::Duration::hours(age_hours),
        }
    }

    #[test]
    fn test_snapshot_from_empty_readings() {
        let snap = ClimateSnapshot::from_readings("Zone", &[], false);
        assert!(snap.is_none());
    }

    #[test]
    fn test_snapshot_from_single_reading() {
        let readings = vec![make_reading(25.0, 60.0, Some(1.2), None, 1)];
        let snap = ClimateSnapshot::from_readings("Kitchen", &readings, false).unwrap();
        assert_eq!(snap.zone_name, "Kitchen");
        assert!((snap.avg_temp_c - 25.0).abs() < 0.01);
        assert!((snap.avg_humidity_pct - 60.0).abs() < 0.01);
        assert!((snap.avg_vpd_kpa - 1.2).abs() < 0.01);
        assert_eq!(snap.reading_count, 1);
        assert_eq!(snap.quality, DataQuality::Fresh);
        assert!(!snap.is_outdoor);
    }

    #[test]
    fn test_snapshot_averages_multiple_readings() {
        let readings = vec![
            make_reading(20.0, 50.0, Some(0.8), None, 1),
            make_reading(24.0, 60.0, Some(1.0), None, 2),
            make_reading(22.0, 55.0, Some(0.9), None, 3),
        ];
        let snap = ClimateSnapshot::from_readings("Z", &readings, false).unwrap();
        assert!((snap.avg_temp_c - 22.0).abs() < 0.01);
        assert!((snap.avg_humidity_pct - 55.0).abs() < 0.01);
        assert!((snap.avg_vpd_kpa - 0.9).abs() < 0.01);
        assert_eq!(snap.reading_count, 3);
    }

    #[test]
    fn test_snapshot_sums_precipitation_outdoor() {
        let readings = vec![
            make_reading(20.0, 60.0, Some(0.8), Some(3.0), 1),
            make_reading(20.0, 60.0, Some(0.8), Some(5.0), 2),
            make_reading(20.0, 60.0, Some(0.8), None, 3), // no precip data
        ];
        let snap = ClimateSnapshot::from_readings("Patio", &readings, true).unwrap();
        assert!(snap.is_outdoor);
        // Only readings with precipitation data contribute
        assert!((snap.precipitation_48h_mm.unwrap() - 8.0).abs() < 0.01);
    }

    #[test]
    fn test_snapshot_no_precipitation_indoor() {
        let readings = vec![make_reading(20.0, 60.0, Some(0.8), Some(10.0), 1)];
        let snap = ClimateSnapshot::from_readings("Room", &readings, false).unwrap();
        assert!(snap.precipitation_48h_mm.is_none());
    }

    #[test]
    fn test_snapshot_stale_quality_from_old_reading() {
        let readings = vec![make_reading(20.0, 60.0, Some(0.8), None, 12)];
        let snap = ClimateSnapshot::from_readings("Z", &readings, false).unwrap();
        assert_eq!(snap.quality, DataQuality::Stale);
    }

    #[test]
    fn test_snapshot_calculates_vpd_when_missing() {
        let readings = vec![make_reading(22.0, 55.0, None, None, 1)];
        let snap = ClimateSnapshot::from_readings("Z", &readings, false).unwrap();
        // Should calculate VPD from temp/humidity
        assert!(snap.avg_vpd_kpa > 0.0);
        // Should be close to reference VPD (~0.95)
        assert!((snap.avg_vpd_kpa - REFERENCE_VPD_KPA).abs() < 0.1);
    }

    // ── Realistic scenario tests ────────────────────────────────────

    #[test]
    fn test_scenario_tropical_greenhouse() {
        // Hot, humid greenhouse: 28°C, 75% RH → VPD ≈ 0.94
        // Sphagnum moss, medium light
        let snap = ClimateSnapshot {
            zone_name: "Greenhouse".into(),
            avg_temp_c: 28.0,
            avg_humidity_pct: 75.0,
            avg_vpd_kpa: 0.94,
            precipitation_48h_mm: None,
            newest_reading_at: Utc::now(),
            reading_count: 48,
            quality: DataQuality::Fresh,
            is_outdoor: false,
        };
        let est = climate_adjusted_frequency(
            7,
            Some(&snap),
            Some(&crate::orchid::PotMedium::SphagnumMoss),
            &LightRequirement::Medium,
            None,
        );
        // VPD factor = 1.19/0.94 ≈ 1.27, sphagnum (1.3) → 7 * 1.27 * 1.3 ≈ 11.5
        assert!(
            est.adjusted_days >= 10 && est.adjusted_days <= 13,
            "Tropical greenhouse with sphagnum should be 10-13 days, got {}",
            est.adjusted_days
        );
    }

    #[test]
    fn test_scenario_dry_windowsill() {
        // Dry winter windowsill: 20°C, 30% RH → VPD ≈ 1.64
        // Bark, high light
        let snap = ClimateSnapshot {
            zone_name: "Windowsill".into(),
            avg_temp_c: 20.0,
            avg_humidity_pct: 30.0,
            avg_vpd_kpa: 1.64,
            precipitation_48h_mm: None,
            newest_reading_at: Utc::now(),
            reading_count: 48,
            quality: DataQuality::Fresh,
            is_outdoor: false,
        };
        let est = climate_adjusted_frequency(
            7,
            Some(&snap),
            Some(&crate::orchid::PotMedium::Bark),
            &LightRequirement::High,
            None,
        );
        // VPD factor = 1.19/1.64 ≈ 0.726, bark 0.85, high light 0.85
        // 7 * 0.726 * 0.85 * 0.85 ≈ 3.67 → 4
        assert!(
            est.adjusted_days <= 5,
            "Dry windowsill with bark should be ≤5 days, got {}",
            est.adjusted_days
        );
    }

    #[test]
    fn test_scenario_outdoor_after_rain() {
        // Outdoor zone after heavy rain: 18°C, 85% RH, 20mm precip
        let snap = ClimateSnapshot {
            zone_name: "Patio".into(),
            avg_temp_c: 18.0,
            avg_humidity_pct: 85.0,
            avg_vpd_kpa: 0.31,
            precipitation_48h_mm: Some(20.0),
            newest_reading_at: Utc::now(),
            reading_count: 48,
            quality: DataQuality::Fresh,
            is_outdoor: true,
        };
        let est = climate_adjusted_frequency(
            7,
            Some(&snap),
            Some(&crate::orchid::PotMedium::Bark),
            &LightRequirement::Medium,
            None,
        );
        // VPD factor = 1.19/0.31 ≈ 2.5 (clamped), rain factor 2.0, bark 0.85
        // 7 * 2.5 * 1.0 * 0.85 * 1.0 * 2.0 = 29.75, clamped to 21 (3x base)
        assert_eq!(
            est.adjusted_days, 21,
            "Outdoor after heavy rain should hit 3x cap, got {}",
            est.adjusted_days
        );
    }

    // ── piecewise_linear tests ──────────────────────────────────────

    #[test]
    fn test_piecewise_linear_below_range_clamps() {
        let points = &[(100.0, 2.0), (200.0, 4.0), (300.0, 6.0)];
        assert!((piecewise_linear(0.0, points) - 2.0).abs() < 1e-9);
        assert!((piecewise_linear(50.0, points) - 2.0).abs() < 1e-9);
    }

    #[test]
    fn test_piecewise_linear_above_range_clamps() {
        let points = &[(100.0, 2.0), (200.0, 4.0), (300.0, 6.0)];
        assert!((piecewise_linear(500.0, points) - 6.0).abs() < 1e-9);
        assert!((piecewise_linear(300.0, points) - 6.0).abs() < 1e-9);
    }

    #[test]
    fn test_piecewise_linear_at_control_points() {
        let points = &[(100.0, 2.0), (200.0, 4.0), (300.0, 6.0)];
        assert!((piecewise_linear(100.0, points) - 2.0).abs() < 1e-9);
        assert!((piecewise_linear(200.0, points) - 4.0).abs() < 1e-9);
    }

    #[test]
    fn test_piecewise_linear_midpoint_interpolation() {
        let points = &[(100.0, 2.0), (200.0, 4.0)];
        // Midpoint: x=150 → y=3.0
        assert!((piecewise_linear(150.0, points) - 3.0).abs() < 1e-9);
        // Quarter point: x=125 → y=2.5
        assert!((piecewise_linear(125.0, points) - 2.5).abs() < 1e-9);
    }

    // ── light_factor_par tests ──────────────────────────────────────

    #[test]
    fn test_light_factor_par_at_200_is_baseline() {
        assert!((light_factor_par(200.0) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_light_factor_par_very_low_clamps() {
        // Below 50 should clamp to 1.20
        assert!((light_factor_par(10.0) - 1.20).abs() < 1e-9);
        assert!((light_factor_par(0.0) - 1.20).abs() < 1e-9);
    }

    #[test]
    fn test_light_factor_par_very_high_clamps() {
        // Above 800 should clamp to 0.70
        assert!((light_factor_par(1000.0) - 0.70).abs() < 1e-9);
        assert!((light_factor_par(2500.0) - 0.70).abs() < 1e-9);
    }

    #[test]
    fn test_light_factor_par_matches_enum_at_reference_points() {
        // PAR 100 ≈ Low light → enum Low = 1.15, PAR = 1.10 (close)
        assert!((light_factor_par(100.0) - 1.10).abs() < 1e-9);
        // PAR 200 ≈ Medium → enum Medium = 1.0, PAR = 1.0 (exact)
        assert!((light_factor_par(200.0) - light_factor(&LightRequirement::Medium)).abs() < 1e-9);
        // PAR 400 ≈ High → enum High = 0.85, PAR = 0.85 (exact)
        assert!((light_factor_par(400.0) - light_factor(&LightRequirement::High)).abs() < 1e-9);
    }

    // ── climate_adjusted_frequency with PAR ─────────────────────────

    #[test]
    fn test_climate_adjusted_frequency_par_overrides_enum() {
        let snap = ClimateSnapshot {
            zone_name: "Test".into(),
            avg_temp_c: 22.0,
            avg_humidity_pct: 55.0,
            avg_vpd_kpa: REFERENCE_VPD_KPA,
            precipitation_48h_mm: None,
            newest_reading_at: Utc::now(),
            reading_count: 10,
            quality: DataQuality::Fresh,
            is_outdoor: false,
        };
        // Without PAR: uses enum Low (factor 1.15)
        let est_enum = climate_adjusted_frequency(
            7, Some(&snap), None, &LightRequirement::Low, None,
        );
        // With PAR 400: factor 0.85 (overrides Low's 1.15)
        let est_par = climate_adjusted_frequency(
            7, Some(&snap), None, &LightRequirement::Low, Some(400.0),
        );
        // PAR 400 → High light → should water sooner (fewer days)
        assert!(
            est_par.adjusted_days < est_enum.adjusted_days,
            "PAR 400 should override Low enum: PAR got {} days, enum got {}",
            est_par.adjusted_days, est_enum.adjusted_days,
        );
    }

    #[test]
    fn test_climate_adjusted_frequency_none_par_uses_enum() {
        let snap = ClimateSnapshot {
            zone_name: "Test".into(),
            avg_temp_c: 22.0,
            avg_humidity_pct: 55.0,
            avg_vpd_kpa: REFERENCE_VPD_KPA,
            precipitation_48h_mm: None,
            newest_reading_at: Utc::now(),
            reading_count: 10,
            quality: DataQuality::Fresh,
            is_outdoor: false,
        };
        // None PAR with Medium should behave identically to the explicit enum path
        let est_none = climate_adjusted_frequency(
            7, Some(&snap), None, &LightRequirement::Medium, None,
        );
        // PAR 200 matches Medium exactly (factor 1.0)
        let est_par = climate_adjusted_frequency(
            7, Some(&snap), None, &LightRequirement::Medium, Some(200.0),
        );
        assert_eq!(
            est_none.adjusted_days, est_par.adjusted_days,
            "PAR 200 should match Medium enum: None={}, PAR={}",
            est_none.adjusted_days, est_par.adjusted_days,
        );
    }
}
