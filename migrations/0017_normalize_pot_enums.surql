-- Normalize pot_medium values from old SurrealValue object format {Variant: {}} to plain strings.
-- Old code stored enums via SurrealValue's untagged format; new code stores as plain strings.
-- Without this, OrchidDbRow deserialization fails: "Failed to decode PotMedium, no variants matched"
UPDATE orchid SET pot_medium = "Bark" WHERE pot_medium.Bark != NONE;
UPDATE orchid SET pot_medium = "SphagnumMoss" WHERE pot_medium.SphagnumMoss != NONE;
UPDATE orchid SET pot_medium = "Leca" WHERE pot_medium.Leca != NONE;
UPDATE orchid SET pot_medium = "Inorganic" WHERE pot_medium.Inorganic != NONE;
UPDATE orchid SET pot_medium = "Unknown" WHERE pot_medium.Unknown != NONE;

-- Normalize pot_size values from old object format to plain strings.
UPDATE orchid SET pot_size = "Small" WHERE pot_size.Small != NONE;
UPDATE orchid SET pot_size = "Medium" WHERE pot_size.Medium != NONE;
UPDATE orchid SET pot_size = "Large" WHERE pot_size.Large != NONE;
UPDATE orchid SET pot_size = "Unknown" WHERE pot_size.Unknown != NONE;

-- Normalize pot_type values from old object format to plain strings (unlikely but defensive).
UPDATE orchid SET pot_type = "Solid" WHERE pot_type.Solid != NONE;
UPDATE orchid SET pot_type = "Slotted" WHERE pot_type.Slotted != NONE;
UPDATE orchid SET pot_type = "Clay" WHERE pot_type.Clay != NONE;
UPDATE orchid SET pot_type = "Mounted" WHERE pot_type.Mounted != NONE;
UPDATE orchid SET pot_type = "Unknown" WHERE pot_type.Unknown != NONE;
