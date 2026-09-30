-- '공식 미공개' is no longer a value; those specs are simply unconfirmed. The
-- estimates and sources recorded in detail are kept.
UPDATE device_spec_values
SET display_value = '미확인'
WHERE display_value = '공식 미공개';
