-- Peak brightness rows (main and sub displays) no longer carry a detail.
UPDATE device_spec_values
SET detail = NULL, updated_at = now()
WHERE spec_key ~ 'PeakBrightness$'
  AND detail IS NOT NULL;
