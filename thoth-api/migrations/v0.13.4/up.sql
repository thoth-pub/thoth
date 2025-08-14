CREATE TYPE contact_type AS ENUM (
    'Accessibility'
);

CREATE TABLE contact (
    contact_id      UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    publisher_id    UUID NOT NULL REFERENCES publisher(publisher_id) ON DELETE CASCADE,
    contact_type    contact_type NOT NULL DEFAULT 'Accessibility',
    email           TEXT NOT NULL CHECK (octet_length(email) >= 1),
    created_at      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT contact_contact_type_publisher_id_uniq UNIQUE (publisher_id, contact_type)
);
SELECT diesel_manage_updated_at('contact');
CREATE INDEX idx_contact_email ON contact (email);

CREATE TABLE contact_history (
    contact_history_id  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    contact_id          UUID NOT NULL REFERENCES contact(contact_id) ON DELETE CASCADE,
    account_id          UUID NOT NULL REFERENCES account(account_id),
    data                JSONB NOT NULL,
    timestamp           TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

ALTER TABLE publisher
    ADD COLUMN accessibility TEXT CHECK (octet_length(accessibility) >= 1);

CREATE TYPE accessibility_standard AS ENUM (
    'wcag-21-aa',
    'wcag-21-aaa',
    'wcag-22-aa',
    'wcag-22-aaa',
    'epub-a11y-10-aa',
    'epub-a11y-10-aaa',
    'epub-a11y-11-aa',
    'epub-a11y-11-aaa',
    'pdf-ua-1',
    'pdf-ua-2'
);

CREATE TYPE accessibility_exception AS ENUM (
    'micro-enterprises',
    'disproportionate-burden',
    'fundamental-alteration'
);

ALTER TABLE publication
    ADD COLUMN accessibility_standard accessibility_standard, -- WCAG only
    ADD COLUMN accessibility_additional_standard accessibility_standard, -- EPUB or PDF only
    ADD COLUMN accessibility_exception accessibility_exception,
    ADD COLUMN accessibility_report_url TEXT,

    -- Either standards or exception (or none, for excluded types)
    ADD CONSTRAINT check_standard_or_exception
        CHECK (
            (
                accessibility_exception IS NULL
                AND accessibility_standard IS NOT NULL
            )
            OR (
                accessibility_exception IS NOT NULL
                AND accessibility_standard IS NULL
                AND accessibility_additional_standard IS NULL
            )
            OR (
                accessibility_exception IS NULL
                AND accessibility_standard IS NULL
                AND accessibility_additional_standard IS NULL
            )
        ),

    -- Ensure additional_standard is only used for PDFs or EPUBs
    ADD CONSTRAINT check_additional_standard_pdf_epub
        CHECK (
            accessibility_additional_standard IS NULL
            OR publication_type IN ('PDF', 'Epub')
        ),

    -- Ensure standards are valid per publication type
    ADD CONSTRAINT check_accessibility_standard_rules
        CHECK (
            CASE publication_type
                WHEN 'Paperback' THEN accessibility_standard IS NULL AND accessibility_additional_standard IS NULL AND accessibility_exception IS NULL
                WHEN 'Hardback'  THEN accessibility_standard IS NULL AND accessibility_additional_standard IS NULL AND accessibility_exception IS NULL
                WHEN 'MP3'       THEN accessibility_standard IS NULL AND accessibility_additional_standard IS NULL AND accessibility_exception IS NULL
                WHEN 'WAV'       THEN accessibility_standard IS NULL AND accessibility_additional_standard IS NULL AND accessibility_exception IS NULL
                WHEN 'PDF'       THEN (
                    (accessibility_standard IS NULL OR accessibility_standard IN (
                        'wcag-21-aa','wcag-21-aaa',
                        'wcag-22-aa','wcag-22-aaa'
                    ))
                    AND
                    (accessibility_additional_standard IS NULL OR accessibility_additional_standard IN ('pdf-ua-1','pdf-ua-2'))
                )
                WHEN 'Epub'      THEN (
                    (accessibility_standard IS NULL OR accessibility_standard IN (
                        'wcag-21-aa','wcag-21-aaa',
                        'wcag-22-aa','wcag-22-aaa'
                    ))
                    AND
                    (accessibility_additional_standard IS NULL OR accessibility_additional_standard IN (
                        'epub-a11y-10-aa','epub-a11y-10-aaa',
                        'epub-a11y-11-aa','epub-a11y-11-aaa'
                    ))
                )
                ELSE (
                    (accessibility_standard IS NULL OR accessibility_standard IN (
                        'wcag-21-aa','wcag-21-aaa',
                        'wcag-22-aa','wcag-22-aaa'
                    ))
                    AND
                    accessibility_additional_standard IS NULL
                )
            END
        );
