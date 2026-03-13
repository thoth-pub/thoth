ALTER TABLE public.language
  ADD COLUMN main_language  boolean DEFAULT false NOT NULL;

ALTER TABLE public.funding
  ADD COLUMN jurisdiction   text,
  ADD CONSTRAINT funding_jurisdiction_check CHECK ((octet_length(jurisdiction) >= 1));

ALTER TABLE public.issue
  DROP COLUMN IF EXISTS issue_number;
