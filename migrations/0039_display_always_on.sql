-- Always On Display (있음 / 없음 / 미확인) for the main display and every
-- existing sub display, read once from the existing 주요 기능 (displayFeatures)
-- text: 있음 when its value lists Always-On, 없음 when only its detail mentions
-- it (every such note says it is missing: "미지원", "없음", "S7부터 정식 도입"),
-- otherwise 미확인. The Always-On notes move to the new row's detail.
-- 주요 기능 then drops the Always-On entries and every brightness (…니트)
-- mention: AOD has its own row and brightness lives in 피크 밝기.
ALTER TABLE device_spec_values
    DROP CONSTRAINT device_spec_values_spec_key_check,
    ADD CONSTRAINT device_spec_values_spec_key_check CHECK (spec_key IN (
        'operatingSystem', 'weight', 'storage', 'stylus',
        'displayPanel', 'displaySize', 'displayResolution', 'refreshRate',
        'displayPeakBrightness', 'displayLamination', 'displayAntiReflective',
        'displayAlwaysOn', 'displayColorGamut', 'displayContrastRatio', 'displaySupplier',
        'displayFeatures', 'processor', 'memory',
        'wiredConnection', 'speakers', 'rearCameras', 'telephoto', 'digitalZoom',
        'frontCamera', 'videoRecording', 'videoPlayback', 'wireless', 'biometrics',
        'waterResistance', 'sim',
        'sub1DisplayName', 'sub1DisplayPanel', 'sub1DisplaySize', 'sub1DisplayResolution',
        'sub1RefreshRate', 'sub1PeakBrightness', 'sub1DisplayLamination',
        'sub1DisplayAntiReflective', 'sub1DisplayAlwaysOn', 'sub1DisplayColorGamut', 'sub1DisplayContrastRatio', 'sub1DisplaySupplier', 'sub1DisplayFeatures',
        'sub2DisplayName', 'sub2DisplayPanel', 'sub2DisplaySize', 'sub2DisplayResolution',
        'sub2RefreshRate', 'sub2PeakBrightness', 'sub2DisplayLamination',
        'sub2DisplayAntiReflective', 'sub2DisplayAlwaysOn', 'sub2DisplayColorGamut', 'sub2DisplayContrastRatio', 'sub2DisplaySupplier', 'sub2DisplayFeatures'
    ));

-- Keeps (keep = true) or drops the ", " / ". " separated parts of `src` matching
-- `pattern`. Separators inside parentheses or between digits ("2,600",
-- "1.25mm") don't split.
CREATE FUNCTION pg_temp.filter_segments(src text, pattern text, keep boolean, sentences_only boolean DEFAULT false) RETURNS text AS $$
DECLARE
    segs text[] := '{}';
    seps text[] := '{}';
    cur text := '';
    sep text;
    c text;
    depth int := 0;
    i int := 1;
    n int := length(src);
    result text := '';
    pending text := '';
BEGIN
    IF src IS NULL THEN
        RETURN NULL;
    END IF;
    WHILE i <= n LOOP
        c := substr(src, i, 1);
        IF c = '(' THEN
            depth := depth + 1;
        ELSIF c = ')' AND depth > 0 THEN
            depth := depth - 1;
        END IF;
        IF depth = 0 AND (c = '.' OR (c = ',' AND NOT sentences_only)) AND (i = n OR substr(src, i + 1, 1) ~ '\s') THEN
            sep := c;
            i := i + 1;
            WHILE i <= n AND substr(src, i, 1) ~ '\s' LOOP
                sep := sep || substr(src, i, 1);
                i := i + 1;
            END LOOP;
            segs := segs || cur;
            seps := seps || sep;
            cur := '';
            CONTINUE;
        END IF;
        cur := cur || c;
        i := i + 1;
    END LOOP;
    segs := segs || cur;
    seps := seps || ''::text;

    FOR j IN 1 .. array_length(segs, 1) LOOP
        CONTINUE WHEN btrim(segs[j]) = '' OR (segs[j] ~* pattern) <> keep;
        result := result || pending || segs[j];
        pending := seps[j];
    END LOOP;
    RETURN nullif(btrim(result), '');
END
$$ LANGUAGE plpgsql;

CREATE TEMP TABLE aod_features ON COMMIT DROP AS
SELECT device_id, spec_key, display_value, detail,
       replace(spec_key, 'Features', 'AlwaysOn') AS aod_key,
       display_value ~* 'always[- ]?on' AS listed,
       -- The value was nothing but "Always-On 디스플레이": its whole detail is about AOD.
       display_value ~* 'always[- ]?on|상시 ?켜짐'
           AND pg_temp.filter_segments(display_value, '^\s*always[- ]?on\s*(display|디스플레이)\s*$', false) IS NULL
           AS aod_only
  FROM device_spec_values
 WHERE spec_key ~ '^(sub[0-9])?[dD]isplayFeatures$';

INSERT INTO device_spec_values (device_id, spec_key, display_value, detail)
SELECT f.device_id, f.aod_key,
       CASE
           WHEN f.listed THEN '있음'
           WHEN f.detail ~* 'always[- ]?on|상시 ?켜짐' THEN '없음'
           ELSE '미확인'
       END,
       pg_temp.filter_segments(
           CASE WHEN f.aod_only THEN f.detail
                -- The 나이트 클록 note is one long sentence with commas: split on sentences only.
                WHEN f.detail ~ '나이트 클록' THEN pg_temp.filter_segments(f.detail, 'always[- ]?on|상시 ?켜짐', true, true)
                ELSE pg_temp.filter_segments(f.detail, 'always[- ]?on|상시 ?켜짐', true) END,
           '니트', false)
  FROM aod_features f;

-- Devices without a 주요 기능 row (sub displays always have one; kept for safety).
INSERT INTO device_spec_values (device_id, spec_key, display_value)
SELECT s.device_id, replace(s.spec_key, 'DisplaySize', 'DisplayAlwaysOn'), '미확인'
  FROM device_spec_values s
 WHERE s.spec_key ~ '^(sub[0-9])?[dD]isplaySize$'
ON CONFLICT (device_id, spec_key) DO NOTHING;

UPDATE device_spec_values v
SET display_value = coalesce(
        pg_temp.filter_segments(v.display_value, '니트|^\s*always[- ]?on\s*(display|디스플레이)\s*$', false),
        '미확인'
    ),
    detail = CASE WHEN f.aod_only THEN NULL
                  WHEN v.detail ~ '나이트 클록' THEN pg_temp.filter_segments(v.detail, '니트|always[- ]?on|상시 ?켜짐', false, true)
                  ELSE pg_temp.filter_segments(v.detail, '니트|always[- ]?on|상시 ?켜짐', false) END,
    updated_at = now()
  FROM aod_features f
 WHERE v.device_id = f.device_id AND v.spec_key = f.spec_key
   AND (coalesce(v.detail, '') || ' ' || v.display_value) ~* '니트|always[- ]?on|상시 ?켜짐';

DROP FUNCTION pg_temp.filter_segments(text, text, boolean, boolean);
