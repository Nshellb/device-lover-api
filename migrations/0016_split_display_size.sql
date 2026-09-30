-- displaySize is now entered as inches + aspect ratio:
--   display_value = '6.3인치', detail = '19.5:9 비율'.
-- Legacy rows stored a metric size ('15.5 cm') with '약 6.1형' in detail; the
-- ratio for those is derived from the same device's displayResolution.
UPDATE device_spec_values AS s
SET display_value = (regexp_match(s.detail, '([0-9]+(?:\.[0-9]+)?)형'))[1] || '인치',
    detail = (
        SELECT trim_scale(round(m[1]::numeric / m[2]::numeric * 18) / 2)::text || ':9 비율'
        FROM device_spec_values r,
             LATERAL regexp_match(r.display_value, '([0-9]+)\s*×\s*([0-9]+)') AS m
        WHERE r.device_id = s.device_id
          AND r.spec_key = 'displayResolution'
          AND m[2]::numeric > 0
    )
WHERE s.spec_key = 'displaySize'
  AND s.display_value ~ '^[0-9.]+ ?cm$'
  AND s.detail ~ '[0-9.]+형';

-- Hand-entered rows that packed both into the value ('6.3인치 19.6:9 비율').
UPDATE device_spec_values
SET display_value = (regexp_match(display_value, '^([0-9.]+)\s*인치'))[1] || '인치',
    detail = (regexp_match(display_value, '([0-9.]+:[0-9.]+)\s*비율'))[1] || ' 비율'
WHERE spec_key = 'displaySize'
  AND display_value ~ '^[0-9.]+\s*인치\s+[0-9.]+:[0-9.]+\s*비율$';
