-- Samsung publishes 4,096 pressure levels for the S Pen of every Galaxy S Ultra
-- that was still 필압 미확인: S21 Ultra (S Pen 별매), S22 / S23 / S24 / S26 Ultra
-- (samsung.com S Pen product pages and Samsung Newsroom).
UPDATE device_spec_values s
SET display_value = '4096단계', updated_at = now()
FROM device_models m
WHERE m.id = s.device_id
  AND s.spec_key = 'stylus'
  AND s.display_value = '필압 미확인'
  AND m.slug IN ('galaxy-s21-ultra', 'galaxy-s22-ultra', 'galaxy-s23-ultra', 'galaxy-s24-ultra', 'galaxy-s26-ultra');
