use crate::db::db;
use surrealdb::types::SurrealValue;
use super::open_meteo;

/// Poll habitat weather for all unique native coordinates across orchids.
/// Called periodically by the background task in main.rs (every 2 hours).
pub async fn poll_habitat_weather() {
    let db = db();
    let client = reqwest::Client::new();

    // 1. Query all distinct (latitude, longitude) pairs from orchids with native coords
    let mut response = match db
        .query(
            "SELECT math::round(native_latitude * 100) / 100 AS lat, \
                    math::round(native_longitude * 100) / 100 AS lon \
             FROM orchid \
             WHERE native_latitude IS NOT NULL AND native_longitude IS NOT NULL \
             GROUP BY lat, lon"
        )
        .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("Habitat poll: failed to query coordinates: {}", e);
            return;
        }
    };

    let errors = response.take_errors();
    if !errors.is_empty() {
        tracing::warn!("Habitat poll: coordinate query errors: {:?}", errors);
        return;
    }

    let coords: Vec<CoordRow> = match response.take(0) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Habitat poll: failed to parse coordinates: {}", e);
            return;
        }
    };

    if coords.is_empty() {
        tracing::debug!("Habitat poll: no orchids with native coordinates");
        return;
    }

    tracing::info!("Habitat poll: fetching weather for {} coordinate pairs", coords.len());

    // 2. Fetch weather for each unique coordinate pair
    for coord in &coords {
        match open_meteo::fetch_habitat_weather(&client, coord.lat, coord.lon).await {
            Ok(reading) => {
                if let Err(e) = db
                    .query(
                        "CREATE habitat_weather SET \
                         latitude = $lat, longitude = $lon, \
                         temperature = $temp, humidity = $humidity, \
                         precipitation = $precip, recorded_at = time::now()"
                    )
                    .bind(("lat", coord.lat))
                    .bind(("lon", coord.lon))
                    .bind(("temp", reading.temperature_c))
                    .bind(("humidity", reading.humidity_pct))
                    .bind(("precip", reading.precipitation_mm))
                    .await
                {
                    tracing::warn!(
                        "Habitat poll: failed to store reading for ({}, {}): {}",
                        coord.lat, coord.lon, e
                    );
                } else {
                    tracing::info!(
                        "Habitat poll: stored reading for ({}, {}): {:.1}C, {:.0}%, {:.1}mm",
                        coord.lat, coord.lon,
                        reading.temperature_c, reading.humidity_pct, reading.precipitation_mm
                    );
                }
            }
            Err(e) => {
                tracing::warn!(
                    "Habitat poll: failed to fetch weather for ({}, {}): {}",
                    coord.lat, coord.lon, e
                );
            }
        }

        // Brief delay between API calls to be respectful
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }

    // 3. Run compaction
    compact_habitat_data().await;

    tracing::info!("Habitat poll completed");
}

