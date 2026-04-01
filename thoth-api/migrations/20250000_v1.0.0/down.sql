-- Squashed down migration for the v1.0.0 baseline.
-- This replays the removed v1.0.0 down migrations in reverse order and then drops the original baseline.

-- -----------------------------------------------------------------------------
-- Source down migration: 20260325_v1.0.0/down.sql
-- -----------------------------------------------------------------------------

ALTER TABLE award
  DROP COLUMN IF EXISTS country,
  DROP COLUMN IF EXISTS jury,
  DROP COLUMN IF EXISTS year;


-- -----------------------------------------------------------------------------
-- Source down migration: 20260312_v1.0.0/down.sql
-- -----------------------------------------------------------------------------

DROP INDEX IF EXISTS endorsement_author_institution_idx;

ALTER TABLE endorsement
  DROP COLUMN IF EXISTS author_institution_id,
  DROP COLUMN IF EXISTS author_orcid;

DROP INDEX IF EXISTS book_review_reviewer_institution_idx;

ALTER TABLE book_review
  DROP COLUMN IF EXISTS page_range,
  DROP COLUMN IF EXISTS reviewer_institution_id,
  DROP COLUMN IF EXISTS reviewer_orcid;

ALTER TABLE additional_resource
  DROP COLUMN IF EXISTS date;

ALTER TABLE award
  DROP COLUMN IF EXISTS role;

ALTER TABLE award
  RENAME COLUMN prize_statement TO note;

DROP TYPE IF EXISTS award_role;


-- -----------------------------------------------------------------------------
-- Source down migration: 20260311_v1.0.0/down.sql
-- -----------------------------------------------------------------------------

ALTER TABLE public.language
  ADD COLUMN main_language  boolean DEFAULT false NOT NULL;

ALTER TABLE public.funding
  ADD COLUMN jurisdiction   text,
  ADD CONSTRAINT funding_jurisdiction_check CHECK ((octet_length(jurisdiction) >= 1));

ALTER TABLE public.issue
  DROP COLUMN IF EXISTS issue_number;


-- -----------------------------------------------------------------------------
-- Source down migration: 20260303_v1.0.0/down.sql
-- -----------------------------------------------------------------------------

ALTER TABLE imprint
  DROP COLUMN IF EXISTS default_currency,
  DROP COLUMN IF EXISTS default_place,
  DROP COLUMN IF EXISTS default_locale;


-- -----------------------------------------------------------------------------
-- Source down migration: 20260223_v1.0.0/down.sql
-- -----------------------------------------------------------------------------

ALTER TABLE file DROP CONSTRAINT IF EXISTS file_work_id_fkey;
ALTER TABLE file
  ADD CONSTRAINT file_work_id_fkey
  FOREIGN KEY (work_id) REFERENCES work(work_id);

ALTER TABLE file DROP CONSTRAINT IF EXISTS file_publication_id_fkey;
ALTER TABLE file
  ADD CONSTRAINT file_publication_id_fkey
  FOREIGN KEY (publication_id) REFERENCES publication(publication_id);

ALTER TABLE file_upload DROP CONSTRAINT IF EXISTS file_upload_work_id_fkey;
ALTER TABLE file_upload
  ADD CONSTRAINT file_upload_work_id_fkey
  FOREIGN KEY (work_id) REFERENCES work(work_id);

ALTER TABLE file_upload DROP CONSTRAINT IF EXISTS file_upload_publication_id_fkey;
ALTER TABLE file_upload
  ADD CONSTRAINT file_upload_publication_id_fkey
  FOREIGN KEY (publication_id) REFERENCES publication(publication_id);


-- -----------------------------------------------------------------------------
-- Source down migration: 20260214_v1.0.0/down.sql
-- -----------------------------------------------------------------------------

DROP TRIGGER IF EXISTS set_work_updated_at_with_relations ON public.work_featured_video;
DROP TRIGGER IF EXISTS set_work_updated_at_with_relations ON public.book_review;
DROP TRIGGER IF EXISTS set_work_updated_at_with_relations ON public.endorsement;
DROP TRIGGER IF EXISTS set_work_updated_at_with_relations ON public.award;
DROP TRIGGER IF EXISTS set_work_updated_at_with_relations ON public.additional_resource;

