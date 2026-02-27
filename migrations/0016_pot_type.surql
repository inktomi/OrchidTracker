-- Add pot type field (Solid, Slotted, Clay, Mounted) missing from 0009.
DEFINE FIELD IF NOT EXISTS pot_type ON orchid TYPE option<string>;
