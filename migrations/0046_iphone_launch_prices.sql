-- iPhone launch prices (KRW at the Korean launch incl. VAT, USD at the US launch)
-- per storage tier. Existing configurations are matched by storage and only get
-- their prices filled; missing tiers are added with the label "128GB" / "1TB".
-- Sources: Apple Newsroom (US/KR) where available, otherwise launch-day press
-- (etnews, inews24, Kyunghyang, Hankyung, ...), GSMArena and Wikipedia.
--   * iPhone 2007-2013 USD are the US two-year-contract prices Apple advertised
--     (no off-contract price existed); KRW are the carrier list prices (출고가).
--     KRW is NULL where there was no Korean launch with a list price (original
--     iPhone, iPhone 3G 16GB).
--   * iPhone 6..XS Max KRW are carrier 출고가 / Apple Korea prices of the first
--     Korean sale; USD are the unlocked prices where Apple published one.
--   * iPhone 12 / 12 mini USD are Apple's unlocked prices ($829 / $729).
--   * iPhone 18 Pro / Pro Max: only the press-confirmed US starting price of the
--     256GB tier; other tiers and KRW are not yet confirmed.
--   * Tiers Apple never sold in Korea / were not listed (iPhone 8 128GB, ...) and
--     unverified prices stay NULL.
CREATE TEMP TABLE iphone_launch_prices (slug text, storage_gb int, price_krw int, price_usd double precision) ON COMMIT DROP;
INSERT INTO iphone_launch_prices VALUES
    ('iphone', 8, NULL, 599),
    ('iphone', 16, NULL, 499),
    ('iphone-3g', 8, NULL, 199),
    ('iphone-3g', 16, NULL, 299),
    ('iphone-3gs', 16, 814000, 199),
    ('iphone-3gs', 32, 946000, 299),
    ('iphone-4', 16, 814000, 199),
    ('iphone-4', 32, 946000, 299),
    ('iphone-4s', 16, 814000, 199),
    ('iphone-4s', 32, 946000, 299),
    ('iphone-4s', 64, 1078000, 399),
    ('iphone-5', 16, 814000, 199),
    ('iphone-5', 32, 946000, 299),
    ('iphone-5', 64, 1078000, 399),
    ('iphone-5c', 16, 704000, 99),
    ('iphone-5c', 32, 814000, 199),
    ('iphone-5s', 16, 814000, 199),
    ('iphone-5s', 32, 946000, 299),
    ('iphone-5s', 64, 1078000, 399),
    ('iphone-6', 16, 789800, 649),
    ('iphone-6', 64, 924000, 749),
    ('iphone-6', 128, 1056000, 849),
    ('iphone-6-plus', 16, 924000, 749),
    ('iphone-6-plus', 64, 1056000, 849),
    ('iphone-6-plus', 128, 1188000, 949),
    ('iphone-6s', 16, 920000, 649),
    ('iphone-6s', 64, 1060000, 749),
    ('iphone-6s', 128, 1200000, 849),
    ('iphone-6s-plus', 16, 1060000, 749),
    ('iphone-6s-plus', 64, 1200000, 849),
    ('iphone-6s-plus', 128, 1340000, 949),
    ('iphone-7', 32, 869000, 649),
    ('iphone-7', 128, 999900, 749),
    ('iphone-7', 256, 1130800, 849),
    ('iphone-7-plus', 32, 1021900, 769),
    ('iphone-7-plus', 128, 1152800, 869),
    ('iphone-7-plus', 256, 1283700, 969),
    ('iphone-8', 64, 990000, 699),
    ('iphone-8', 256, 1200000, 849),
    ('iphone-8-plus', 64, 1130000, 799),
    ('iphone-8-plus', 256, 1340000, 949),
    ('iphone-x', 64, 1420000, 999),
    ('iphone-x', 256, 1630000, 1149),
    ('iphone-xr', 64, 990000, 749),
    ('iphone-xr', 128, NULL, 799),
    ('iphone-xr', 256, 1200000, 899),
    ('iphone-xs', 64, 1370000, 999),
    ('iphone-xs', 256, NULL, 1149),
    ('iphone-xs', 512, 1850000, 1349),
    ('iphone-xs-max', 64, 1500000, 1099),
    ('iphone-xs-max', 256, NULL, 1249),
    ('iphone-xs-max', 512, 1980000, 1449),
    ('iphone-11', 64, 990000, 699),
    ('iphone-11', 128, 1060000, 749),
    ('iphone-11', 256, 1200000, 849),
    ('iphone-11-pro', 64, 1390000, 999),
    ('iphone-11-pro', 256, 1600000, 1149),
    ('iphone-11-pro', 512, 1870000, 1349),
    ('iphone-11-pro-max', 64, 1550000, 1099),
    ('iphone-11-pro-max', 256, 1760000, 1249),
    ('iphone-11-pro-max', 512, 2030000, 1449),
    ('iphone-12', 64, 1090000, 829),
    ('iphone-12', 128, 1160000, 879),
    ('iphone-12', 256, 1300000, 979),
    ('iphone-12-mini', 64, 950000, 729),
    ('iphone-12-mini', 128, 1020000, 779),
    ('iphone-12-mini', 256, 1160000, 879),
    ('iphone-12-pro', 128, 1350000, 999),
    ('iphone-12-pro', 256, 1490000, 1099),
    ('iphone-12-pro', 512, 1760000, 1299),
    ('iphone-12-pro-max', 128, 1490000, 1099),
    ('iphone-12-pro-max', 256, 1630000, 1199),
    ('iphone-12-pro-max', 512, 1900000, 1399),
    ('iphone-13', 128, 1090000, 799),
    ('iphone-13', 256, 1230000, 899),
    ('iphone-13', 512, 1500000, 1099),
    ('iphone-13-mini', 128, 950000, 699),
    ('iphone-13-mini', 256, 1090000, 799),
    ('iphone-13-mini', 512, 1360000, 999),
    ('iphone-13-pro', 128, 1350000, 999),
    ('iphone-13-pro', 256, 1490000, 1099),
    ('iphone-13-pro', 512, 1760000, 1299),
    ('iphone-13-pro', 1024, 2030000, 1499),
    ('iphone-13-pro-max', 128, 1490000, 1099),
    ('iphone-13-pro-max', 256, 1630000, 1199),
    ('iphone-13-pro-max', 512, 1900000, 1399),
    ('iphone-13-pro-max', 1024, 2170000, 1599),
    ('iphone-14', 128, 1250000, 799),
    ('iphone-14', 256, 1400000, 899),
    ('iphone-14', 512, 1700000, 1099),
    ('iphone-14-plus', 128, 1350000, 899),
    ('iphone-14-plus', 256, 1500000, 999),
    ('iphone-14-plus', 512, 1800000, 1199),
    ('iphone-14-pro', 128, 1550000, 999),
    ('iphone-14-pro', 256, 1700000, 1099),
    ('iphone-14-pro', 512, 2000000, 1299),
    ('iphone-14-pro', 1024, 2300000, 1499),
    ('iphone-14-pro-max', 128, 1750000, 1099),
    ('iphone-14-pro-max', 256, 1900000, 1199),
    ('iphone-14-pro-max', 512, 2200000, 1399),
    ('iphone-14-pro-max', 1024, 2500000, 1599),
    ('iphone-15', 128, 1250000, 799),
    ('iphone-15', 256, 1400000, 899),
    ('iphone-15', 512, 1700000, 1099),
    ('iphone-15-plus', 128, 1350000, 899),
    ('iphone-15-plus', 256, 1500000, 999),
    ('iphone-15-plus', 512, 1800000, 1199),
    ('iphone-15-pro', 128, 1550000, 999),
    ('iphone-15-pro', 256, 1700000, 1099),
    ('iphone-15-pro', 512, 2000000, 1299),
    ('iphone-15-pro', 1024, 2300000, 1499),
    ('iphone-15-pro-max', 256, 1900000, 1199),
    ('iphone-15-pro-max', 512, 2200000, 1399),
    ('iphone-15-pro-max', 1024, 2500000, 1599),
    ('iphone-16', 128, 1250000, 799),
    ('iphone-16', 256, 1400000, 899),
    ('iphone-16', 512, 1700000, 1099),
    ('iphone-16-plus', 128, 1350000, 899),
    ('iphone-16-plus', 256, 1500000, 999),
    ('iphone-16-plus', 512, 1800000, 1199),
    ('iphone-16-pro', 128, 1550000, 999),
    ('iphone-16-pro', 256, 1700000, 1099),
    ('iphone-16-pro', 512, 2000000, 1299),
    ('iphone-16-pro', 1024, 2300000, 1499),
    ('iphone-16-pro-max', 256, 1900000, 1199),
    ('iphone-16-pro-max', 512, 2200000, 1399),
    ('iphone-16-pro-max', 1024, 2500000, 1599),
    ('iphone-16e', 128, 990000, 599),
    ('iphone-16e', 256, 1140000, 699),
    ('iphone-16e', 512, 1440000, 899),
    ('iphone-17', 256, 1290000, 799),
    ('iphone-17', 512, 1590000, 999),
    ('iphone-air', 256, 1590000, 999),
    ('iphone-air', 512, 1890000, 1199),
    ('iphone-air', 1024, 2190000, 1399),
    ('iphone-17-pro', 256, 1790000, 1099),
    ('iphone-17-pro', 512, 2090000, 1299),
    ('iphone-17-pro', 1024, 2390000, 1499),
    ('iphone-17-pro-max', 256, 1990000, 1199),
    ('iphone-17-pro-max', 512, 2290000, 1399),
    ('iphone-17-pro-max', 1024, 2590000, 1599),
    ('iphone-17-pro-max', 2048, 3190000, 1999),
    ('iphone-17e', 256, 990000, 599),
    ('iphone-17e', 512, NULL, 799),
    ('iphone-18-pro', 256, NULL, 1199),
    ('iphone-18-pro-max', 256, NULL, 1299);

