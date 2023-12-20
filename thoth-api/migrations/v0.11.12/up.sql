ALTER TABLE price ADD CONSTRAINT price_publication_id_currency_code_uniq
  UNIQUE (publication_id, currency_code);
