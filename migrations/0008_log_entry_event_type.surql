-- Add event_type to log entries for categorization
DEFINE FIELD IF NOT EXISTS event_type ON log_entry TYPE option<string>
    ASSERT $value = NONE OR $value IN [
        "Flowering","NewGrowth","Repotted","Fertilized",
        "PestTreatment","Purchased","Watered","Note"
    ];

-- Track first bloom date on orchids
DEFINE FIELD IF NOT EXISTS first_bloom_at ON orchid TYPE option<datetime>;

-- Index for efficient log entry lookups by orchid + time
DEFINE INDEX IF NOT EXISTS idx_log_orchid_time ON log_entry FIELDS orchid, timestamp;
