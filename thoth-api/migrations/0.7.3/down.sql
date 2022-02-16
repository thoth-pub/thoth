ALTER TABLE publication
    DROP CONSTRAINT publication_non_physical_no_weight_g,
    DROP CONSTRAINT publication_non_physical_no_weight_oz,
    DROP COLUMN weight_g,
    DROP COLUMN weight_oz;
