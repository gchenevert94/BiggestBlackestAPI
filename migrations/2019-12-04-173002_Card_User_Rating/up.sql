-- Your SQL goes here
CREATE TABLE bb.user_card_rating (
  user_id SERIAL NOT NULL CONSTRAINT FK_user_card_rating_user REFERENCES bb."user"(id),
  card_id SERIAL NOT NULL CONSTRAINT FK_user_card_rating_card REFERENCES bb.card(id),
  rating REAL NOT NULL,
  created_date TIMESTAMP NOT NULL DEFAULT NOW(),
  is_active BOOLEAN NOT NULL DEFAULT TRUE,
  last_modified TIMESTAMP NOT NULL DEFAULT NOW(),
  CONSTRAINT PK_user_card_rating PRIMARY KEY (user_id, card_id)
);


ALTER TABLE bb.card ADD average_rating REAL;
ALTER TABLE bb.card ADD total_votes INT NOT NULL DEFAULT 0;
