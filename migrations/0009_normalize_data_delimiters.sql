-- Use a comma followed by a space for separators inside catalog data.
UPDATE device_spec_values
SET raw_value = CASE
        WHEN raw_value::text LIKE '% · %'
            THEN replace(raw_value::text, ' · ', ', ')::jsonb
        ELSE raw_value
    END,
    display_value = replace(display_value, ' · ', ', '),
    detail = replace(detail, ' · ', ', '),
    updated_at = now()
WHERE raw_value::text LIKE '% · %'
   OR display_value LIKE '% · %'
   OR detail LIKE '% · %';

UPDATE device_configurations
SET label = replace(label, ' · ', ', '),
    updated_at = now()
WHERE label LIKE '% · %';

UPDATE device_models
SET summary_variant_label = replace(summary_variant_label, ' · ', ', '),
    updated_at = now()
WHERE summary_variant_label LIKE '% · %';

UPDATE camera_models
SET continuous_shooting_note = replace(continuous_shooting_note, ' · ', ', '),
    video_spec = replace(video_spec, ' · ', ', '),
    updated_at = now()
WHERE continuous_shooting_note LIKE '% · %'
   OR video_spec LIKE '% · %';
