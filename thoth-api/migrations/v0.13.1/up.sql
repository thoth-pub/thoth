-- Indexes account table
CREATE INDEX idx_account_email ON account (email);

-- Indexes publisher_account table
CREATE INDEX  ON publisher_account (account_id);

-- Indexes work table
CREATE INDEX idx_work_doi ON work (doi);
CREATE INDEX idx_work_reference ON work (reference);
CREATE INDEX idx_work_short_abstract_substr ON work (substring(short_abstract FROM 1 FOR 255));
CREATE INDEX idx_work_long_abstract_substr ON work (substring(long_abstract FROM 1 FOR 255));
CREATE INDEX idx_work_landing_page ON work (landing_page);
CREATE INDEX idx_work_imprint_id ON work (imprint_id);
CREATE INDEX idx_work_updated_at_with_relations_desc ON work (updated_at_with_relations DESC, work_id);
CREATE INDEX idx_work_full_title_asc ON work (full_title ASC, work_id);
CREATE INDEX idx_work_publication_date_asc ON work (publication_date ASC, work_id);
CREATE INDEX idx_work_publication_date_desc ON work (publication_date DESC, work_id);
CREATE INDEX idx_work_type_status_pub_date_desc
    ON work (work_type, work_status, publication_date DESC);
CREATE INDEX idx_work_books_pub_date_desc
    ON work (publication_date DESC)
    WHERE work_type IN ('monograph', 'edited-book', 'textbook') AND work_status = 'active';

-- Indexes work_relation table
CREATE INDEX idx_work_relation_relation_ordinal_relator_relation_type_asc
    ON work_relation (relation_ordinal ASC, relator_work_id, relation_type);
CREATE INDEX idx_work_relation_relation_ordinal_related_relation_type_asc
    ON work_relation (relation_ordinal ASC, related_work_id, relation_type);

-- Indexes publisher table
CREATE INDEX idx_publisher_publisher_name ON publisher (publisher_name);
CREATE INDEX idx_publisher_publisher_shortname ON publisher (publisher_shortname);

-- Indexes imprint table
CREATE INDEX idx_imprint_imprint_name ON imprint (imprint_name);
CREATE INDEX idx_imprint_imprint_url ON imprint (imprint_url);
CREATE INDEX idx_imprint_publisher_id ON imprint (publisher_id);

-- Indexes subject table
CREATE INDEX idx_subject_subject_code_asc ON subject (subject_code ASC, work_id);
CREATE INDEX idx_subject_subject_ordinal_asc ON subject (subject_ordinal ASC, work_id);

-- Indexes publication table
CREATE INDEX idx_publication_work_id ON publication (work_id);
CREATE INDEX idx_publication_isbn ON publication (isbn);
CREATE INDEX idx_publication_publication_type ON publication (publication_type);

-- Indexes location table
CREATE INDEX idx_location_location_platform_asc ON location (location_platform ASC, publication_id);

-- Indexes price table
CREATE INDEX idx_price_currency_code_asc ON price (currency_code ASC, publication_id);

-- Indexes contributor table
CREATE INDEX idx_contributor_full_name ON contributor (full_name);
CREATE INDEX idx_contributor_last_name ON contributor (last_name);
CREATE INDEX idx_contributor_orcid ON contributor (orcid);

-- Indexes contribution table
CREATE INDEX idx_contribution_work_id ON contribution (work_id);
CREATE INDEX idx_contribution_contributor_id ON contribution (contributor_id);
CREATE INDEX idx_contribution_ordinal_asc ON contribution (contribution_ordinal ASC, work_id);

-- Indexes affiliation table
CREATE INDEX idx_affiliation_contribution_id ON affiliation (contribution_id);
CREATE INDEX idx_affiliation_ordinal_asc ON affiliation (affiliation_ordinal ASC, contribution_id);

-- Indexes contributor table
CREATE INDEX idx_institution_institution_name ON institution (institution_name);
CREATE INDEX idx_institution_ror ON institution (ror);
CREATE INDEX idx_institution_institution_doi ON institution (institution_doi);

-- Indexes funding table
CREATE INDEX idx_funding_work_id ON funding (work_id);
CREATE INDEX idx_funding_program ON funding (program);

-- Indexes series table
CREATE INDEX idx_series_series_name ON series (series_name);
CREATE INDEX idx_series_issn_print ON series (issn_print);
CREATE INDEX idx_series_issn_digital ON series (issn_digital);
CREATE INDEX idx_series_series_url ON series (series_url);
CREATE INDEX idx_series_series_description ON series (series_description);
CREATE INDEX idx_series_imprint_id ON series (imprint_id);

-- Indexes issue table
CREATE INDEX idx_issue_ordinal_work_id_asc ON issue (issue_ordinal ASC, work_id);
CREATE INDEX idx_issue_ordinal_series_id_asc ON issue (issue_ordinal ASC, series_id);

-- Indexes language table
CREATE INDEX idx_language_language_code_asc ON language (language_code ASC, work_id);

-- Indexes reference table
CREATE INDEX idx_reference_work_id ON reference (work_id);
CREATE INDEX idx_reference_doi ON reference (doi);
CREATE INDEX idx_reference_unstructured_citation ON reference (unstructured_citation);
CREATE INDEX idx_reference_issn ON reference (issn);
CREATE INDEX idx_reference_isbn ON reference (isbn);
CREATE INDEX idx_reference_journal_title ON reference (journal_title);
CREATE INDEX idx_reference_article_title ON reference (article_title);
CREATE INDEX idx_reference_series_title ON reference (series_title);
CREATE INDEX idx_reference_volume_title ON reference (volume_title);
CREATE INDEX idx_reference_author_substr ON reference ((substring(author FROM 1 FOR 255)));
CREATE INDEX idx_reference_standard_designator ON reference (standard_designator);
CREATE INDEX idx_reference_standards_body_name ON reference (standards_body_name);
CREATE INDEX idx_reference_standards_body_acronym ON reference (standards_body_acronym);
