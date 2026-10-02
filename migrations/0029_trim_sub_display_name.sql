-- Sub display names are now stored without the trailing word ("커버"); the UIs
-- render them as "<이름> 디스플레이".
UPDATE device_spec_values
SET display_value = btrim(regexp_replace(display_value, '\s*디스플레이$', ''))
WHERE spec_key IN ('sub1DisplayName', 'sub2DisplayName')
  AND display_value ~ '디스플레이$'
  AND length(btrim(regexp_replace(display_value, '\s*디스플레이$', ''))) > 0;
