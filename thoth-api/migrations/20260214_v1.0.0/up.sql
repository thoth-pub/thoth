ALTER TYPE file_type ADD VALUE IF NOT EXISTS 'additional_resource';
ALTER TYPE file_type ADD VALUE IF NOT EXISTS 'work_featured_video';

CREATE TYPE resource_type AS ENUM (
  'AUDIO',
  'VIDEO',
  'IMAGE',
  'BLOG',
  'WEBSITE',
  'DOCUMENT',
  'BOOK',
  'ARTICLE',
  'MAP',
  'SOURCE',
  'DATASET',
  'SPREADSHEET',
  'OTHER'
);

CREATE TABLE additional_resource (
  additional_resource_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  work_id UUID NOT NULL REFERENCES work(work_id) ON DELETE CASCADE,

  title TEXT NOT NULL CHECK (octet_length(title) >= 1),
  description TEXT,
  attribution TEXT,
  resource_type resource_type NOT NULL,

  doi TEXT,
  handle TEXT,
  url TEXT,

  resource_ordinal INTEGER NOT NULL DEFAULT 1 CHECK (resource_ordinal > 0),

  created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,

  CONSTRAINT additional_resource_resource_ordinal_work_id_uniq
    UNIQUE (work_id, resource_ordinal)
    DEFERRABLE INITIALLY IMMEDIATE
);
SELECT diesel_manage_updated_at('additional_resource');

CREATE TABLE award (
  award_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  work_id UUID NOT NULL REFERENCES work(work_id) ON DELETE CASCADE,

  title TEXT NOT NULL CHECK (octet_length(title) >= 1),
  url TEXT,
  category TEXT,
  note TEXT,

  award_ordinal INTEGER NOT NULL DEFAULT 1 CHECK (award_ordinal > 0),

  created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,

  CONSTRAINT award_award_ordinal_work_id_uniq
    UNIQUE (work_id, award_ordinal)
    DEFERRABLE INITIALLY IMMEDIATE
);
SELECT diesel_manage_updated_at('award');

CREATE TABLE endorsement (
  endorsement_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  work_id UUID NOT NULL REFERENCES work(work_id) ON DELETE CASCADE,

  author_name TEXT,
  author_role TEXT,
  url TEXT,
  text TEXT,

  endorsement_ordinal INTEGER NOT NULL DEFAULT 1 CHECK (endorsement_ordinal > 0),

  created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,

  CONSTRAINT endorsement_endorsement_ordinal_work_id_uniq
    UNIQUE (work_id, endorsement_ordinal)
    DEFERRABLE INITIALLY IMMEDIATE
);
SELECT diesel_manage_updated_at('endorsement');

CREATE TABLE book_review (
  book_review_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  work_id UUID NOT NULL REFERENCES work(work_id) ON DELETE CASCADE,

  title TEXT,
  author_name TEXT,
  url TEXT,
  doi TEXT,
  review_date DATE,
  journal_name TEXT,
  journal_volume TEXT,
  journal_number TEXT,
  journal_issn TEXT,
  text TEXT,

  review_ordinal INTEGER NOT NULL DEFAULT 1 CHECK (review_ordinal > 0),

  created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,

  CONSTRAINT book_review_review_ordinal_work_id_uniq
    UNIQUE (work_id, review_ordinal)
    DEFERRABLE INITIALLY IMMEDIATE
);
SELECT diesel_manage_updated_at('book_review');

