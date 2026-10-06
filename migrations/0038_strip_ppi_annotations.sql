-- PPI now has its own row (split out of displayResolution's detail), so the
-- "약 416ppi(비공식 계산치)" annotations are reduced to a plain "416ppi".
UPDATE device_spec_values
SET detail = regexp_replace(
        detail,
        '(약\s*)?([0-9]+(\.[0-9]+)?)\s*ppi(\s*\([^)]*\))?',
        '\2ppi',
        'gi'
    )
WHERE spec_key ~ '^(sub[0-9])?[dD]isplayResolution$'
  AND detail ~* '[0-9]\s*ppi';
