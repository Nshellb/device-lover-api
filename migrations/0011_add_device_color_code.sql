-- A CSS-compatible color code (e.g. "#FFD700") an admin registers per color,
-- so the FO can render an actual color swatch on the selection button
-- instead of just the color's name. Optional: nullable until an admin fills
-- it in via the BO, same treatment as device_colors.image_url.
ALTER TABLE device_colors
    ADD COLUMN color_code TEXT CHECK (color_code IS NULL OR length(btrim(color_code)) > 0);
