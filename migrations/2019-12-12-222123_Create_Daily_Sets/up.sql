-- Your SQL goes here
CREATE OR REPLACE VIEW bb.v_card_ratings AS
  SELECT
    r.card_id
    , SUM(CASE WHEN rating < 0.5 THEN 1 ELSE 0 END) AS "downvotes"
    , SUM(CASE WHEN rating > 0.5 THEN 1 ELSE 0 END) AS "upvotes"
    , AVG(CASE WHEN rating < 0.5 THEN rating ELSE NULL END) AS "low_votes"
    , AVG(CASE WHEN rating > 0.5 THEN rating ELSE NULL END) AS "high_votes"
    , AVG(rating) AS "average_rating"
    , COUNT(*) AS "total_votes"
    FROM bb.user_card_rating as r
   GROUP BY r.card_id;

CREATE OR REPLACE PROCEDURE bb.generate_daily_set(top_n_cards INT)
AS $$
BEGIN
  with
    inserted_set AS (
      INSERT INTO bb.parent_set (name)
        VALUES ('Daily Set: ' || TO_CHAR(NOW()::DATE, 'yyyy-mm-dd'))
      RETURNING id)
    , card_ratings AS (
      SELECT
        r.card_id
        , SUM(CASE WHEN rating < 0.5 THEN 1 ELSE 0 END) AS "downvotes"
        , SUM(CASE WHEN rating > 0.5 THEN 1 ELSE 0 END) AS "upvotes"
        , AVG(CASE WHEN rating < 0.5 THEN rating ELSE NULL END) AS "low_votes"
        , AVG(CASE WHEN rating > 0.5 THEN rating ELSE NULL END) AS "high_votes"
      FROM bb.user_card_rating as r
      GROUP BY r.card_id
    )
    , top_cards AS (
      SELECT c.id
      FROM bb.card AS c
        INNER JOIN card_ratings AS r ON r.card_id = c.id
      WHERE DATE_PART('DAY', created_date - NOW()::timestamp) <= 1
        AND c.submitted_by_user_id IS NOT NULL
      ORDER BY ( r.upvotes * (r.high_votes - 0.5) ) + ( r.downvotes * (r.low_votes - 0.5) )
      LIMIT top_n_cards
    )
  INSERT INTO bb.parent_set_card (parent_set_id, card_id)
    SELECT
      s.id,
      c.id
    FROM inserted_set AS s
      CROSS JOIN top_cards as c;
END;
$$
LANGUAGE 'plpgsql';
