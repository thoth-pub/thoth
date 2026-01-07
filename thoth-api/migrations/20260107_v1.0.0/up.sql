-- Drop foreign key constraints
ALTER TABLE abstract_history           DROP CONSTRAINT IF EXISTS abstract_history_account_id_fkey;
ALTER TABLE affiliation_history        DROP CONSTRAINT IF EXISTS affiliation_history_account_id_fkey;
ALTER TABLE biography_history          DROP CONSTRAINT IF EXISTS biography_history_account_id_fkey;
ALTER TABLE contact_history            DROP CONSTRAINT IF EXISTS contact_history_account_id_fkey;
ALTER TABLE contribution_history       DROP CONSTRAINT IF EXISTS contribution_history_account_id_fkey;
ALTER TABLE contributor_history        DROP CONSTRAINT IF EXISTS contributor_history_account_id_fkey;
ALTER TABLE funding_history            DROP CONSTRAINT IF EXISTS funding_history_account_id_fkey;
ALTER TABLE imprint_history            DROP CONSTRAINT IF EXISTS imprint_history_account_id_fkey;
ALTER TABLE institution_history        DROP CONSTRAINT IF EXISTS institution_history_account_id_fkey;
ALTER TABLE institution_history        DROP CONSTRAINT IF EXISTS funder_history_account_id_fkey; -- historical
ALTER TABLE issue_history              DROP CONSTRAINT IF EXISTS issue_history_account_id_fkey;
ALTER TABLE language_history           DROP CONSTRAINT IF EXISTS language_history_account_id_fkey;
ALTER TABLE location_history           DROP CONSTRAINT IF EXISTS location_history_account_id_fkey;
ALTER TABLE price_history              DROP CONSTRAINT IF EXISTS price_history_account_id_fkey;
ALTER TABLE publication_history        DROP CONSTRAINT IF EXISTS publication_history_account_id_fkey;
ALTER TABLE publisher_history          DROP CONSTRAINT IF EXISTS publisher_history_account_id_fkey;
ALTER TABLE reference_history          DROP CONSTRAINT IF EXISTS reference_history_account_id_fkey;
ALTER TABLE series_history             DROP CONSTRAINT IF EXISTS series_history_account_id_fkey;
ALTER TABLE subject_history            DROP CONSTRAINT IF EXISTS subject_history_account_id_fkey;
ALTER TABLE title_history              DROP CONSTRAINT IF EXISTS title_history_account_id_fkey;
ALTER TABLE work_history               DROP CONSTRAINT IF EXISTS work_history_account_id_fkey;
ALTER TABLE work_relation_history      DROP CONSTRAINT IF EXISTS work_relation_history_account_id_fkey;

-- Rename column account_id to user_id and change type to TEXT
ALTER TABLE abstract_history           RENAME COLUMN account_id TO user_id;
ALTER TABLE affiliation_history        RENAME COLUMN account_id TO user_id;
ALTER TABLE biography_history          RENAME COLUMN account_id TO user_id;
ALTER TABLE contact_history            RENAME COLUMN account_id TO user_id;
ALTER TABLE contribution_history       RENAME COLUMN account_id TO user_id;
ALTER TABLE contributor_history        RENAME COLUMN account_id TO user_id;
ALTER TABLE funding_history            RENAME COLUMN account_id TO user_id;
ALTER TABLE imprint_history            RENAME COLUMN account_id TO user_id;
ALTER TABLE institution_history        RENAME COLUMN account_id TO user_id;
ALTER TABLE issue_history              RENAME COLUMN account_id TO user_id;
ALTER TABLE language_history           RENAME COLUMN account_id TO user_id;
ALTER TABLE location_history           RENAME COLUMN account_id TO user_id;
ALTER TABLE price_history              RENAME COLUMN account_id TO user_id;
ALTER TABLE publication_history        RENAME COLUMN account_id TO user_id;
ALTER TABLE publisher_history          RENAME COLUMN account_id TO user_id;
ALTER TABLE reference_history          RENAME COLUMN account_id TO user_id;
ALTER TABLE series_history             RENAME COLUMN account_id TO user_id;
ALTER TABLE subject_history            RENAME COLUMN account_id TO user_id;
ALTER TABLE title_history              RENAME COLUMN account_id TO user_id;
ALTER TABLE work_history               RENAME COLUMN account_id TO user_id;
ALTER TABLE work_relation_history      RENAME COLUMN account_id TO user_id;

ALTER TABLE abstract_history           ALTER COLUMN user_id TYPE TEXT;
ALTER TABLE affiliation_history        ALTER COLUMN user_id TYPE TEXT;
ALTER TABLE biography_history          ALTER COLUMN user_id TYPE TEXT;
ALTER TABLE contact_history            ALTER COLUMN user_id TYPE TEXT;
ALTER TABLE contribution_history       ALTER COLUMN user_id TYPE TEXT;
ALTER TABLE contributor_history        ALTER COLUMN user_id TYPE TEXT;
ALTER TABLE funding_history            ALTER COLUMN user_id TYPE TEXT;
ALTER TABLE imprint_history            ALTER COLUMN user_id TYPE TEXT;
ALTER TABLE institution_history        ALTER COLUMN user_id TYPE TEXT;
ALTER TABLE issue_history              ALTER COLUMN user_id TYPE TEXT;
ALTER TABLE language_history           ALTER COLUMN user_id TYPE TEXT;
ALTER TABLE location_history           ALTER COLUMN user_id TYPE TEXT;
ALTER TABLE price_history              ALTER COLUMN user_id TYPE TEXT;
ALTER TABLE publication_history        ALTER COLUMN user_id TYPE TEXT;
ALTER TABLE publisher_history          ALTER COLUMN user_id TYPE TEXT;
ALTER TABLE reference_history          ALTER COLUMN user_id TYPE TEXT;
ALTER TABLE series_history             ALTER COLUMN user_id TYPE TEXT;
ALTER TABLE subject_history            ALTER COLUMN user_id TYPE TEXT;
ALTER TABLE title_history              ALTER COLUMN user_id TYPE TEXT;
ALTER TABLE work_history               ALTER COLUMN user_id TYPE TEXT;
ALTER TABLE work_relation_history      ALTER COLUMN user_id TYPE TEXT;

-- Drop the obsolete tables
DROP TABLE IF EXISTS publisher_account;
DROP TABLE IF EXISTS account;