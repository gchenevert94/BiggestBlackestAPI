-- Your SQL goes here
DROP PROCEDURE "bb"."GetCardRange";
DROP PROCEDURE "bb"."DrawNthCard";
DROP PROCEDURE "bb"."GetCardById";
DROP PROCEDURE "bb"."GetDecks";
DROP PROCEDURE "bb"."GetCardsByDeck";

DROP TYPE bb."IdTable";

ALTER TABLE bb.card RENAME isblack TO is_black;
ALTER TABLE bb.card RENAME formattext TO format_text;
ALTER TABLE bb.card RENAME isactive TO is_active;
ALTER TABLE bb.card RENAME lastmodified TO last_modified;

ALTER TABLE bb.parent_set RENAME isactive TO is_active;
ALTER TABLE bb.parent_set RENAME lastmodified TO last_modified;

ALTER TABLE bb.parent_set_card RENAME parentsetid TO parent_set_id;
ALTER TABLE bb.parent_set_card RENAME cardid TO card_id;
ALTER TABLE bb.parent_set_card RENAME isactive TO is_active;
ALTER TABLE bb.parent_set_card RENAME lastmodified TO last_modified;

ALTER TABLE bb."user" RENAME isactive TO is_active;
ALTER TABLE bb."user" RENAME lastmodified TO last_modified;