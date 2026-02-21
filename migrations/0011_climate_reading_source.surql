-- Add source field to distinguish reading origin (wizard, manual, hardware, weather_api)
DEFINE FIELD IF NOT EXISTS source ON climate_reading TYPE option<string>;
DEFINE INDEX IF NOT EXISTS idx_reading_source ON climate_reading FIELDS zone, source, recorded_at;
