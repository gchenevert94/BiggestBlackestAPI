-- Your SQL goes here
ALTER TABLE bb.card ADD CONSTRAINT UX_card_color_text UNIQUE (is_black, format_text);

CREATE TABLE IF NOT EXISTS bb.user_card_combination_rating (
  user_id INT NOT NULL CONSTRAINT FK_user_card_combination_rating_user REFERENCES bb."user"(id),
  white_card_id INT NOT NULL CONSTRAINT FK_user_card_combination_rating_white_card REFERENCES bb.card(id),
  black_card_id INT NOT NULL CONSTRAINT FK_user_card_combination_rating_black_card REFERENCES bb.card(id),
  rating REAL NOT NULL,
  last_modified TIMESTAMP NOT NULL DEFAULT NOW(),
  is_active BOOLEAN NOT NULL DEFAULT TRUE,
  CONSTRAINT PK_user_card_combination_rating PRIMARY KEY (user_id,white_card_id,black_card_id)
);

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
