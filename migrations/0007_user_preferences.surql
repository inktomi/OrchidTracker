-- User preferences (temp_unit etc.)
DEFINE TABLE IF NOT EXISTS user_preference SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS owner ON user_preference TYPE record<user>;
DEFINE FIELD IF NOT EXISTS temp_unit ON user_preference TYPE string DEFAULT 'C';
DEFINE INDEX IF NOT EXISTS idx_user_preference_owner ON user_preference FIELDS owner UNIQUE;
