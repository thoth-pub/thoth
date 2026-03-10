ALTER TABLE imprint
  DROP COLUMN IF EXISTS default_currency,
  DROP COLUMN IF EXISTS default_place,
  DROP COLUMN IF EXISTS default_locale;