DROP INDEX IF EXISTS file_upload_work_featured_video_idx;
DROP INDEX IF EXISTS file_upload_additional_resource_idx;
DROP INDEX IF EXISTS file_work_featured_video_unique_idx;
DROP INDEX IF EXISTS file_additional_resource_unique_idx;

ALTER TABLE file DROP CONSTRAINT IF EXISTS file_type_check;
ALTER TABLE file_upload DROP CONSTRAINT IF EXISTS file_upload_type_check;

ALTER TABLE file
  DROP COLUMN IF EXISTS work_featured_video_id,
  DROP COLUMN IF EXISTS additional_resource_id;

ALTER TABLE file_upload
  DROP COLUMN IF EXISTS work_featured_video_id,
  DROP COLUMN IF EXISTS additional_resource_id;

ALTER TABLE file
  ADD CONSTRAINT file_type_check
  CHECK (
    (file_type = 'frontcover' AND work_id IS NOT NULL AND publication_id IS NULL) OR
    (file_type = 'publication' AND publication_id IS NOT NULL AND work_id IS NULL)
  );

ALTER TABLE file_upload
  ADD CONSTRAINT file_upload_type_check
  CHECK (
    (file_type = 'frontcover' AND work_id IS NOT NULL AND publication_id IS NULL) OR
    (file_type = 'publication' AND publication_id IS NOT NULL AND work_id IS NULL)
  );

DROP TABLE IF EXISTS work_featured_video_history;
DROP TABLE IF EXISTS book_review_history;
DROP TABLE IF EXISTS endorsement_history;
DROP TABLE IF EXISTS award_history;
DROP TABLE IF EXISTS additional_resource_history;

DROP TABLE IF EXISTS work_featured_video;
DROP TABLE IF EXISTS book_review;
DROP TABLE IF EXISTS endorsement;
DROP TABLE IF EXISTS award;
DROP TABLE IF EXISTS additional_resource;

DROP TYPE IF EXISTS resource_type;

ALTER TABLE work
  DROP COLUMN IF EXISTS resources_description;


-- -----------------------------------------------------------------------------
-- Source down migration: 20260210_v1.0.0/down.sql
-- -----------------------------------------------------------------------------

DROP TABLE IF EXISTS file_upload;
DROP TABLE IF EXISTS file;
DROP TYPE IF EXISTS file_type;
DROP FUNCTION IF EXISTS file_upload_work_updated_at_with_relations();
DROP FUNCTION IF EXISTS file_work_updated_at_with_relations();

ALTER TABLE imprint
  DROP CONSTRAINT IF EXISTS imprint_storage_cfg_all_or_none,
  DROP COLUMN IF EXISTS s3_bucket,
  DROP COLUMN IF EXISTS cdn_domain,
  DROP COLUMN IF EXISTS cloudfront_dist_id;


-- -----------------------------------------------------------------------------
-- Source down migration: 20260107_v1.0.0/down.sql
-- -----------------------------------------------------------------------------

