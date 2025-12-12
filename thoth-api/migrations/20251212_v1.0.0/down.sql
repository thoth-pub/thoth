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
