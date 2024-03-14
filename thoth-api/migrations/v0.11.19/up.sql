ALTER TABLE imprint
    ADD COLUMN crossmark_doi TEXT CHECK (crossmark_doi ~* 'https:\/\/doi.org\/10.\d{4,9}\/[-._\;\(\)\/:a-zA-Z0-9]+$');
