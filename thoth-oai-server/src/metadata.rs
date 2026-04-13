use quick_xml::escape::escape;
use thoth_api::markup::{convert_from_jats, ConversionLimit, MarkupFormat};
use thoth_client::{
    AbstractType, ContributionType, LanguageRelation, PublicationType, RelationType, SubjectType,
    Work, WorkAbstracts, WorkContributions, WorkLanguages, WorkTitles,
};
use thoth_errors::ThothResult;

fn xml_escape(value: &str) -> String {
    escape(value).into_owned()
}

fn push_text_element(xml: &mut String, name: &str, text: &str) {
    xml.push('<');
    xml.push_str(name);
    xml.push('>');
    xml.push_str(&xml_escape(text));
    xml.push_str("</");
    xml.push_str(name);
    xml.push('>');
}

fn push_text_element_attrs(xml: &mut String, name: &str, attrs: &[(&str, String)], text: &str) {
    xml.push('<');
    xml.push_str(name);
    for (key, value) in attrs {
        xml.push(' ');
        xml.push_str(key);
        xml.push_str("=\"");
        xml.push_str(&xml_escape(value));
        xml.push('"');
    }
    xml.push('>');
    xml.push_str(&xml_escape(text));
    xml.push_str("</");
    xml.push_str(name);
    xml.push('>');
}

fn push_open_tag(xml: &mut String, name: &str, attrs: &[(&str, String)]) {
    xml.push('<');
    xml.push_str(name);
    for (key, value) in attrs {
        xml.push(' ');
        xml.push_str(key);
        xml.push_str("=\"");
        xml.push_str(&xml_escape(value));
        xml.push('"');
    }
    xml.push('>');
}

fn push_close_tag(xml: &mut String, name: &str) {
    xml.push_str("</");
    xml.push_str(name);
    xml.push('>');
}

fn doi_url(doi: &thoth_api::model::Doi) -> String {
    format!("https://doi.org/{doi}")
}

fn orcid_url(orcid: &thoth_api::model::Orcid) -> String {
    format!("https://orcid.org/{orcid}")
}

fn ror_url(ror: &thoth_api::model::Ror) -> String {
    format!("https://ror.org/{ror}")
}

fn work_url(work: &Work) -> String {
    format!("https://thoth.pub/books/{}", work.work_id)
}

fn canonical_title(work: &Work) -> Option<&WorkTitles> {
    work.titles
        .iter()
        .find(|title| title.canonical)
        .or_else(|| work.titles.first())
}

fn canonical_long_abstract(work: &Work) -> Option<&WorkAbstracts> {
    work.abstracts
        .iter()
        .find(|abstract_record| {
            abstract_record.abstract_type == AbstractType::LONG && abstract_record.canonical
        })
        .or_else(|| {
            work.abstracts
                .iter()
                .find(|abstract_record| abstract_record.abstract_type == AbstractType::LONG)
        })
}

fn canonical_short_abstract(work: &Work) -> Option<&WorkAbstracts> {
    work.abstracts
        .iter()
        .find(|abstract_record| {
            abstract_record.abstract_type == AbstractType::SHORT && abstract_record.canonical
        })
        .or_else(|| {
            work.abstracts
                .iter()
                .find(|abstract_record| abstract_record.abstract_type == AbstractType::SHORT)
        })
}

fn abstract_text(abstract_record: Option<&WorkAbstracts>) -> ThothResult<Option<String>> {
    abstract_record
        .map(|abstract_record| {
            convert_from_jats(
                &abstract_record.content,
                MarkupFormat::PlainText,
                ConversionLimit::Abstract,
            )
        })
        .transpose()
}

fn creators(work: &Work) -> impl Iterator<Item = &WorkContributions> {
    work.contributions
        .iter()
        .filter(|contribution| contribution.contribution_type == ContributionType::AUTHOR)
}

fn contributors(work: &Work) -> impl Iterator<Item = &WorkContributions> {
    work.contributions
        .iter()
        .filter(|contribution| contribution.contribution_type != ContributionType::AUTHOR)
}

