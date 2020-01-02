-- Your SQL goes here
CREATE OR REPLACE FUNCTION bb.user_rate_card(
	in_user_id integer,
	in_card_id integer,
	in_rating real,
	OUT out_total_votes integer,
	OUT out_average_rating real)
RETURNS record
AS $$
DECLARE
  existing_rating bb.user_card_rating.rating%TYPE;
BEGIN
  SELECT r.rating
    FROM bb.user_card_rating AS r
    WHERE
      r.user_id = in_user_id
      AND r.card_id = in_card_id
  INTO existing_rating;

  IF existing_rating IS NULL THEN
    INSERT INTO bb.user_card_rating (user_id, card_id, rating)
    VALUES (in_user_id, in_card_id, in_rating);

    UPDATE ONLY bb.card
       SET
    average_rating = ((total_votes * COALESCE(average_rating, 0)) + in_rating) / (total_votes + 1)
    , total_votes = total_votes + 1
     WHERE id = in_card_id
           RETURNING average_rating, total_votes INTO out_average_rating, out_total_votes;
  ELSE
    UPDATE ONLY bb.user_card_rating
      SET
        rating = in_rating
      WHERE user_id = in_user_id AND card_id = in_card_id;
    UPDATE ONLY bb.card
      SET
        average_rating = average_rating + ((in_rating - existing_rating) / total_votes)
      WHERE id = in_card_id
      RETURNING average_rating, total_votes INTO out_average_rating, out_total_votes;
  END IF;
END
$$
LANGUAGE 'plpgsql';
