-- 1) Launch price per configuration (KRW, NULL = unknown).
ALTER TABLE device_configurations
    ADD COLUMN price_krw INTEGER CHECK (price_krw IS NULL OR price_krw > 0);

-- 2) Materials: any number of (part, material, note) entries per device.
CREATE TABLE device_materials (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    device_id UUID NOT NULL REFERENCES device_models(id) ON DELETE CASCADE,
    position INTEGER NOT NULL CHECK (position BETWEEN 0 AND 20),
    part TEXT NOT NULL CHECK (length(btrim(part)) BETWEEN 1 AND 40),
    material TEXT NOT NULL CHECK (length(btrim(material)) BETWEEN 1 AND 40),
    note TEXT CHECK (note IS NULL OR length(btrim(note)) > 0),
    UNIQUE (device_id, position)
);

-- 3) Battery / charging as numbers (NULL = unknown, 0 = not supported for charging).
CREATE TABLE device_power (
    device_id UUID PRIMARY KEY REFERENCES device_models(id) ON DELETE CASCADE,
    battery_mah INTEGER CHECK (battery_mah IS NULL OR battery_mah > 0),
    battery_note TEXT,
    wired_w DOUBLE PRECISION CHECK (wired_w IS NULL OR wired_w >= 0),
    wired_note TEXT,
    wireless_w DOUBLE PRECISION CHECK (wireless_w IS NULL OR wireless_w >= 0),
    wireless_note TEXT
);

INSERT INTO device_power (device_id, battery_mah, battery_note, wired_w, wired_note, wireless_w, wireless_note)
SELECT dm.id,
       (SELECT replace((regexp_match(b.display_value, '([0-9][0-9,]*) ?mAh'))[1], ',', '')::integer
          FROM device_spec_values b WHERE b.device_id = dm.id AND b.spec_key = 'batteryCapacity'),
       (SELECT NULLIF(concat_ws(', ',
                 CASE WHEN b.display_value !~ '^[0-9,]+ ?mAh$' THEN b.display_value END,
                 b.detail), '')
          FROM device_spec_values b WHERE b.device_id = dm.id AND b.spec_key = 'batteryCapacity'),
       (SELECT CASE WHEN f.display_value = '미지원' THEN 0
                    ELSE (regexp_match(f.display_value, '([0-9.]+) ?W'))[1]::double precision END
          FROM device_spec_values f WHERE f.device_id = dm.id AND f.spec_key = 'fastCharging'),
       (SELECT NULLIF(concat_ws(', ',
                 CASE WHEN f.display_value !~ '^(최대 )?[0-9.]+ ?W$' AND f.display_value <> '미지원' THEN f.display_value END,
                 f.detail), '')
          FROM device_spec_values f WHERE f.device_id = dm.id AND f.spec_key = 'fastCharging'),
       (SELECT CASE WHEN w.display_value = '미지원' THEN 0
                    ELSE (SELECT max(x[1]::double precision)
                            FROM regexp_matches(w.display_value, '([0-9.]+) ?W', 'g') AS x) END
          FROM device_spec_values w WHERE w.device_id = dm.id AND w.spec_key = 'wirelessCharging'),
       (SELECT NULLIF(concat_ws(', ',
                 CASE WHEN w.display_value !~ '^[0-9.]+ ?W$' AND w.display_value <> '미지원' THEN w.display_value END,
                 w.detail), '')
          FROM device_spec_values w WHERE w.device_id = dm.id AND w.spec_key = 'wirelessCharging')
  FROM device_models dm;

