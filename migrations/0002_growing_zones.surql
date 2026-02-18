-- Growing zones table
DEFINE TABLE IF NOT EXISTS growing_zone SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS owner ON growing_zone TYPE record<user>;
DEFINE FIELD IF NOT EXISTS name ON growing_zone TYPE string;
DEFINE FIELD IF NOT EXISTS light_level ON growing_zone TYPE string
    ASSERT $value IN ["Low", "Medium", "High"];
DEFINE FIELD IF NOT EXISTS location_type ON growing_zone TYPE string
    ASSERT $value IN ["Indoor", "Outdoor"];
DEFINE FIELD IF NOT EXISTS temperature_range ON growing_zone TYPE string DEFAULT "";
DEFINE FIELD IF NOT EXISTS humidity ON growing_zone TYPE string DEFAULT "";
DEFINE FIELD IF NOT EXISTS description ON growing_zone TYPE string DEFAULT "";
DEFINE FIELD IF NOT EXISTS sort_order ON growing_zone TYPE int DEFAULT 0;
DEFINE FIELD IF NOT EXISTS created_at ON growing_zone TYPE datetime DEFAULT time::now();
DEFINE INDEX IF NOT EXISTS idx_zone_owner ON growing_zone FIELDS owner;
DEFINE INDEX IF NOT EXISTS idx_zone_owner_name ON growing_zone FIELDS owner, name UNIQUE;

-- Remove old ASSERT on orchid.placement to allow any zone name string
REMOVE FIELD IF EXISTS placement ON orchid;
DEFINE FIELD IF NOT EXISTS placement ON orchid TYPE string DEFAULT "";
