CREATE TABLE device_popularity_daily (
    stat_date DATE NOT NULL,
    category TEXT NOT NULL CHECK (category IN ('smartphone', 'camera')),
    device_slug TEXT COLLATE "C" NOT NULL
        CHECK (
            device_slug ~ '^[a-z0-9]+(-[a-z0-9]+)*$'
            AND char_length(device_slug) <= 160
        ),
    view_count BIGINT NOT NULL DEFAULT 0 CHECK (view_count >= 0),
    compare_count BIGINT NOT NULL DEFAULT 0 CHECK (compare_count >= 0),
    last_selected_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (stat_date, category, device_slug)
);

CREATE INDEX device_popularity_daily_ranking_idx
    ON device_popularity_daily (category, stat_date DESC, last_selected_at DESC);
