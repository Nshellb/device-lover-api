-- Generalize camera_models beyond Canon-only DSLRs. Existing manufacturer-
-- specific enums cannot express other brands' cameras (e.g. Samsung's NX
-- mirrorless line has no DSLR camera_type, no EF-family lens_mount, and no
-- fixed-lens-compact mount at all). All 17 existing seeded rows are a subset
-- of the widened values below, so this is a pure constraint-loosening
-- migration — no backfill needed. Constraint names verified against the
-- live DB before writing this (`\d+ camera_models`).

-- series: was a Canon-only 3-value enum. It's a marketing/product-line
-- label with no real finite cross-brand vocabulary, so loosen to free text
-- (same shape as image_processor's check).
ALTER TABLE camera_models
    DROP CONSTRAINT camera_models_series_check,
    ADD CONSTRAINT camera_models_series_check
        CHECK (length(btrim(series)) BETWEEN 1 AND 160);

-- camera_type: was hard-locked to the single literal 'DSLR'. Widen to the
-- genuinely finite, brand-agnostic camera-body categories (Samsung's NX
-- line was mirrorless).
ALTER TABLE camera_models
    DROP CONSTRAINT camera_models_camera_type_check,
    ADD CONSTRAINT camera_models_camera_type_check
        CHECK (camera_type IN ('DSLR', 'mirrorless', 'compact'));

-- sensor_format: widen the existing enum with two more standard,
-- brand-agnostic sensor-size categories.
ALTER TABLE camera_models
    DROP CONSTRAINT camera_models_sensor_format_check,
    ADD CONSTRAINT camera_models_sensor_format_check
        CHECK (sensor_format IN ('full_frame', 'aps_c', 'micro_four_thirds', 'one_inch'));

-- lens_mount: mount names are brand+system-specific (Canon EF/EF-S/RF,
-- Nikon F/Z, Sony E, Samsung NX, ...), so an enum cannot scale; loosen to
-- free text. Also make it nullable: a fixed-lens 'compact' camera has no
-- interchangeable mount at all.
ALTER TABLE camera_models
    ALTER COLUMN lens_mount DROP NOT NULL,
    DROP CONSTRAINT camera_models_lens_mount_check,
    ADD CONSTRAINT camera_models_lens_mount_check
        CHECK (lens_mount IS NULL OR length(btrim(lens_mount)) BETWEEN 1 AND 120);
