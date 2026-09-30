-- RAM availability is now expressed by ram_gb itself: NULL means unknown or
-- undisclosed. Dropping the column also drops the CHECK that tied it to ram_gb.
ALTER TABLE device_configurations
    DROP COLUMN ram_status;
