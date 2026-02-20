-- Fertilizer and repotting tracking fields on orchid
DEFINE FIELD IF NOT EXISTS last_fertilized_at ON orchid TYPE option<datetime>;
DEFINE FIELD IF NOT EXISTS fertilize_frequency_days ON orchid TYPE option<int>;
DEFINE FIELD IF NOT EXISTS fertilizer_type ON orchid TYPE option<string>;
DEFINE FIELD IF NOT EXISTS last_repotted_at ON orchid TYPE option<datetime>;
DEFINE FIELD IF NOT EXISTS pot_medium ON orchid TYPE option<string>;
DEFINE FIELD IF NOT EXISTS pot_size ON orchid TYPE option<string>;
