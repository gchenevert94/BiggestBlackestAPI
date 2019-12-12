-- Your SQL goes here
CREATE OR REPLACE FUNCTION bb.get_sets(
  search TEXT,
  n_results INT,
  cursor INT
) RETURNS TABLE (
  id INT,
  name TEXT
)
AS $$
BEGIN
  RETURN QUERY SELECT
    s.id,
    s.name
  FROM bb.parent_set AS s
  WHERE 
    (search IS NULL OR text_searchable_name @@ to_tsquery('english', search))
    AND (cursor IS NULL OR s.id > cursor)
  ORDER by s.id
  LIMIT n_results;
END;
$$
LANGUAGE 'plpgsql';