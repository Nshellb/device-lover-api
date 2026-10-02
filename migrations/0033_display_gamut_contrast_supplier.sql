-- Display color gamut, contrast ratio and panel supplier (main + sub displays).
-- Existing devices start as 미확인.
ALTER TABLE device_spec_values
    DROP CONSTRAINT device_spec_values_spec_key_check,
    ADD CONSTRAINT device_spec_values_spec_key_check CHECK (spec_key IN (
        'operatingSystem', 'weight', 'storage', 'stylus',
        'displayPanel', 'displaySize', 'displayResolution', 'refreshRate',
        'displayPeakBrightness', 'displayLamination', 'displayAntiReflective',
        'displayColorGamut', 'displayContrastRatio', 'displaySupplier',
        'displayFeatures', 'processor', 'memory',
        'wiredConnection', 'speakers', 'rearCameras', 'telephoto', 'digitalZoom',
        'frontCamera', 'videoRecording', 'videoPlayback', 'wireless', 'biometrics',
        'waterResistance', 'sim',
        'sub1DisplayName', 'sub1DisplayPanel', 'sub1DisplaySize', 'sub1DisplayResolution',
        'sub1RefreshRate', 'sub1PeakBrightness', 'sub1DisplayLamination',
        'sub1DisplayAntiReflective', 'sub1DisplayColorGamut', 'sub1DisplayContrastRatio', 'sub1DisplaySupplier', 'sub1DisplayFeatures',
        'sub2DisplayName', 'sub2DisplayPanel', 'sub2DisplaySize', 'sub2DisplayResolution',
        'sub2RefreshRate', 'sub2PeakBrightness', 'sub2DisplayLamination',
        'sub2DisplayAntiReflective', 'sub2DisplayColorGamut', 'sub2DisplayContrastRatio', 'sub2DisplaySupplier', 'sub2DisplayFeatures'
    ));

INSERT INTO device_spec_values (device_id, spec_key, display_value)
SELECT dm.id, t.key, '미확인'
  FROM device_models dm,
       (VALUES ('displayColorGamut'), ('displayContrastRatio'), ('displaySupplier')) AS t(key);

INSERT INTO device_spec_values (device_id, spec_key, display_value)
SELECT s.device_id, 'sub' || g.n || t.key, '미확인'
  FROM device_spec_values s,
       (VALUES (1), (2)) AS g(n),
       (VALUES ('DisplayColorGamut'), ('DisplayContrastRatio'), ('DisplaySupplier')) AS t(key)
 WHERE s.spec_key = 'sub' || g.n || 'DisplaySize';
