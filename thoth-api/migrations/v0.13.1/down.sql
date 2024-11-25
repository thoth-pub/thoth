-- Remove indexes from account table
DROP INDEX IF EXISTS idx_account_email;

-- Remove indexes from publisher_account table
DROP INDEX IF EXISTS idx_publisher_account_account_id;

-- Remove indexes from work table
DROP INDEX IF EXISTS idx_work_work_id;
DROP INDEX IF EXISTS idx_work_doi;
DROP INDEX IF EXISTS idx_work_reference;
DROP INDEX IF EXISTS idx_work_short_abstract_substr;
DROP INDEX IF EXISTS idx_work_long_abstract_substr;
DROP INDEX IF EXISTS idx_work_landing_page;
DROP INDEX IF EXISTS idx_work_imprint_id;
DROP INDEX IF EXISTS idx_work_updated_at_with_relations_desc;
DROP INDEX IF EXISTS idx_work_full_title_asc;
DROP INDEX IF EXISTS idx_work_publication_date_asc;
DROP INDEX IF EXISTS idx_work_publication_date_desc;

-- Remove indexes from work_relation table
DROP INDEX IF EXISTS idx_work_relation_relation_ordinal_relator_asc;
DROP INDEX IF EXISTS idx_work_relation_relation_ordinal_related_asc;

-- Remove indexes from publisher table
DROP INDEX IF EXISTS idx_publisher_publisher_id;
DROP INDEX IF EXISTS idx_publisher_publisher_name;
DROP INDEX IF EXISTS idx_publisher_publisher_shortname;

-- Remove indexes from imprint table
DROP INDEX IF EXISTS idx_imprint_id;
DROP INDEX IF EXISTS idx_imprint_imprint_name;
DROP INDEX IF EXISTS idx_imprint_imprint_url;
DROP INDEX IF EXISTS idx_imprint_publisher_id;

-- Remove indexes from subject table
DROP INDEX IF EXISTS idx_subject_subject_code_asc;

-- Remove indexes from publication table
DROP INDEX IF EXISTS idx_publication_work_id;
DROP INDEX IF EXISTS idx_publication_isbn;
DROP INDEX IF EXISTS idx_publication_publication_type;

-- Remove indexes from location table
DROP INDEX IF EXISTS idx_location_location_platform_asc;

-- Remove indexes from price table
DROP INDEX IF EXISTS idx_price_currency_code_asc;

-- Remove indexes from contributor table
DROP INDEX IF EXISTS idx_contributor_full_name;
DROP INDEX IF EXISTS idx_contributor_last_name;
DROP INDEX IF EXISTS idx_contributor_orcid;

-- Remove indexes from contribution table
DROP INDEX IF EXISTS idx_contribution_work_id;
DROP INDEX IF EXISTS idx_contribution_contributor_id;
DROP INDEX IF EXISTS idx_contribution_ordinal_asc;

-- Remove indexes from affiliation table
DROP INDEX IF EXISTS idx_affiliation_contribution_id;
DROP INDEX IF EXISTS idx_affiliation_ordinal_asc;

-- Remove indexes from institution table
DROP INDEX IF EXISTS idx_institution_institution_name;
DROP INDEX IF EXISTS idx_institution_ror;
DROP INDEX IF EXISTS idx_institution_institution_doi;

-- Remove indexes from funding table
DROP INDEX IF EXISTS idx_funding_work_id;
DROP INDEX IF EXISTS idx_funding_program;

-- Remove indexes from series table
DROP INDEX IF EXISTS idx_series_series_name;
DROP INDEX IF EXISTS idx_series_issn_print;
DROP INDEX IF EXISTS idx_series_issn_digital;
DROP INDEX IF EXISTS idx_series_series_url;
DROP INDEX IF EXISTS idx_series_series_description;
DROP INDEX IF EXISTS idx_series_imprint_id;

-- Remove indexes from issue table
DROP INDEX IF EXISTS idx_issue_ordinal_work_id_asc;
DROP INDEX IF EXISTS idx_issue_ordinal_series_id_asc;

-- Remove indexes from language table
DROP INDEX IF EXISTS idx_language_language_code_asc;

-- Remove indexes from reference table
DROP INDEX IF EXISTS idx_reference_work_id;
DROP INDEX IF EXISTS idx_reference_doi;
DROP INDEX IF EXISTS idx_reference_unstructured_citation;
DROP INDEX IF EXISTS idx_reference_issn;
DROP INDEX IF EXISTS idx_reference_isbn;
DROP INDEX IF EXISTS idx_reference_journal_title;
DROP INDEX IF EXISTS idx_reference_article_title;
DROP INDEX IF EXISTS idx_reference_series_title;
DROP INDEX IF EXISTS idx_reference_volume_title;
DROP INDEX IF EXISTS idx_reference_author_substr;
DROP INDEX IF EXISTS idx_reference_standard_designator;
DROP INDEX IF EXISTS idx_reference_standards_body_name;
DROP INDEX IF EXISTS idx_reference_standards_body_acronym;
