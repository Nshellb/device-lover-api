-- Canon Camera Museum data checked on 2026-09-23 (Asia/Seoul).
-- release_month is the Japan release month, not an invented day or a KR release date.
-- body_weight_g excludes the battery and memory card; EOS 6D uses the WG variant,
-- and EOS 70D uses the Wi-Fi variant. See docs/canon-cameras.md for source notes.
-- EOS 20Da / EOS 60Da are separate astrophotography variants and are out of scope.
-- Preserve existing brands and camera records when the seed is replayed.
INSERT INTO brands (slug, name)
VALUES ('canon', 'Canon')
ON CONFLICT (slug) DO NOTHING;

INSERT INTO camera_models (
    brand_id,
    slug,
    name,
    series,
    release_month,
    camera_type,
    sensor_format,
    effective_megapixels,
    image_processor,
    lens_mount,
    max_continuous_fps,
    continuous_shooting_note,
    video_spec,
    body_weight_g,
    source_url,
    source_title,
    checked_at
)
SELECT
    brands.id,
    seed.slug,
    seed.name,
    seed.series,
    seed.release_month,
    'DSLR',
    seed.sensor_format,
    seed.effective_megapixels,
    seed.image_processor,
    seed.lens_mount,
    seed.max_continuous_fps,
    seed.continuous_shooting_note,
    seed.video_spec,
    seed.body_weight_g,
    seed.source_url,
    'Canon Camera Museum - ' || seed.name,
    TIMESTAMPTZ '2026-09-23 03:45:04+09'
FROM brands
CROSS JOIN (VALUES
    ('canon-eos-5d', 'EOS 5D', 'EOS 5D', '2005-09', 'full_frame', 12.8,
     'DIGIC II', 'EF', 3.0, NULL, '미지원', 810,
     'https://global.canon/en/c-museum/product/dslr791.html'),
    ('canon-eos-5d-mark-ii', 'EOS 5D Mark II', 'EOS 5D', '2008-11', 'full_frame', 21.1,
     'DIGIC 4', 'EF', 3.9, NULL, 'Full HD 30p', 810,
     'https://global.canon/en/c-museum/product/dslr800.html'),
    ('canon-eos-5d-mark-iii', 'EOS 5D Mark III', 'EOS 5D', '2012-03', 'full_frame', 22.3,
     'DIGIC 5+', 'EF', 6.0, NULL, 'Full HD 30p', 860,
     'https://global.canon/en/c-museum/product/dslr808.html'),
    ('canon-eos-5d-mark-iv', 'EOS 5D Mark IV', 'EOS 5D', '2016-09', 'full_frame', 30.4,
     'DIGIC 6+', 'EF', 7.0, NULL, 'DCI 4K 30p', 800,
     'https://global.canon/en/c-museum/product/dslr849.html'),
    ('canon-eos-5ds', 'EOS 5DS', 'EOS 5D', '2015-06', 'full_frame', 50.6,
     'Dual DIGIC 6', 'EF', 5.0, NULL, 'Full HD 30p', 845,
     'https://global.canon/en/c-museum/product/dslr828.html'),
    ('canon-eos-5ds-r', 'EOS 5DS R', 'EOS 5D', '2015-06', 'full_frame', 50.6,
     'Dual DIGIC 6', 'EF', 5.0, NULL, 'Full HD 30p', 845,
     'https://global.canon/en/c-museum/product/dslr828.html'),
    ('canon-eos-6d', 'EOS 6D', 'EOS 6D', '2012-11', 'full_frame', 20.2,
     'DIGIC 5+', 'EF', 4.5, NULL, 'Full HD 30p', 680,
     'https://global.canon/en/c-museum/product/dslr813.html'),
    ('canon-eos-6d-mark-ii', 'EOS 6D Mark II', 'EOS 6D', '2017-08', 'full_frame', 26.2,
     'DIGIC 7', 'EF', 6.5, NULL, 'Full HD 60p (4K 타임랩스 지원)', 685,
     'https://global.canon/en/c-museum/product/dslr866.html'),
    ('canon-eos-10d', 'EOS 10D', 'EOS x0D', '2003-03', 'aps_c', 6.3,
     'DIGIC', 'EF', 3.0, NULL, '미지원', 790,
     'https://global.canon/en/c-museum/product/dslr783.html'),
    ('canon-eos-20d', 'EOS 20D', 'EOS x0D', '2004-09', 'aps_c', 8.2,
     'DIGIC II', 'EF/EF-S', 5.0, NULL, '미지원', 685,
     'https://global.canon/en/c-museum/product/dslr786.html'),
    ('canon-eos-30d', 'EOS 30D', 'EOS x0D', '2006-03', 'aps_c', 8.2,
     'DIGIC II', 'EF/EF-S', 5.0, NULL, '미지원', 700,
     'https://global.canon/en/c-museum/product/dslr792.html'),
    ('canon-eos-40d', 'EOS 40D', 'EOS x0D', '2007-09', 'aps_c', 10.1,
     'DIGIC III', 'EF/EF-S', 6.5, NULL, '미지원', 740,
     'https://global.canon/en/c-museum/product/dslr795.html'),
    ('canon-eos-50d', 'EOS 50D', 'EOS x0D', '2008-09', 'aps_c', 15.1,
     'DIGIC 4', 'EF/EF-S', 6.3, NULL, '미지원', 730,
     'https://global.canon/en/c-museum/product/dslr799.html'),
    -- The English Museum page says 3.7 fps; the Japanese specification says 5.3 fps.
    ('canon-eos-60d', 'EOS 60D', 'EOS x0D', '2010-09', 'aps_c', 18.0,
     'DIGIC 4', 'EF/EF-S', 5.3, NULL, 'Full HD 30p', 675,
     'https://global.canon/ja/c-museum/product/dslr805.html'),
    ('canon-eos-70d', 'EOS 70D', 'EOS x0D', '2013-08', 'aps_c', 20.2,
     'DIGIC 5+', 'EF/EF-S', 7.0, NULL, 'Full HD 30p', 675,
     'https://global.canon/en/c-museum/product/dslr816.html'),
    ('canon-eos-80d', 'EOS 80D', 'EOS x0D', '2016-03', 'aps_c', 24.2,
     'DIGIC 6', 'EF/EF-S', 7.0, NULL, 'Full HD 60p', 650,
     'https://global.canon/en/c-museum/product/dslr844.html'),
    ('canon-eos-90d', 'EOS 90D', 'EOS x0D', '2019-09', 'aps_c', 32.5,
     'DIGIC 8', 'EF/EF-S', 11.0, '라이브뷰 · One-Shot AF 기준 (뷰파인더 최대 10fps)',
     '4K UHD 30p', 619,
     'https://global.canon/en/c-museum/product/dslr887.html')
) AS seed (
    slug, name, series, release_month, sensor_format, effective_megapixels,
    image_processor, lens_mount, max_continuous_fps, continuous_shooting_note,
    video_spec, body_weight_g, source_url
)
WHERE brands.slug = 'canon'
ON CONFLICT (slug) DO NOTHING;
