-- This file should undo anything in `up.sql`
DROP PROCEDURE bb.user_rate_card_combination;
DROP TABLE bb.user_card_combination_rating;
ALTER TABLE bb.card DROP CONSTRAINT UX_card_color_text;
