ALTER TABLE imprint
    ADD crossmark_doi TEXT CHECK (doi ~* 'https:\/\/doi.org\/10.\d{4,9}\/[-._\;\(\)\/:a-zA-Z0-9]+$');