fn main_language(work: &Work) -> Option<&WorkLanguages> {
    match work.languages.as_slice() {
        [] => None,
        [language] => Some(language),
        _ => work
            .languages
            .iter()
            .min_by_key(|language| match language.language_relation {
                LanguageRelation::TRANSLATED_INTO => 0,
                LanguageRelation::ORIGINAL => 1,
                LanguageRelation::TRANSLATED_FROM => 2,
                _ => 3,
            }),
    }
}

fn personal_name(contribution: &WorkContributions) -> String {
    match contribution.first_name.as_deref() {
        Some(first_name) if !first_name.is_empty() && !contribution.last_name.is_empty() => {
            format!("{}, {}", contribution.last_name, first_name)
        }
        _ if !contribution.full_name.is_empty() => contribution.full_name.clone(),
        _ => contribution.last_name.clone(),
    }
}

fn publication_type_value(publication_type: &PublicationType) -> &'static str {
    match publication_type {
        PublicationType::HARDBACK => "hardback",
        PublicationType::PAPERBACK => "paperback",
        PublicationType::PDF => "application/pdf",
        PublicationType::EPUB => "application/epub+zip",
        PublicationType::XML => "text/xml",
        PublicationType::HTML => "text/html",
        PublicationType::DOCX => {
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
        }
        PublicationType::MP3 => "audio/mpeg",
        PublicationType::WAV => "audio/wav",
        PublicationType::MOBI => "application/x-mobipocket-ebook",
        PublicationType::AZW3 => "application/vnd.amazon.ebook",
        PublicationType::FICTION_BOOK => "application/x-fictionbook+xml",
        PublicationType::Other(_) => "application/octet-stream",
    }
}

fn dc_type(work: &Work) -> &'static str {
    match work.work_type {
        thoth_client::WorkType::JOURNAL_ISSUE => "issue",
        thoth_client::WorkType::BOOK_CHAPTER => "chapter",
        thoth_client::WorkType::Other(_) => "book",
        _ => "book",
    }
}

fn openaire_resource_type(work: &Work) -> Option<(&'static str, &'static str)> {
    match work.work_type {
        thoth_client::WorkType::JOURNAL_ISSUE => {
            Some(("http://purl.org/coar/resource_type/c_0640", "journal"))
        }
        thoth_client::WorkType::BOOK_CHAPTER => {
            Some(("http://purl.org/coar/resource_type/c_3248", "book part"))
        }
        thoth_client::WorkType::MONOGRAPH
        | thoth_client::WorkType::TEXTBOOK
        | thoth_client::WorkType::EDITED_BOOK
        | thoth_client::WorkType::BOOK_SET => {
            Some(("http://purl.org/coar/resource_type/c_2f33", "book"))
        }
        thoth_client::WorkType::Other(_) => None,
    }
}

fn normalized_license_name(license: &str) -> &str {
    match license.trim_end_matches('/') {
        "http://creativecommons.org/publicdomain/zero/1.0" => "CC0 1.0 Universal",
        "http://creativecommons.org/licenses/by/4.0" => "CC BY 4.0",
        "http://creativecommons.org/licenses/by-sa/4.0" => "CC BY-SA 4.0",
        "http://creativecommons.org/licenses/by-nc/4.0" => "CC BY-NC 4.0",
        "http://creativecommons.org/licenses/by-nc-sa/4.0" => "CC BY-NC-SA 4.0",
        "http://creativecommons.org/licenses/by-nd/4.0" => "CC BY-ND 4.0",
        "http://creativecommons.org/licenses/by-nc-nd/4.0" => "CC BY-NC-ND 4.0",
        "http://creativecommons.org/licenses/by/3.0" => "CC BY 3.0",
        "http://creativecommons.org/licenses/by-sa/3.0" => "CC BY-SA 3.0",
        "http://creativecommons.org/licenses/by-nc/3.0" => "CC BY-NC 3.0",
        "http://creativecommons.org/licenses/by-nc-sa/3.0" => "CC BY-NC-SA 3.0",
        "http://creativecommons.org/licenses/by-nd/3.0" => "CC BY-ND 3.0",
        "http://creativecommons.org/licenses/by-nc-nd/3.0" => "CC BY-NC-ND 3.0",
        _ => license,
    }
}

