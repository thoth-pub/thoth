ALTER TABLE publication
    ADD COLUMN weight double precision CHECK (weight > 0.0),
    ADD CONSTRAINT publication_non_physical_no_weight CHECK
        (weight IS NULL OR publication_type = 'Paperback' OR publication_type = 'Hardback');
