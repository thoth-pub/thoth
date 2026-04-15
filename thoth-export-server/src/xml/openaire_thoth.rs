use super::{write_element_block, XmlElementBlock, XmlSpecification};
use std::collections::HashSet;
use std::io::Write;

use thoth_api::markup::{convert_from_jats, ConversionLimit, MarkupFormat};
use thoth_client::{
    AbstractType, ContributionType, LanguageRelation, PublicationType, RelationType, SubjectType,
    Work, WorkAbstracts, WorkContributions, WorkLanguages, WorkTitles,
};
use thoth_errors::{ThothError, ThothResult};
use uuid::Uuid;
use xml::writer::events::StartElementBuilder;
#[cfg(test)]
use xml::writer::EmitterConfig;
use xml::writer::{EventWriter, XmlEvent};

const OPENAIRE_ERROR: &str = "openaire::thoth";
const OAI_IDENTIFIER_PREFIX: &str = "oai:thoth.pub";
const BY_WORK_ONLY_MESSAGE: &str = "Output can only be generated for one work at a time";
const OPENAIRE_NS: &[(&str, &str)] = &[
    ("xmlns:rdf", "http://www.w3.org/1999/02/22-rdf-syntax-ns#"),
    ("xmlns:xsi", "http://www.w3.org/2001/XMLSchema-instance"),
    ("xmlns:dc", "http://purl.org/dc/elements/1.1/"),
    ("xmlns:dcterms", "http://purl.org/dc/terms/"),
    ("xmlns:datacite", "http://datacite.org/schema/kernel-4"),
    ("xmlns:oaire", "http://namespace.openaire.eu/schema/oaire/"),
    (
        "xsi:schemaLocation",
        "http://namespace.openaire.eu/schema/oaire/ https://www.openaire.eu/schema/repo-lit/4.0/openaire.xsd",
    ),
];

#[derive(Copy, Clone)]
pub(crate) struct OpenaireThoth;

impl XmlSpecification for OpenaireThoth {
    fn handle_event<W: Write>(w: &mut EventWriter<W>, works: &[Work]) -> ThothResult<()> {
        match works {
            [] => Err(ThothError::IncompleteMetadataRecord(
                OPENAIRE_ERROR.to_string(),
                "Not enough data".to_string(),
            )),
            [work] => XmlElementBlock::<OpenaireThoth>::xml_element(work, w),
            _ => Err(ThothError::IncompleteMetadataRecord(
                OPENAIRE_ERROR.to_string(),
                BY_WORK_ONLY_MESSAGE.to_string(),
            )),
        }
    }
}

impl XmlElementBlock<OpenaireThoth> for Work {
    fn xml_element<W: Write>(&self, w: &mut EventWriter<W>) -> ThothResult<()> {
        write_openaire(self, w)
    }
}

fn push_text_element<W: Write>(
    xml: &mut EventWriter<W>,
    name: &str,
    text: &str,
) -> ThothResult<()> {
    write_element_block(name, xml, |xml| {
        xml.write(XmlEvent::Characters(text)).map_err(|e| e.into())
    })
}

fn push_text_element_attrs<W: Write>(
    xml: &mut EventWriter<W>,
    name: &str,
    attrs: &[(&str, String)],
    text: &str,
) -> ThothResult<()> {
    let mut event_builder: StartElementBuilder = XmlEvent::start_element(name);
    for (key, value) in attrs {
        event_builder = event_builder.attr(*key, value.as_str());
    }
    let event: XmlEvent = event_builder.into();
    xml.write(event)?;
    xml.write(XmlEvent::Characters(text))?;
    xml.write(XmlEvent::end_element()).map_err(|e| e.into())
}

fn push_open_tag<W: Write>(
    xml: &mut EventWriter<W>,
    name: &str,
    attrs: &[(&str, String)],
) -> ThothResult<()> {
    let mut event_builder: StartElementBuilder = XmlEvent::start_element(name);
    for (key, value) in attrs {
        event_builder = event_builder.attr(*key, value.as_str());
    }
    let event: XmlEvent = event_builder.into();
    xml.write(event).map_err(|e| e.into())
}

