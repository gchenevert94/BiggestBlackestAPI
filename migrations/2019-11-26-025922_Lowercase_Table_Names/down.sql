-- This file should undo anything in `up.sql`
ALTER TABLE bb.card RENAME TO "Card";
ALTER TABLE bb.draw_card RENAME TO "DrawCard";
ALTER TABLE bb.parent_set RENAME TO "ParentSet";
ALTER TABLE bb.parent_set_card RENAME TO "ParentSetCard";
ALTER TABLE bb."user" RENAME TO "User"
