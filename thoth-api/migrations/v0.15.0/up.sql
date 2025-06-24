-- Create MarkupFormat enum
CREATE TYPE markup_format AS ENUM (
    'html',
    'markdown',
    'plain_text',
    'jats_xml'
);

-- Add markup_format column to title table with default value
ALTER TABLE title 
    ADD COLUMN markup_format markup_format NOT NULL DEFAULT 'plain_text';

-- Update existing titles to have plain_text as default markup format
UPDATE title 
    SET markup_format = 'jats_xml'::::locale_code, 
    WHERE markup_format IS NULL;
