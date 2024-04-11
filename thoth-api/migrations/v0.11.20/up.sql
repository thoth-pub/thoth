ALTER TABLE work
    ADD COLUMN withdrawn_date DATE;

UPDATE work                              
    SET withdrawn_date = updated_at    
    WHERE (work_status = 'withdrawn-from-sale' 
    OR work_status = 'out-of-print');

ALTER TABLE work
    ADD CONSTRAINT work_withdrawn_date_check CHECK
        (((work_status = 'withdrawn-from-sale' OR work_status = 'out-of-print')
        AND withdrawn_date IS NOT NULL)
        OR (work_status NOT IN ('withdrawn-from-sale', 'out-of-print') AND withdrawn_date IS NULL));