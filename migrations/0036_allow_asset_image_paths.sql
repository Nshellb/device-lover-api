-- Device and color images may be site-relative paths under the FO's public/asset
-- folder (e.g. '/asset/galaxy-z-fold-6.jpg') in addition to https:// URLs.
ALTER TABLE device_models
    DROP CONSTRAINT device_models_image_url_check,
    ADD CONSTRAINT device_models_image_url_check CHECK (
        image_url IS NULL OR image_url LIKE 'https://%' OR image_url LIKE '/asset/%'
    );

ALTER TABLE device_colors
    DROP CONSTRAINT device_colors_image_url_check,
    ADD CONSTRAINT device_colors_image_url_check CHECK (
        image_url IS NULL OR image_url LIKE 'https://%' OR image_url LIKE '/asset/%'
    );