UPDATE device_configurations c
SET price_krw = p.price_krw, price_usd = p.price_usd, updated_at = now()
FROM iphone_launch_prices p
JOIN device_models m ON m.slug = p.slug
WHERE c.device_id = m.id AND c.storage_gb = p.storage_gb;

INSERT INTO device_configurations (device_id, label, storage_gb, price_krw, price_usd, position)
SELECT m.id,
       CASE WHEN p.storage_gb >= 1024 THEN (p.storage_gb / 1024) || 'TB' ELSE p.storage_gb || 'GB' END,
       p.storage_gb, p.price_krw, p.price_usd,
       1000 + (row_number() OVER (PARTITION BY m.id ORDER BY p.storage_gb))::int
FROM iphone_launch_prices p
JOIN device_models m ON m.slug = p.slug
WHERE NOT EXISTS (
    SELECT 1 FROM device_configurations c
     WHERE c.device_id = m.id AND c.storage_gb = p.storage_gb
);

-- Smallest storage first. (device_id, position) is unique, so shift out of the way first.
UPDATE device_configurations c
SET position = c.position + 1000
FROM device_models m
WHERE m.id = c.device_id AND c.position < 1000
  AND m.slug IN (SELECT DISTINCT slug FROM iphone_launch_prices);

UPDATE device_configurations c
SET position = r.pos
FROM (
    SELECT c2.id, (row_number() OVER (PARTITION BY c2.device_id ORDER BY c2.storage_gb) - 1)::int AS pos
      FROM device_configurations c2
      JOIN device_models m ON m.id = c2.device_id
     WHERE m.slug IN (SELECT DISTINCT slug FROM iphone_launch_prices)
) r
WHERE c.id = r.id;
