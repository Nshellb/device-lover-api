-- wiredConnection choices are now USB Type-C / Lightning / micro-USB / Apple 30-pin
-- (or free text), replacing the 0023 labels. Details 0023 added are undone.
UPDATE device_spec_values
SET display_value = 'USB Type-C'
WHERE spec_key = 'wiredConnection' AND display_value = 'USB-C';

UPDATE device_spec_values
SET display_value = 'Lightning',
    detail = NULLIF(regexp_replace(detail, '^Lightning(, |$)', ''), '')
WHERE spec_key = 'wiredConnection' AND display_value = '8핀';

UPDATE device_spec_values
SET display_value = 'micro-USB'
WHERE spec_key = 'wiredConnection' AND display_value = '마이크로 5핀';

UPDATE device_spec_values
SET display_value = 'micro-USB',
    detail = concat_ws(', ', 'USB 3.0', detail)
WHERE spec_key = 'wiredConnection' AND display_value = '마이크로 USB 3.0';

UPDATE device_spec_values
SET display_value = 'Apple 30-pin'
WHERE spec_key = 'wiredConnection' AND display_value = '30핀 Dock 커넥터';
