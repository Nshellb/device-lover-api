-- Fills the iPhone launch-price gaps left by 0046 from 나무위키 (cross-checked
-- against the values 0046 already holds, which matched wherever both had one):
--   * iPhone XR 128GB, XS 256GB, XS Max 256GB (Apple Korea prices; XR 128GB uses
--     the 1,060,000 figure consistent with the 256GB value already stored)
--   * iPhone 17e 512GB (Korean launch price of 2026-03-03)
--   * iPhone 18 Pro / Pro Max all tiers (the aggregator values 0046 left out
--     are identical on 나무위키)
-- 나무위키 lists iPhone 17e's US price as $699 / $899 (a later price change, as
-- with its Korean 1,150,000 / 1,450,000 from 2026-09-10); Apple's launch
-- announcement said $599 / $799, which stays.
UPDATE device_configurations c
SET price_krw = coalesce(p.price_krw, c.price_krw),
    price_usd = coalesce(p.price_usd, c.price_usd),
    updated_at = now()
FROM (VALUES
    ('iphone-xr', 128, 1060000, NULL::double precision),
    ('iphone-xs', 256, 1580000, NULL),
    ('iphone-xs-max', 256, 1710000, NULL),
    ('iphone-17e', 512, 1290000, NULL),
    ('iphone-18-pro', 256, 1990000, 1199),
    ('iphone-18-pro', 512, 2290000, 1399),
    ('iphone-18-pro', 1024, 2890000, 1799),
    ('iphone-18-pro', 2048, 3790000, 2399),
    ('iphone-18-pro-max', 256, 2190000, 1299),
    ('iphone-18-pro-max', 512, 2490000, 1499),
    ('iphone-18-pro-max', 1024, 3090000, 1899),
    ('iphone-18-pro-max', 2048, 3990000, 2499)
) AS p(slug, storage_gb, price_krw, price_usd)
JOIN device_models m ON m.slug = p.slug
WHERE c.device_id = m.id AND c.storage_gb = p.storage_gb;

INSERT INTO device_configurations (device_id, label, storage_gb, price_krw, price_usd, position)
SELECT m.id,
       CASE WHEN p.storage_gb >= 1024 THEN (p.storage_gb / 1024) || 'TB' ELSE p.storage_gb || 'GB' END,
       p.storage_gb, p.price_krw, p.price_usd,
       2000 + (row_number() OVER (PARTITION BY m.id ORDER BY p.storage_gb))::int
FROM (VALUES
    ('iphone-18-pro', 512, 2290000, 1399::double precision),
    ('iphone-18-pro', 1024, 2890000, 1799),
    ('iphone-18-pro', 2048, 3790000, 2399),
    ('iphone-18-pro-max', 256, 2190000, 1299),
    ('iphone-18-pro-max', 512, 2490000, 1499),
    ('iphone-18-pro-max', 1024, 3090000, 1899),
    ('iphone-18-pro-max', 2048, 3990000, 2499)
) AS p(slug, storage_gb, price_krw, price_usd)
JOIN device_models m ON m.slug = p.slug
WHERE NOT EXISTS (
    SELECT 1 FROM device_configurations c
     WHERE c.device_id = m.id AND c.storage_gb = p.storage_gb
);

UPDATE device_configurations c
SET position = c.position + 1000
FROM device_models m
WHERE m.id = c.device_id AND c.position < 1000
  AND m.slug IN ('iphone-18-pro', 'iphone-18-pro-max');

UPDATE device_configurations c
SET position = r.pos
FROM (
    SELECT c2.id, (row_number() OVER (PARTITION BY c2.device_id ORDER BY c2.storage_gb) - 1)::int AS pos
      FROM device_configurations c2
      JOIN device_models m ON m.id = c2.device_id
     WHERE m.slug IN ('iphone-18-pro', 'iphone-18-pro-max')
) r
WHERE c.id = r.id;
