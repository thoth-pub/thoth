ALTER TABLE work DROP CONSTRAINT work_place_check;
ALTER TABLE work ADD CONSTRAINT work_reference_check1 CHECK (octet_length(reference) >= 1);