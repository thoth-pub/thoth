ALTER TABLE publication
    DROP CONSTRAINT publication_non_physical_no_weight_g,
    DROP CONSTRAINT publication_non_physical_no_weight_oz,
    DROP CONSTRAINT publication_weight_g_not_missing,
    DROP CONSTRAINT publication_weight_oz_not_missing,
    DROP COLUMN weight_g,
    DROP COLUMN weight_oz;
