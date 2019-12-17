-- This file should undo anything in `up.sql`

CREATE OR REPLACE PROCEDURE bb.user_rate_card_combination(user_id INT, white_card_id INT, black_card_id INT, rating REAL)
AS $$
BEGIN
  INSERT INTO
    bb.user_card_combination_rating
      (user_id, white_card_id, black_card_id, rating)
  VALUES (user_id, white_card_id, black_card_id, rating)
    ON CONFLICT
    ON CONSTRAINT PK_user_card_combination_rating
    DO UPDATE SET rating = EXCLUDED.rating, last_modified = NOW();
END;
$$
LANGUAGE 'plpgsql';

ALTER TABLE bb.user_card_combination_rating
  DROP CONSTRAINT PK_user_card_combination_rating;

ALTER TABLE bb.user_card_combination_rating DROP ordinal;

ALTER TABLE bb.user_card_combination_rating
  ADD CONSTRAINT PK_user_card_combination_rating
  PRIMARY KEY (user_id, white_card_id, black_card_id);