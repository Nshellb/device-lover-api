-- Display treatments: lamination and anti-reflective (있음 / 없음 / 미확인) for the
-- main display and every existing sub display. Existing text is scanned once.
ALTER TABLE device_spec_values
    DROP CONSTRAINT device_spec_values_spec_key_check,
    ADD CONSTRAINT device_spec_values_spec_key_check CHECK (spec_key IN (
        'operatingSystem', 'weight', 'storage', 'stylus',
        'displayPanel', 'displaySize', 'displayResolution', 'refreshRate',
        'displayPeakBrightness', 'displayLamination', 'displayAntiReflective',
        'displayFeatures', 'processor', 'memory',
        'wiredConnection', 'speakers', 'rearCameras', 'telephoto', 'digitalZoom',
        'frontCamera', 'videoRecording', 'videoPlayback', 'wireless', 'biometrics',
        'waterResistance', 'sim',
        'sub1DisplayName', 'sub1DisplayPanel', 'sub1DisplaySize', 'sub1DisplayResolution',
        'sub1RefreshRate', 'sub1PeakBrightness', 'sub1DisplayLamination',
        'sub1DisplayAntiReflective', 'sub1DisplayFeatures',
        'sub2DisplayName', 'sub2DisplayPanel', 'sub2DisplaySize', 'sub2DisplayResolution',
        'sub2RefreshRate', 'sub2PeakBrightness', 'sub2DisplayLamination',
        'sub2DisplayAntiReflective', 'sub2DisplayFeatures'
    ));

INSERT INTO device_spec_values (device_id, spec_key, display_value)
SELECT dm.id, t.key,
       CASE WHEN EXISTS (
           SELECT 1 FROM device_spec_values s
            WHERE s.device_id = dm.id AND s.spec_key IN ('displayPanel', 'displayFeatures')
              AND (coalesce(s.detail, '') || ' ' || s.display_value) ~* t.pattern
       ) THEN '있음' ELSE '미확인' END
  FROM device_models dm,
       (VALUES ('displayLamination', '라미네|laminat'),
               ('displayAntiReflective', '반사 ?방지|anti-?reflect|gorilla armor')) AS t(key, pattern);

INSERT INTO device_spec_values (device_id, spec_key, display_value)
SELECT s.device_id, 'sub' || g.n || t.key,
       CASE WHEN EXISTS (
           SELECT 1 FROM device_spec_values f
            WHERE f.device_id = s.device_id
              AND f.spec_key IN ('sub' || g.n || 'DisplayPanel', 'sub' || g.n || 'DisplayFeatures')
              AND (coalesce(f.detail, '') || ' ' || f.display_value) ~* t.pattern
       ) THEN '있음' ELSE '미확인' END
  FROM device_spec_values s,
       (VALUES (1), (2)) AS g(n),
       (VALUES ('DisplayLamination', '라미네|laminat'),
               ('DisplayAntiReflective', '반사 ?방지|anti-?reflect|gorilla armor')) AS t(key, pattern)
 WHERE s.spec_key = 'sub' || g.n || 'DisplaySize';
