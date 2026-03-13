ALTER TABLE public.language
  DROP COLUMN IF EXISTS main_language;

ALTER TABLE public.funding
  DROP CONSTRAINT IF EXISTS funding_jurisdiction_check,
  DROP COLUMN IF EXISTS jurisdiction;

ALTER TABLE public.issue
  ADD COLUMN issue_number   integer;
