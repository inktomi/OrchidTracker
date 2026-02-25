-- Add precipitation field to climate_reading table
DEFINE FIELD IF NOT EXISTS precipitation ON climate_reading TYPE option<float>;
