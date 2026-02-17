-- Users
DEFINE TABLE IF NOT EXISTS user SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS username ON user TYPE string;
DEFINE FIELD IF NOT EXISTS email ON user TYPE string;
DEFINE FIELD IF NOT EXISTS password_hash ON user TYPE string;
DEFINE FIELD IF NOT EXISTS created_at ON user TYPE datetime DEFAULT time::now();
DEFINE INDEX IF NOT EXISTS idx_user_username ON user FIELDS username UNIQUE;
DEFINE INDEX IF NOT EXISTS idx_user_email ON user FIELDS email UNIQUE;

-- Orchids
DEFINE TABLE IF NOT EXISTS orchid SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS owner ON orchid TYPE record<user>;
DEFINE FIELD IF NOT EXISTS name ON orchid TYPE string;
DEFINE FIELD IF NOT EXISTS species ON orchid TYPE string;
DEFINE FIELD IF NOT EXISTS water_frequency_days ON orchid TYPE int;
DEFINE FIELD IF NOT EXISTS light_requirement ON orchid TYPE string
    ASSERT $value IN ["Low", "Medium", "High"];
DEFINE FIELD IF NOT EXISTS notes ON orchid TYPE string DEFAULT "";
DEFINE FIELD IF NOT EXISTS placement ON orchid TYPE string
    ASSERT $value IN ["Low", "Medium", "High", "Patio", "OutdoorRack"];
DEFINE FIELD IF NOT EXISTS light_lux ON orchid TYPE string DEFAULT "";
DEFINE FIELD IF NOT EXISTS temperature_range ON orchid TYPE string DEFAULT "";
DEFINE FIELD IF NOT EXISTS conservation_status ON orchid TYPE option<string>;
DEFINE FIELD IF NOT EXISTS created_at ON orchid TYPE datetime DEFAULT time::now();
DEFINE FIELD IF NOT EXISTS updated_at ON orchid TYPE datetime DEFAULT time::now();
DEFINE INDEX IF NOT EXISTS idx_orchid_owner ON orchid FIELDS owner;

-- Log entries
DEFINE TABLE IF NOT EXISTS log_entry SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS orchid ON log_entry TYPE record<orchid>;
DEFINE FIELD IF NOT EXISTS owner ON log_entry TYPE record<user>;
DEFINE FIELD IF NOT EXISTS timestamp ON log_entry TYPE datetime DEFAULT time::now();
DEFINE FIELD IF NOT EXISTS note ON log_entry TYPE string DEFAULT "";
DEFINE FIELD IF NOT EXISTS image_filename ON log_entry TYPE option<string>;
DEFINE INDEX IF NOT EXISTS idx_log_orchid ON log_entry FIELDS orchid;
DEFINE INDEX IF NOT EXISTS idx_log_owner ON log_entry FIELDS owner;

-- Migration tracking
DEFINE TABLE IF NOT EXISTS migration SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS name ON migration TYPE string;
DEFINE FIELD IF NOT EXISTS applied_at ON migration TYPE datetime DEFAULT time::now();
DEFINE INDEX IF NOT EXISTS idx_migration_name ON migration FIELDS name UNIQUE;
