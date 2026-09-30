-- 0016 derived the ratio as width/height of the stored resolution string, which
-- inverted it (e.g. '5:9' instead of '16:9') for resolutions written as
-- 'short × long'. Recompute those from the longer side over the shorter side.
UPDATE device_spec_values AS s
SET detail = (
    SELECT trim_scale(round(greatest(m[1]::numeric, m[2]::numeric)
                            / least(m[1]::numeric, m[2]::numeric) * 18) / 2)::text || ':9 비율'
    FROM device_spec_values r,
         LATERAL regexp_match(r.display_value, '([0-9]+)\s*×\s*([0-9]+)') AS m
    WHERE r.device_id = s.device_id
      AND r.spec_key = 'displayResolution'
      AND least(m[1]::numeric, m[2]::numeric) > 0
)
WHERE s.spec_key = 'displaySize'
  AND (regexp_match(s.detail, '^([0-9.]+):9 비율$'))[1]::numeric < 9;
