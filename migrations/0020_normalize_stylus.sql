-- stylus is now a fixed choice: 미지원 / 필압 미확인 / 1024·2048·4096·8192단계.
-- Legacy pen descriptions ('S Pen 내장', ...) move into detail; the pressure level
-- is taken from the text when it names one, otherwise it becomes 필압 미확인.
UPDATE device_spec_values
SET display_value = CASE WHEN detail ~ '4,?096단계' THEN '4096단계' ELSE '필압 미확인' END,
    detail = concat_ws(', ', display_value, detail)
WHERE spec_key = 'stylus'
  AND display_value <> '미지원';
