CREATE TYPE award_role AS ENUM (
  'SHORT_LISTED',
  'WINNER',
  'LONG_LISTED',
  'COMMENDED',
  'RUNNER_UP',
  'JOINT_WINNER',
  'NOMINATED'
);

ALTER TABLE award
  RENAME COLUMN note TO prize_statement;

ALTER TABLE award
  ADD COLUMN role award_role;

ALTER TABLE additional_resource
  ADD COLUMN date DATE;

ALTER TABLE book_review
  ADD COLUMN reviewer_orcid TEXT,
  ADD COLUMN reviewer_institution_id UUID REFERENCES institution(institution_id) ON DELETE SET NULL,
  ADD COLUMN page_range TEXT;

CREATE INDEX book_review_reviewer_institution_idx
  ON book_review (reviewer_institution_id)
  WHERE reviewer_institution_id IS NOT NULL;

ALTER TABLE endorsement
  ADD COLUMN author_orcid TEXT,
  ADD COLUMN author_institution_id UUID REFERENCES institution(institution_id) ON DELETE SET NULL;

CREATE INDEX endorsement_author_institution_idx
  ON endorsement (author_institution_id)
  WHERE author_institution_id IS NOT NULL;
