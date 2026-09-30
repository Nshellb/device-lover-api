-- wiredConnection is now a fixed choice (USB-C / 마이크로 5핀 / 8핀) or free text
-- ('그외', e.g. '30핀 Dock 커넥터'). Legacy version/brand names move into detail.
UPDATE device_spec_values
SET display_value = 'USB-C'
WHERE spec_key = 'wiredConnection' AND display_value = 'USB Type-C';

UPDATE device_spec_values
SET display_value = '8핀',
    detail = concat_ws(', ', 'Lightning', detail)
WHERE spec_key = 'wiredConnection' AND display_value = 'Lightning';

UPDATE device_spec_values
SET detail = concat_ws(', ', CASE WHEN display_value = '마이크로 USB 2.0' THEN 'USB 2.0' END, detail),
    display_value = '마이크로 5핀'
WHERE spec_key = 'wiredConnection' AND display_value IN ('micro-USB', '마이크로 USB 2.0');
