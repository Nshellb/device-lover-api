-- refreshRate is now entered as min/max numbers and stored as '60Hz' (equal) or
-- '1~120Hz' (range). Legacy '최대 120Hz' rows take the range from detail when it
-- names one (e.g. '1~120Hz 적응형'), otherwise just the maximum.
UPDATE device_spec_values
SET display_value = COALESCE(
    (SELECT m[1] || '~' || m[2] || 'Hz' FROM regexp_match(detail, '([0-9]+)\s*~\s*([0-9]+)\s*Hz') AS m),
    (SELECT m[1] || 'Hz' FROM regexp_match(display_value, '([0-9]+)\s*Hz') AS m),
    display_value
)
WHERE spec_key = 'refreshRate'
  AND display_value <> '60Hz';
