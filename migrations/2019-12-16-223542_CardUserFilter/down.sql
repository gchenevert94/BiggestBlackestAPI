-- This file should undo anything in `up.sql`
CREATE OR REPLACE FUNCTION bb.get_cards(
	search text,
	filter_black boolean,
	previous_cursor integer,
	n_cards integer,
	card_sets integer[],
	get_random boolean,
	random_seed real)
    RETURNS TABLE(id integer, format_text text, is_black boolean, parent_set_id integer, parent_set_name text, total_votes integer, average_rating real) 
    LANGUAGE 'plpgsql'

    COST 100
    VOLATILE 
    ROWS 1000
    
AS $BODY$
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
$BODY$;

