-- This file should undo anything in `up.sql`
DROP TABLE "bb"."ParentSetCard";
ALTER TABLE "bb"."Card" ADD COLUMN ParentSetId INT NOT NULL CONSTRAINT FK_Card_ParentSet REFERENCES "bb"."ParentSet"(Id) DEFAULT 1;
ALTER TABLE "bb"."Card" ALTER COLUMN ParentSetId DROP DEFAULT;
