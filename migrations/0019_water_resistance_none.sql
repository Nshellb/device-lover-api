-- waterResistance without an IP rating is now stored as an explicit sentence
-- (BO leaves both IP digits empty) instead of the generic '미지원'.
UPDATE device_spec_values
SET display_value = '방수·방진 지원 안 함'
WHERE spec_key = 'waterResistance'
  AND display_value = '미지원';
