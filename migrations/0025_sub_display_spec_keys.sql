-- Foldables may carry up to two sub displays, stored as optional spec keys
-- (sub1* / sub2*) next to the 26 fixed ones; the API enforces all-or-none per group.
ALTER TABLE device_spec_values
    DROP CONSTRAINT device_spec_values_spec_key_check,
    ADD CONSTRAINT device_spec_values_spec_key_check CHECK (spec_key IN (
        'operatingSystem', 'dimensions', 'weight', 'storage', 'stylus',
        'displayPanel', 'displaySize', 'displayResolution', 'refreshRate',
        'displayFeatures', 'processor', 'memory', 'wiredConnection', 'speakers',
        'rearCameras', 'telephoto', 'digitalZoom', 'frontCamera', 'videoRecording',
        'batteryCapacity', 'videoPlayback', 'fastCharging', 'wirelessCharging',
        'wireless', 'biometrics', 'waterResistance',
        'sub1DisplayPanel', 'sub1DisplaySize', 'sub1DisplayResolution',
        'sub1RefreshRate', 'sub1DisplayFeatures',
        'sub2DisplayPanel', 'sub2DisplaySize', 'sub2DisplayResolution',
        'sub2RefreshRate', 'sub2DisplayFeatures'
    ));
