-- Launch price in USD per configuration (NULL = unknown).
ALTER TABLE device_configurations
    ADD COLUMN price_usd DOUBLE PRECISION CHECK (price_usd IS NULL OR price_usd > 0);