fn parent_work(work: &Work) -> Option<&thoth_client::WorkRelationsRelatedWork> {
    work.relations
        .iter()
        .find(|relation| relation.relation_type == RelationType::IS_CHILD_OF)
        .map(|relation| &relation.related_work)
}

pub(crate) fn map_oai_dc(work: &Work) -> ThothResult<String> {
    let mut xml = String::from(
        r#"<oai_dc:dc xmlns:oai_dc="http://www.openarchives.org/OAI/2.0/oai_dc/" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:schemaLocation="http://www.openarchives.org/OAI/2.0/oai_dc/ http://www.openarchives.org/OAI/2.0/oai_dc.xsd">"#,
    );

    if let Some(title) = canonical_title(work) {
        push_text_element(&mut xml, "dc:title", &title.full_title);
    }

    for creator in creators(work) {
        push_text_element(&mut xml, "dc:creator", &creator.full_name);
    }

    for subject in work
        .subjects
        .iter()
        .filter(|subject| subject.subject_type == SubjectType::KEYWORD)
    {
        push_text_element(&mut xml, "dc:subject", &subject.subject_code);
    }

    if let Some(description) = abstract_text(canonical_long_abstract(work))? {
        push_text_element(&mut xml, "dc:description", &description);
    }

    push_text_element(
        &mut xml,
        "dc:publisher",
        &work.imprint.publisher.publisher_name,
    );

    for contributor in contributors(work) {
        push_text_element(&mut xml, "dc:contributor", &contributor.full_name);
    }

    if let Some(publication_date) = &work.publication_date {
        push_text_element(&mut xml, "dc:date", &publication_date.to_string());
    }

    push_text_element(&mut xml, "dc:type", dc_type(work));

    for publication in &work.publications {
        push_text_element(
            &mut xml,
            "dc:format",
            publication_type_value(&publication.publication_type),
        );
    }

    push_text_element(&mut xml, "dc:identifier", &work_url(work));
    if let Some(doi) = &work.doi {
        push_text_element(&mut xml, "dc:identifier", &doi_url(doi));
    }
    for publication in &work.publications {
        if let Some(isbn) = &publication.isbn {
            push_text_element(&mut xml, "dc:identifier", &format!("urn:isbn:{isbn}"));
        }
    }

    if let Some(language) = main_language(work) {
        push_text_element(
            &mut xml,
            "dc:language",
            &language.language_code.to_string().to_lowercase(),
        );
    }

    for relation in &work.relations {
        if let Some(doi) = &relation.related_work.doi {
            push_text_element(&mut xml, "dc:relation", &doi_url(doi));
        }
        for publication in &relation.related_work.publications {
            if let Some(isbn) = &publication.isbn {
                push_text_element(&mut xml, "dc:relation", &format!("urn:isbn:{isbn}"));
            }
        }
    }

    if let Some(license) = &work.license {
        push_text_element(&mut xml, "dc:rights", license);
    }

    xml.push_str("</oai_dc:dc>");
    Ok(xml)
}

