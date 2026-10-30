-- MET-WP1-06: Publisher-platform approval foundation (issue #878).
--
-- Additive and initially inactive. Creates the closed approval-status enum
-- and the single `metric_publisher_platform_approval` table: one durable
-- record of whether one publisher has an approval relationship with one
-- Metrics platform, and whether that approval permits usage submissions,
-- sales submissions, or both.
--
-- This slice stores approval structure only. There is deliberately no
-- approval-transition/administration behaviour, no `PUBLISHER_CONTROLLED`
-- platform-ownership enforcement, no package/capability entitlement check
-- and no publisher-import authorization: those belong to later bounded
-- WP3/WP5 work. The module exposes no GraphQL or administration surface, so
-- the enum below is not a `juniper::GraphQLEnum`.
--
-- Foreign-key decision (reviewed): both foreign keys — to canonical
-- `publisher` and to `metric_platform` — are direct and deliberately
-- non-cascading, so deleting a referenced publisher or platform fails
-- instead of silently deleting approval/audit state.
--
-- Uniqueness decision (reviewed): the approved design fixes exactly one
-- approval relationship per `(publisher_id, platform_id)` pair, enforced by
-- a UNIQUE constraint. No other uniqueness rule is invented.
--
-- Index decision (reviewed): the complete intended index set is exactly the
-- primary key plus the index PostgreSQL creates to enforce the
-- `(publisher_id, platform_id)` UNIQUE constraint. No speculative secondary
-- index is created: the approved design defines no other
-- publisher-platform-approval access pattern at this foundation stage.
--
-- `approved_by` decision (reviewed): `approved_by` is preserved from the
-- approved design as nullable UUID with deliberately **no** foreign key.
-- Current Thoth authentication exposes authenticated ZITADEL actor identity
-- as string-based application identity, and no canonical local user
-- table/relationship has been approved for this field. No conversion rule,
-- identity table or default-generation rule is invented here; a later
-- separately reviewed administrative approval write-path specification must
-- resolve actor/audit semantics before this field is populated by any write
-- path.
--
-- No trigger, stored procedure or CHECK constraint encodes any deferred
-- approval semantic (for example that `APPROVED` requires `approved_by` or
-- `approved_at`, that `PENDING`/`REVOKED` requires audit fields to be
-- null/non-null, that at least one submission flag must be true, or that the
-- referenced platform must currently be `PUBLISHER_CONTROLLED`). Those
-- questions belong to later bounded runtime/administrative work.
--
-- No approval row is seeded.

CREATE TYPE public.metric_publisher_platform_approval_status AS ENUM (
    'PENDING',
    'APPROVED',
    'REVOKED'
);

-- metric_publisher_platform_approval is one durable record of whether one
-- publisher has an approval relationship with one Metrics platform, and
-- whether that approval permits usage submissions, sales submissions, or
-- both. `usage_submission_enabled` and `sales_submission_enabled` are
-- independently representable non-null booleans with no invented default;
-- `approval_status` is the closed status above with no invented default.
-- `approved_by`, `approved_at` and `notes` remain nullable, with no invented
-- default and, for `approved_by`, deliberately no foreign key (see above).
CREATE TABLE public.metric_publisher_platform_approval (
    publisher_platform_approval_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    publisher_id uuid NOT NULL,
    platform_id uuid NOT NULL,
    usage_submission_enabled boolean NOT NULL,
    sales_submission_enabled boolean NOT NULL,
    approval_status public.metric_publisher_platform_approval_status NOT NULL,
    approved_by uuid,
    approved_at timestamp with time zone,
    notes text,
    CONSTRAINT metric_publisher_platform_approval_pkey PRIMARY KEY (publisher_platform_approval_id),
    CONSTRAINT metric_publisher_platform_approval_publisher_id_platform_id_key
        UNIQUE (publisher_id, platform_id),
    CONSTRAINT metric_publisher_platform_approval_publisher_id_fkey FOREIGN KEY (publisher_id)
        REFERENCES public.publisher(publisher_id),
    CONSTRAINT metric_publisher_platform_approval_platform_id_fkey FOREIGN KEY (platform_id)
        REFERENCES public.metric_platform(platform_id)
);
