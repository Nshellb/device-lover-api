-- Spec availability is now expressed by the display value itself (e.g.
-- "정보 없음", "공식 미공개", "해당 없음"); the API derives `muted` from it.
ALTER TABLE device_spec_values
    DROP COLUMN status;
