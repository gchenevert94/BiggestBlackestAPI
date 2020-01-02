-- This file should undo anything in `up.sql`
ALTER TABLE bb.card DROP COLUMN created_date;
ALTER TABLE bb.card DROP COLUMN submitted_by_user_id;

DROP FUNCTION bb.create_card;
