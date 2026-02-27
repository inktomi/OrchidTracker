-- PAR (Photosynthetically Active Radiation) sensor reading.
-- Stores PPFD in µmol/m²/s. When present, continuous functions
-- replace the 3-tier LightRequirement enum in algorithms.
DEFINE FIELD IF NOT EXISTS par_ppfd ON orchid TYPE option<float>;
