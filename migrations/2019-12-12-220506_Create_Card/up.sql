-- Your SQL goes here
ALTER TABLE bb.card ADD created_date TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT NOW();
ALTER TABLE bb.card ADD submitted_by_user_id INT CONSTRAINT FK_card_user REFERENCES bb.user(id);

CREATE OR REPLACE FUNCTION bb.create_card(
  format_text TEXT,
  is_black BOOLEAN,
  submitted_by_user_id INT
) RETURNS INTEGER
AS $$
  INSERT INTO bb.card
    (format_text, is_black, submitted_by_user_id)
  VALUES (format_text, is_black, submitted_by_user_id)
  RETURNING id;
$$
LANGUAGE SQL;
