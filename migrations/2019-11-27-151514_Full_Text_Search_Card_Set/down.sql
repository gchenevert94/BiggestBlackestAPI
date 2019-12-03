-- This file should undo anything in `up.sql`
ALTER TABLE bb.card DROP COLUMN text_searchable_format_text;
DROP TRIGGER tsvectorupdate_card ON bb.card;

ALTER TABLE bb.parent_set DROP COLUMN text_searchable_name;
DROP TRIGGER tsvectorupdate_parent_set ON bb.parent_set;
