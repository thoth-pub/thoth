ALTER TABLE series
    -- Description of the series
    ADD COLUMN series_description TEXT CHECK (octet_length(series_description) >= 1),
    -- Call for proposals URL
    ADD COLUMN series_cfp_url TEXT CHECK (series_cfp_url ~* '^[^:]*:\/\/(?:[^\/:]*:[^\/@]*@)?(?:[^\/:.]*\.)+([^:\/]+)');

ALTER TYPE publication_type ADD VALUE IF NOT EXISTS 'AZW3';
ALTER TYPE publication_type ADD VALUE IF NOT EXISTS 'DOCX',
ALTER TYPE publication_type ADD VALUE IF NOT EXISTS 'FictionBook';