/// Compact raw readings into daily/weekly/monthly summaries.
/// - Raw readings older than 7 days → daily summaries
/// - Daily summaries older than 30 days → weekly summaries
/// - Weekly summaries older than 90 days → monthly summaries
pub async fn compact_habitat_data() {
    let db = db();

    // --- Raw → Daily: readings older than 7 days ---
    let compact_daily = db
        .query(
            "LET $old_readings = (SELECT * FROM habitat_weather WHERE recorded_at < time::now() - 7d); \
             IF array::len($old_readings) > 0 { \
                 LET $groups = (SELECT \
                     latitude, longitude, \
                     time::floor(recorded_at, 1d) AS period_start, \
                     math::mean(temperature) AS avg_temperature, \
                     math::min(temperature) AS min_temperature, \
                     math::max(temperature) AS max_temperature, \
                     math::mean(humidity) AS avg_humidity, \
                     math::sum(precipitation) AS total_precipitation, \
                     count() AS sample_count \
                 FROM habitat_weather \
                 WHERE recorded_at < time::now() - 7d \
                 GROUP BY latitude, longitude, period_start); \
                 FOR $g IN $groups { \
                     CREATE habitat_weather_summary SET \
                         latitude = $g.latitude, longitude = $g.longitude, \
                         period_type = 'daily', period_start = $g.period_start, \
                         avg_temperature = $g.avg_temperature, \
                         min_temperature = $g.min_temperature, \
                         max_temperature = $g.max_temperature, \
                         avg_humidity = $g.avg_humidity, \
                         total_precipitation = $g.total_precipitation, \
                         sample_count = $g.sample_count; \
                 }; \
                 DELETE habitat_weather WHERE recorded_at < time::now() - 7d; \
             };"
        )
        .await;

    if let Err(e) = compact_daily {
        tracing::warn!("Habitat compact: raw→daily failed: {}", e);
    }

    // --- Daily → Weekly: daily summaries older than 30 days ---
    let compact_weekly = db
        .query(
            "LET $old_daily = (SELECT * FROM habitat_weather_summary WHERE period_type = 'daily' AND period_start < time::now() - 30d); \
             IF array::len($old_daily) > 0 { \
                 LET $groups = (SELECT \
                     latitude, longitude, \
                     time::floor(period_start, 1w) AS week_start, \
                     math::sum(avg_temperature * sample_count) / math::sum(sample_count) AS avg_temperature, \
                     math::min(min_temperature) AS min_temperature, \
                     math::max(max_temperature) AS max_temperature, \
                     math::sum(avg_humidity * sample_count) / math::sum(sample_count) AS avg_humidity, \
                     math::sum(total_precipitation) AS total_precipitation, \
                     math::sum(sample_count) AS sample_count \
                 FROM habitat_weather_summary \
                 WHERE period_type = 'daily' AND period_start < time::now() - 30d \
                 GROUP BY latitude, longitude, week_start); \
                 FOR $g IN $groups { \
                     CREATE habitat_weather_summary SET \
                         latitude = $g.latitude, longitude = $g.longitude, \
                         period_type = 'weekly', period_start = $g.week_start, \
                         avg_temperature = $g.avg_temperature, \
                         min_temperature = $g.min_temperature, \
                         max_temperature = $g.max_temperature, \
                         avg_humidity = $g.avg_humidity, \
                         total_precipitation = $g.total_precipitation, \
                         sample_count = $g.sample_count; \
                 }; \
                 DELETE habitat_weather_summary WHERE period_type = 'daily' AND period_start < time::now() - 30d; \
             };"
        )
        .await;

    if let Err(e) = compact_weekly {
        tracing::warn!("Habitat compact: daily→weekly failed: {}", e);
    }

    // --- Weekly → Monthly: weekly summaries older than 90 days ---
    let compact_monthly = db
        .query(
            "LET $old_weekly = (SELECT * FROM habitat_weather_summary WHERE period_type = 'weekly' AND period_start < time::now() - 90d); \
             IF array::len($old_weekly) > 0 { \
                 LET $groups = (SELECT \
                     latitude, longitude, \
                     time::floor(period_start, 1mo) AS month_start, \
                     math::sum(avg_temperature * sample_count) / math::sum(sample_count) AS avg_temperature, \
                     math::min(min_temperature) AS min_temperature, \
                     math::max(max_temperature) AS max_temperature, \
                     math::sum(avg_humidity * sample_count) / math::sum(sample_count) AS avg_humidity, \
                     math::sum(total_precipitation) AS total_precipitation, \
                     math::sum(sample_count) AS sample_count \
                 FROM habitat_weather_summary \
                 WHERE period_type = 'weekly' AND period_start < time::now() - 90d \
                 GROUP BY latitude, longitude, month_start); \
                 FOR $g IN $groups { \
                     CREATE habitat_weather_summary SET \
                         latitude = $g.latitude, longitude = $g.longitude, \
                         period_type = 'monthly', period_start = $g.month_start, \
                         avg_temperature = $g.avg_temperature, \
                         min_temperature = $g.min_temperature, \
                         max_temperature = $g.max_temperature, \
                         avg_humidity = $g.avg_humidity, \
                         total_precipitation = $g.total_precipitation, \
                         sample_count = $g.sample_count; \
                 }; \
                 DELETE habitat_weather_summary WHERE period_type = 'weekly' AND period_start < time::now() - 90d; \
             };"
        )
        .await;

    if let Err(e) = compact_monthly {
        tracing::warn!("Habitat compact: weekly→monthly failed: {}", e);
    }
}

#[derive(serde::Deserialize, SurrealValue)]
#[surreal(crate = "surrealdb::types")]
struct CoordRow {
    lat: f64,
    lon: f64,
}
