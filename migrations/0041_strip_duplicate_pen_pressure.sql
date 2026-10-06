-- 펜 지원 value already carries the pressure level ("4096단계"), so the pen's
-- detail must not repeat it. Drops the trailing sentence / comma clause that
-- mentions 필압 and the inline "·4,096단계 필압" phrase.
UPDATE device_spec_values
SET detail = nullif(btrim(
        regexp_replace(
            regexp_replace(
                regexp_replace(detail, '(?<![0-9]),\s*(필압|[0-9,]+단계)[^,]*$', ''),
                '\.\s*[^.]*필압[^.]*$', ''),
            '[·,]?\s*[0-9,]+단계\s*필압', '', 'g'),
        ' ,.'), ''),
    updated_at = now()
WHERE spec_key = 'stylus'
  AND display_value ~ '단계$'
  AND detail ~ '필압';
