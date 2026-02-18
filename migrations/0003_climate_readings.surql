-- Climate readings history
DEFINE TABLE IF NOT EXISTS climate_reading SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS zone ON climate_reading TYPE record<growing_zone>;
DEFINE FIELD IF NOT EXISTS zone_name ON climate_reading TYPE string DEFAULT "";
DEFINE FIELD IF NOT EXISTS temperature ON climate_reading TYPE float;
DEFINE FIELD IF NOT EXISTS humidity ON climate_reading TYPE float;
DEFINE FIELD IF NOT EXISTS vpd ON climate_reading TYPE option<float>;
DEFINE FIELD IF NOT EXISTS recorded_at ON climate_reading TYPE datetime DEFAULT time::now();
DEFINE INDEX IF NOT EXISTS idx_reading_zone_time ON climate_reading FIELDS zone, recorded_at;

-- Add data source fields to growing_zone
DEFINE FIELD IF NOT EXISTS data_source_type ON growing_zone TYPE option<string>;
DEFINE FIELD IF NOT EXISTS data_source_config ON growing_zone TYPE string DEFAULT "";
