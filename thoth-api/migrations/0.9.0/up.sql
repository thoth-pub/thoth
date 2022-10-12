CREATE TABLE reference (
    reference_id            UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    work_id                 UUID NOT NULL REFERENCES work(work_id) ON DELETE CASCADE,
    reference_ordinal       INTEGER NOT NULL CHECK (reference_ordinal > 0),
    doi                     TEXT CHECK (doi ~ '^https:\/\/doi\.org\/10\.\d{4,9}\/[-._\;\(\)\/:a-zA-Z0-9]+$'),
    unstructured_citation   TEXT CHECK (octet_length(unstructured_citation) >= 1),
    issn                    TEXT CHECK (issn ~ '\d{4}\-\d{3}(\d|X)'),
    isbn                    TEXT CHECK (octet_length(isbn) = 17),
    journal_title           TEXT CHECK (octet_length(journal_title) => 1),
    article_title           TEXT CHECK (octet_length(article_title) => 1),
    series_title            TEXT CHECK (octet_length(series_title) => 1),
    volume_title            TEXT CHECK (octet_length(volume_title) => 1),
    edition                 INTEGER CHECK (edition > 0),
    author                  TEXT CHECK (octet_length(author) => 1),
    volume                  TEXT CHECK (octet_length(volume) => 1),
    issue                   TEXT CHECK (octet_length(issue) => 1),
    first_page              TEXT CHECK (octet_length(first_page) => 1),
    component_number        TEXT CHECK (octet_length(component_number) => 1),
    standard_designator     TEXT CHECK (octet_length(standard_designator) => 1),
    standards_body_name     TEXT CHECK (octet_length(standards_body_name) => 1),
    standards_body_acronym  TEXT CHECK (octet_length(standards_body_acronym) => 1),
    url                     TEXT CHECK (url ~ '^[^:]*:\/\/(?:[^\/:]*:[^\/@]*@)?(?:[^\/:.]*\.)+([^:\/]+)'),
    publication_date        DATE,
    retrieval_date          DATE,
    created_at              TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at              TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT reference_reference_ordinal_work_id_uniq UNIQUE (work_id, reference_ordinal),
    CONSTRAINT reference_doi_andor_unstructured_citation CHECK
        (doi IS NOT NULL OR unstructured_citation IS NOT NULL)
);
SELECT diesel_manage_updated_at('reference');

CREATE TABLE reference_history (
    reference_history_id     UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    reference_id             UUID NOT NULL REFERENCES reference(reference_id) ON DELETE CASCADE,
    account_id               UUID NOT NULL REFERENCES account(account_id),
    data                     JSONB NOT NULL,
    timestamp                TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);