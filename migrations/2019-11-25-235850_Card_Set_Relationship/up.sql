-- Your SQL goes here
CREATE TABLE "bb"."ParentSetCard" (
  ParentSetId SERIAL NOT NULL CONSTRAINT FK_ParentSetCard_ParentSet REFERENCES "bb"."ParentSet"(Id),
  CardId SERIAL NOT NULL CONSTRAINT FK_ParentSetCard_Card REFERENCES "bb"."Card"(Id),
  IsActive BOOLEAN NOT NULL DEFAULT TRUE,
  LastModified TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT NOW(),
  CONSTRAINT PK_ParentSetCard PRIMARY KEY (ParentSetId, CardId)
);

ALTER TABLE "bb"."Card" DROP CONSTRAINT FK_Card_ParentSet;
ALTER TABLE "bb"."Card" DROP COLUMN ParentSetId;
