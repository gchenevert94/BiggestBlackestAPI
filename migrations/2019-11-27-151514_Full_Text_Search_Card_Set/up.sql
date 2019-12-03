-- Your SQL goes here
ALTER TABLE bb.card ADD COLUMN text_searchable_format_text TSVECTOR;

UPDATE bb.card SET text_searchable_format_text = to_tsvector('english', formattext);

ALTER TABLE bb.card ALTER COLUMN text_searchable_format_text SET NOT NULL;

CREATE INDEX formattext_search_idx ON bb.card USING GIN (text_searchable_format_text);

CREATE TRIGGER tsvectorupdate_card BEFORE INSERT OR UPDATE
  ON bb.card FOR EACH ROW EXECUTE PROCEDURE
  tsvector_update_trigger(text_searchable_format_text,
                          'pg_catalog.english',
                          formattext);

ALTER TABLE bb.parent_set ADD COLUMN text_searchable_name TSVECTOR;

UPDATE bb.parent_set SET text_searchable_name = to_tsvector('english', name);

ALTER TABLE bb.parent_set ALTER COLUMN text_searchable_name SET NOT NULL;

CREATE INDEX name_search_idx ON bb.parent_set USING GIN (text_searchable_name);

CREATE TRIGGER tsvectorupdate_parent_set BEFORE INSERT OR UPDATE
  ON bb.parent_set FOR EACH ROW EXECUTE PROCEDURE
                     tsvector_update_trigger(text_searchable_name,
                                             'pg_catalog.english',
                     name);
