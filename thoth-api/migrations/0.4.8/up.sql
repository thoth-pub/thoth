CREATE TYPE location_platform AS ENUM (
    'Project MUSE',
    'OAPEN',
    'DOAB',
    'JSTOR',
    'EBSCO Host',
    'OCLC KB',
    'ProQuest KB',
    'ProQuest ExLibris',
    'EBSCO KB',
    'JISC KB',
    'Other'
);

CREATE TABLE location (
    location_id         UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    publication_id      UUID NOT NULL REFERENCES publication(publication_id) ON DELETE CASCADE,
    landing_page        TEXT CHECK (landing_page ~* '^[^:]*:\/\/(?:[^\/:]*:[^\/@]*@)?(?:[^\/:.]*\.)+([^:\/]+)'),
    full_text_url       TEXT CHECK (full_text_url ~* '^[^:]*:\/\/(?:[^\/:]*:[^\/@]*@)?(?:[^\/:.]*\.)+([^:\/]+)'),
    location_platform   location_platform NOT NULL DEFAULT 'Other',
    canonical           BOOLEAN NOT NULL DEFAULT False,
    created_at          TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at          TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- Location must contain at least one of landing_page or full_text_url
    CONSTRAINT location_url_check CHECK (landing_page IS NOT NULL OR full_text_url IS NOT NULL)
);
SELECT diesel_manage_updated_at('location');

-- Only allow one canonical location per publication
CREATE UNIQUE INDEX location_uniq_canonical_true_idx ON location(publication_id)
    WHERE canonical;

-- Only allow one instance of each platform (except 'Other') per publication
CREATE UNIQUE INDEX location_uniq_platform_idx ON location(publication_id, location_platform)
    WHERE NOT location_platform = 'Other';

