-- Your SQL goes here
DROP TRIGGER tsvectorupdate_card ON bb.card;

CREATE TRIGGER tsvectorupdate_card
    BEFORE INSERT OR UPDATE 
    ON bb.card
    FOR EACH ROW
    EXECUTE PROCEDURE tsvector_update_trigger('text_searchable_format_text', 'pg_catalog.english', 'format_text');