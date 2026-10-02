-- Dimensions become structured, searchable numbers (up to 3 entries per device,
-- e.g. 펼친 상태 / 접은 상태) instead of the free-text '146.7 × 71.5 × 7.65 mm'
-- spec row, so width/height/depth can be filtered individually later.
CREATE TABLE device_dimensions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    device_id UUID NOT NULL REFERENCES device_models(id) ON DELETE CASCADE,
    position INTEGER NOT NULL CHECK (position BETWEEN 0 AND 2),
    label TEXT NOT NULL CHECK (length(btrim(label)) BETWEEN 1 AND 40),
    width_mm DOUBLE PRECISION NOT NULL CHECK (width_mm > 0),
    height_mm DOUBLE PRECISION NOT NULL CHECK (height_mm > 0),
    depth_mm DOUBLE PRECISION NOT NULL CHECK (depth_mm > 0),
    note TEXT CHECK (note IS NULL OR length(btrim(note)) > 0),
    UNIQUE (device_id, position)
);
CREATE INDEX device_dimensions_size_idx ON device_dimensions (width_mm, height_mm, depth_mm);

-- Main entry from the old 'W × H × D mm' value. Foldables (detail mentions the
-- folded size) become '펼친 상태'; other devices keep their detail as a note.
INSERT INTO device_dimensions (device_id, position, label, width_mm, height_mm, depth_mm, note)
SELECT s.device_id, 0,
       CASE WHEN s.detail ~ '(접었을 때|접은 상태)' THEN '펼친 상태' ELSE '본체' END,
       m[1]::double precision, m[2]::double precision, m[3]::double precision,
       CASE WHEN s.detail ~ '(접었을 때|접은 상태)' THEN NULL ELSE s.detail END
  FROM device_spec_values s,
       LATERAL regexp_match(s.display_value, '^([0-9.]+) × ([0-9.]+) × ([0-9.]+) mm$') AS m
 WHERE s.spec_key = 'dimensions' AND m IS NOT NULL;

-- Folded entry; a thickness range ('13.8~16.8') keeps its thickest value.
INSERT INTO device_dimensions (device_id, position, label, width_mm, height_mm, depth_mm)
SELECT s.device_id, 1, '접은 상태',
       m[1]::double precision, m[2]::double precision,
       COALESCE(m[4], m[3])::double precision
  FROM device_spec_values s,
       LATERAL regexp_match(
           s.detail,
           '(?:접었을 때|접은 상태) ([0-9.]+) × ([0-9.]+) × ([0-9.]+)(?:~([0-9.]+))?'
       ) AS m
 WHERE s.spec_key = 'dimensions' AND m IS NOT NULL;

DELETE FROM device_spec_values WHERE spec_key = 'dimensions';

ALTER TABLE device_spec_values
    DROP CONSTRAINT device_spec_values_spec_key_check,
    ADD CONSTRAINT device_spec_values_spec_key_check CHECK (spec_key IN (
        'operatingSystem', 'weight', 'storage', 'stylus',
        'displayPanel', 'displaySize', 'displayResolution', 'refreshRate',
        'displayFeatures', 'processor', 'memory', 'wiredConnection', 'speakers',
        'rearCameras', 'telephoto', 'digitalZoom', 'frontCamera', 'videoRecording',
        'batteryCapacity', 'videoPlayback', 'fastCharging', 'wirelessCharging',
        'wireless', 'biometrics', 'waterResistance',
        'sub1DisplayName', 'sub1DisplayPanel', 'sub1DisplaySize', 'sub1DisplayResolution',
        'sub1RefreshRate', 'sub1DisplayFeatures',
        'sub2DisplayName', 'sub2DisplayPanel', 'sub2DisplaySize', 'sub2DisplayResolution',
        'sub2RefreshRate', 'sub2DisplayFeatures'
    ));