CREATE TABLE work_featured_video (
  work_featured_video_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  work_id UUID NOT NULL UNIQUE REFERENCES work(work_id) ON DELETE CASCADE,

  title TEXT,
  url TEXT,
  width INTEGER NOT NULL DEFAULT 560 CHECK (width > 0),
  height INTEGER NOT NULL DEFAULT 315 CHECK (height > 0),

  created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
SELECT diesel_manage_updated_at('work_featured_video');

ALTER TABLE work
  ADD COLUMN resources_description TEXT;

ALTER TABLE file
  ADD COLUMN additional_resource_id UUID REFERENCES additional_resource(additional_resource_id) ON DELETE CASCADE,
  ADD COLUMN work_featured_video_id UUID REFERENCES work_featured_video(work_featured_video_id) ON DELETE CASCADE;

ALTER TABLE file_upload
  ADD COLUMN additional_resource_id UUID REFERENCES additional_resource(additional_resource_id) ON DELETE CASCADE,
  ADD COLUMN work_featured_video_id UUID REFERENCES work_featured_video(work_featured_video_id) ON DELETE CASCADE;

ALTER TABLE file DROP CONSTRAINT IF EXISTS file_type_check;
ALTER TABLE file_upload DROP CONSTRAINT IF EXISTS file_upload_type_check;

ALTER TABLE file
  ADD CONSTRAINT file_type_check
  CHECK (
    (file_type = 'frontcover' AND work_id IS NOT NULL AND publication_id IS NULL AND additional_resource_id IS NULL AND work_featured_video_id IS NULL) OR
    (file_type = 'publication' AND publication_id IS NOT NULL AND work_id IS NULL AND additional_resource_id IS NULL AND work_featured_video_id IS NULL) OR
    (
      file_type NOT IN ('frontcover', 'publication') AND
      work_id IS NULL AND
      publication_id IS NULL AND
      (
        (additional_resource_id IS NOT NULL AND work_featured_video_id IS NULL) OR
        (work_featured_video_id IS NOT NULL AND additional_resource_id IS NULL)
      )
    )
  );

ALTER TABLE file_upload
  ADD CONSTRAINT file_upload_type_check
  CHECK (
    (file_type = 'frontcover' AND work_id IS NOT NULL AND publication_id IS NULL AND additional_resource_id IS NULL AND work_featured_video_id IS NULL) OR
    (file_type = 'publication' AND publication_id IS NOT NULL AND work_id IS NULL AND additional_resource_id IS NULL AND work_featured_video_id IS NULL) OR
    (
      file_type NOT IN ('frontcover', 'publication') AND
      work_id IS NULL AND
      publication_id IS NULL AND
      (
        (additional_resource_id IS NOT NULL AND work_featured_video_id IS NULL) OR
        (work_featured_video_id IS NOT NULL AND additional_resource_id IS NULL)
      )
    )
  );

CREATE UNIQUE INDEX file_additional_resource_unique_idx
  ON file (additional_resource_id)
  WHERE additional_resource_id IS NOT NULL;

CREATE UNIQUE INDEX file_work_featured_video_unique_idx
  ON file (work_featured_video_id)
  WHERE work_featured_video_id IS NOT NULL;

CREATE INDEX file_upload_additional_resource_idx
  ON file_upload (additional_resource_id)
  WHERE additional_resource_id IS NOT NULL;

CREATE INDEX file_upload_work_featured_video_idx
  ON file_upload (work_featured_video_id)
  WHERE work_featured_video_id IS NOT NULL;

CREATE TABLE additional_resource_history (
  additional_resource_history_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  additional_resource_id UUID NOT NULL REFERENCES additional_resource(additional_resource_id) ON DELETE CASCADE,
  user_id TEXT NOT NULL,
  data JSONB NOT NULL,
  timestamp TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE award_history (
  award_history_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  award_id UUID NOT NULL REFERENCES award(award_id) ON DELETE CASCADE,
  user_id TEXT NOT NULL,
  data JSONB NOT NULL,
  timestamp TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE endorsement_history (
  endorsement_history_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  endorsement_id UUID NOT NULL REFERENCES endorsement(endorsement_id) ON DELETE CASCADE,
  user_id TEXT NOT NULL,
  data JSONB NOT NULL,
  timestamp TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE book_review_history (
  book_review_history_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  book_review_id UUID NOT NULL REFERENCES book_review(book_review_id) ON DELETE CASCADE,
  user_id TEXT NOT NULL,
  data JSONB NOT NULL,
  timestamp TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE work_featured_video_history (
  work_featured_video_history_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  work_featured_video_id UUID NOT NULL REFERENCES work_featured_video(work_featured_video_id) ON DELETE CASCADE,
  user_id TEXT NOT NULL,
  data JSONB NOT NULL,
  timestamp TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TRIGGER set_work_updated_at_with_relations
  AFTER INSERT OR DELETE OR UPDATE ON public.additional_resource
  FOR EACH ROW EXECUTE FUNCTION public.work_updated_at_with_relations();

CREATE TRIGGER set_work_updated_at_with_relations
  AFTER INSERT OR DELETE OR UPDATE ON public.award
  FOR EACH ROW EXECUTE FUNCTION public.work_updated_at_with_relations();

CREATE TRIGGER set_work_updated_at_with_relations
  AFTER INSERT OR DELETE OR UPDATE ON public.endorsement
  FOR EACH ROW EXECUTE FUNCTION public.work_updated_at_with_relations();

CREATE TRIGGER set_work_updated_at_with_relations
  AFTER INSERT OR DELETE OR UPDATE ON public.book_review
  FOR EACH ROW EXECUTE FUNCTION public.work_updated_at_with_relations();

CREATE TRIGGER set_work_updated_at_with_relations
  AFTER INSERT OR DELETE OR UPDATE ON public.work_featured_video
  FOR EACH ROW EXECUTE FUNCTION public.work_updated_at_with_relations();
