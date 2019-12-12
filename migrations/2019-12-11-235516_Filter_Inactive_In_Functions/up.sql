-- Your SQL goes here
CREATE OR REPLACE FUNCTION bb.get_cards(
  search TEXT,
  filter_black BOOLEAN, 
  previous_cursor INT,
  n_cards INT,
  card_sets INT[],
  get_random BOOLEAN,
  random_seed REAL
) RETURNS TABLE (
  id INT,
  format_text TEXT,
  is_black BOOLEAN,
  parent_set_id INT,
  parent_set_name TEXT,
  total_votes INT,
  average_rating REAL
)
AS $$
BEGIN

  IF get_random THEN

    IF random_seed IS NOT NULL THEN
      PERFORM setseed(random_seed);
    END IF;

    RETURN QUERY SELECT
        c.id,
        c.format_text,
        c.is_black,
        p.parent_set_id AS "parent_set_id",
        ps.name AS "parent_set_name",
        c.total_votes,
        c.average_rating
      FROM bb.card AS c
        INNER JOIN bb.parent_set_card AS p ON p.card_id = c.id
        INNER JOIN bb.parent_set AS ps ON ps.id = p.parent_set_id
      WHERE
        (search IS NULL OR to_tsquery('english', search) @@ text_searchable_format_text)
        AND (filter_black IS NULL OR c.is_black = filter_black)
        AND (card_sets IS NULL OR p.parent_set_id = ANY(card_sets))
        AND c.is_active = true AND p.is_active = true AND ps.is_active = true
      ORDER BY RANDOM()
      LIMIT n_cards OFFSET previous_cursor;
  ELSE

    RETURN QUERY SELECT
      c.id,
      c.format_text AS "format_text",
      c.is_black AS "is_black",
      p.parent_set_id AS "parent_set_id",
      ps.name AS "parent_set_name",
      c.total_votes,
      c.average_rating
    FROM bb.card AS c
      INNER JOIN bb.parent_set_card AS p ON p.card_id = c.id
      INNER JOIN bb.parent_set AS ps ON ps.id = p.parent_set_id
    WHERE
      (search IS NULL OR to_tsquery('english', search) @@ text_searchable_format_text)
      AND (filter_black IS NULL OR c.is_black = filter_black)
      AND (card_sets IS NULL OR p.parent_set_id = ANY(card_sets))
      AND c.is_active = true AND p.is_active = true AND ps.is_active = true
      AND (previous_cursor IS NULL OR c.id > previous_cursor)
    ORDER BY c.id
    LIMIT n_cards;

  END IF;
END;
$$
LANGUAGE 'plpgsql';

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
    AND s.is_active = true
    ORDER by s.id
    LIMIT n_results;
END;
$$
LANGUAGE 'plpgsql';
