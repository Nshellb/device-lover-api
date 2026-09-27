-- Moves per-device colors out of the fixed-key device_spec_values EAV table
-- into a dedicated child table, so each color can carry its own image.
-- Mirrors device_configurations (position-ordered, created_at/updated_at),
-- plus an `exclusive` boolean that previously lived inside
-- device_spec_values.raw_value for spec_key='colors'.
CREATE TABLE device_colors (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    device_id UUID NOT NULL REFERENCES device_models(id) ON DELETE CASCADE,
    name TEXT NOT NULL CHECK (length(btrim(name)) BETWEEN 1 AND 160),
    image_url TEXT CHECK (image_url IS NULL OR image_url LIKE 'https://%'),
    exclusive BOOLEAN NOT NULL DEFAULT false,
    position INTEGER NOT NULL DEFAULT 0 CHECK (position >= 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (device_id, name),
    UNIQUE (device_id, position)
);

-- Backfill: one device_spec_values row per device (spec_key='colors',
-- raw_value a JSON array of {name, exclusive}) becomes N device_colors rows,
-- preserving array order as `position`. image_url starts NULL for all of
-- them — no per-color image data has ever existed anywhere in this project;
-- an admin fills it in later via the BO form.
--
-- Real data has the same color name appear twice in several Galaxy devices'
-- raw_value (once as a regular color, once more as an "exclusive" one, e.g.
-- Galaxy S24's "사파이어 블루") — an artifact of how the research pipeline's
-- colors() parser appends exclusive-detail names after the regular list
-- instead of updating the existing entry. Collapse by name per device
-- (bool_or so any exclusive:true occurrence wins) before inserting, since
-- device_colors has one row per distinct color name.
WITH expanded AS (
    SELECT
        sv.device_id,
        elem.value ->> 'name' AS name,
        COALESCE((elem.value ->> 'exclusive')::boolean, false) AS exclusive,
        elem.ordinality
    FROM device_spec_values sv
    CROSS JOIN LATERAL jsonb_array_elements(sv.raw_value) WITH ORDINALITY AS elem(value, ordinality)
    WHERE sv.spec_key = 'colors'
      AND sv.raw_value IS NOT NULL
),
deduped AS (
    SELECT
        device_id,
        name,
        bool_or(exclusive) AS exclusive,
        min(ordinality) AS first_ordinality
    FROM expanded
    GROUP BY device_id, name
)
INSERT INTO device_colors (device_id, name, exclusive, position)
SELECT
    device_id,
    name,
    exclusive,
    (row_number() OVER (PARTITION BY device_id ORDER BY first_ordinality) - 1)::integer
FROM deduped;

-- Drop the old EAV rows and shrink the fixed spec_key set from 27 to 26 —
-- colors is no longer one of the fixed keys every device must report.
DELETE FROM device_spec_values WHERE spec_key = 'colors';

ALTER TABLE device_spec_values
    DROP CONSTRAINT device_spec_values_spec_key_check,
    ADD CONSTRAINT device_spec_values_spec_key_check CHECK (spec_key IN (
        'operatingSystem', 'dimensions', 'weight', 'storage', 'stylus',
        'displayPanel', 'displaySize', 'displayResolution', 'refreshRate',
        'displayFeatures', 'processor', 'memory', 'wiredConnection', 'speakers',
        'rearCameras', 'telephoto', 'digitalZoom', 'frontCamera', 'videoRecording',
        'batteryCapacity', 'videoPlayback', 'fastCharging', 'wirelessCharging',
        'wireless', 'biometrics', 'waterResistance'
    ));
