CREATE TABLE brands (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slug TEXT COLLATE "C" NOT NULL UNIQUE
        CHECK (slug ~ '^[a-z0-9]+(-[a-z0-9]+)*$' AND length(slug) <= 80),
    name TEXT NOT NULL CHECK (length(btrim(name)) BETWEEN 1 AND 120)
);

CREATE TABLE device_models (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    brand_id UUID NOT NULL REFERENCES brands(id) ON DELETE RESTRICT,
    category TEXT NOT NULL DEFAULT 'smartphone' CHECK (category = 'smartphone'),
    slug TEXT COLLATE "C" NOT NULL UNIQUE
        CHECK (
            slug ~ '^[a-z0-9]+(-[a-z0-9]+)*$'
            AND length(slug) <= 120
            AND slug !~ '-vs(-|$)'
        ),
    name TEXT NOT NULL CHECK (length(btrim(name)) BETWEEN 1 AND 160),
    market_code TEXT NOT NULL DEFAULT 'KR' CHECK (market_code = 'KR'),
    release_date DATE,
    summary_variant_label TEXT CHECK (
        summary_variant_label IS NULL OR length(btrim(summary_variant_label)) > 0
    ),
    image_url TEXT CHECK (image_url IS NULL OR image_url LIKE 'https://%'),
    publication_status TEXT NOT NULL DEFAULT 'draft'
        CHECK (publication_status IN ('draft', 'published', 'archived')),
    verified_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (publication_status <> 'published' OR
        (release_date IS NOT NULL AND verified_at IS NOT NULL))
);

CREATE INDEX device_models_public_latest_idx
    ON device_models (category, release_date DESC, slug)
    WHERE publication_status = 'published';
CREATE INDEX device_models_public_brand_latest_idx
    ON device_models (brand_id, category, release_date DESC, slug)
    WHERE publication_status = 'published';

CREATE TABLE device_aliases (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    device_id UUID NOT NULL REFERENCES device_models(id) ON DELETE CASCADE,
    value TEXT NOT NULL CHECK (length(btrim(value)) BETWEEN 1 AND 160),
    kind TEXT NOT NULL CHECK (kind IN ('alias', 'model_number', 'hardware_identifier')),
    position INTEGER NOT NULL DEFAULT 0 CHECK (position >= 0),
    UNIQUE (device_id, value),
    UNIQUE (device_id, position)
);

CREATE TABLE device_identifiers (
    route_key TEXT COLLATE "C" PRIMARY KEY
        CHECK (length(route_key) BETWEEN 1 AND 160 AND route_key !~ '-vs(-|$)'),
    device_id UUID NOT NULL REFERENCES device_models(id) ON DELETE CASCADE,
    search_key TEXT COLLATE "C" NOT NULL CHECK (length(search_key) BETWEEN 1 AND 160)
);
CREATE INDEX device_identifiers_device_idx ON device_identifiers (device_id);
CREATE INDEX device_identifiers_search_idx ON device_identifiers (search_key);

CREATE TABLE device_configurations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    device_id UUID NOT NULL REFERENCES device_models(id) ON DELETE CASCADE,
    label TEXT NOT NULL CHECK (length(btrim(label)) BETWEEN 1 AND 160),
    storage_gb INTEGER NOT NULL CHECK (storage_gb > 0),
    ram_gb INTEGER CHECK (ram_gb > 0),
    ram_status TEXT NOT NULL CHECK (ram_status IN ('known', 'unknown', 'not_disclosed')),
    position INTEGER NOT NULL DEFAULT 0 CHECK (position >= 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK ((ram_status = 'known' AND ram_gb IS NOT NULL)
        OR (ram_status <> 'known' AND ram_gb IS NULL)),
    UNIQUE NULLS NOT DISTINCT (device_id, storage_gb, ram_gb),
    UNIQUE (device_id, position)
);

CREATE TABLE device_sources (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    device_id UUID NOT NULL REFERENCES device_models(id) ON DELETE CASCADE,
    url TEXT NOT NULL CHECK (url LIKE 'https://%'),
    title TEXT NOT NULL CHECK (length(btrim(title)) BETWEEN 1 AND 240),
    checked_at TIMESTAMPTZ,
    is_primary BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (device_id, url),
    UNIQUE (id, device_id)
);
CREATE UNIQUE INDEX device_sources_one_primary_idx
    ON device_sources (device_id) WHERE is_primary;

CREATE TABLE device_spec_values (
    device_id UUID NOT NULL REFERENCES device_models(id) ON DELETE CASCADE,
    spec_key TEXT NOT NULL CHECK (spec_key IN (
        'operatingSystem', 'colors', 'dimensions', 'weight', 'storage', 'stylus',
        'displayPanel', 'displaySize', 'displayResolution', 'refreshRate',
        'displayFeatures', 'processor', 'memory', 'wiredConnection', 'speakers',
        'rearCameras', 'telephoto', 'digitalZoom', 'frontCamera', 'videoRecording',
        'batteryCapacity', 'videoPlayback', 'fastCharging', 'wirelessCharging',
        'wireless', 'biometrics', 'waterResistance'
    )),
    status TEXT NOT NULL CHECK (status IN (
        'known', 'unknown', 'not_disclosed', 'not_applicable'
    )),
    raw_value JSONB,
    display_value TEXT NOT NULL CHECK (length(btrim(display_value)) > 0),
    detail TEXT CHECK (detail IS NULL OR length(btrim(detail)) > 0),
    source_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (device_id, spec_key),
    FOREIGN KEY (source_id, device_id) REFERENCES device_sources(id, device_id),
    CHECK (
        (status = 'known' AND raw_value IS NOT NULL AND raw_value <> 'null'::jsonb)
        OR (status <> 'known' AND raw_value IS NULL)
    )
);
CREATE INDEX device_spec_values_source_idx
    ON device_spec_values (source_id, device_id) WHERE source_id IS NOT NULL;
