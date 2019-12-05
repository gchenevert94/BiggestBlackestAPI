-- This file should undo anything in `up.sql`
ALTER TABLE bb.card DROP COLUMN average_rating;
ALTER TABLE bb.card DROP COLUMN total_votes;

DROP TABLE bb.user_card_rating;
