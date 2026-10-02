CREATE TABLE wireless_technologies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    category TEXT NOT NULL CHECK (category IN ('network', 'wifi', 'bluetooth', 'uwb', 'nfc')),
    value TEXT NOT NULL CHECK (length(btrim(value)) BETWEEN 1 AND 80),
    label TEXT NOT NULL CHECK (length(btrim(label)) BETWEEN 1 AND 160),
    sort_order INTEGER NOT NULL DEFAULT 0 CHECK (sort_order BETWEEN 0 AND 100000),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (category, value)
);

CREATE INDEX wireless_technologies_category_order_idx
    ON wireless_technologies (category, sort_order, label);

INSERT INTO wireless_technologies (category, value, label, sort_order) VALUES
    ('network', '5G', '5G', 0),
    ('network', '4G', '4G', 10),
    ('network', '3G', '3G', 20),
    ('wifi', 'Wi-Fi 7', 'Wi-Fi 7 (802.11be)', 0),
    ('wifi', 'Wi-Fi 6E', 'Wi-Fi 6E (802.11ax, 6GHz)', 10),
    ('wifi', 'Wi-Fi 6', 'Wi-Fi 6 (802.11ax)', 20),
    ('wifi', 'Wi-Fi 5', 'Wi-Fi 5 (802.11ac)', 30),
    ('wifi', 'Wi-Fi 4', 'Wi-Fi 4 (802.11n)', 40),
    ('wifi', 'Wi-Fi 3', 'Wi-Fi 3 (802.11g)', 50),
    ('wifi', 'Wi-Fi 2', 'Wi-Fi 2 (802.11a)', 60),
    ('wifi', 'Wi-Fi 1', 'Wi-Fi 1 (802.11b)', 70),
    ('bluetooth', 'Bluetooth 6.0', '6.0', 0),
    ('bluetooth', 'Bluetooth 5.4', '5.4', 10),
    ('bluetooth', 'Bluetooth 5.3', '5.3', 20),
    ('bluetooth', 'Bluetooth 5.2', '5.2', 30),
    ('bluetooth', 'Bluetooth 5.1', '5.1', 40),
    ('bluetooth', 'Bluetooth 5.0', '5.0', 50),
    ('bluetooth', 'Bluetooth 4.2', '4.2', 60),
    ('bluetooth', 'Bluetooth 4.1', '4.1', 70),
    ('bluetooth', 'Bluetooth 4.0', '4.0', 80),
    ('bluetooth', 'Bluetooth 3.0', '3.0', 90),
    ('bluetooth', 'Bluetooth 2.1', '2.1', 100),
    ('bluetooth', 'Bluetooth 2.0', '2.0', 110),
    ('uwb', '지원', '지원', 0),
    ('uwb', '미지원', '미지원', 10),
    ('nfc', '지원', '지원', 0),
    ('nfc', '미지원', '미지원', 10);