fn push_close_tag<W: Write>(xml: &mut EventWriter<W>, _name: &str) -> ThothResult<()> {
    let event: XmlEvent = XmlEvent::end_element().into();
    xml.write(event).map_err(|e| e.into())
}

fn normalize_value(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_string())
}

fn push_unique(values: &mut Vec<String>, seen: &mut HashSet<String>, value: impl Into<String>) {
    if let Some(value) = normalize_value(&value.into()) {
        if seen.insert(value.clone()) {
            values.push(value);
        }
    }
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

fn oai_identifier(work_id: Uuid) -> String {
    format!("{OAI_IDENTIFIER_PREFIX}:{work_id}")
}

fn canonical_title(work: &Work) -> Option<&WorkTitles> {
    work.titles
        .iter()
        .find(|title| title.canonical)
        .or_else(|| work.titles.first())
}

fn ordered_titles(work: &Work) -> Vec<&WorkTitles> {
    let mut titles = work.titles.iter().collect::<Vec<_>>();
    titles.sort_by(|left, right| {
        right
            .canonical
            .cmp(&left.canonical)
            .then_with(|| {
                left.locale_code
                    .to_string()
                    .cmp(&right.locale_code.to_string())
            })
            .then_with(|| left.full_title.cmp(&right.full_title))
    });
    titles
}

fn ordered_abstracts(work: &Work) -> Vec<&WorkAbstracts> {
    fn priority(abstract_type: &AbstractType) -> u8 {
        match abstract_type {
            AbstractType::SHORT => 0,
            AbstractType::LONG => 1,
            _ => 2,
        }
    }

    let mut abstracts = work.abstracts.iter().collect::<Vec<_>>();
    abstracts.sort_by(|left, right| {
        priority(&left.abstract_type)
            .cmp(&priority(&right.abstract_type))
            .then_with(|| right.canonical.cmp(&left.canonical))
            .then_with(|| {
                left.locale_code
                    .to_string()
                    .cmp(&right.locale_code.to_string())
            })
            .then_with(|| left.content.cmp(&right.content))
    });
    abstracts
}

fn ordered_languages(work: &Work) -> Vec<&WorkLanguages> {
    fn priority(language_relation: &LanguageRelation) -> u8 {
        match language_relation {
            LanguageRelation::ORIGINAL => 0,
            LanguageRelation::TRANSLATED_FROM => 1,
            LanguageRelation::TRANSLATED_INTO => 2,
            _ => 3,
        }
    }

    let mut languages = work.languages.iter().collect::<Vec<_>>();
    languages.sort_by(|left, right| {
        priority(&left.language_relation)
            .cmp(&priority(&right.language_relation))
            .then_with(|| {
                left.language_code
                    .to_string()
                    .cmp(&right.language_code.to_string())
            })
    });
    languages
}

fn convert_abstract_to_text(abstract_record: &WorkAbstracts) -> ThothResult<String> {
    convert_from_jats(
        &abstract_record.content,
        MarkupFormat::PlainText,
        ConversionLimit::Abstract,
    )
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

fn timestamp_rfc3339(timestamp: thoth_api::model::Timestamp) -> String {
    timestamp.to_rfc3339().replace("+00:00", "Z")
}

fn reference_citation(reference: &thoth_client::WorkReferences) -> Option<String> {
    if let Some(unstructured) = reference.unstructured_citation.as_deref() {
        return normalize_value(unstructured);
    }

    let mut parts = Vec::new();
    if let Some(author) = reference.author.as_deref().and_then(normalize_value) {
        parts.push(author);
    }
    if let Some(article_title) = reference.article_title.as_deref().and_then(normalize_value) {
        parts.push(article_title);
    }
    if let Some(journal_title) = reference.journal_title.as_deref().and_then(normalize_value) {
        parts.push(journal_title);
    }
    if let Some(series_title) = reference.series_title.as_deref().and_then(normalize_value) {
        parts.push(series_title);
    }
    if let Some(volume_title) = reference.volume_title.as_deref().and_then(normalize_value) {
        parts.push(volume_title);
    }
    if let Some(volume) = reference.volume.as_deref().and_then(normalize_value) {
        parts.push(format!("vol. {volume}"));
    }
    if let Some(issue) = reference.issue.as_deref().and_then(normalize_value) {
        parts.push(format!("issue {issue}"));
    }
    if let Some(first_page) = reference.first_page.as_deref().and_then(normalize_value) {
        parts.push(format!("p. {first_page}"));
    }
    if let Some(component_number) = reference
        .component_number
        .as_deref()
        .and_then(normalize_value)
    {
        parts.push(format!("component {component_number}"));
    }
    if let Some(edition) = reference.edition {
        parts.push(format!("{edition} ed."));
    }
    if let Some(publication_date) = reference.publication_date {
        parts.push(publication_date.to_string());
    }
    if let Some(doi) = &reference.doi {
        parts.push(doi_url(doi));
    }
    if let Some(isbn) = &reference.isbn {
        parts.push(format!("ISBN {isbn}"));
    }
    if let Some(issn) = reference.issn.as_deref().and_then(normalize_value) {
        parts.push(format!("ISSN {issn}"));
    }
    if let Some(standard_designator) = reference
        .standard_designator
        .as_deref()
        .and_then(normalize_value)
    {
        parts.push(format!("std. {standard_designator}"));
    }
    if let Some(standards_body_name) = reference
        .standards_body_name
        .as_deref()
        .and_then(normalize_value)
    {
        parts.push(standards_body_name);
    }
    if let Some(standards_body_acronym) = reference
        .standards_body_acronym
        .as_deref()
        .and_then(normalize_value)
    {
        parts.push(standards_body_acronym);
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join(". "))
    }
}

fn write_openaire<W: Write>(work: &Work, xml: &mut EventWriter<W>) -> ThothResult<()> {
    let root_attrs = OPENAIRE_NS
        .iter()
        .map(|(key, value)| (*key, (*value).to_string()))
        .collect::<Vec<_>>();
    push_open_tag(xml, "oaire:resource", &root_attrs)?;

    push_text_element_attrs(
        xml,
        "datacite:identifier",
        &[("identifierType", "URL".to_string())],
        &work_url(work),
    )?;

    let mut title_entries = Vec::new();
    let mut title_seen = HashSet::new();
    for (index, title) in ordered_titles(work).iter().enumerate() {
        if index == 0 {
            let canonical_title = normalize_value(&title.title)
                .or_else(|| normalize_value(&title.full_title))
                .unwrap_or_else(|| title.full_title.clone());
            if title_seen.insert(format!("canonical:{canonical_title}")) {
                title_entries.push((Vec::new(), canonical_title));
            }
            if let Some(subtitle) = title.subtitle.as_deref().and_then(normalize_value) {
                if title_seen.insert(format!("subtitle:{subtitle}")) {
                    title_entries.push((
                        vec![("titleType", "Subtitle".to_string())],
                        subtitle.to_string(),
                    ));
                }
            }
        } else if let Some(full_title) = normalize_value(&title.full_title) {
            if title_seen.insert(format!("alternative:{full_title}")) {
                title_entries.push((
                    vec![("titleType", "AlternativeTitle".to_string())],
                    full_title,
                ));
            }
        }
    }
    if !title_entries.is_empty() {
        push_open_tag(xml, "datacite:titles", &[])?;
        for (attrs, value) in title_entries {
            if attrs.is_empty() {
                push_text_element(xml, "datacite:title", &value)?;
            } else {
                let attrs = attrs
                    .iter()
                    .map(|(key, value)| (*key, value.clone()))
                    .collect::<Vec<_>>();
                push_text_element_attrs(xml, "datacite:title", &attrs, &value)?;
            }
        }
        push_close_tag(xml, "datacite:titles")?;
    }

    let creators = creators(work).collect::<Vec<_>>();
    if !creators.is_empty() {
        push_open_tag(xml, "datacite:creators", &[])?;
        for creator in creators {
            push_open_tag(xml, "datacite:creator", &[])?;
            push_text_element_attrs(
                xml,
                "datacite:creatorName",
                &[("nameType", "Personal".to_string())],
                &personal_name(creator),
            )?;
            if let Some(first_name) = creator.first_name.as_deref() {
                if !first_name.is_empty() {
                    push_text_element(xml, "datacite:givenName", first_name)?;
                }
            }
            if !creator.last_name.is_empty() {
                push_text_element(xml, "datacite:familyName", &creator.last_name)?;
            }
            if let Some(orcid) = &creator.contributor.orcid {
                push_text_element_attrs(
                    xml,
                    "datacite:nameIdentifier",
                    &[
                        ("nameIdentifierScheme", "ORCID".to_string()),
                        ("schemeURI", "https://orcid.org/".to_string()),
                    ],
                    &orcid_url(orcid),
                )?;
            }
            for affiliation in &creator.affiliations {
                if let Some(ror) = &affiliation.institution.ror {
                    push_text_element_attrs(
                        xml,
                        "datacite:affiliation",
                        &[("affiliationIdentifier", ror_url(ror))],
                        &affiliation.institution.institution_name,
                    )?;
                } else {
                    push_text_element(
                        xml,
                        "datacite:affiliation",
                        &affiliation.institution.institution_name,
                    )?;
                }
            }
            push_close_tag(xml, "datacite:creator")?;
        }
        push_close_tag(xml, "datacite:creators")?;
    }

    let contributors = contributors(work).collect::<Vec<_>>();
    if !contributors.is_empty() {
        push_open_tag(xml, "datacite:contributors", &[])?;
        for contributor in contributors {
            let contributor_type = if contributor.contribution_type == ContributionType::EDITOR {
                "Editor"
            } else {
                "Other"
            };
            push_open_tag(
                xml,
                "datacite:contributor",
                &[("contributorType", contributor_type.to_string())],
            )?;
            push_text_element_attrs(
                xml,
                "datacite:contributorName",
                &[("nameType", "Personal".to_string())],
                &personal_name(contributor),
            )?;
            if let Some(first_name) = contributor.first_name.as_deref() {
                if !first_name.is_empty() {
                    push_text_element(xml, "datacite:givenName", first_name)?;
                }
            }
            if !contributor.last_name.is_empty() {
                push_text_element(xml, "datacite:familyName", &contributor.last_name)?;
            }
            if let Some(orcid) = &contributor.contributor.orcid {
                push_text_element_attrs(
                    xml,
                    "datacite:nameIdentifier",
                    &[
                        ("nameIdentifierScheme", "ORCID".to_string()),
                        ("schemeURI", "https://orcid.org/".to_string()),
                    ],
                    &orcid_url(orcid),
                )?;
            }
            for affiliation in &contributor.affiliations {
                if let Some(ror) = &affiliation.institution.ror {
                    push_text_element_attrs(
                        xml,
                        "datacite:affiliation",
                        &[("affiliationIdentifier", ror_url(ror))],
                        &affiliation.institution.institution_name,
                    )?;
                } else {
                    push_text_element(
                        xml,
                        "datacite:affiliation",
                        &affiliation.institution.institution_name,
                    )?;
                }
            }
            push_close_tag(xml, "datacite:contributor")?;
        }
        push_close_tag(xml, "datacite:contributors")?;
    }

    if !work.fundings.is_empty() {
        push_open_tag(xml, "oaire:fundingReferences", &[])?;
        for funding in &work.fundings {
            push_open_tag(xml, "oaire:fundingReference", &[])?;
            push_text_element(
                xml,
                "oaire:funderName",
                &funding.institution.institution_name,
            )?;
            if let Some(ror) = &funding.institution.ror {
                push_text_element_attrs(
                    xml,
                    "oaire:funderIdentifier",
                    &[("funderIdentifierType", "ROR".to_string())],
                    &ror_url(ror),
                )?;
            }
            if let Some(grant_number) = &funding.grant_number {
                push_text_element(xml, "oaire:awardNumber", grant_number)?;
            }
            if let Some(project_name) = &funding.project_name {
                push_text_element(xml, "oaire:awardTitle", project_name)?;
            }
            push_close_tag(xml, "oaire:fundingReference")?;
        }
        push_close_tag(xml, "oaire:fundingReferences")?;
    }

    let mut alternate_identifiers = Vec::new();
    let mut alternate_identifier_seen = HashSet::new();
    let mut push_alternate_identifier = |identifier_type: &str, value: String| {
        if let Some(value) = normalize_value(&value) {
            let key = (identifier_type.to_string(), value.clone());
            if alternate_identifier_seen.insert(key.clone()) {
                alternate_identifiers.push(key);
            }
        }
    };
    if let Some(doi) = &work.doi {
        push_alternate_identifier("DOI", doi_url(doi));
    }
    if let Some(landing_page) = work.landing_page.as_deref() {
        push_alternate_identifier("URL", landing_page.to_string());
    }
    push_alternate_identifier("OAI", oai_identifier(work.work_id));
    if let Some(lccn) = work.lccn.as_deref() {
        push_alternate_identifier("LCCN", lccn.to_string());
    }
    if let Some(oclc) = work.oclc.as_deref() {
        push_alternate_identifier("OCLC", oclc.to_string());
    }
    for publication in &work.publications {
        if let Some(isbn) = &publication.isbn {
            push_alternate_identifier("ISBN", isbn.to_string());
        }
    }
    if !alternate_identifiers.is_empty() {
        push_open_tag(xml, "datacite:alternateIdentifiers", &[])?;
        for (identifier_type, value) in alternate_identifiers {
            push_text_element_attrs(
                xml,
                "datacite:alternateIdentifier",
                &[("alternateIdentifierType", identifier_type)],
                &value,
            )?;
        }
        push_close_tag(xml, "datacite:alternateIdentifiers")?;
    }

    let mut related_identifiers = Vec::new();
    let mut related_identifier_seen = HashSet::new();
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
            let value = doi_url(doi);
            if let Some(value) = normalize_value(&value) {
                let key = ("DOI".to_string(), relation_type.to_string(), value.clone());
                if related_identifier_seen.insert(key.clone()) {
                    related_identifiers.push(key);
                }
            }
        }
        if let Some(landing_page) = relation.related_work.landing_page.as_deref() {
            if let Some(value) = normalize_value(landing_page) {
                let key = ("URL".to_string(), relation_type.to_string(), value.clone());
                if related_identifier_seen.insert(key.clone()) {
                    related_identifiers.push(key);
                }
            }
        }
        for publication in &relation.related_work.publications {
            if let Some(isbn) = &publication.isbn {
                let value = isbn.to_string();
                if let Some(value) = normalize_value(&value) {
                    let key = ("ISBN".to_string(), relation_type.to_string(), value.clone());
                    if related_identifier_seen.insert(key.clone()) {
                        related_identifiers.push(key);
                    }
                }
            }
        }
    }
    if !related_identifiers.is_empty() {
        push_open_tag(xml, "datacite:relatedIdentifiers", &[])?;
        for (identifier_type, relation_type, value) in related_identifiers {
            push_text_element_attrs(
                xml,
                "datacite:relatedIdentifier",
                &[
                    ("relatedIdentifierType", identifier_type),
                    ("relationType", relation_type),
                ],
                &value,
            )?;
        }
        push_close_tag(xml, "datacite:relatedIdentifiers")?;
    }

    let mut language_values = Vec::new();
    let mut language_seen = HashSet::new();
    for language in ordered_languages(work) {
        push_unique(
            &mut language_values,
            &mut language_seen,
            language.language_code.to_string().to_lowercase(),
        );
    }
    for language in language_values {
        push_text_element(xml, "dc:language", &language)?;
    }

    push_text_element(xml, "dc:publisher", &work.imprint.publisher.publisher_name)?;

    if let Some(publication_date) = &work.publication_date {
        push_text_element_attrs(
            xml,
            "datacite:date",
            &[("dateType", "Issued".to_string())],
            &publication_date.to_string(),
        )?;
    }
    push_text_element(
        xml,
        "dcterms:modified",
        &timestamp_rfc3339(work.updated_at_with_relations),
    )?;

    if let Some((uri, value)) = openaire_resource_type(work) {
        push_text_element_attrs(
            xml,
            "oaire:resourceType",
            &[
                ("resourceTypeGeneral", "literature".to_string()),
                ("uri", uri.to_string()),
            ],
            value,
        )?;
    }

    let mut description_values = Vec::new();
    let mut description_seen = HashSet::new();
    for abstract_record in ordered_abstracts(work) {
        push_unique(
            &mut description_values,
            &mut description_seen,
            convert_abstract_to_text(abstract_record)?,
        );
    }
    if let Some(toc) = work.toc.as_deref() {
        push_unique(
            &mut description_values,
            &mut description_seen,
            toc.to_string(),
        );
    }
    if let Some(general_note) = work.general_note.as_deref() {
        push_unique(
            &mut description_values,
            &mut description_seen,
            general_note.to_string(),
        );
    }
    if let Some(bibliography_note) = work.bibliography_note.as_deref() {
        push_unique(
            &mut description_values,
            &mut description_seen,
            bibliography_note.to_string(),
        );
    }
    if let Some(cover_caption) = work.cover_caption.as_deref() {
        push_unique(
            &mut description_values,
            &mut description_seen,
            cover_caption.to_string(),
        );
    }
    if let Some(page_breakdown) = work.page_breakdown.as_deref() {
        push_unique(
            &mut description_values,
            &mut description_seen,
            page_breakdown.to_string(),
        );
    }
    for description in description_values {
        push_text_element(xml, "dc:description", &description)?;
    }

    let mut format_values = Vec::new();
    let mut format_seen = HashSet::new();
    for publication in &work.publications {
        push_unique(
            &mut format_values,
            &mut format_seen,
            publication_type_value(&publication.publication_type).to_string(),
        );
    }
    for format_value in format_values {
        push_text_element(xml, "dc:format", &format_value)?;
    }

    if let Some(license) = &work.license {
        push_text_element_attrs(
            xml,
            "datacite:rights",
            &[(
                "rightsURI",
                "http://purl.org/coar/access_right/c_abf2".to_string(),
            )],
            "open access",
        )?;
        push_text_element_attrs(
            xml,
            "oaire:licenseCondition",
            &[("uri", license.clone())],
            normalized_license_name(license),
        )?;
    } else {
        push_text_element_attrs(
            xml,
            "datacite:rights",
            &[(
                "rightsURI",
                "http://purl.org/coar/access_right/c_16ec".to_string(),
            )],
            "restricted access",
        )?;
    }
    if let Some(copyright_holder) = work.copyright_holder.as_deref() {
        push_text_element(
            xml,
            "datacite:rights",
            &format!("Copyright holder: {copyright_holder}"),
        )?;
    }

    let mut subject_entries = Vec::new();
    let mut subject_seen = HashSet::new();
    for subject in &work.subjects {
        let entry = match subject.subject_type {
            SubjectType::KEYWORD | SubjectType::CUSTOM => {
                (Vec::new(), subject.subject_code.to_string())
            }
            SubjectType::THEMA => (
                vec![("subjectScheme", "Thema".to_string())],
                subject.subject_code.to_string(),
            ),
            _ => (
                vec![("subjectScheme", subject.subject_type.to_string())],
                subject.subject_code.to_string(),
            ),
        };
        let signature = format!(
            "{}|{}",
            entry
                .0
                .iter()
                .map(|(key, value)| format!("{key}={value}"))
                .collect::<Vec<_>>()
                .join("&"),
            entry.1
        );
        if subject_seen.insert(signature) {
            subject_entries.push(entry);
        }
    }
    for (attrs, value) in subject_entries {
        if attrs.is_empty() {
            push_text_element(xml, "datacite:subject", &value)?;
        } else {
            let attrs = attrs
                .iter()
                .map(|(key, value)| (*key, value.clone()))
                .collect::<Vec<_>>();
            push_text_element_attrs(xml, "datacite:subject", &attrs, &value)?;
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
        push_open_tag(xml, "datacite:sizes", &[])?;
        for size in sizes {
            push_text_element(xml, "datacite:size", &size)?;
        }
        push_close_tag(xml, "datacite:sizes")?;
    }

    for publication in &work.publications {
        for location in &publication.locations {
            if let Some(full_text_url) = &location.full_text_url {
                push_text_element_attrs(
                    xml,
                    "oaire:file",
                    &[
                        (
                            "mimeType",
                            publication_type_value(&publication.publication_type).to_string(),
                        ),
                        ("objectType", "fulltext".to_string()),
                    ],
                    full_text_url,
                )?;
            }
        }
    }

    let issue = work.issues.first();
    if work.work_type == thoth_client::WorkType::BOOK_CHAPTER {
        if let Some(parent_work) = parent_work(work) {
            if let Some(parent_title) = parent_work
                .titles
                .iter()
                .find(|title| title.canonical)
                .or_else(|| parent_work.titles.first())
            {
                push_text_element(xml, "oaire:citationTitle", &parent_title.full_title)?;
            } else if let Some(title) = canonical_title(work) {
                push_text_element(xml, "oaire:citationTitle", &title.full_title)?;
            }
            if let Some(edition) = parent_work.edition.or(work.edition) {
                push_text_element(xml, "oaire:citationEdition", &edition.to_string())?;
            }
        } else if let Some(title) = canonical_title(work) {
            push_text_element(xml, "oaire:citationTitle", &title.full_title)?;
        }
    } else if let Some(issue) = issue {
        push_text_element(xml, "oaire:citationTitle", &issue.series.series_name)?;
        let citation_issue = issue
            .issue_number
            .map(|value| value.to_string())
            .and_then(|value| normalize_value(&value))
            .unwrap_or_else(|| issue.issue_ordinal.to_string());
        push_text_element(xml, "oaire:citationIssue", &citation_issue)?;
    } else if let Some(title) = canonical_title(work) {
        push_text_element(xml, "oaire:citationTitle", &title.full_title)?;
    }

    if let Some(first_page) = &work.first_page {
        push_text_element(xml, "oaire:citationStartPage", first_page)?;
    }
    if let Some(last_page) = &work.last_page {
        push_text_element(xml, "oaire:citationEndPage", last_page)?;
    }

    let mut citation_values = Vec::new();
    let mut citation_seen = HashSet::new();
    for reference in &work.references {
        if let Some(citation) = reference_citation(reference) {
            push_unique(&mut citation_values, &mut citation_seen, citation);
        }
    }
    for citation in citation_values {
        push_text_element(xml, "dcterms:bibliographicCitation", &citation)?;
    }

    push_close_tag(xml, "oaire:resource")
}