pub(crate) fn map_oai_openaire(work: &Work) -> ThothResult<String> {
    let mut xml = String::from(
        r#"<oaire:resource xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:dcterms="http://purl.org/dc/terms/" xmlns:datacite="http://datacite.org/schema/kernel-4" xmlns:oaire="http://namespace.openaire.eu/schema/oaire/" xsi:schemaLocation="http://namespace.openaire.eu/schema/oaire/ https://www.openaire.eu/schema/repo-lit/4.0/openaire.xsd">"#,
    );

    push_text_element_attrs(
        &mut xml,
        "datacite:identifier",
        &[("identifierType", "URL".to_string())],
        &work_url(work),
    );

    push_open_tag(&mut xml, "datacite:titles", &[]);
    if let Some(title) = canonical_title(work) {
        push_text_element(&mut xml, "datacite:title", &title.title);
        if let Some(subtitle) = &title.subtitle {
            push_text_element_attrs(
                &mut xml,
                "datacite:title",
                &[("titleType", "Subtitle".to_string())],
                subtitle,
            );
        }
    }
    push_close_tag(&mut xml, "datacite:titles");

    push_open_tag(&mut xml, "datacite:creators", &[]);
    for creator in creators(work) {
        push_open_tag(&mut xml, "datacite:creator", &[]);
        push_text_element_attrs(
            &mut xml,
            "datacite:creatorName",
            &[("nameType", "Personal".to_string())],
            &personal_name(creator),
        );
        if let Some(first_name) = creator.first_name.as_deref() {
            if !first_name.is_empty() {
                push_text_element(&mut xml, "datacite:givenName", first_name);
            }
        }
        if !creator.last_name.is_empty() {
            push_text_element(&mut xml, "datacite:familyName", &creator.last_name);
        }
        if let Some(orcid) = &creator.contributor.orcid {
            push_text_element_attrs(
                &mut xml,
                "datacite:nameIdentifier",
                &[
                    ("nameIdentifierScheme", "ORCID".to_string()),
                    ("schemeURI", "https://orcid.org/".to_string()),
                ],
                &orcid_url(orcid),
            );
        }
        for affiliation in &creator.affiliations {
            if let Some(ror) = &affiliation.institution.ror {
                push_text_element_attrs(
                    &mut xml,
                    "datacite:affiliation",
                    &[("affiliationIdentifier", ror_url(ror))],
                    &affiliation.institution.institution_name,
                );
            } else {
                push_text_element(
                    &mut xml,
                    "datacite:affiliation",
                    &affiliation.institution.institution_name,
                );
            }
        }
        push_close_tag(&mut xml, "datacite:creator");
    }
    push_close_tag(&mut xml, "datacite:creators");

    push_open_tag(&mut xml, "datacite:contributors", &[]);
    for contributor in contributors(work) {
        let contributor_type = if contributor.contribution_type == ContributionType::EDITOR {
            "Editor"
        } else {
            "Other"
        };
        push_open_tag(
            &mut xml,
            "datacite:contributor",
            &[("contributorType", contributor_type.to_string())],
        );
        push_text_element_attrs(
            &mut xml,
            "datacite:creatorName",
            &[("nameType", "Personal".to_string())],
            &personal_name(contributor),
        );
        if let Some(first_name) = contributor.first_name.as_deref() {
            if !first_name.is_empty() {
                push_text_element(&mut xml, "datacite:givenName", first_name);
            }
        }
        if !contributor.last_name.is_empty() {
            push_text_element(&mut xml, "datacite:familyName", &contributor.last_name);
        }
        if let Some(orcid) = &contributor.contributor.orcid {
            push_text_element_attrs(
                &mut xml,
                "datacite:nameIdentifier",
                &[
                    ("nameIdentifierScheme", "ORCID".to_string()),
                    ("schemeURI", "https://orcid.org/".to_string()),
                ],
                &orcid_url(orcid),
            );
        }
        for affiliation in &contributor.affiliations {
            if let Some(ror) = &affiliation.institution.ror {
                push_text_element_attrs(
                    &mut xml,
                    "datacite:affiliation",
                    &[("affiliationIdentifier", ror_url(ror))],
                    &affiliation.institution.institution_name,
                );
            } else {
                push_text_element(
                    &mut xml,
                    "datacite:affiliation",
                    &affiliation.institution.institution_name,
                );
            }
        }
        push_close_tag(&mut xml, "datacite:contributor");
    }
    push_close_tag(&mut xml, "datacite:contributors");

    push_open_tag(&mut xml, "oaire:fundingReferences", &[]);
    for funding in &work.fundings {
        push_open_tag(&mut xml, "oaire:fundingReference", &[]);
        push_text_element(
            &mut xml,
            "oaire:funderName",
            &funding.institution.institution_name,
        );
        if let Some(ror) = &funding.institution.ror {
            push_text_element_attrs(
                &mut xml,
                "oaire:funderIdentifier",
                &[("funderIdentifierType", "ROR".to_string())],
                &ror_url(ror),
            );
        }
        if let Some(grant_number) = &funding.grant_number {
            push_text_element(&mut xml, "oaire:awardNumber", grant_number);
        }
        if let Some(project_name) = &funding.project_name {
            push_text_element(&mut xml, "oaire:awardTitle", project_name);
        }
        push_close_tag(&mut xml, "oaire:fundingReference");
    }
    push_close_tag(&mut xml, "oaire:fundingReferences");

    push_open_tag(&mut xml, "datacite:alternateIdentifiers", &[]);
    if let Some(doi) = &work.doi {
        push_text_element_attrs(
            &mut xml,
            "datacite:alternateIdentifier",
            &[("alternateIdentifierType", "DOI".to_string())],
            &doi_url(doi),
        );
    }
    if let Some(landing_page) = &work.landing_page {
        push_text_element_attrs(
            &mut xml,
            "datacite:alternateIdentifier",
            &[("alternateIdentifierType", "URL".to_string())],
            landing_page,
        );
    }
    for publication in &work.publications {
        if let Some(isbn) = &publication.isbn {
            push_text_element_attrs(
                &mut xml,
                "datacite:alternateIdentifier",
                &[("alternateIdentifierType", "ISBN".to_string())],
                &isbn.to_string(),
            );
        }
    }
    push_close_tag(&mut xml, "datacite:alternateIdentifiers");

    push_open_tag(&mut xml, "datacite:relatedIdentifiers", &[]);
    for relation in &work.relations {
        let relation_type = if matches!(
            relation.relation_type,
            RelationType::HAS_CHILD | RelationType::HAS_PART
        ) {
            "HasPart"
        } else {
            "IsPartOf"
        };
        if let Some(doi) = &relation.related_work.doi {
            push_text_element_attrs(
                &mut xml,
                "datacite:relatedIdentifier",
                &[
                    ("relatedIdentifierType", "DOI".to_string()),
                    ("relationType", relation_type.to_string()),
                ],
                &doi_url(doi),
            );
        }
        if let Some(landing_page) = &relation.related_work.landing_page {
            push_text_element_attrs(
                &mut xml,
                "datacite:relatedIdentifier",
                &[
                    ("relatedIdentifierType", "URL".to_string()),
                    ("relationType", relation_type.to_string()),
                ],
                landing_page,
            );
        }
        for publication in &relation.related_work.publications {
            if let Some(isbn) = &publication.isbn {
                push_text_element_attrs(
                    &mut xml,
                    "datacite:relatedIdentifier",
                    &[
                        ("relatedIdentifierType", "ISBN".to_string()),
                        ("relationType", relation_type.to_string()),
                    ],
                    &isbn.to_string(),
                );
            }
        }
    }
    push_close_tag(&mut xml, "datacite:relatedIdentifiers");

    for language in &work.languages {
        push_text_element(
            &mut xml,
            "dc:language",
            &language.language_code.to_string().to_lowercase(),
        );
    }

    push_text_element(
        &mut xml,
        "dc:publisher",
        &work.imprint.publisher.publisher_name,
    );

    if let Some(publication_date) = &work.publication_date {
        push_text_element_attrs(
            &mut xml,
            "datacite:date",
            &[("dateType", "Issued".to_string())],
            &publication_date.to_string(),
        );
    }

    if let Some((uri, value)) = openaire_resource_type(work) {
        push_text_element_attrs(
            &mut xml,
            "oaire:resourceType",
            &[
                ("resourceTypeGeneral", "literature".to_string()),
                ("uri", uri.to_string()),
            ],
            value,
        );
    }

    if let Some(short_abstract) = abstract_text(canonical_short_abstract(work))? {
        push_text_element(&mut xml, "dc:description", &short_abstract);
    }
    if let Some(long_abstract) = abstract_text(canonical_long_abstract(work))? {
        push_text_element(&mut xml, "dc:description", &long_abstract);
    }
    if let Some(toc) = &work.toc {
        push_text_element(&mut xml, "dc:description", toc);
    }

    for publication in &work.publications {
        push_text_element(
            &mut xml,
            "dc:format",
            publication_type_value(&publication.publication_type),
        );
    }

    if let Some(license) = &work.license {
        push_text_element_attrs(
            &mut xml,
            "datacite:rights",
            &[(
                "rightsURI",
                "http://purl.org/coar/access_right/c_abf2".to_string(),
            )],
            "open access",
        );
        push_text_element_attrs(
            &mut xml,
            "oaire:licenseCondition",
            &[("uri", license.clone())],
            normalized_license_name(license),
        );
    } else {
        push_text_element_attrs(
            &mut xml,
            "datacite:rights",
            &[(
                "rightsURI",
                "http://purl.org/coar/access_right/c_16ec".to_string(),
            )],
            "restricted access",
        );
    }

    for subject in &work.subjects {
        match subject.subject_type {
            SubjectType::KEYWORD | SubjectType::CUSTOM => {
                push_text_element(&mut xml, "datacite:subject", &subject.subject_code);
            }
            SubjectType::THEMA => {
                push_text_element_attrs(
                    &mut xml,
                    "datacite:subject",
                    &[("subjectScheme", "Thema".to_string())],
                    &subject.subject_code,
                );
            }
            _ => {
                push_text_element_attrs(
                    &mut xml,
                    "datacite:subject",
                    &[("subjectScheme", subject.subject_type.to_string())],
                    &subject.subject_code,
                );
            }
        }
    }

    let mut sizes = Vec::new();
    if let Some(page_count) = work.page_count {
        sizes.push(format!("{page_count} pages"));
    }
    if let Some(image_count) = work.image_count {
        sizes.push(format!("{image_count} images"));
    }
    if let Some(table_count) = work.table_count {
        sizes.push(format!("{table_count} tables"));
    }
    if let Some(audio_count) = work.audio_count {
        sizes.push(format!("{audio_count} audios"));
    }
    if let Some(video_count) = work.video_count {
        sizes.push(format!("{video_count} videos"));
    }
    if !sizes.is_empty() {
        push_open_tag(&mut xml, "datacite:sizes", &[]);
        for size in sizes {
            push_text_element(&mut xml, "datacite:size", &size);
        }
        push_close_tag(&mut xml, "datacite:sizes");
    }

    for publication in &work.publications {
        for location in &publication.locations {
            if let Some(full_text_url) = &location.full_text_url {
                push_text_element_attrs(
                    &mut xml,
                    "oaire:file",
                    &[
                        (
                            "mimeType",
                            publication_type_value(&publication.publication_type).to_string(),
                        ),
                        ("objectType", "fulltext".to_string()),
                    ],
                    full_text_url,
                );
            }
        }
    }

    if work.work_type == thoth_client::WorkType::BOOK_CHAPTER {
        if let Some(parent_work) = parent_work(work) {
            if let Some(parent_title) = parent_work
                .titles
                .iter()
                .find(|title| title.canonical)
                .or_else(|| parent_work.titles.first())
            {
                push_text_element(&mut xml, "oaire:citationTitle", &parent_title.full_title);
            }
            if let Some(edition) = parent_work.edition {
                push_text_element(&mut xml, "oaire:citationEdition", &edition.to_string());
            }
        }
    } else if let Some(issue) = work.issues.first() {
        push_text_element(&mut xml, "oaire:citationTitle", &issue.series.series_name);
        push_text_element(
            &mut xml,
            "oaire:citationIssue",
            &issue.issue_ordinal.to_string(),
        );
    }

    if let Some(first_page) = &work.first_page {
        push_text_element(&mut xml, "oaire:citationStartPage", first_page);
    }
    if let Some(last_page) = &work.last_page {
        push_text_element(&mut xml, "oaire:citationEndPage", last_page);
    }

    xml.push_str("</oaire:resource>");
    Ok(xml)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xml_publication_type_maps_to_text_xml() {
        assert_eq!(publication_type_value(&PublicationType::XML), "text/xml");
    }
}
