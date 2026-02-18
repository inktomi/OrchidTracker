-- Migration 0005: Native habitat weather tracking
-- Adds native habitat fields to orchid and creates habitat weather tables

-- New fields on orchid table
DEFINE FIELD IF NOT EXISTS native_region ON orchid TYPE option<string>;
DEFINE FIELD IF NOT EXISTS native_latitude ON orchid TYPE option<float>;
DEFINE FIELD IF NOT EXISTS native_longitude ON orchid TYPE option<float>;

-- Raw habitat weather readings (pruned after 7 days via compaction)
DEFINE TABLE IF NOT EXISTS habitat_weather SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS latitude ON habitat_weather TYPE float;
DEFINE FIELD IF NOT EXISTS longitude ON habitat_weather TYPE float;
DEFINE FIELD IF NOT EXISTS temperature ON habitat_weather TYPE float;
DEFINE FIELD IF NOT EXISTS humidity ON habitat_weather TYPE float;
DEFINE FIELD IF NOT EXISTS precipitation ON habitat_weather TYPE float DEFAULT 0.0;
DEFINE FIELD IF NOT EXISTS recorded_at ON habitat_weather TYPE datetime DEFAULT time::now();
DEFINE INDEX IF NOT EXISTS idx_habitat_coords_time ON habitat_weather FIELDS latitude, longitude, recorded_at;

-- Compacted summaries (daily/weekly/monthly)
DEFINE TABLE IF NOT EXISTS habitat_weather_summary SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS latitude ON habitat_weather_summary TYPE float;
DEFINE FIELD IF NOT EXISTS longitude ON habitat_weather_summary TYPE float;
DEFINE FIELD IF NOT EXISTS period_type ON habitat_weather_summary TYPE string;
DEFINE FIELD IF NOT EXISTS period_start ON habitat_weather_summary TYPE datetime;
DEFINE FIELD IF NOT EXISTS avg_temperature ON habitat_weather_summary TYPE float;
DEFINE FIELD IF NOT EXISTS min_temperature ON habitat_weather_summary TYPE float;
DEFINE FIELD IF NOT EXISTS max_temperature ON habitat_weather_summary TYPE float;
DEFINE FIELD IF NOT EXISTS avg_humidity ON habitat_weather_summary TYPE float;
DEFINE FIELD IF NOT EXISTS total_precipitation ON habitat_weather_summary TYPE float DEFAULT 0.0;
DEFINE FIELD IF NOT EXISTS sample_count ON habitat_weather_summary TYPE int;
DEFINE INDEX IF NOT EXISTS idx_summary_coords_period ON habitat_weather_summary FIELDS latitude, longitude, period_type, period_start;
