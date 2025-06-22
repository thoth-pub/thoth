-- Remove markup_format column from title table
ALTER TABLE title 
    DROP COLUMN markup_format;

-- Drop MarkupFormat enum
DROP TYPE markup_format;
