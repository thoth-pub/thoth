CREATE TYPE resource_type AS ENUM (
  'AUDIO',
  'VIDEO',
  'IMAGE',
  'BLOG',
  'WEBSITE',
  'DOCUMENT',
  'BOOK',
  'ARTICLE',
  'MAP',
  'SOURCE',
  'DATASET',
  'SPREADSHEET',
  'OTHER'
);

CREATE TABLE additional_resource (
  additional_resource_id   UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  work_id                  UUID NOT NULL REFERENCES work(work_id) ON DELETE CASCADE,
  title                    TEXT NOT NULL CHECK (octet_length(title) >= 1),
  description              TEXT,
  attribution              TEXT,
  resource_type            resource_type NOT NULL,
  doi                      TEXT,
  handle                   TEXT,
  url                      TEXT,
  resource_ordinal         INTEGER NOT NULL DEFAULT 1 CHECK (resource_ordinal > 0),
  created_at               TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at               TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
SELECT diesel_manage_updated_at('additional_resource');

CREATE TABLE award (
    award_id       UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    work_id        UUID NOT NULL REFERENCES work(work_id) ON DELETE CASCADE,
    title          TEXT NOT NULL CHECK (octet_length(title) >= 1),
    url            TEXT,
    category       TEXT,
    note           TEXT,
    award_ordinal  INTEGER NOT NULL DEFAULT 1 CHECK (award_ordinal > 0),
    created_at     TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
SELECT diesel_manage_updated_at('award');

CREATE TABLE endorsement (
    endorsement_id      UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    work_id             UUID NOT NULL REFERENCES work(work_id) ON DELETE CASCADE,
    author_name         TEXT,
    author_role         TEXT,
    url                 TEXT,
    text                TEXT,
    endorsement_ordinal INTEGER NOT NULL DEFAULT 1 CHECK (endorsement_ordinal > 0),
    created_at          TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
SELECT diesel_manage_updated_at('endorsement');

CREATE TABLE book_review (
    book_review_id      UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    work_id             UUID NOT NULL REFERENCES work(work_id) ON DELETE CASCADE,
    title               TEXT,
    author_name         TEXT,
    url                 TEXT,
    doi                 TEXT,
    review_date         DATE,
    journal_name        TEXT,
    journal_volume      TEXT,
    journal_number      TEXT,
    journal_issn        TEXT,
    text                TEXT,
    review_ordinal      INTEGER NOT NULL DEFAULT 1 CHECK (review_ordinal > 0),
    created_at          TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
SELECT diesel_manage_updated_at('book_review');

CREATE TABLE work_featured_video (
  work_featured_video_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  work_id                UUID NOT NULL UNIQUE REFERENCES work(work_id) ON DELETE CASCADE,
  video_id               TEXT,
  title                  TEXT,
  width                  INTEGER NOT NULL DEFAULT 560 CHECK (width > 0),
  height                 INTEGER NOT NULL DEFAULT 315 CHECK (height > 0),

  created_at             TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at             TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
SELECT diesel_manage_updated_at('work_featured_video');

ALTER TABLE work
ADD COLUMN resources_description TEXT;

CREATE TABLE additional_resource_history (
    additional_resource_history_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    additional_resource_id UUID NOT NULL REFERENCES additional_resource(additional_resource_id) ON DELETE CASCADE,
    account_id UUID NOT NULL REFERENCES account(account_id),
    data JSONB NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE award_history (
    award_history_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    award_id UUID NOT NULL REFERENCES award(award_id) ON DELETE CASCADE,
    account_id UUID NOT NULL REFERENCES account(account_id),
    data JSONB NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE endorsement_history (
    endorsement_history_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    endorsement_id UUID NOT NULL REFERENCES endorsement(endorsement_id) ON DELETE CASCADE,
    account_id UUID NOT NULL REFERENCES account(account_id),
    data JSONB NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE book_review_history (
    book_review_history_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    book_review_id UUID NOT NULL REFERENCES book_review(book_review_id) ON DELETE CASCADE,
    account_id UUID NOT NULL REFERENCES account(account_id),
    data JSONB NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE work_featured_video_history (
    work_featured_video_history_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    work_featured_video_id UUID NOT NULL REFERENCES work_featured_video(work_featured_video_id) ON DELETE CASCADE,
    account_id UUID NOT NULL REFERENCES account(account_id),
    data JSONB NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX idx_additional_resource_workid_ordinal ON additional_resource (work_id, resource_ordinal);
CREATE UNIQUE INDEX idx_award_workid_ordinal ON award (work_id, award_ordinal);
CREATE UNIQUE INDEX idx_endorsement_workid_ordinal ON endorsement (work_id, endorsement_ordinal);
CREATE UNIQUE INDEX idx_book_review_workid_ordinal ON book_review (work_id, review_ordinal);

