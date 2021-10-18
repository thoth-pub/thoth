ALTER TABLE publication
    DROP CONSTRAINT publication_publication_type_work_id_uniq;

-- Migrate location URLs back into publication table as far as possible before dropping location table:
-- set the landing page of the canonical location (if any) as the main publication_url,
-- then create duplicate publications to store all other location URLs (landing page/full text).
UPDATE publication
   SET publication_url = location.landing_page
      FROM location
      WHERE publication.publication_id = location.publication_id
      AND location.canonical
      AND location.full_text_url IS NULL;
INSERT INTO publication(publication_type, work_id, publication_url)
    SELECT publication.publication_type, publication.work_id, location.landing_page FROM publication, location
    WHERE publication.publication_id = location.publication_id
    AND NOT location.canonical;
INSERT INTO publication(publication_type, work_id, publication_url)
    SELECT publication.publication_type, publication.work_id, location.full_text_url FROM publication, location
    WHERE publication.publication_id = location.publication_id
    AND location.full_text_url IS NOT NULL;

DROP TABLE location_history;
DROP TRIGGER set_updated_at ON location;
DROP TABLE location;
DROP TYPE IF EXISTS location_platform;
