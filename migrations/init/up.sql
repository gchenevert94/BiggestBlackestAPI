-- For database "biggest_blackest"
CREATE SCHEMA "bb";

CREATE IF NOT EXISTS TABLE "bb"."User" (
  Id SERIAL NOT NULL CONSTRAINT PK_User PRIMARY KEY,
  Username TEXT NOT NULL CONSTRAINT UX_User_Name UNIQUE,
  IsActive BOOLEAN NOT NULL DEFAULT TRUE,
  LastModified TIMESTAMP NOT NULL DEFAULT (NOW() AT TIME ZONE UTC)
);

CREATE IF NOT EXISTS TABLE "bb"."ParentSet" (
  Id SERIAL NOT NULL CONSTRAINT PK_ParentSet PRIMARY KEY,
  Name TEXT NOT NULL CONSTRAINT UX_ParentSet_Name UNIQUE,
  IsActive BOOLEAN NOT NULL DEFAULT TRUE,
  LastModified TIMESTAMP NOT NULL DEFAULT (NOW() AT TIME ZONE UTC)
);

CREATE IF NOT EXISTS TABLE "bb"."Card" (
  Id SERIAL NOT NULL CONSTRAINT PK_Card PRIMARY KEY,
  IsBlack BOOLEAN NOT NULL,
  ParentSetId INT NOT NULL CONSTRAINT FK_Card_ParentSet REFERENCES "bb"."ParentSet"(Id),
  FormatText TEXT NOT NULL,
  IsActive BOOLEAN NOT NULL DEFAULT TRUE,
  LastModified TIMESTAMP NOT NULL DEFAULT (NOW() AT TIME ZONE UTC)
);

CREATE IF NOT EXISTS TABLE "bb"."DrawCard" (
  Id SERIAL NOT NULL CONSTRAINT PK_DrawCard PRIMARY KEY,
  CardId INT NOT NULL CONSTRAINT FK_DrawCard REFERENCES "bb"."Card"(Id),
  DrawDate TIMESTAMP NOT NULL,
  SessionKey TEXT,
  IsActive BOOLEAN NOT NULL DEFAULT TRUE,
  LastModified TIMESTAMP NOT NULL DEFAULT (NOW() AT TIME ZONE UTC)
);

CREATE IF NOT EXISTS TYPE "bb"."IdTable" AS RANGE (
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
  DECLARE r = SELECT
    c.Id,
    c.IsBlack,
    p.ParentSetId,
    p.Name AS "ParentSet",
    c.FormatText
  FROM "bb"."Card" AS c
    INNER JOIN @sets AS s ON s.Id = c.ParentSetId
    INNER JOIN "bb"."ParentSet" AS p ON p.Id = c.ParentSetId
  WHERE IsBlack = @is_black AND ROW_NUMBER() OVER (ORDER BY Id) = @n
$$;

CREATE OR REPLACE PROCEDURE "bb"."GetCardById"(id INT)
AS $$
  SELECT
    Id,
    IsBlack,
    p.ParentSetId,
    p.Name AS "ParentSet",
    FormatText
  FROM "bb"."Card" AS c
    INNER JOIN "bb"."ParentSet" AS p ON p.Id = c.ParentId
  WHERE c.Id = id
$$;

CREATE OR REPLACE PROCEDURE "bb"."GetDecks"
AS $$
  SELECT
    Id,
    Name
  FROM ParentSet
$$;

CREATE OR REPLACE PROCEDURE "bb"."GetCardsByDeck"(sets "bb"."IdTable", isBlack BOOLEAN)
AS $$
  SELECT
    Id,
    IsBlack,
    p.ParentSetId,
    p.Name AS "ParentSet",
    FormatText
  FROM "bb"."Card" AS c
    INNER JOIN "bb"."ParentSet" AS p ON p.Id = c.ParentId
    INNER JOIN sets AS s ON s.Id = p.Id
  WHERE isBlack IS NULL OR c.IsBlack = isBlack
$$;