-- Your SQL goes here
ALTER TABLE bb.user_card_combination_rating ADD ordinal INT;

ALTER TABLE bb.user_card_combination_rating
  DROP CONSTRAINT PK_user_card_combination_rating;
ALTER TABLE bb.user_card_combination_rating
  ADD CONSTRAINT PK_user_card_combination_rating
  PRIMARY KEY (user_id, white_card_id, black_card_id, ordinal);

CREATE OR REPLACE PROCEDURE bb.user_rate_card_combination(user_id INT, white_card_id INT, black_card_id INT, rating REAL, ordinal INT)
AS $$
BEGIN
  INSERT INTO
    bb.user_card_combination_rating
      (user_id, white_card_id, black_card_id, rating, ordinal)
  VALUES (user_id, white_card_id, black_card_id, rating, ordinal)
    ON CONFLICT
    ON CONSTRAINT PK_user_card_combination_rating
    DO UPDATE SET rating = EXCLUDED.rating, last_modified = NOW();
END;
$$
LANGUAGE 'plpgsql';