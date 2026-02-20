-- Seasonal care fields on orchid
DEFINE FIELD IF NOT EXISTS rest_start_month ON orchid TYPE option<int>;
DEFINE FIELD IF NOT EXISTS rest_end_month ON orchid TYPE option<int>;
DEFINE FIELD IF NOT EXISTS bloom_start_month ON orchid TYPE option<int>;
DEFINE FIELD IF NOT EXISTS bloom_end_month ON orchid TYPE option<int>;
DEFINE FIELD IF NOT EXISTS rest_water_multiplier ON orchid TYPE option<float>;
DEFINE FIELD IF NOT EXISTS rest_fertilizer_multiplier ON orchid TYPE option<float>;
DEFINE FIELD IF NOT EXISTS active_water_multiplier ON orchid TYPE option<float>;
DEFINE FIELD IF NOT EXISTS active_fertilizer_multiplier ON orchid TYPE option<float>;

-- Hemisphere preference on user_preference
DEFINE FIELD IF NOT EXISTS hemisphere ON user_preference TYPE string DEFAULT 'N';
