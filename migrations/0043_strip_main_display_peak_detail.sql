-- A peak brightness detail that only says "메인 디스플레이" adds nothing: the row
-- is already the main display's.
UPDATE device_spec_values
SET detail = NULL, updated_at = now()
WHERE spec_key ~ '^(sub[0-9])?([dD]isplayPeakBrightness|PeakBrightness)$'
  AND btrim(detail) = '메인 디스플레이';
