CREATE OR REPLACE FUNCTION publication_location_canonical_urls() RETURNS trigger AS $$
BEGIN
    IF (
        NEW.publication_type <> 'Hardback' AND
        NEW.publication_type <> 'Paperback' AND
        (SELECT COUNT(*) FROM location
            WHERE location.publication_id = NEW.publication_id
            AND location.canonical
            AND (location.landing_page IS NULL OR location.full_text_url IS NULL)
        ) > 0
    ) THEN
        RAISE EXCEPTION 'Digital publications must have both Landing Page and Full Text URL in all their canonical locations';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER publication_location_canonical_urls_check BEFORE UPDATE ON publication
    FOR EACH ROW EXECUTE PROCEDURE publication_location_canonical_urls();
