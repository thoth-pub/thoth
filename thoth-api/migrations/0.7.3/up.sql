ALTER TABLE publication
    ADD COLUMN weight_g double precision CHECK (weight_g > 0.0),
    ADD COLUMN weight_oz double precision CHECK (weight_oz > 0.0),
    ADD CONSTRAINT publication_non_physical_no_weight_g CHECK
        (weight_g IS NULL OR publication_type = 'Paperback' OR publication_type = 'Hardback'),
    ADD CONSTRAINT publication_non_physical_no_weight_oz CHECK
        (weight_oz IS NULL OR publication_type = 'Paperback' OR publication_type = 'Hardback'),
    ADD CONSTRAINT publication_weight_g_not_missing CHECK
        (weight_g IS NOT NULL OR weight_oz IS NULL),
    ADD CONSTRAINT publication_weight_oz_not_missing CHECK
        (weight_oz IS NOT NULL OR weight_g IS NULL);
