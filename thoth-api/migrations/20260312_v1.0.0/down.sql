DROP INDEX IF EXISTS endorsement_author_institution_idx;

ALTER TABLE endorsement
  DROP COLUMN IF EXISTS author_institution_id,
  DROP COLUMN IF EXISTS author_orcid;

DROP INDEX IF EXISTS book_review_reviewer_institution_idx;

ALTER TABLE book_review
  DROP COLUMN IF EXISTS page_range,
  DROP COLUMN IF EXISTS reviewer_institution_id,
  DROP COLUMN IF EXISTS reviewer_orcid;

ALTER TABLE additional_resource
  DROP COLUMN IF EXISTS date;

ALTER TABLE award
  DROP COLUMN IF EXISTS role;

ALTER TABLE award
  RENAME COLUMN prize_statement TO note;

DROP TYPE IF EXISTS award_role;
