DELETE FROM price WHERE unit_price = 0.0;

ALTER TABLE price
    ADD CONSTRAINT price_unit_price_check CHECK (unit_price > 0.0);
