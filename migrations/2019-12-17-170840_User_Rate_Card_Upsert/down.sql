-- This file should undo anything in `up.sql`
CREATE OR REPLACE FUNCTION bb.user_rate_card(
	in_user_id integer,
	in_card_id integer,
	in_rating real,
	OUT out_total_votes integer,
	OUT out_average_rating real)
  RETURNS record
  LANGUAGE 'plpgsql'

  COST 100
  VOLATILE 
  
AS $BODY$
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
$BODY$;
