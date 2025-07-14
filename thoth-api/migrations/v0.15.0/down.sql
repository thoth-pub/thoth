-- Recreate short_abstract and long_abstract columns in the work table
ALTER TABLE work
    ADD COLUMN short_abstract TEXT CHECK (octet_length(short_abstract) >= 1),
    ADD COLUMN long_abstract TEXT CHECK (octet_length(long_abstract) >= 1);

-- Migrate data back from the abstract table to the work table
UPDATE work
SET short_abstract = abstract.content
FROM abstract
WHERE abstract.work_id = work.work_id
  AND abstract.abstract_type = 'short';

UPDATE work
SET long_abstract = abstract.content
FROM abstract
WHERE abstract.work_id = work.work_id
  AND abstract.abstract_type = 'long';

-- Drop the abstract table
DROP TABLE IF EXISTS abstract;

-- Drop the AbstractType enum
DROP TYPE IF EXISTS abstract_type;

-- Drop unique indexes created for the abstract table
DROP INDEX IF EXISTS abstract_unique_canonical_true_idx;
DROP INDEX IF EXISTS abstract_uniq_locale_idx;

DROP TABLE IF EXISTS abstract;