ALTER TABLE publication
    DROP CONSTRAINT publication_non_physical_no_weight,
    DROP COLUMN weight;
