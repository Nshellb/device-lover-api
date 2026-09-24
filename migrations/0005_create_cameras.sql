CREATE TABLE camera_models (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    brand_id UUID NOT NULL REFERENCES brands(id) ON DELETE RESTRICT,
    slug TEXT COLLATE "C" NOT NULL UNIQUE
        CHECK (slug ~ '^[a-z0-9]+(-[a-z0-9]+)*$' AND length(slug) <= 120),
    name TEXT NOT NULL CHECK (length(btrim(name)) BETWEEN 1 AND 160),
    series TEXT NOT NULL CHECK (series IN ('EOS 5D', 'EOS 6D', 'EOS x0D')),
    -- Canon Camera Museum gives a marketing month, rather than an exact day.
    release_month TEXT COLLATE "C" NOT NULL
        CHECK (release_month ~ '^[1-9][0-9]{3}-(0[1-9]|1[0-2])$'),
    camera_type TEXT NOT NULL CHECK (camera_type = 'DSLR'),
    sensor_format TEXT NOT NULL CHECK (sensor_format IN ('full_frame', 'aps_c')),
    effective_megapixels DOUBLE PRECISION NOT NULL
        CHECK (effective_megapixels > 0 AND effective_megapixels < 'Infinity'::double precision),
    image_processor TEXT NOT NULL CHECK (length(btrim(image_processor)) BETWEEN 1 AND 120),
    lens_mount TEXT NOT NULL CHECK (lens_mount IN ('EF', 'EF/EF-S')),
    max_continuous_fps DOUBLE PRECISION NOT NULL
        CHECK (max_continuous_fps > 0 AND max_continuous_fps < 'Infinity'::double precision),
    continuous_shooting_note TEXT CHECK (
        continuous_shooting_note IS NULL OR length(btrim(continuous_shooting_note)) BETWEEN 1 AND 240
    ),
    video_spec TEXT NOT NULL CHECK (length(btrim(video_spec)) BETWEEN 1 AND 240),
    -- Body only; excludes the battery, memory card and lens.
    body_weight_g INTEGER NOT NULL CHECK (body_weight_g > 0),
    source_url TEXT NOT NULL CHECK (
        source_url ~ '^https://[^/[:space:]]+(/[^[:space:]]*)?$'
        AND length(source_url) <= 2048
    ),
    source_title TEXT NOT NULL CHECK (length(btrim(source_title)) BETWEEN 1 AND 240),
    checked_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX camera_models_latest_idx
    ON camera_models (release_month DESC, name COLLATE "C", slug);
CREATE INDEX camera_models_series_latest_idx
    ON camera_models (series, release_month DESC, name COLLATE "C", slug);
CREATE INDEX camera_models_brand_idx ON camera_models (brand_id);
