ALTER TABLE imprint
  ADD COLUMN default_currency   currency_code,
  ADD COLUMN default_place      text,
  ADD COLUMN default_locale     locale_code;
