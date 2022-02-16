ALTER TABLE publication
    ADD COLUMN weight_g double precision CHECK (weight_g > 0.0),
    ADD COLUMN weight_oz double precision CHECK (weight_oz > 0.0),
    ADD CONSTRAINT publication_non_physical_no_weight_g CHECK
        (weight_g IS NULL OR publication_type = 'Paperback' OR publication_type = 'Hardback'),
    ADD CONSTRAINT publication_non_physical_no_weight_oz CHECK
        (weight_oz IS NULL OR publication_type = 'Paperback' OR publication_type = 'Hardback');
