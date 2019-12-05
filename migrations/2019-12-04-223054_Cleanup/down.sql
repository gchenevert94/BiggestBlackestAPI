-- This file should undo anything in `up.plpgsql`
CREATE OR REPLACE PROCEDURE "bb"."GetCardRange"() AS $$ BEGIN RAISE EXCEPTION 'INVALID PROCEDURE'; END; $$ LANGUAGE 'plpgsql';
CREATE OR REPLACE PROCEDURE "bb"."DrawNthCard"() AS $$ BEGIN RAISE EXCEPTION 'INVALID PROCEDURE'; END; $$ LANGUAGE 'plpgsql' ;
CREATE OR REPLACE PROCEDURE "bb"."GetCardById"() AS $$ BEGIN RAISE EXCEPTION 'INVALID PROCEDURE'; END; $$ LANGUAGE 'plpgsql';
CREATE OR REPLACE PROCEDURE "bb"."GetDecks"() AS $$ BEGIN RAISE EXCEPTION 'INVALID PROCEDURE'; END; $$ LANGUAGE 'plpgsql';
CREATE OR REPLACE PROCEDURE "bb"."GetCardsByDeck"() AS $$ BEGIN RAISE EXCEPTION 'INVALID PROCEDURE'; END; $$ LANGUAGE 'plpgsql';

CREATE TYPE bb."IdTable" AS RANGE(
  SUBTYPE=int4,
  SUBTYPE_OPCLASS = int4_ops
);

ALTER TABLE bb.card RENAME is_black TO isblack;
ALTER TABLE bb.card RENAME format_text TO formattext;
ALTER TABLE bb.card RENAME is_active TO isactive;
ALTER TABLE bb.card RENAME last_modified TO lastmodified;

ALTER TABLE bb.parent_set RENAME is_active TO isactive;
ALTER TABLE bb.parent_set RENAME last_modified TO lastmodified;

ALTER TABLE bb.parent_set_card RENAME parent_set_id TO parentsetid;
ALTER TABLE bb.parent_set_card RENAME card_id TO cardid;
ALTER TABLE bb.parent_set_card RENAME is_active TO isactive;
ALTER TABLE bb.parent_set_card RENAME last_modified TO lastmodified;

ALTER TABLE bb."user" RENAME is_active TO isactive;
ALTER TABLE bb."user" RENAME last_modified TO lastmodified;