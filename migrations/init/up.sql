-- For database "biggest_blackest"
CREATE SCHEMA "bb";

CREATE TABLE "bb"."ParentSet" (
  Id SERIAL NOT NULL CONSTRAINT PK_ParentSet PRIMARY KEY,
  Name TEXT NOT NULL CONSTRAINT UX_ParentSet UNIQUE
);

CREATE TABLE "bb"."Card" (
  Id SERIAL NOT NULL CONSTRAINT PK_Card PRIMARY KEY,
  IsBlack BOOLEAN NOT NULL,
  ParentSetId INT NOT NULL CONSTRAINT FK_Card_ParentSet REFERENCES "bb"."ParentSet",
  FormatText TEXT NOT NULL
);

CREATE TABLE "bb"."DrawCard" (
  Id SERIAL NOT NULL CONSTRAINT PK_DrawCard PRIMARY KEY,
  CardId INT NOT NULL CONSTRAINT FK_DrawCard REFERENCES "bb"."Card"(Id),
  DrawDate TIMESTAMP NOT NULL
);

CREATE TYPE "bb"."IdTable" AS RANGE (
  SUBTYPE = INT
);

CREATE OR REPLACE PROCEDURE "bb"."GetCardRange"(sets "bb"."IdTable", is_black BOOLEAN)
AS $$
  SELECT
    COUNT(*)
  FROM "bb"."Card"
    INNER JOIN @sets AS s ON s.Id = c.ParentSetId
  WHERE IsBlack = @is_black
$$;

CREATE OR REPLACE PROCEDURE "bb"."DrawNthCard"(n INT, sets "bb"."IdTable", is_black BOOLEAN)
AS $$
  SELECT
    c.Id,
    c.IsBlack,
    p.ParentSetId,
    p.ParentSet,
    c.FormatText
  FROM "bb"."Card" AS c
    INNER JOIN @sets AS s ON s.Id = c.ParentSetId
    INNER JOIN "bb"."ParentSet" AS p ON p.Id = c.ParentSetId
  WHERE IsBlack = @is_black AND ROW_NUMBER() = @n
$$;
