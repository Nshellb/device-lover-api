-- Dimensions are stored as a device held upright: 세로 (height) is the longest,
-- 가로 (width) the second, 두께 (depth) the thinnest, regardless of the order
-- they were entered in. Sorting the three values avoids the float drift of a
-- sum-minus-extremes median.
UPDATE device_dimensions d
   SET height_mm = s.sizes[1], width_mm = s.sizes[2], depth_mm = s.sizes[3]
  FROM (
      SELECT id, array_agg(size ORDER BY size DESC) AS sizes
        FROM device_dimensions,
             unnest(ARRAY[width_mm, height_mm, depth_mm]) AS size
       GROUP BY id
  ) s
 WHERE d.id = s.id
   AND (d.height_mm < d.width_mm OR d.width_mm < d.depth_mm);
