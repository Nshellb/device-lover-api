-- Alt text for the device's representative image (NULL = fall back to the device name).
ALTER TABLE device_models
    ADD COLUMN image_alt TEXT CHECK (image_alt IS NULL OR length(btrim(image_alt)) BETWEEN 1 AND 200);
