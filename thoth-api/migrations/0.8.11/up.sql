ALTER TABLE work DROP CONSTRAINT work_reference_check1;
ALTER TABLE work ADD CONSTRAINT work_place_check CHECK (octet_length(place) >= 1);