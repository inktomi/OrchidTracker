-- Hardware device table: shared devices across zones
DEFINE TABLE IF NOT EXISTS hardware_device SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS owner ON hardware_device TYPE record<user>;
DEFINE FIELD IF NOT EXISTS name ON hardware_device TYPE string;
DEFINE FIELD IF NOT EXISTS device_type ON hardware_device TYPE string
    ASSERT $value IN ["tempest", "ac_infinity"];
DEFINE FIELD IF NOT EXISTS config ON hardware_device TYPE string DEFAULT "";
DEFINE FIELD IF NOT EXISTS created_at ON hardware_device TYPE datetime DEFAULT time::now();
DEFINE INDEX IF NOT EXISTS idx_device_owner ON hardware_device FIELDS owner;

-- Add hardware device reference fields to growing_zone
DEFINE FIELD IF NOT EXISTS hardware_device ON growing_zone TYPE option<record<hardware_device>>;
DEFINE FIELD IF NOT EXISTS hardware_port ON growing_zone TYPE option<int>;