-- Recreate the `account` table
CREATE TABLE account (
     account_id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
     name                TEXT NOT NULL CHECK (octet_length(name) >= 1),
     surname             TEXT NOT NULL CHECK (octet_length(surname) >= 1),
     email               TEXT NOT NULL CHECK (octet_length(email) >= 1),
     hash                BYTEA NOT NULL,
     salt                TEXT NOT NULL CHECK (octet_length(salt) >= 1),
     is_superuser        BOOLEAN NOT NULL DEFAULT False,
     is_bot              BOOLEAN NOT NULL DEFAULT False,
     is_active           BOOLEAN NOT NULL DEFAULT True,
     token               TEXT NULL CHECK (OCTET_LENGTH(token) >= 1),
     created_at          TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
     updated_at          TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
SELECT diesel_manage_updated_at('account');

-- case-insensitive UNIQ index on email
CREATE UNIQUE INDEX email_uniq_idx ON account(lower(email));

-- Recreate the `publisher_account` table
CREATE TABLE publisher_account (
   account_id          UUID NOT NULL REFERENCES account(account_id) ON DELETE CASCADE,
   publisher_id        UUID NOT NULL REFERENCES publisher(publisher_id) ON DELETE CASCADE,
   is_admin            BOOLEAN NOT NULL DEFAULT False,
   created_at          TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
   updated_at          TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
   PRIMARY KEY (account_id, publisher_id)
);
SELECT diesel_manage_updated_at('publisher_account');

-- Rename column user_id → account_id and change type to UUID
ALTER TABLE abstract_history           ALTER COLUMN user_id TYPE UUID USING user_id::uuid;
ALTER TABLE affiliation_history        ALTER COLUMN user_id TYPE UUID USING user_id::uuid;
ALTER TABLE biography_history          ALTER COLUMN user_id TYPE UUID USING user_id::uuid;
ALTER TABLE contact_history            ALTER COLUMN user_id TYPE UUID USING user_id::uuid;
ALTER TABLE contribution_history       ALTER COLUMN user_id TYPE UUID USING user_id::uuid;
ALTER TABLE contributor_history        ALTER COLUMN user_id TYPE UUID USING user_id::uuid;
ALTER TABLE funding_history            ALTER COLUMN user_id TYPE UUID USING user_id::uuid;
ALTER TABLE imprint_history            ALTER COLUMN user_id TYPE UUID USING user_id::uuid;
ALTER TABLE institution_history        ALTER COLUMN user_id TYPE UUID USING user_id::uuid;
ALTER TABLE issue_history              ALTER COLUMN user_id TYPE UUID USING user_id::uuid;
ALTER TABLE language_history           ALTER COLUMN user_id TYPE UUID USING user_id::uuid;
ALTER TABLE location_history           ALTER COLUMN user_id TYPE UUID USING user_id::uuid;
ALTER TABLE price_history              ALTER COLUMN user_id TYPE UUID USING user_id::uuid;
ALTER TABLE publication_history        ALTER COLUMN user_id TYPE UUID USING user_id::uuid;
ALTER TABLE publisher_history          ALTER COLUMN user_id TYPE UUID USING user_id::uuid;
ALTER TABLE reference_history          ALTER COLUMN user_id TYPE UUID USING user_id::uuid;
ALTER TABLE series_history             ALTER COLUMN user_id TYPE UUID USING user_id::uuid;
ALTER TABLE subject_history            ALTER COLUMN user_id TYPE UUID USING user_id::uuid;
ALTER TABLE title_history              ALTER COLUMN user_id TYPE UUID USING user_id::uuid;
ALTER TABLE work_history               ALTER COLUMN user_id TYPE UUID USING user_id::uuid;
ALTER TABLE work_relation_history      ALTER COLUMN user_id TYPE UUID USING user_id::uuid;

ALTER TABLE abstract_history           RENAME COLUMN user_id TO account_id;
ALTER TABLE affiliation_history        RENAME COLUMN user_id TO account_id;
ALTER TABLE biography_history          RENAME COLUMN user_id TO account_id;
ALTER TABLE contact_history            RENAME COLUMN user_id TO account_id;
ALTER TABLE contribution_history       RENAME COLUMN user_id TO account_id;
ALTER TABLE contributor_history        RENAME COLUMN user_id TO account_id;
ALTER TABLE funding_history            RENAME COLUMN user_id TO account_id;
ALTER TABLE imprint_history            RENAME COLUMN user_id TO account_id;
ALTER TABLE institution_history        RENAME COLUMN user_id TO account_id;
ALTER TABLE issue_history              RENAME COLUMN user_id TO account_id;
ALTER TABLE language_history           RENAME COLUMN user_id TO account_id;
ALTER TABLE location_history           RENAME COLUMN user_id TO account_id;
ALTER TABLE price_history              RENAME COLUMN user_id TO account_id;
ALTER TABLE publication_history        RENAME COLUMN user_id TO account_id;
ALTER TABLE publisher_history          RENAME COLUMN user_id TO account_id;
ALTER TABLE reference_history          RENAME COLUMN user_id TO account_id;
ALTER TABLE series_history             RENAME COLUMN user_id TO account_id;
ALTER TABLE subject_history            RENAME COLUMN user_id TO account_id;
ALTER TABLE title_history              RENAME COLUMN user_id TO account_id;
ALTER TABLE work_history               RENAME COLUMN user_id TO account_id;
ALTER TABLE work_relation_history      RENAME COLUMN user_id TO account_id;

-- Restore foreign key constraints
ALTER TABLE abstract_history           ADD CONSTRAINT abstract_history_account_id_fkey           FOREIGN KEY (account_id) REFERENCES account(account_id);
ALTER TABLE affiliation_history        ADD CONSTRAINT affiliation_history_account_id_fkey        FOREIGN KEY (account_id) REFERENCES account(account_id);
ALTER TABLE biography_history          ADD CONSTRAINT biography_history_account_id_fkey          FOREIGN KEY (account_id) REFERENCES account(account_id);
ALTER TABLE contact_history            ADD CONSTRAINT contact_history_account_id_fkey            FOREIGN KEY (account_id) REFERENCES account(account_id);
ALTER TABLE contribution_history       ADD CONSTRAINT contribution_history_account_id_fkey       FOREIGN KEY (account_id) REFERENCES account(account_id);
ALTER TABLE contributor_history        ADD CONSTRAINT contributor_history_account_id_fkey        FOREIGN KEY (account_id) REFERENCES account(account_id);
ALTER TABLE funding_history            ADD CONSTRAINT funding_history_account_id_fkey            FOREIGN KEY (account_id) REFERENCES account(account_id);
ALTER TABLE imprint_history            ADD CONSTRAINT imprint_history_account_id_fkey            FOREIGN KEY (account_id) REFERENCES account(account_id);
ALTER TABLE institution_history        ADD CONSTRAINT institution_history_account_id_fkey        FOREIGN KEY (account_id) REFERENCES account(account_id);
ALTER TABLE issue_history              ADD CONSTRAINT issue_history_account_id_fkey              FOREIGN KEY (account_id) REFERENCES account(account_id);
ALTER TABLE language_history           ADD CONSTRAINT language_history_account_id_fkey           FOREIGN KEY (account_id) REFERENCES account(account_id);
ALTER TABLE location_history           ADD CONSTRAINT location_history_account_id_fkey           FOREIGN KEY (account_id) REFERENCES account(account_id);
ALTER TABLE price_history              ADD CONSTRAINT price_history_account_id_fkey              FOREIGN KEY (account_id) REFERENCES account(account_id);
ALTER TABLE publication_history        ADD CONSTRAINT publication_history_account_id_fkey        FOREIGN KEY (account_id) REFERENCES account(account_id);
ALTER TABLE publisher_history          ADD CONSTRAINT publisher_history_account_id_fkey          FOREIGN KEY (account_id) REFERENCES account(account_id);
ALTER TABLE reference_history          ADD CONSTRAINT reference_history_account_id_fkey          FOREIGN KEY (account_id) REFERENCES account(account_id);
ALTER TABLE series_history             ADD CONSTRAINT series_history_account_id_fkey             FOREIGN KEY (account_id) REFERENCES account(account_id);
ALTER TABLE subject_history            ADD CONSTRAINT subject_history_account_id_fkey            FOREIGN KEY (account_id) REFERENCES account(account_id);
ALTER TABLE title_history              ADD CONSTRAINT title_history_account_id_fkey              FOREIGN KEY (account_id) REFERENCES account(account_id);
ALTER TABLE work_history               ADD CONSTRAINT work_history_account_id_fkey               FOREIGN KEY (account_id) REFERENCES account(account_id);
ALTER TABLE work_relation_history      ADD CONSTRAINT work_relation_history_account_id_fkey      FOREIGN KEY (account_id) REFERENCES account(account_id);

-- Remove ZITADEL organisation id column/index from publisher
DROP INDEX IF EXISTS publisher_zitadel_id_key;
ALTER TABLE publisher
    DROP COLUMN IF EXISTS zitadel_id;

-- -----------------------------------------------------------------------------
-- Source down migration: 20251212_v1.0.0/down.sql
-- -----------------------------------------------------------------------------

DROP TABLE contact_history;
DROP TABLE contact;

ALTER TABLE publisher
    DROP COLUMN accessibility_statement,
    DROP COLUMN accessibility_report_url;

ALTER TABLE publication
    DROP CONSTRAINT check_accessibility_standard_rules,
    DROP CONSTRAINT check_additional_standard_pdf_epub,
    DROP CONSTRAINT check_standard_or_exception,
    DROP COLUMN accessibility_standard,
    DROP COLUMN accessibility_additional_standard,
    DROP COLUMN accessibility_exception,
    DROP COLUMN accessibility_report_url;

DROP TYPE contact_type;
DROP TYPE accessibility_exception;
DROP TYPE accessibility_standard;


-- -----------------------------------------------------------------------------
-- Source down migration: 20251205_v1.0.0/down.sql
-- -----------------------------------------------------------------------------

-- Add title-related columns back to the work table
ALTER TABLE work
    ADD COLUMN full_title TEXT CHECK (octet_length(full_title) >= 1),
    ADD COLUMN title TEXT CHECK (octet_length(title) >= 1),
    ADD COLUMN subtitle TEXT CHECK (octet_length(subtitle) >= 1);

-- Migrate data back from title table to work table
UPDATE work w
SET 
    full_title = regexp_replace(t.full_title, '^<full_title>(.*)</full_title>$', '\\1'),
    title = regexp_replace(t.title, '^<title>(.*)</title>$', '\\1'),
    subtitle = CASE WHEN t.subtitle IS NOT NULL THEN regexp_replace(t.subtitle, '^<subtitle>(.*)</subtitle>$', '\\1') ELSE NULL END
FROM title t
WHERE w.work_id = t.work_id
    AND t.canonical = TRUE;

-- Drop the unique index for canonical titles
DROP INDEX IF EXISTS title_uniq_locale_idx;
-- Drop the unique index for locale codes
DROP INDEX IF EXISTS title_unique_canonical_true_idx;

-- Drop the title_history table
DROP TABLE title_history;

-- Drop the title table
DROP TABLE title;

-- Recreate short_abstract and long_abstract columns in the work table
ALTER TABLE work
    ADD COLUMN short_abstract TEXT CHECK (octet_length(short_abstract) >= 1),
    ADD COLUMN long_abstract TEXT CHECK (octet_length(long_abstract) >= 1);

-- -----------------------------------------------------------------------------
-- Reverse Conversion Function
-- -----------------------------------------------------------------------------
-- This function attempts to convert a JATS XML string back into a format that
-- resembles the original plaintext or Markdown. This is the reverse of the
-- `convert_to_jats` function from the `up` migration.
--
-- NOTE: This is a best-effort reversal. The primary goal is to make the data
-- readable and usable, not to restore the original format with 100% fidelity.
-- -----------------------------------------------------------------------------
CREATE OR REPLACE FUNCTION convert_from_jats(jats_in TEXT)
RETURNS TEXT AS $$
DECLARE
    processed_content TEXT := jats_in;
BEGIN
    -- Return NULL immediately if input is NULL or empty.
    IF processed_content IS NULL OR processed_content = '' THEN
        RETURN NULL;
    END IF;

    -- The order of replacements is important to handle nested tags correctly.

    -- Convert JATS tags back to a Markdown-like format.
    processed_content := regexp_replace(processed_content, '<ext-link xlink:href="([^"]+)">([^<]+)</ext-link>', '[\2](\1)', 'gi');
    processed_content := regexp_replace(processed_content, '<bold>([^<]+)</bold>', '**\1**', 'gi');
    processed_content := regexp_replace(processed_content, '<italic>([^<]+)</italic>', '*\1*', 'gi');
    processed_content := regexp_replace(processed_content, '<monospace>([^<]+)</monospace>', '`\1`', 'gi');
    processed_content := regexp_replace(processed_content, '<sc>([^<]+)</sc>', '\1', 'gi'); -- Revert small-caps to original text
    processed_content := regexp_replace(processed_content, '<sup[^>]*>([^<]+)</sup>', '^\1^', 'gi'); -- A possible representation for superscript
    processed_content := regexp_replace(processed_content, '<sub[^>]*>([^<]+)</sub>', '~\1~', 'gi'); -- A possible representation for subscript
    processed_content := regexp_replace(processed_content, '<break\s*/>', E'\n', 'gi');

    -- Remove paragraph tags and handle the spacing.
    -- Replace closing tags with double newlines to separate paragraphs.
    processed_content := regexp_replace(processed_content, '</p>', E'\n\n', 'gi');
    -- Strip any remaining opening paragraph tags.
    processed_content := regexp_replace(processed_content, '<p>', '', 'gi');

    -- Clean up any leftover simple HTML tags that were not converted.
    processed_content := regexp_replace(processed_content, '<[^>]+>', '', 'g');

    -- Trim leading/trailing whitespace that may result from tag removal.
    processed_content := trim(processed_content);

    RETURN processed_content;
END;
$$ LANGUAGE plpgsql;


-- Migrate data back from the abstract table to the work table using the reverse conversion
UPDATE work
SET
    short_abstract = convert_from_jats(abstract.content)
FROM
    abstract
WHERE
    abstract.work_id = work.work_id
    AND abstract.abstract_type = 'short'
    AND abstract.canonical = TRUE;

UPDATE work
SET
    long_abstract = convert_from_jats(abstract.content)
FROM
    abstract
WHERE
    abstract.work_id = work.work_id
    AND abstract.abstract_type = 'long'
    AND abstract.canonical = TRUE;

-- Drop unique indexes created for the abstract table
DROP INDEX IF EXISTS abstract_unique_canonical_true_idx;
DROP INDEX IF EXISTS abstract_uniq_locale_idx;

-- Drop the abstract_history table
DROP TABLE abstract_history;
-- Drop the abstract table and its related objects
DROP TABLE IF EXISTS abstract;

-- Drop the AbstractType enum
DROP TYPE IF EXISTS abstract_type;

ALTER TABLE contribution
    ADD COLUMN biography TEXT CHECK (octet_length(biography) >= 1);

-- Migrate data back from the abstract table to the work table using the reverse conversion
UPDATE contribution
SET
    biography = convert_from_jats(biography.content)
FROM
    biography
WHERE
    biography.contribution_id = contribution.contribution_id
    AND biography.canonical = TRUE;

-- Drop unique indexes created for the biography table
DROP INDEX IF EXISTS biography_unique_canonical_true_idx;
DROP INDEX IF EXISTS biography_uniq_locale_idx;

-- Drop the biography_history table
DROP TABLE biography_history;
-- Drop the biography table and its related objects
DROP TABLE IF EXISTS biography;

-- Drop the locale_code enum type
DROP TYPE locale_code;

-- Clean up the reverse conversion function
DROP FUNCTION convert_from_jats(TEXT);

-- -----------------------------------------------------------------------------
-- Source down migration: 20251204_v1.0.0/down.sql
-- -----------------------------------------------------------------------------

ALTER TABLE affiliation
    DROP CONSTRAINT affiliation_affiliation_ordinal_contribution_id_uniq;

CREATE UNIQUE INDEX affiliation_uniq_ord_in_contribution_idx ON affiliation(contribution_id, affiliation_ordinal);

ALTER TABLE contribution
    DROP CONSTRAINT contribution_contribution_ordinal_work_id_uniq,
    ADD CONSTRAINT contribution_contribution_ordinal_work_id_uniq UNIQUE (contribution_ordinal, work_id);

ALTER TABLE issue
    DROP CONSTRAINT issue_issue_ordinal_series_id_uniq;

CREATE UNIQUE INDEX issue_uniq_ord_in_series_idx ON issue(series_id, issue_ordinal);

ALTER TABLE reference
    DROP CONSTRAINT reference_reference_ordinal_work_id_uniq,
    ADD CONSTRAINT reference_reference_ordinal_work_id_uniq UNIQUE (work_id, reference_ordinal);

ALTER TABLE subject
    DROP CONSTRAINT subject_ordinal_type_uniq;

ALTER TABLE work_relation
    DROP CONSTRAINT work_relation_ordinal_type_uniq,
    ADD CONSTRAINT work_relation_ordinal_type_uniq UNIQUE (relation_ordinal, relator_work_id, relation_type);


-- -----------------------------------------------------------------------------
-- Source down migration: 20251203_v1.0.0/down.sql
-- -----------------------------------------------------------------------------

-------------------------------------------------------------------------------
-- 1. Drop the current deterministic work_relation_work_updated_at_with_relations
--    and its trigger
-------------------------------------------------------------------------------

DROP TRIGGER IF EXISTS set_work_relation_updated_at_with_relations ON work_relation;
DROP FUNCTION IF EXISTS work_relation_work_updated_at_with_relations() CASCADE;

-------------------------------------------------------------------------------
-- 2. Restore the previous work_relation_work_updated_at_with_relations()
--    that bumps all involved works whenever a relation row changes
-------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION work_relation_work_updated_at_with_relations() RETURNS trigger AS $$
BEGIN
    IF (
        NEW IS DISTINCT FROM OLD
        ) THEN
        UPDATE work
        SET updated_at_with_relations = current_timestamp
        WHERE work_id = OLD.relator_work_id OR work_id = NEW.relator_work_id
           OR work_id = OLD.related_work_id OR work_id = NEW.related_work_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER set_work_updated_at_with_relations AFTER INSERT OR UPDATE OR DELETE ON work_relation
    FOR EACH ROW EXECUTE PROCEDURE work_relation_work_updated_at_with_relations();

-------------------------------------------------------------------------------
-- 3. Restore work_work_updated_at_with_relations() and its trigger on work
-------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION work_work_updated_at_with_relations() RETURNS trigger AS $$
BEGIN
    IF (
        NEW IS DISTINCT FROM OLD
        ) THEN
        UPDATE work
        SET updated_at_with_relations = current_timestamp
        FROM work_relation
        -- The positions of relator/related IDs in this statement don't matter, as
        -- every work_relation record has a mirrored record with relator/related IDs swapped
        WHERE work.work_id = work_relation.relator_work_id AND work_relation.related_work_id = NEW.work_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS set_work_updated_at_with_relations ON work;

CREATE TRIGGER set_work_updated_at_with_relations
    AFTER UPDATE ON work
    FOR EACH ROW EXECUTE PROCEDURE work_work_updated_at_with_relations();


-- -----------------------------------------------------------------------------
-- Source down migration: 20250000_v1.0.0/down.sql
-- -----------------------------------------------------------------------------

-- Drop tables
DROP TABLE IF EXISTS public.work_relation_history CASCADE;
DROP TABLE IF EXISTS public.work_relation CASCADE;
DROP TABLE IF EXISTS public.work_history CASCADE;
DROP TABLE IF EXISTS public.work CASCADE;
DROP TABLE IF EXISTS public.subject_history CASCADE;
DROP TABLE IF EXISTS public.subject CASCADE;
DROP TABLE IF EXISTS public.series_history CASCADE;
DROP TABLE IF EXISTS public.series CASCADE;
DROP TABLE IF EXISTS public.reference_history CASCADE;
DROP TABLE IF EXISTS public.reference CASCADE;
DROP TABLE IF EXISTS public.publisher_history CASCADE;
DROP TABLE IF EXISTS public.publisher_account CASCADE;
DROP TABLE IF EXISTS public.publisher CASCADE;
DROP TABLE IF EXISTS public.publication_history CASCADE;
DROP TABLE IF EXISTS public.publication CASCADE;
DROP TABLE IF EXISTS public.price_history CASCADE;
DROP TABLE IF EXISTS public.price CASCADE;
DROP TABLE IF EXISTS public.location_history CASCADE;
DROP TABLE IF EXISTS public.location CASCADE;
DROP TABLE IF EXISTS public.language_history CASCADE;
DROP TABLE IF EXISTS public.language CASCADE;
DROP TABLE IF EXISTS public.issue_history CASCADE;
DROP TABLE IF EXISTS public.issue CASCADE;
DROP TABLE IF EXISTS public.institution_history CASCADE;
DROP TABLE IF EXISTS public.institution CASCADE;
DROP TABLE IF EXISTS public.imprint_history CASCADE;
DROP TABLE IF EXISTS public.imprint CASCADE;
DROP TABLE IF EXISTS public.funding_history CASCADE;
DROP TABLE IF EXISTS public.funding CASCADE;
DROP TABLE IF EXISTS public.contributor_history CASCADE;
DROP TABLE IF EXISTS public.contributor CASCADE;
DROP TABLE IF EXISTS public.contribution_history CASCADE;
DROP TABLE IF EXISTS public.contribution CASCADE;
DROP TABLE IF EXISTS public.affiliation_history CASCADE;
DROP TABLE IF EXISTS public.affiliation CASCADE;
DROP TABLE IF EXISTS public.account CASCADE;

-- Drop functions
DROP FUNCTION IF EXISTS public.affiliation_work_updated_at_with_relations() CASCADE;
DROP FUNCTION IF EXISTS public.biography_work_updated_at_with_relations() CASCADE;
DROP FUNCTION IF EXISTS public.contributor_work_updated_at_with_relations() CASCADE;
DROP FUNCTION IF EXISTS public.imprint_work_updated_at_with_relations() CASCADE;
DROP FUNCTION IF EXISTS public.institution_work_updated_at_with_relations() CASCADE;
DROP FUNCTION IF EXISTS public.location_work_updated_at_with_relations() CASCADE;
DROP FUNCTION IF EXISTS public.price_work_updated_at_with_relations() CASCADE;
DROP FUNCTION IF EXISTS public.publisher_work_updated_at_with_relations() CASCADE;
DROP FUNCTION IF EXISTS public.series_work_updated_at_with_relations() CASCADE;
DROP FUNCTION IF EXISTS public.work_relation_work_updated_at_with_relations() CASCADE;
DROP FUNCTION IF EXISTS public.work_work_updated_at_with_relations() CASCADE;
DROP FUNCTION IF EXISTS public.work_updated_at_with_relations() CASCADE;
DROP FUNCTION IF EXISTS public.work_set_updated_at() CASCADE;
DROP FUNCTION IF EXISTS public.publication_chapter_no_dimensions() CASCADE;
DROP FUNCTION IF EXISTS public.publication_location_canonical_urls() CASCADE;
DROP FUNCTION IF EXISTS public.diesel_set_updated_at() CASCADE;
DROP FUNCTION IF EXISTS public.diesel_manage_updated_at(regclass) CASCADE;

-- Drop enum types
DROP TYPE IF EXISTS public.work_type;
DROP TYPE IF EXISTS public.work_status;
DROP TYPE IF EXISTS public.subject_type;
DROP TYPE IF EXISTS public.series_type;
DROP TYPE IF EXISTS public.relation_type;
DROP TYPE IF EXISTS public.publication_type;
DROP TYPE IF EXISTS public.location_platform;
DROP TYPE IF EXISTS public.language_relation;
DROP TYPE IF EXISTS public.language_code;
DROP TYPE IF EXISTS public.currency_code;
DROP TYPE IF EXISTS public.country_code;
DROP TYPE IF EXISTS public.contribution_type;

-- Drop extension
DROP EXTENSION IF EXISTS "uuid-ossp" CASCADE;
