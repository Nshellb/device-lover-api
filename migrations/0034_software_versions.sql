-- Operating system / UX versions become managed master data (software_versions)
-- linked to devices (device_software): the launch version plus every version the
-- device can be upgraded to. The old 'operatingSystem' spec row is converted.
CREATE TABLE software_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    category TEXT NOT NULL CHECK (category IN ('os', 'ux')),
    value TEXT NOT NULL CHECK (length(btrim(value)) BETWEEN 1 AND 80),
    label TEXT NOT NULL CHECK (length(btrim(label)) BETWEEN 1 AND 160),
    sort_order INTEGER NOT NULL DEFAULT 0 CHECK (sort_order BETWEEN 0 AND 100000),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (category, value)
);
CREATE INDEX software_versions_category_order_idx
    ON software_versions (category, sort_order, label);

CREATE TABLE device_software (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    device_id UUID NOT NULL REFERENCES device_models(id) ON DELETE CASCADE,
    version_id UUID NOT NULL REFERENCES software_versions(id) ON DELETE RESTRICT,
    position INTEGER NOT NULL CHECK (position BETWEEN 0 AND 60),
    is_launch BOOLEAN NOT NULL DEFAULT false,
    note TEXT CHECK (note IS NULL OR length(btrim(note)) > 0),
    UNIQUE (device_id, version_id),
    UNIQUE (device_id, position)
);

-- Masters from existing values; sort_order = major*100 + minor so old -> new.
INSERT INTO software_versions (category, value, label, sort_order)
SELECT 'os', v, v,
       COALESCE((regexp_match(v, '([0-9]+)'))[1]::int * 100
                + COALESCE((regexp_match(v, '[0-9]+\.([0-9]+)'))[1]::int, 0), 0)
  FROM (SELECT DISTINCT display_value AS v FROM device_spec_values
         WHERE spec_key = 'operatingSystem') t
ON CONFLICT DO NOTHING;

INSERT INTO software_versions (category, value, label, sort_order)
SELECT 'ux', v, v,
       COALESCE((regexp_match(v, '([0-9]+)'))[1]::int * 100
                + COALESCE((regexp_match(v, '[0-9]+\.([0-9]+)'))[1]::int, 0), 0)
  FROM (SELECT DISTINCT (regexp_match(detail, '(One UI [0-9.]+[0-9])'))[1] AS v
          FROM device_spec_values
         WHERE spec_key = 'operatingSystem') t
 WHERE v IS NOT NULL
ON CONFLICT DO NOTHING;

INSERT INTO device_software (device_id, version_id, position, is_launch, note)
SELECT s.device_id, v.id, 0, true, NULLIF(btrim(s.detail), '')
  FROM device_spec_values s
  JOIN software_versions v ON v.category = 'os' AND v.value = s.display_value
 WHERE s.spec_key = 'operatingSystem';

INSERT INTO device_software (device_id, version_id, position, is_launch)
SELECT s.device_id, v.id, 1, true
  FROM device_spec_values s
  JOIN software_versions v
    ON v.category = 'ux' AND v.value = (regexp_match(s.detail, '(One UI [0-9.]+[0-9])'))[1]
 WHERE s.spec_key = 'operatingSystem';

DELETE FROM device_spec_values WHERE spec_key = 'operatingSystem';

ALTER TABLE device_spec_values
    DROP CONSTRAINT device_spec_values_spec_key_check,
    ADD CONSTRAINT device_spec_values_spec_key_check CHECK (spec_key IN (
        'weight', 'storage', 'stylus',
        'displayPanel', 'displaySize', 'displayResolution', 'refreshRate',
        'displayPeakBrightness', 'displayLamination', 'displayAntiReflective',
        'displayColorGamut', 'displayContrastRatio', 'displaySupplier',
        'displayFeatures', 'processor', 'memory',
        'wiredConnection', 'speakers', 'rearCameras', 'telephoto', 'digitalZoom',
        'frontCamera', 'videoRecording', 'videoPlayback', 'wireless', 'biometrics',
        'waterResistance', 'sim',
        'sub1DisplayName', 'sub1DisplayPanel', 'sub1DisplaySize', 'sub1DisplayResolution',
        'sub1RefreshRate', 'sub1PeakBrightness', 'sub1DisplayLamination',
        'sub1DisplayAntiReflective', 'sub1DisplayColorGamut', 'sub1DisplayContrastRatio',
        'sub1DisplaySupplier', 'sub1DisplayFeatures',
        'sub2DisplayName', 'sub2DisplayPanel', 'sub2DisplaySize', 'sub2DisplayResolution',
        'sub2RefreshRate', 'sub2PeakBrightness', 'sub2DisplayLamination',
        'sub2DisplayAntiReflective', 'sub2DisplayColorGamut', 'sub2DisplayContrastRatio',
        'sub2DisplaySupplier', 'sub2DisplayFeatures'
    ));