-- Temporarily allow old and new keys together while data moves.
ALTER TABLE device_spec_values
    DROP CONSTRAINT device_spec_values_spec_key_check,
    ADD CONSTRAINT device_spec_values_spec_key_check CHECK (spec_key IN (
        'operatingSystem', 'weight', 'storage', 'stylus',
        'displayPanel', 'displaySize', 'displayResolution', 'refreshRate',
        'displayPeakBrightness', 'displayFeatures', 'processor', 'memory',
        'wiredConnection', 'speakers', 'rearCameras', 'telephoto', 'digitalZoom',
        'frontCamera', 'videoRecording', 'videoPlayback', 'wireless', 'biometrics',
        'waterResistance', 'sim', 'batteryCapacity', 'fastCharging', 'wirelessCharging',
        'sub1DisplayName', 'sub1DisplayPanel', 'sub1DisplaySize', 'sub1DisplayResolution',
        'sub1RefreshRate', 'sub1PeakBrightness', 'sub1DisplayFeatures',
        'sub2DisplayName', 'sub2DisplayPanel', 'sub2DisplaySize', 'sub2DisplayResolution',
        'sub2RefreshRate', 'sub2PeakBrightness', 'sub2DisplayFeatures'
    ));

-- 4) Peak brightness (main + each sub display) and SIM become fixed spec keys;
--    the three battery/charging keys move to device_power.
INSERT INTO device_spec_values (device_id, spec_key, display_value)
SELECT dm.id, 'displayPeakBrightness',
       COALESCE((
           SELECT replace((regexp_match(coalesce(s.detail, '') || ' ' || s.display_value, '([0-9][0-9,]*) ?(?:니트|nit)'))[1], ',', '') || '니트'
             FROM device_spec_values s
            WHERE s.device_id = dm.id AND s.spec_key IN ('displayFeatures', 'displayPanel')
              AND (coalesce(s.detail, '') || ' ' || s.display_value) ~ '[0-9][0-9,]* ?(?:니트|nit)'
            ORDER BY s.spec_key LIMIT 1
       ), '미확인')
  FROM device_models dm;

INSERT INTO device_spec_values (device_id, spec_key, display_value)
SELECT dm.id, 'sim', '미확인' FROM device_models dm;

INSERT INTO device_spec_values (device_id, spec_key, display_value)
SELECT s.device_id, 'sub' || n || 'PeakBrightness',
       COALESCE((
           SELECT replace((regexp_match(coalesce(f.detail, '') || ' ' || f.display_value, '([0-9][0-9,]*) ?(?:니트|nit)'))[1], ',', '') || '니트'
             FROM device_spec_values f
            WHERE f.device_id = s.device_id AND f.spec_key = 'sub' || n || 'DisplayFeatures'
              AND (coalesce(f.detail, '') || ' ' || f.display_value) ~ '[0-9][0-9,]* ?(?:니트|nit)'
       ), '미확인')
  FROM device_spec_values s, (VALUES (1), (2)) AS g(n)
 WHERE s.spec_key = 'sub' || n || 'DisplaySize';

DELETE FROM device_spec_values
 WHERE spec_key IN ('batteryCapacity', 'fastCharging', 'wirelessCharging');

ALTER TABLE device_spec_values
    DROP CONSTRAINT device_spec_values_spec_key_check,
    ADD CONSTRAINT device_spec_values_spec_key_check CHECK (spec_key IN (
        'operatingSystem', 'weight', 'storage', 'stylus',
        'displayPanel', 'displaySize', 'displayResolution', 'refreshRate',
        'displayPeakBrightness', 'displayFeatures', 'processor', 'memory',
        'wiredConnection', 'speakers', 'rearCameras', 'telephoto', 'digitalZoom',
        'frontCamera', 'videoRecording', 'videoPlayback', 'wireless', 'biometrics',
        'waterResistance', 'sim',
        'sub1DisplayName', 'sub1DisplayPanel', 'sub1DisplaySize', 'sub1DisplayResolution',
        'sub1RefreshRate', 'sub1PeakBrightness', 'sub1DisplayFeatures',
        'sub2DisplayName', 'sub2DisplayPanel', 'sub2DisplaySize', 'sub2DisplayResolution',
        'sub2RefreshRate', 'sub2PeakBrightness', 'sub2DisplayFeatures'
    ));
