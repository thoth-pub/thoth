CREATE TYPE length_unit AS ENUM (
    'mm',
    'cm',
    'in'
);

ALTER TABLE work
    ALTER COLUMN width TYPE double precision,
    ALTER COLUMN height TYPE double precision;
