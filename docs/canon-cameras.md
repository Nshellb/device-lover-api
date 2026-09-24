# Canon DSLR seed data

Verified against Canon's own Camera Museum on 2026-09-23 (Asia/Seoul). Migration `0006_seed_canon_cameras.sql` inserts 17 models and reuses the `canon` brand. Existing rows with the same slug are preserved.

## Data definitions

- `release_month` uses the Museum's release month in Japan, with month precision. It is not a Korean release date. Canon also explicitly identifies the [EOS 5 series release dates as Japanese dates](https://global.canon/en/news/2025/20250728.html).
- `effective_megapixels` stores Canon's approximate effective sensor resolution in megapixels, not total pixels or the JPEG output dimensions. EOS 20D uses the Museum overview's 8.2 MP value (its detailed table uses 8.25 MP).
- `body_weight_g` uses the body-only figure, excluding the battery and memory card. EOS 6D uses the WG (Wi-Fi/GPS) version, and EOS 70D uses the Wi-Fi version.
- `lens_mount` communicates compatible Canon lens families. `EF/EF-S` means EF mount with EF-S compatibility. EOS 10D accepts EF lenses only despite its APS-C sensor.
- `max_continuous_fps` is the maximum advertised still-image continuous shooting rate. EOS 90D's 11 fps requires Live View with One-Shot AF (manual focus also supports it); its viewfinder maximum is 10 fps. That condition is stored in `continuous_shooting_note`.
- `video_spec` summarizes the highest normal movie resolution and its maximum frame rate, using the familiar 30p/60p labels for 29.97p/59.94p where applicable. EOS 6D Mark II's 4K support is explicitly marked as time-lapse only. EOS 5D Mark IV uses DCI 4K (4096 × 2160); EOS 90D uses UHD (3840 × 2160).
- EOS 20Da and EOS 60Da are separate astrophotography variants and are not in this inventory.

## Inventory and primary sources

All models are DSLRs. Each source link is also persisted on the corresponding database row.

| Model | Japan release | Sensor / effective MP | Processor | Compatible lenses | Max fps | Body g | Movie summary | Canon source |
| --- | --- | --- | --- | --- | ---: | ---: | --- | --- |
| EOS 5D | 2005-09 | Full frame / 12.8 | DIGIC II | EF | 3 | 810 | Unsupported | [Camera Museum](https://global.canon/en/c-museum/product/dslr791.html) |
| EOS 5D Mark II | 2008-11 | Full frame / 21.1 | DIGIC 4 | EF | 3.9 | 810 | Full HD 30p | [Camera Museum](https://global.canon/en/c-museum/product/dslr800.html) |
| EOS 5D Mark III | 2012-03 | Full frame / 22.3 | DIGIC 5+ | EF | 6 | 860 | Full HD 30p | [Camera Museum](https://global.canon/en/c-museum/product/dslr808.html) |
| EOS 5D Mark IV | 2016-09 | Full frame / 30.4 | DIGIC 6+ | EF | 7 | 800 | DCI 4K 30p | [Camera Museum](https://global.canon/en/c-museum/product/dslr849.html) |
| EOS 5DS | 2015-06 | Full frame / 50.6 | Dual DIGIC 6 | EF | 5 | 845 | Full HD 30p | [Camera Museum](https://global.canon/en/c-museum/product/dslr828.html) |
| EOS 5DS R | 2015-06 | Full frame / 50.6 | Dual DIGIC 6 | EF | 5 | 845 | Full HD 30p | [Camera Museum](https://global.canon/en/c-museum/product/dslr828.html) |
| EOS 6D | 2012-11 | Full frame / 20.2 | DIGIC 5+ | EF | 4.5 | 680 | Full HD 30p | [Camera Museum](https://global.canon/en/c-museum/product/dslr813.html) |
| EOS 6D Mark II | 2017-08 | Full frame / 26.2 | DIGIC 7 | EF | 6.5 | 685 | Full HD 60p; 4K time-lapse | [Camera Museum](https://global.canon/en/c-museum/product/dslr866.html) |
| EOS 10D | 2003-03 | APS-C / 6.3 | DIGIC | EF | 3 | 790 | Unsupported | [Camera Museum](https://global.canon/en/c-museum/product/dslr783.html) |
| EOS 20D | 2004-09 | APS-C / 8.2 | DIGIC II | EF/EF-S | 5 | 685 | Unsupported | [Camera Museum](https://global.canon/en/c-museum/product/dslr786.html) |
| EOS 30D | 2006-03 | APS-C / 8.2 | DIGIC II | EF/EF-S | 5 | 700 | Unsupported | [Camera Museum](https://global.canon/en/c-museum/product/dslr792.html) |
| EOS 40D | 2007-09 | APS-C / 10.1 | DIGIC III | EF/EF-S | 6.5 | 740 | Unsupported | [Camera Museum](https://global.canon/en/c-museum/product/dslr795.html) |
| EOS 50D | 2008-09 | APS-C / 15.1 | DIGIC 4 | EF/EF-S | 6.3 | 730 | Unsupported | [Camera Museum](https://global.canon/en/c-museum/product/dslr799.html) |
| EOS 60D | 2010-09 | APS-C / 18.0 | DIGIC 4 | EF/EF-S | 5.3 | 675 | Full HD 30p | [Camera Museum (Japanese)](https://global.canon/ja/c-museum/product/dslr805.html) |
| EOS 70D | 2013-08 | APS-C / 20.2 | DIGIC 5+ | EF/EF-S | 7 | 675 | Full HD 30p | [Camera Museum](https://global.canon/en/c-museum/product/dslr816.html) |
| EOS 80D | 2016-03 | APS-C / 24.2 | DIGIC 6 | EF/EF-S | 7 | 650 | Full HD 60p | [Camera Museum](https://global.canon/en/c-museum/product/dslr844.html) |
| EOS 90D | 2019-09 | APS-C / 32.5 | DIGIC 8 | EF/EF-S | 11 (Live View) | 619 | UHD 4K 30p | [Camera Museum](https://global.canon/en/c-museum/product/dslr887.html) |

## Source-specific checks

- EOS 60D: the English Museum specification incorrectly lists 3.7 fps. The Japanese version lists 5.3 fps, so the Japanese page is the stored source.
- EOS 6D Mark II: the Museum page links to its [official specification PDF](https://global.canon/ja/c-museum/wp-content/uploads/2018/08/dslr866_en.pdf). Printed pages 574 and 579 distinguish normal Full HD movies from 4K time-lapse and give the 685 g body-only weight.
- EOS 90D: the Museum page links to its [official supplemental specification PDF](https://global.canon/ja/c-museum/wp-content/uploads/2020/10/dslr887_en.pdf). Printed pages 20, 22 and 26 confirm continuous shooting, movie modes and the 619 g body-only weight. [Canon Latin America's specification table](https://www.cla.canon.com/en/p/eos-90d) additionally identifies One-Shot AF/manual focus for the 11 fps Live View rate.
- EOS 5DS and EOS 5DS R share one Museum page and the summarized specifications stored here. They remain separate models because the R variant cancels the optical low-pass filter effect.
