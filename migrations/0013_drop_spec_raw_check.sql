-- The API no longer reads or writes device_spec_values.raw_value, so the
-- "known requires raw_value, everything else requires NULL" invariant from
-- 0003 would reject every known spec written without it.
ALTER TABLE device_spec_values
    DROP CONSTRAINT IF EXISTS device_spec_values_check;
