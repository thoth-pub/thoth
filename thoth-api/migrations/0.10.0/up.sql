ALTER TABLE work
    ADD COLUMN IF NOT EXISTS bibliography_note TEXT CHECK (octet_length(bibliography_note) >= 1);
