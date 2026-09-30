-- speakers is now entered as a type (스피커 없음 / 모노 / 스테레오) plus a count and
-- stored as '스테레오 2개' / '모노 1개' / '스피커 없음'. Legacy rows carried only the
-- type, so the count defaults to the usual one (mono 1, stereo 2).
UPDATE device_spec_values
SET display_value = '스테레오 2개',
    detail = CASE WHEN detail = '스테레오' THEN NULL ELSE detail END
WHERE spec_key = 'speakers'
  AND (display_value IN ('스테레오', '스테레오 스피커')
       OR (display_value = '2개' AND detail = '스테레오'));

UPDATE device_spec_values
SET display_value = '모노 1개'
WHERE spec_key = 'speakers'
  AND display_value IN ('모노', '모노 스피커');
