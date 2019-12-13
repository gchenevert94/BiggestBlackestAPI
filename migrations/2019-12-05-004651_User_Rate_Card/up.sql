-- Your SQL goes here
CREATE OR REPLACE FUNCTION bb.user_rate_card(
  in_user_id INT,
  in_card_id INT,
  in_rating REAL,
  out_total_votes OUT INT,
  out_average_rating OUT REAL
)
AS $$
BEGIN
  INSERT INTO bb.user_card_rating (user_id, card_id, rating)
  VALUES (in_user_id, in_card_id, in_rating);

  UPDATE ONLY bb.card
  SET
    average_rating = ((total_votes * COALESCE(average_rating, 0)) + in_rating) / (total_votes + 1)
    , total_votes = total_votes + 1
  WHERE id = in_card_id
  RETURNING average_rating, total_votes INTO out_average_rating, out_total_votes;
END;
$$
LANGUAGE 'plpgsql';
