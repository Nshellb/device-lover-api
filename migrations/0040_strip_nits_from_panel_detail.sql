-- Brightness lives in 피크 밝기; the panel detail of Galaxy Z Fold6 still
-- carried "최대 2,600니트" after 0039 cleaned 주요 기능.
UPDATE device_spec_values
SET detail = nullif(btrim(regexp_replace(detail, '(,\s*)?최대\s*[0-9,]+\s*니트', '', 'g'), ' ,'), ''),
    updated_at = now()
WHERE spec_key !~ 'PeakBrightness$'
  AND detail ~ '최대\s*[0-9,]+\s*니트';
