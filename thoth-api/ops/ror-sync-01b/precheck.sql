-- ROR-SYNC-01B (thoth-pub/thoth#1023): read-only pre-check queries.
--
-- Run them before upgrading to the release that merges institutions sharing a
-- ROR ID, to see in advance what the migration will merge and whether anything
-- blocks it. They only read. A group is blocked when two of its rows have
-- Metrics references, when two hold a DOI, or when its Metrics-referenced row
-- and its DOI holder differ; query 2 then marks no survivor and no rule,
-- because the migration aborts. See README.md beside this file.

-- 1. Foreign keys on institution: must be exactly the nine of spec §1,
--    as (referencing table, column, delete action: c cascade, n set null, a no action).
SELECT c.conrelid::regclass AS referencing_table, a.attname AS column_name, c.confdeltype
FROM pg_constraint c
JOIN pg_attribute a ON a.attrelid = c.conrelid AND a.attnum = ANY (c.conkey)
WHERE c.contype = 'f' AND c.confrelid = 'institution'::regclass
ORDER BY 1, 2;

-- 2. Rows of every ROR ID held more than once: references, the S4 survivor,
--    the deciding rule, and whether the group is blocked (S3).
WITH grp AS (
  SELECT institution_id FROM institution
  WHERE ror IN (SELECT ror FROM institution WHERE ror IS NOT NULL GROUP BY ror HAVING count(*) > 1)
), refs AS (
  SELECT institution_id AS id, 'affiliation' AS t FROM affiliation WHERE institution_id IN (SELECT institution_id FROM grp)
  UNION ALL SELECT institution_id, 'funding' FROM funding WHERE institution_id IN (SELECT institution_id FROM grp)
  UNION ALL SELECT reviewer_institution_id, 'book_review' FROM book_review WHERE reviewer_institution_id IN (SELECT institution_id FROM grp)
  UNION ALL SELECT author_institution_id, 'endorsement' FROM endorsement WHERE author_institution_id IN (SELECT institution_id FROM grp)
  UNION ALL SELECT institution_id, 'metrics' FROM metric_record WHERE institution_id IN (SELECT institution_id FROM grp)
  UNION ALL SELECT institution_id, 'metrics' FROM metric_rollup_work_day WHERE institution_id IN (SELECT institution_id FROM grp)
  UNION ALL SELECT institution_id, 'metrics' FROM metric_rollup_work_institution_month WHERE institution_id IN (SELECT institution_id FROM grp)
  UNION ALL SELECT institution_id, 'metrics' FROM metric_rollup_work_institution_year WHERE institution_id IN (SELECT institution_id FROM grp)
), per_row AS (
  SELECT i.ror, i.institution_id, i.institution_name, i.institution_doi, i.country_code, i.created_at,
         count(r.t) FILTER (WHERE r.t = 'affiliation') AS affiliations,
         count(r.t) FILTER (WHERE r.t = 'funding') AS fundings,
         count(r.t) FILTER (WHERE r.t = 'book_review') AS book_reviews,
         count(r.t) FILTER (WHERE r.t = 'endorsement') AS endorsements,
         count(r.t) FILTER (WHERE r.t = 'metrics') AS metrics_references
  FROM institution i
  JOIN grp USING (institution_id)
  LEFT JOIN refs r ON r.id = i.institution_id
  GROUP BY i.institution_id
), ranked AS (
  SELECT p.*,
         p.affiliations + p.fundings + p.book_reviews + p.endorsements AS references_total,
         count(*) FILTER (WHERE p.metrics_references > 0) OVER g AS metrics_rows_in_group,
         count(p.institution_doi) OVER g AS doi_holders_in_group,
         bool_or(p.metrics_references > 0 AND p.institution_doi IS NOT NULL) OVER g AS one_row_has_both,
         row_number() OVER s AS position,
         nth_value(p.affiliations + p.fundings + p.book_reviews + p.endorsements, 1) OVER s_all AS references_1,
         nth_value(p.affiliations + p.fundings + p.book_reviews + p.endorsements, 2) OVER s_all AS references_2,
         nth_value(p.created_at, 1) OVER s_all AS created_1,
         nth_value(p.created_at, 2) OVER s_all AS created_2
  FROM per_row p
  WINDOW g AS (PARTITION BY p.ror),
         s AS (PARTITION BY p.ror
               ORDER BY (p.metrics_references > 0) DESC,
                        (p.institution_doi IS NOT NULL) DESC,
                        p.affiliations + p.fundings + p.book_reviews + p.endorsements DESC,
                        p.created_at, p.institution_id),
         s_all AS (s ROWS BETWEEN UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING)
), decided AS (
  SELECT r.*,
         (r.metrics_rows_in_group > 1
          OR r.doi_holders_in_group > 1
          OR (r.metrics_rows_in_group = 1 AND r.doi_holders_in_group = 1 AND NOT r.one_row_has_both)) AS group_blocked
  FROM ranked r
)
SELECT ror, institution_id, institution_name, institution_doi, country_code, created_at,
       affiliations, fundings, book_reviews, endorsements, metrics_references,
       CASE WHEN group_blocked THEN NULL ELSE position = 1 END AS survivor,
       CASE WHEN group_blocked THEN NULL
            WHEN metrics_rows_in_group > 0 THEN 0
            WHEN doi_holders_in_group > 0 THEN 1
            WHEN references_1 > references_2 THEN 2
            WHEN created_1 <> created_2 THEN 3
            ELSE 4 END AS deciding_rule,
       group_blocked
FROM decided
ORDER BY ror, institution_id;

-- 3. Distinct works the merge bumps, with their status. The committed text repeats
--    query 2's WITH clause, from grp to decided, verbatim where "..." stands here,
--    so that no input is built by hand.
WITH grp AS (
  SELECT institution_id FROM institution
  WHERE ror IN (SELECT ror FROM institution WHERE ror IS NOT NULL GROUP BY ror HAVING count(*) > 1)
), refs AS (
  SELECT institution_id AS id, 'affiliation' AS t FROM affiliation WHERE institution_id IN (SELECT institution_id FROM grp)
  UNION ALL SELECT institution_id, 'funding' FROM funding WHERE institution_id IN (SELECT institution_id FROM grp)
  UNION ALL SELECT reviewer_institution_id, 'book_review' FROM book_review WHERE reviewer_institution_id IN (SELECT institution_id FROM grp)
  UNION ALL SELECT author_institution_id, 'endorsement' FROM endorsement WHERE author_institution_id IN (SELECT institution_id FROM grp)
  UNION ALL SELECT institution_id, 'metrics' FROM metric_record WHERE institution_id IN (SELECT institution_id FROM grp)
  UNION ALL SELECT institution_id, 'metrics' FROM metric_rollup_work_day WHERE institution_id IN (SELECT institution_id FROM grp)
  UNION ALL SELECT institution_id, 'metrics' FROM metric_rollup_work_institution_month WHERE institution_id IN (SELECT institution_id FROM grp)
  UNION ALL SELECT institution_id, 'metrics' FROM metric_rollup_work_institution_year WHERE institution_id IN (SELECT institution_id FROM grp)
), per_row AS (
  SELECT i.ror, i.institution_id, i.institution_name, i.institution_doi, i.country_code, i.created_at,
         count(r.t) FILTER (WHERE r.t = 'affiliation') AS affiliations,
         count(r.t) FILTER (WHERE r.t = 'funding') AS fundings,
         count(r.t) FILTER (WHERE r.t = 'book_review') AS book_reviews,
         count(r.t) FILTER (WHERE r.t = 'endorsement') AS endorsements,
         count(r.t) FILTER (WHERE r.t = 'metrics') AS metrics_references
  FROM institution i
  JOIN grp USING (institution_id)
  LEFT JOIN refs r ON r.id = i.institution_id
  GROUP BY i.institution_id
), ranked AS (
  SELECT p.*,
         p.affiliations + p.fundings + p.book_reviews + p.endorsements AS references_total,
         count(*) FILTER (WHERE p.metrics_references > 0) OVER g AS metrics_rows_in_group,
         count(p.institution_doi) OVER g AS doi_holders_in_group,
         bool_or(p.metrics_references > 0 AND p.institution_doi IS NOT NULL) OVER g AS one_row_has_both,
         row_number() OVER s AS position,
         nth_value(p.affiliations + p.fundings + p.book_reviews + p.endorsements, 1) OVER s_all AS references_1,
         nth_value(p.affiliations + p.fundings + p.book_reviews + p.endorsements, 2) OVER s_all AS references_2,
         nth_value(p.created_at, 1) OVER s_all AS created_1,
         nth_value(p.created_at, 2) OVER s_all AS created_2
  FROM per_row p
  WINDOW g AS (PARTITION BY p.ror),
         s AS (PARTITION BY p.ror
               ORDER BY (p.metrics_references > 0) DESC,
                        (p.institution_doi IS NOT NULL) DESC,
                        p.affiliations + p.fundings + p.book_reviews + p.endorsements DESC,
                        p.created_at, p.institution_id),
         s_all AS (s ROWS BETWEEN UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING)
), decided AS (
  SELECT r.*,
         (r.metrics_rows_in_group > 1
          OR r.doi_holders_in_group > 1
          OR (r.metrics_rows_in_group = 1 AND r.doi_holders_in_group = 1 AND NOT r.one_row_has_both)) AS group_blocked
  FROM ranked r
),
merged AS (
  SELECT institution_id FROM decided WHERE position > 1 AND NOT group_blocked
)
SELECT w.work_id, w.work_status
FROM work w
WHERE w.work_id IN (
  SELECT c.work_id FROM affiliation a JOIN contribution c USING (contribution_id) WHERE a.institution_id IN (SELECT institution_id FROM merged)
  UNION SELECT f.work_id FROM funding f WHERE f.institution_id IN (SELECT institution_id FROM merged)
  UNION SELECT b.work_id FROM book_review b WHERE b.reviewer_institution_id IN (SELECT institution_id FROM merged)
  UNION SELECT e.work_id FROM endorsement e WHERE e.author_institution_id IN (SELECT institution_id FROM merged)
)
ORDER BY w.work_id;
