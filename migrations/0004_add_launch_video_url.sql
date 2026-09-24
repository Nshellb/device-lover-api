ALTER TABLE device_models
    ADD COLUMN launch_video_url TEXT
        CHECK (launch_video_url IS NULL OR launch_video_url LIKE 'https://%');
