ALTER TABLE series
    -- Descrition of the series
    ADD COLUMN series_description TEXT CHECK (octet_length(series_description) >= 1),
    -- Call for proposals URL
    ADD COLUMN series_cfp_url TEXT CHECK (series_cfp_url ~* '^[^:]*:\/\/(?:[^\/:]*:[^\/@]*@)?(?:[^\/:.]*\.)+([^:\/]+)');

