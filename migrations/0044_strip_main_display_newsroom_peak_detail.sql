-- Peak brightness detail "메인 디스플레이, 삼성 글로벌 뉴스룸 기준" removed as a whole.
UPDATE device_spec_values
SET detail = NULL, updated_at = now()
WHERE spec_key = 'displayPeakBrightness'
  AND btrim(detail) = '메인 디스플레이, 삼성 글로벌 뉴스룸 기준';
