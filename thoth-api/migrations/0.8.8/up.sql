ALTER TABLE work
    ALTER COLUMN copyright_holder DROP NOT NULL;

UPDATE work SET page_interval = REPLACE(page_interval, '-', '–');