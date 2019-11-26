-- For database "biggest_blackest"
CREATE SCHEMA IF NOT EXISTS "bb";

CREATE TABLE IF NOT EXISTS "bb"."User" (
  Id SERIAL NOT NULL CONSTRAINT PK_User PRIMARY KEY,
  Username TEXT NOT NULL CONSTRAINT UX_User_Name UNIQUE,
  IsActive BOOLEAN NOT NULL DEFAULT TRUE,
  LastModified TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS "bb"."ParentSet" (
  Id SERIAL NOT NULL CONSTRAINT PK_ParentSet PRIMARY KEY,
  Name TEXT NOT NULL CONSTRAINT UX_ParentSet_Name UNIQUE,
  IsActive BOOLEAN NOT NULL DEFAULT TRUE,
  LastModified TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS "bb"."Card" (
  Id SERIAL NOT NULL CONSTRAINT PK_Card PRIMARY KEY,
  IsBlack BOOLEAN NOT NULL,
  ParentSetId INT NOT NULL CONSTRAINT FK_Card_ParentSet REFERENCES "bb"."ParentSet"(Id),
  FormatText TEXT NOT NULL,
  IsActive BOOLEAN NOT NULL DEFAULT TRUE,
  LastModified TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS "bb"."DrawCard" (
  Id SERIAL NOT NULL CONSTRAINT PK_DrawCard PRIMARY KEY,
  CardId INT NOT NULL CONSTRAINT FK_DrawCard REFERENCES "bb"."Card"(Id),
  DrawDate TIMESTAMP NOT NULL,
  SessionKey TEXT,
  IsActive BOOLEAN NOT NULL DEFAULT TRUE,
  LastModified TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TYPE "bb"."IdTable" AS RANGE (
  SUBTYPE = INTEGER
);

CREATE OR REPLACE PROCEDURE "bb"."GetCardRange"(cardSets "bb"."IdTable", is_black BOOLEAN)
AS $$
  SELECT
    COUNT(*)
  FROM "bb"."Card"
  WHERE IsBlack = is_black AND Id <@ cardSets
$$
LANGUAGE sql;

CREATE OR REPLACE PROCEDURE "bb"."DrawNthCard"(n INT, cardSets "bb"."IdTable", is_black BOOLEAN)
AS $$
  SELECT
    c.Id,
    c.IsBlack,
    p.Id AS "ParentSetId",
    p.Name AS "ParentSet",
    c.FormatText
  FROM "bb"."Card" AS c
    INNER JOIN "bb"."ParentSet" AS p ON p.Id = c.ParentSetId
  WHERE IsBlack = is_black AND p.Id <@ cardSets
  ORDER BY c.Id
  LIMIT 1
  OFFSET n
$$
LANGUAGE sql;

CREATE OR REPLACE PROCEDURE "bb"."GetCardById"(psid INT)
AS $$
  SELECT
    c.Id,
    IsBlack,
    p.Id AS "ParentSetId",
    p.Name AS "ParentSet",
    FormatText
  FROM "bb"."Card" AS c
    INNER JOIN "bb"."ParentSet" AS p ON p.Id = c.ParentSetId
  WHERE c.Id = psid
$$
LANGUAGE sql;

CREATE OR REPLACE PROCEDURE "bb"."GetDecks"()
AS $$
  SELECT
    Id,
    Name
  FROM "bb"."ParentSet"
$$
LANGUAGE sql;

CREATE OR REPLACE PROCEDURE "bb"."GetCardsByDeck"(cardSets "bb"."IdTable", isBlack BOOLEAN)
AS $$
  SELECT
    c.Id,
    IsBlack,
    p.Id AS "ParentSetId",
    p.Name AS "ParentSet",
    FormatText
  FROM "bb"."Card" AS c
    INNER JOIN "bb"."ParentSet" AS p ON p.Id = c.ParentSetId
  WHERE p.Id <@ cardSets AND ( isBlack IS NULL OR c.IsBlack = isBlack )
$$
LANGUAGE sql;
