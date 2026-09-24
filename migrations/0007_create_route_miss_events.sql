CREATE TABLE route_miss_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    requested_path TEXT NOT NULL
        CHECK (
            char_length(requested_path) BETWEEN 1 AND 512
            AND requested_path LIKE '/%'
            AND requested_path !~ '[[:cntrl:]]'
        ),
    referrer TEXT
        CHECK (referrer IS NULL OR char_length(referrer) BETWEEN 1 AND 1024),
    locale TEXT
        CHECK (locale IS NULL OR char_length(locale) BETWEEN 1 AND 35),
    viewport_class TEXT
        CHECK (viewport_class IS NULL OR viewport_class IN ('mobile', 'tablet', 'desktop')),
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX route_miss_events_occurred_at_idx
    ON route_miss_events (occurred_at DESC);

CREATE INDEX route_miss_events_path_occurred_at_idx
    ON route_miss_events (requested_path, occurred_at DESC);
