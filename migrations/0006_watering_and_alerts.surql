-- 0006_watering_and_alerts.surql
-- Adds watering tracking fields to orchid, plus alert and push subscription tables.

-- Watering + structured climate range fields on orchid
DEFINE FIELD IF NOT EXISTS last_watered_at ON orchid TYPE option<datetime>;
DEFINE FIELD IF NOT EXISTS temp_min ON orchid TYPE option<float>;
DEFINE FIELD IF NOT EXISTS temp_max ON orchid TYPE option<float>;
DEFINE FIELD IF NOT EXISTS humidity_min ON orchid TYPE option<float>;
DEFINE FIELD IF NOT EXISTS humidity_max ON orchid TYPE option<float>;

-- Push subscription table
DEFINE TABLE IF NOT EXISTS push_subscription SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS owner ON push_subscription TYPE record<user>;
DEFINE FIELD IF NOT EXISTS endpoint ON push_subscription TYPE string;
DEFINE FIELD IF NOT EXISTS p256dh ON push_subscription TYPE string;
DEFINE FIELD IF NOT EXISTS auth ON push_subscription TYPE string;
DEFINE FIELD IF NOT EXISTS created_at ON push_subscription TYPE datetime DEFAULT time::now();
DEFINE INDEX IF NOT EXISTS idx_push_owner ON push_subscription FIELDS owner;

-- Alert table
DEFINE TABLE IF NOT EXISTS alert SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS owner ON alert TYPE record<user>;
DEFINE FIELD IF NOT EXISTS orchid ON alert TYPE option<record<orchid>>;
DEFINE FIELD IF NOT EXISTS zone ON alert TYPE option<record<growing_zone>>;
DEFINE FIELD IF NOT EXISTS alert_type ON alert TYPE string;
DEFINE FIELD IF NOT EXISTS severity ON alert TYPE string;
DEFINE FIELD IF NOT EXISTS message ON alert TYPE string;
DEFINE FIELD IF NOT EXISTS created_at ON alert TYPE datetime DEFAULT time::now();
DEFINE FIELD IF NOT EXISTS acknowledged_at ON alert TYPE option<datetime>;
DEFINE INDEX IF NOT EXISTS idx_alert_owner ON alert FIELDS owner;
