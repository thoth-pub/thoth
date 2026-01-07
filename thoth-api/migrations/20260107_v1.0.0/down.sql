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