CREATE TABLE location_history (
    location_history_id      UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    location_id              UUID NOT NULL REFERENCES location(location_id) ON DELETE CASCADE,
    account_id               UUID NOT NULL REFERENCES account(account_id),
    data                     JSONB NOT NULL,
    timestamp                TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

--------------------------------------------------------------------------------
--- START - Data migration for live database. Delete this patch after migration
--------------------------------------------------------------------------------

-- Migrate punctum PDF publications (publisher ID 9c41b13c-cecc-4f6a-a151-be4682915ef5):

-- Each work that has publications should have a canonical PDF publication under a URL beginning "https://cloud.punctumbooks.com/s/". Create a new canonical location for each of these, using the URL as the full_text_url and the work's landing page as the landing_page.

INSERT INTO location(publication_id, landing_page, full_text_url, canonical)
    SELECT publication_id, landing_page, publication_url, True
    FROM publication
        INNER JOIN work ON publication.work_id = work.work_id
        INNER JOIN imprint ON work.imprint_id = imprint.imprint_id
    WHERE imprint.publisher_id = '9c41b13c-cecc-4f6a-a151-be4682915ef5'
    AND publication.publication_type = 'PDF'
    AND publication.publication_url ILIKE 'https://cloud.punctumbooks.com/s/%';

-- Some works may have additional PDF publications under Project MUSE or JSTOR URLs. Create a new non-canonical location for each of these, using the URL as the landing_page, omitting the full_text_url, and linking to the canonical (punctum cloud) publication.

INSERT INTO location(publication_id, landing_page, canonical, location_platform)
    SELECT a.publication_id, b.publication_url, False, 'Project MUSE'
    FROM publication a, publication b
        INNER JOIN work ON b.work_id = work.work_id
        INNER JOIN imprint ON work.imprint_id = imprint.imprint_id
    WHERE imprint.publisher_id = '9c41b13c-cecc-4f6a-a151-be4682915ef5'
    AND a.publication_type = 'PDF'
    AND a.work_id = b.work_id
    AND a.publication_url ILIKE 'https://cloud.punctumbooks.com/s/%'
    AND b.publication_url ILIKE 'https://muse.jhu.edu/book/%';

INSERT INTO location(publication_id, landing_page, canonical, location_platform)
    SELECT a.publication_id, b.publication_url, False, 'JSTOR'
    FROM publication a, publication b
        INNER JOIN work ON b.work_id = work.work_id
        INNER JOIN imprint ON work.imprint_id = imprint.imprint_id
    WHERE imprint.publisher_id = '9c41b13c-cecc-4f6a-a151-be4682915ef5'
    AND a.publication_type = 'PDF'
    AND a.work_id = b.work_id
    AND a.publication_url ILIKE 'https://cloud.punctumbooks.com/s/%'
    AND b.publication_url ILIKE 'https://www.jstor.org/stable/%';

-- Some works may have additional PDF publications under OAPEN URLs. Usually, one publication stores the OAPEN publication landing page, and another stores the OAPEN full text link. Create a new non-canonical location for each pair of OAPEN publications, using each URL as either the landing_page or the full_text_url as appropriate, and linking to the canonical (punctum cloud) publication.

INSERT INTO location(publication_id, landing_page, full_text_url, canonical, location_platform)
    SELECT a.publication_id, b.publication_url, c.publication_url, False, 'OAPEN'
    FROM publication a, publication b, publication c
        INNER JOIN work ON c.work_id = work.work_id
        INNER JOIN imprint ON work.imprint_id = imprint.imprint_id
    WHERE imprint.publisher_id = '9c41b13c-cecc-4f6a-a151-be4682915ef5'
    AND a.publication_type = 'PDF'
    AND a.work_id = b.work_id
    AND a.work_id = c.work_id
    AND a.publication_url ILIKE 'https://cloud.punctumbooks.com/s/%'
    AND (b.publication_url ILIKE 'https://library.oapen.org/handle/%' OR b.publication_url ILIKE 'http://library.oapen.org/handle/%')
    AND (c.publication_url ILIKE 'https://library.oapen.org/bitstream/%' OR c.publication_url ILIKE 'http://library.oapen.org/bitstream/%');

-- All MUSE, JSTOR and OAPEN PDF publications should now have had their URL data migrated to location objects. They should not contain any additional (ISBN/price, non-duplicated) data so should be safe to delete.
-- In a small number of cases, the OAPEN publications have been misclassified as 'Paperback' rather than 'PDF', so don't restrict the type when deleting.

DELETE FROM publication USING work, imprint
    WHERE publication.work_id = work.work_id
    AND work.imprint_id = imprint.imprint_id
    AND imprint.publisher_id = '9c41b13c-cecc-4f6a-a151-be4682915ef5'
    AND (publication_url ILIKE 'https://muse.jhu.edu/book/%' OR publication_url ILIKE 'https://www.jstor.org/stable/%' OR publication_url ILIKE 'https://library.oapen.org/handle/%' OR publication_url ILIKE 'http://library.oapen.org/handle/%' OR publication_url ILIKE 'https://library.oapen.org/bitstream/%' OR publication_url ILIKE 'http://library.oapen.org/bitstream/%')
    AND (isbn IS NULL OR EXISTS (
        SELECT * FROM publication b
        WHERE publication.work_id = b.work_id
        AND publication.isbn = b.isbn
        AND b.publication_url ILIKE 'https://cloud.punctumbooks.com/s/%'))
    AND NOT EXISTS (SELECT * FROM price WHERE publication.publication_id = price.publication_id);

-- All canonical (punctum cloud) publications should now have had their URL data migrated to location objects. Their publication_url fields should therefore be safe to clear.

UPDATE publication SET publication_url = NULL
    FROM work, imprint
    WHERE publication.work_id = work.work_id
    AND work.imprint_id = imprint.imprint_id
    AND imprint.publisher_id = '9c41b13c-cecc-4f6a-a151-be4682915ef5'
    AND publication_type = 'PDF'
    AND publication_url ILIKE 'https://cloud.punctumbooks.com/s/%'
    AND EXISTS (SELECT * FROM location WHERE publication.publication_id = location.publication_id AND publication.publication_url = location.full_text_url);

-- Migrate punctum paperback publications (publisher ID 9c41b13c-cecc-4f6a-a151-be4682915ef5):

-- If a work only has one paperback publication, assume that it is the canonical one. Create a new canonical location for each of these, using the URL as the landing_page.

INSERT INTO location(publication_id, landing_page, canonical)
    SELECT publication_id, publication_url, True
    FROM publication
        INNER JOIN work ON publication.work_id = work.work_id
        INNER JOIN imprint ON work.imprint_id = imprint.imprint_id
    WHERE imprint.publisher_id = '9c41b13c-cecc-4f6a-a151-be4682915ef5'
    AND publication_type = 'Paperback'
    AND publication_url IS NOT NULL
    AND NOT EXISTS
    (SELECT * FROM publication b
        WHERE publication.work_id = b.work_id
        AND NOT publication.publication_id = b.publication_id
        AND b.publication_type = 'Paperback');

-- Some works have multiple paperback publications. Inspection of the data shows that there are never more than two, they never have more than one distinct ISBN between them, and they never have more than one distinct set of prices between them (although they may have more than one distinct URL).
-- Assume that the main publication in these cases is the only one with prices, or else the only one with a URL, or else the one where the URL is a punctumbooks.com landing page (or, if all else is equal, the first one found). Create a canonical location for this publication.

INSERT INTO location(publication_id, landing_page, canonical)
    SELECT a.publication_id, a.publication_url, True
    FROM publication a
        LEFT JOIN price aprice ON a.publication_id = aprice.publication_id
        INNER JOIN work ON a.work_id = work.work_id
        INNER JOIN imprint ON work.imprint_id = imprint.imprint_id,
    publication b
        LEFT JOIN price bprice ON b.publication_id = bprice.publication_id
    WHERE imprint.publisher_id = '9c41b13c-cecc-4f6a-a151-be4682915ef5'
    AND a.publication_type = 'Paperback'
    AND b.publication_type = 'Paperback'
    AND a.work_id = b.work_id
    AND NOT a.publication_id = b.publication_id
    AND a.publication_url IS NOT NULL
    AND ((aprice.publication_id IS NOT NULL AND bprice.publication_id IS NULL)
        OR ((aprice.currency_code IS NOT DISTINCT FROM bprice.currency_code AND aprice.unit_price IS NOT DISTINCT FROM bprice.unit_price)
            AND (b.publication_url IS NULL OR b.publication_url NOT ILIKE 'https://punctumbooks.com/titles/%')));

-- A single work (ID 98ce9caa-487e-4391-86c9-e5d8129be5b6) has one paperback publication with prices but no URL, and another with a URL but no prices, so it is not covered by the above. Make a canonical location for it manually, attached to the publication with prices, then remove the publication without prices.

INSERT INTO location(publication_id, landing_page, canonical)
    SELECT a.publication_id, b.publication_url, True
    FROM publication a, publication b
    WHERE a.work_id = '98ce9caa-487e-4391-86c9-e5d8129be5b6'
    AND b.work_id = '98ce9caa-487e-4391-86c9-e5d8129be5b6'
    AND a.publication_type = 'Paperback'
    AND b.publication_type = 'Paperback'
    AND NOT a.publication_id = b.publication_id
    AND a.publication_url IS NULL
    AND b.publication_url IS NOT NULL
    AND EXISTS (SELECT * FROM price WHERE price.publication_id = a.publication_id)
    AND NOT EXISTS (SELECT * FROM price WHERE price.publication_id = b.publication_id);

DELETE FROM publication
    WHERE work_id = '98ce9caa-487e-4391-86c9-e5d8129be5b6'
    AND publication_type = 'Paperback'
    AND NOT EXISTS (SELECT * FROM price WHERE price.publication_id = publication_id);

-- Create non-canonical locations under the main publication for all the other URLs associated with this work.

INSERT INTO location(publication_id, landing_page, canonical)
    SELECT a.publication_id, b.publication_url, False
    FROM publication a
        INNER JOIN work ON a.work_id = work.work_id
        INNER JOIN imprint ON work.imprint_id = imprint.imprint_id,
    publication b
    WHERE imprint.publisher_id = '9c41b13c-cecc-4f6a-a151-be4682915ef5'
    AND a.publication_type = 'Paperback'
    AND b.publication_type = 'Paperback'
    AND a.work_id = b.work_id
    AND NOT a.publication_id = b.publication_id
    AND EXISTS (SELECT * FROM location WHERE a.publication_id = location.publication_id)
    AND b.publication_url IS NOT NULL
    AND b.publication_url IS DISTINCT FROM a.publication_url;

-- For any case where the main publication lacks an ISBN, carry over the ISBN (if any) from the other publication.

UPDATE publication
    SET isbn = b.isbn
    FROM publication b, work, imprint
    WHERE publication.work_id = work.work_id
    AND work.imprint_id = imprint.imprint_id
    AND imprint.publisher_id = '9c41b13c-cecc-4f6a-a151-be4682915ef5'
    AND publication.publication_type = 'Paperback'
    AND b.publication_type = 'Paperback'
    AND publication.work_id = b.work_id
    AND NOT publication.publication_id = b.publication_id
    AND EXISTS (SELECT * FROM location WHERE publication.publication_id = location.publication_id)
    AND publication.isbn IS NULL;

-- All price, ISBN and URL information in non-main publications should now either be duplicated on the main publication or stored in the location table. Delete these publications.

DELETE FROM publication USING work, imprint, publication b
    WHERE publication.work_id = work.work_id
    AND work.imprint_id = imprint.imprint_id
    AND imprint.publisher_id = '9c41b13c-cecc-4f6a-a151-be4682915ef5'
    AND publication.publication_type = 'Paperback'
    AND b.publication_type = 'Paperback'
    AND publication.work_id = b.work_id
    AND NOT publication.publication_id = b.publication_id
    AND NOT EXISTS (SELECT * FROM location WHERE publication.publication_id = location.publication_id)
    AND (publication.publication_url IS NULL OR EXISTS (SELECT * FROM location WHERE b.publication_id = location.publication_id AND publication.publication_url = location.landing_page))
    AND (publication.isbn IS NOT DISTINCT FROM b.isbn OR publication.isbn IS NULL)
    AND NOT EXISTS (SELECT unit_price, currency_code FROM price WHERE price.publication_id = publication.publication_id EXCEPT SELECT unit_price, currency_code FROM price WHERE price.publication_id = b.publication_id);

-- All remaining publication_urls should now be listed in the location table as the canonical URL for that publication. Remove them from the publications.

UPDATE publication SET publication_url = NULL
    FROM work, imprint
    WHERE publication.work_id = work.work_id
    AND work.imprint_id = imprint.imprint_id
    AND imprint.publisher_id = '9c41b13c-cecc-4f6a-a151-be4682915ef5'
    AND publication_type = 'Paperback'
    AND publication_url IS NOT NULL
    AND EXISTS (SELECT * FROM location WHERE publication.publication_id = location.publication_id AND publication.publication_url = location.landing_page);

-- Migrate remaining duplicate publications:

-- A single meson press work (ID 38872158-58b9-4ddf-a90e-f6001ac6c62d) accounts for all remaining duplicate publications. Inspection of the data shows two PDFs with differing URLs, identical ISBNs and no prices, and three paperbacks with differing URLs, identical ISBNs and two different prices (each in a different currency) between them. Handle these individually.

-- PDFs: one has a meson.press URL, the other an OAPEN URL. Assume that the former is the main one. Create a canonical location for it, create a secondary location for the other one, and then delete the other one and remove the main one's publication_url.

INSERT INTO location(publication_id, landing_page, full_text_url, canonical)
    SELECT publication_id, landing_page, publication_url, True
    FROM publication
        INNER JOIN work ON publication.work_id = work.work_id
    WHERE publication.work_id = '38872158-58b9-4ddf-a90e-f6001ac6c62d'
    AND publication.publication_type = 'PDF'
    AND publication.publication_url ILIKE 'https://meson.press/wp-content/uploads/%';

INSERT INTO location(publication_id, landing_page, canonical, location_platform)
    SELECT a.publication_id, b.publication_url, False, 'OAPEN'
    FROM publication a, publication b
    WHERE a.work_id = '38872158-58b9-4ddf-a90e-f6001ac6c62d'
    AND b.work_id = '38872158-58b9-4ddf-a90e-f6001ac6c62d'
    AND a.publication_type = 'PDF'
    AND b.publication_type = 'PDF'
    AND a.publication_url ILIKE 'https://meson.press/wp-content/uploads/%'
    AND b.publication_url ILIKE 'https://library.oapen.org/bitstream/%';

DELETE FROM publication
    WHERE publication.work_id = '38872158-58b9-4ddf-a90e-f6001ac6c62d'
    AND publication.publication_type = 'PDF'
    AND publication.publication_url ILIKE 'https://library.oapen.org/bitstream/%'
    AND (isbn IS NULL OR EXISTS (
        SELECT * FROM publication b
        WHERE publication.work_id = b.work_id
        AND publication.isbn = b.isbn
        AND b.publication_url ILIKE 'https://meson.press/wp-content/uploads/%'))
    AND NOT EXISTS (SELECT * FROM price WHERE publication.publication_id = price.publication_id);

UPDATE publication SET publication_url = NULL
    WHERE publication.work_id = '38872158-58b9-4ddf-a90e-f6001ac6c62d'
    AND publication.publication_type = 'PDF'
    AND publication.publication_url ILIKE 'https://meson.press/wp-content/uploads/%'
    AND EXISTS (SELECT * FROM location WHERE publication.publication_id = location.publication_id AND publication.publication_url = location.full_text_url);

-- Paperbacks: none of the URLs are meson.press, so assume that the first publication entered (which has ID 1382662a-ae40-47ae-98a0-34e03ae71366) is the main one. Create a canonical location for it.

INSERT INTO location(publication_id, landing_page, canonical)
    SELECT publication_id, publication_url, True
    FROM publication
    WHERE publication.publication_id = '1382662a-ae40-47ae-98a0-34e03ae71366';

-- Create non-canonical locations for the other publications, linked to the main one.

INSERT INTO location(publication_id, landing_page, canonical)
    SELECT '1382662a-ae40-47ae-98a0-34e03ae71366', publication_url, False
    FROM publication
    WHERE publication.work_id = '38872158-58b9-4ddf-a90e-f6001ac6c62d'
    AND publication.publication_type = 'Paperback'
    AND NOT publication.publication_id = '1382662a-ae40-47ae-98a0-34e03ae71366';

-- One of the prices linked to a non-main publication is not duplicated on the main publication. Move it to the main publication.

UPDATE price SET publication_id = '1382662a-ae40-47ae-98a0-34e03ae71366'
    WHERE publication_id = '49003581-5829-457a-b626-a5ab30df9a55';

-- The non-main paperback publications can now be deleted, and the main publication_url cleared.

DELETE FROM publication
    WHERE publication.work_id = '38872158-58b9-4ddf-a90e-f6001ac6c62d'
    AND publication.publication_type = 'Paperback'
    AND NOT publication.publication_id = '1382662a-ae40-47ae-98a0-34e03ae71366';

UPDATE publication SET publication_url = NULL WHERE publication_id = '1382662a-ae40-47ae-98a0-34e03ae71366';

-- Migrate all remaining publications:

-- All remaining publications across all publishers should now be unique per work/publication type. Therefore, any URLs which they have can be converted to canonical locations. For hard copy types, convert the publication_url to the location landing_page. For soft copy types, convert the publication_url to the location full_text_url and use the work landing_page as the location landing_page.
-- Double-check that no location entry already exists for the publication, and no duplicate publication exists.

INSERT INTO location(publication_id, landing_page, canonical)
    SELECT publication_id, publication_url, True
    FROM publication
    WHERE (publication.publication_type = 'Paperback' OR publication.publication_type = 'Hardback')
    AND publication_url IS NOT NULL
    AND NOT EXISTS (SELECT * FROM publication b
        WHERE publication.work_id = b.work_id
        AND NOT publication.publication_id = b.publication_id
        AND publication.publication_type = b.publication_type)
    AND NOT EXISTS (SELECT * FROM location WHERE publication.publication_id = location.publication_id AND publication.publication_url = location.landing_page);

INSERT INTO location(publication_id, landing_page, full_text_url, canonical)
    SELECT publication_id, landing_page, publication_url, True
    FROM publication
        INNER JOIN work ON publication.work_id = work.work_id
    WHERE (publication.publication_type = 'PDF' OR publication.publication_type = 'Epub' OR publication.publication_type = 'XML' OR publication.publication_type = 'Mobi' OR publication.publication_type = 'HTML')
    AND publication_url IS NOT NULL
    AND NOT EXISTS (SELECT * FROM publication b
        WHERE publication.work_id = b.work_id
        AND NOT publication.publication_id = b.publication_id
        AND publication.publication_type = b.publication_type)
    AND NOT EXISTS (SELECT * FROM location WHERE publication.publication_id = location.publication_id AND publication.publication_url = location.landing_page);

-- All these publications can now have their URLs cleared.

UPDATE publication SET publication_url = NULL
    FROM work
    WHERE publication_url IS NOT NULL
    AND NOT EXISTS (SELECT * FROM publication b
        WHERE publication.work_id = b.work_id
        AND NOT publication.publication_id = b.publication_id
        AND publication.publication_type = b.publication_type)
    AND EXISTS (SELECT * FROM location WHERE publication.publication_id = location.publication_id AND (publication.publication_url = location.landing_page OR publication.publication_url = location.full_text_url));
-----------------------------------------------------------------------------
--- END - Data migration for live database. Delete this patch after migration
-----------------------------------------------------------------------------

ALTER TABLE publication
    -- Only allow one publication of each type per work (existing data may breach this)
    -- To check for records which breach this constraint:
    -- `select * from publication a where (select count(*) from publication b where a.publication_type = b.publication_type and a.work_id = b.work_id) > 1 order by work_id, publication_type;`
    ADD CONSTRAINT publication_publication_type_work_id_uniq UNIQUE (publication_type, work_id),
    -- Remove publication_url column (all data should have been migrated to location table above)
    DROP COLUMN publication_url;