#[cfg(test)]
fn map_openaire(work: &Work) -> ThothResult<String> {
    let mut buffer = Vec::new();
    let mut writer = EmitterConfig::new()
        .perform_indent(true)
        .create_writer(&mut buffer);
    XmlElementBlock::<OpenaireThoth>::xml_element(work, &mut writer)
        .map(|_| buffer)
        .and_then(|xml| {
            String::from_utf8(xml)
                .map_err(|_| ThothError::InternalError("Could not parse XML".to_string()))
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::XML_DECLARATION;
    use crate::xml::dublincore_thoth::test_support::{assert_valid_against_schema, fixture_work};

    fn assert_precedes(xml: &str, first: &str, second: &str) {
        let first_pos = xml
            .find(first)
            .unwrap_or_else(|| panic!("Could not find `{first}` in XML output"));
        let second_pos = xml
            .find(second)
            .unwrap_or_else(|| panic!("Could not find `{second}` in XML output"));
        assert!(
            first_pos < second_pos,
            "Expected `{first}` to appear before `{second}`"
        );
    }

    #[test]
    fn openaire_mapping_is_exhaustive_and_schema_clean() {
        let work = fixture_work();
        let xml = map_openaire(&work).expect("map openaire");

        assert!(xml.contains("<datacite:title>Canonical Title</datacite:title>"));
        assert!(xml.contains(
            "<datacite:title titleType=\"AlternativeTitle\">Alternativer Titel</datacite:title>"
        ));
        assert!(xml.contains("<datacite:contributorName nameType=\"Personal\">Hopper, Grace</datacite:contributorName>"));
        assert!(!xml.contains(
            "<datacite:creatorName nameType=\"Personal\">Hopper, Grace</datacite:creatorName>"
        ));
        assert!(xml.contains("<datacite:alternateIdentifier alternateIdentifierType=\"OAI\">oai:thoth.pub:00000000-0000-0000-0000-000000000111</datacite:alternateIdentifier>"));
        assert!(xml.contains("<datacite:alternateIdentifier alternateIdentifierType=\"LCCN\">2023123456</datacite:alternateIdentifier>"));
        assert!(xml.contains("<datacite:alternateIdentifier alternateIdentifierType=\"OCLC\">123456789</datacite:alternateIdentifier>"));
        assert!(xml.contains("<dcterms:modified>2024-12-31T12:00:00Z</dcterms:modified>"));
        assert!(xml.contains("<dcterms:bibliographicCitation>Doe, J. (2020). The Open Knowledge Handbook.</dcterms:bibliographicCitation>"));
        assert!(xml.contains("<dcterms:bibliographicCitation>Smith, Jane. Structured Citation Patterns. Metadata Quarterly."));
        assert!(xml.contains("<dc:description>Ausführliche Zusammenfassung.</dc:description>"));
        assert!(xml.contains("<oaire:file mimeType=\"application/pdf\" objectType=\"fulltext\">https://example.org/books/111.pdf</oaire:file>"));
        assert!(xml.contains("<oaire:licenseCondition uri=\"http://creativecommons.org/licenses/by/4.0/\">CC BY 4.0</oaire:licenseCondition>"));
        assert!(!xml.contains("<dcterms:spatial>"));
        assert_precedes(
            &xml,
            "<datacite:identifier identifierType=\"URL\">",
            "<datacite:titles>",
        );
        assert_precedes(&xml, "<datacite:creators>", "<datacite:contributors>");
        assert_precedes(
            &xml,
            "<oaire:citationTitle>Open Access Series</oaire:citationTitle>",
            "<oaire:citationIssue>7</oaire:citationIssue>",
        );

        assert_valid_against_schema(&xml, "oai_openaire.xsd");
    }

    #[test]
    fn generator_returns_single_work_xml_with_declaration() {
        let xml = OpenaireThoth {}
            .generate(&[fixture_work()], None)
            .expect("single openaire");
        assert!(xml.starts_with(XML_DECLARATION));
        assert!(!xml.starts_with(&format!("{XML_DECLARATION}\n")));
        assert!(xml.contains("<oaire:resource "));
    }

    #[test]
    fn generator_rejects_multiple_works() {
        let work = fixture_work();
        let result = OpenaireThoth {}.generate(&[work.clone(), work], None);
        assert!(matches!(
            result,
            Err(ThothError::IncompleteMetadataRecord(spec, message))
                if spec == OPENAIRE_ERROR && message == BY_WORK_ONLY_MESSAGE
        ));
    }
}
