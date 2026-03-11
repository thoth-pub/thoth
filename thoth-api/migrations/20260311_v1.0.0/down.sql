ALTER TABLE public.language
  ADD COLUMN main_language  boolean DEFAULT false NOT NULL;

ALTER TABLE public.funding
  ADD COLUMN jurisdiction   text;

ALTER TABLE public.issue
  DROP COLUMN IF EXISTS issue_number;
