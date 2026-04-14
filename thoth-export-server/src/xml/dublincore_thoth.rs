use std::collections::HashSet;

use quick_xml::escape::escape;
use thoth_api::markup::{convert_from_jats, ConversionLimit, MarkupFormat};
use thoth_client::{
    AbstractType, ContributionType, LanguageRelation, PublicationType, SubjectType, Work,
    WorkAbstracts, WorkContributions, WorkLanguages, WorkTitles,
};
use thoth_errors::{ThothError, ThothResult};

use crate::record::XML_DECLARATION;

const DUBLIN_CORE_ERROR: &str = "dublin_core::thoth";
const BY_WORK_ONLY_MESSAGE: &str = "Output can only be generated for one work at a time";

#[derive(Copy, Clone)]
pub(crate) struct DublinCoreThoth;

impl DublinCoreThoth {
    pub(crate) fn generate(&self, works: &[Work]) -> ThothResult<String> {
        match works {
            [] => Err(ThothError::IncompleteMetadataRecord(
                DUBLIN_CORE_ERROR.to_string(),
                "Not enough data".to_string(),
            )),
            [work] => Ok(format!("{XML_DECLARATION}\n{}", map_dublin_core(work)?)),
            _ => Err(ThothError::IncompleteMetadataRecord(
                DUBLIN_CORE_ERROR.to_string(),
                BY_WORK_ONLY_MESSAGE.to_string(),
            )),
        }
    }
}

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

fn work_url(work: &Work) -> String {
    format!("https://thoth.pub/books/{}", work.work_id)
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

fn map_dublin_core(work: &Work) -> ThothResult<String> {
    let mut xml = String::from(
        r#"<oai_dc:dc xmlns:oai_dc="http://www.openarchives.org/OAI/2.0/oai_dc/" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:schemaLocation="http://www.openarchives.org/OAI/2.0/oai_dc/ http://www.openarchives.org/OAI/2.0/oai_dc.xsd">"#,
    );

    let mut title_values = Vec::new();
    let mut title_seen = HashSet::new();
    for title in ordered_titles(work) {
        push_unique(&mut title_values, &mut title_seen, title.full_title.clone());
    }
    for title in title_values {
        push_text_element(&mut xml, "dc:title", &title);
    }

    for creator in creators(work) {
        push_text_element(&mut xml, "dc:creator", &creator.full_name);
    }

    let mut subject_values = Vec::new();
    let mut subject_seen = HashSet::new();
    for subject in &work.subjects {
        let value = match subject.subject_type {
            SubjectType::KEYWORD | SubjectType::CUSTOM => subject.subject_code.clone(),
            SubjectType::THEMA => format!("THEMA:{}", subject.subject_code),
            _ => format!("{}:{}", subject.subject_type, subject.subject_code),
        };
        push_unique(&mut subject_values, &mut subject_seen, value);
    }
    for subject in subject_values {
        push_text_element(&mut xml, "dc:subject", &subject);
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
        push_text_element(&mut xml, "dc:format", &format_value);
    }

    let mut identifier_values = Vec::new();
    let mut identifier_seen = HashSet::new();
    push_unique(&mut identifier_values, &mut identifier_seen, work_url(work));
    if let Some(doi) = &work.doi {
        push_unique(&mut identifier_values, &mut identifier_seen, doi_url(doi));
    }
    if let Some(landing_page) = work.landing_page.as_deref() {
        push_unique(
            &mut identifier_values,
            &mut identifier_seen,
            landing_page.to_string(),
        );
    }
    for publication in &work.publications {
        if let Some(isbn) = &publication.isbn {
            push_unique(
                &mut identifier_values,
                &mut identifier_seen,
                format!("urn:isbn:{isbn}"),
            );
        }
    }
    if let Some(lccn) = work.lccn.as_deref() {
        push_unique(
            &mut identifier_values,
            &mut identifier_seen,
            format!("urn:lccn:{lccn}"),
        );
    }
    if let Some(oclc) = work.oclc.as_deref() {
        push_unique(
            &mut identifier_values,
            &mut identifier_seen,
            format!("urn:oclc:{oclc}"),
        );
    }
    for identifier in identifier_values {
        push_text_element(&mut xml, "dc:identifier", &identifier);
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
        push_text_element(&mut xml, "dc:language", &language);
    }

    let mut relation_values = Vec::new();
    let mut relation_seen = HashSet::new();
    for relation in &work.relations {
        if let Some(doi) = &relation.related_work.doi {
            push_unique(&mut relation_values, &mut relation_seen, doi_url(doi));
        }
        if let Some(landing_page) = relation.related_work.landing_page.as_deref() {
            push_unique(
                &mut relation_values,
                &mut relation_seen,
                landing_page.to_string(),
            );
        }
        for publication in &relation.related_work.publications {
            if let Some(isbn) = &publication.isbn {
                push_unique(
                    &mut relation_values,
                    &mut relation_seen,
                    format!("urn:isbn:{isbn}"),
                );
            }
        }
    }
    for relation in relation_values {
        push_text_element(&mut xml, "dc:relation", &relation);
    }

    let mut rights_values = Vec::new();
    let mut rights_seen = HashSet::new();
    if let Some(license) = work.license.as_deref() {
        push_unique(&mut rights_values, &mut rights_seen, license.to_string());
        push_unique(
            &mut rights_values,
            &mut rights_seen,
            normalized_license_name(license).to_string(),
        );
    }
    if let Some(copyright_holder) = work.copyright_holder.as_deref() {
        push_unique(
            &mut rights_values,
            &mut rights_seen,
            format!("Copyright holder: {copyright_holder}"),
        );
    }
    for rights in rights_values {
        push_text_element(&mut xml, "dc:rights", &rights);
    }

    xml.push_str("</oai_dc:dc>");
    Ok(xml)
}

#[cfg(test)]
pub(crate) mod test_support {
    use super::*;
    use serde_json::json;
    use std::{
        fs,
        path::PathBuf,
        process::Command,
        time::{SystemTime, UNIX_EPOCH},
    };

    pub(crate) fn fixture_work() -> Work {
        let value = json!({
            "workId": "00000000-0000-0000-0000-000000000111",
            "updatedAtWithRelations": "2024-12-31T12:00:00Z",
            "workStatus": "ACTIVE",
            "workType": "MONOGRAPH",
            "reference": "BK-111",
            "edition": 2,
            "doi": "https://doi.org/10.12345/example.111",
            "publicationDate": "2024-02-15",
            "withdrawnDate": null,
            "license": "http://creativecommons.org/licenses/by/4.0/",
            "copyrightHolder": "Example Author",
            "generalNote": "General availability note.",
            "bibliographyNote": "Includes bibliographical references.",
            "place": "London",
            "pageCount": 210,
            "pageBreakdown": "xii + 198 pages",
            "firstPage": "1",
            "lastPage": "198",
            "pageInterval": null,
            "imageCount": 12,
            "tableCount": 3,
            "audioCount": null,
            "videoCount": null,
            "landingPage": "https://example.org/books/111",
            "toc": "Part I; Part II",
            "lccn": "2023123456",
            "oclc": "123456789",
            "coverUrl": "https://example.org/cover.png",
            "coverCaption": "Front cover image.",
            "titles": [
                {
                    "titleId": "00000000-0000-0000-0000-000000000201",
                    "localeCode": "EN",
                    "fullTitle": "Canonical Title: A Story",
                    "title": "Canonical Title",
                    "subtitle": "A Story",
                    "canonical": true
                },
                {
                    "titleId": "00000000-0000-0000-0000-000000000202",
                    "localeCode": "DE",
                    "fullTitle": "Alternativer Titel",
                    "title": "Alternativer Titel",
                    "subtitle": null,
                    "canonical": false
                }
            ],
            "abstracts": [
                {
                    "abstractId": "00000000-0000-0000-0000-000000000301",
                    "workId": "00000000-0000-0000-0000-000000000111",
                    "content": "<p>Short abstract text.</p>",
                    "localeCode": "EN",
                    "abstractType": "SHORT",
                    "canonical": true
                },
                {
                    "abstractId": "00000000-0000-0000-0000-000000000302",
                    "workId": "00000000-0000-0000-0000-000000000111",
                    "content": "<p>Long abstract text.</p>",
                    "localeCode": "EN",
                    "abstractType": "LONG",
                    "canonical": true
                },
                {
                    "abstractId": "00000000-0000-0000-0000-000000000303",
                    "workId": "00000000-0000-0000-0000-000000000111",
                    "content": "<p>Ausführliche Zusammenfassung.</p>",
                    "localeCode": "DE",
                    "abstractType": "LONG",
                    "canonical": false
                }
            ],
            "imprint": {
                "imprintName": "Example Imprint",
                "imprintUrl": null,
                "crossmarkDoi": null,
                "defaultCurrency": "EUR",
                "defaultPlace": "London",
                "defaultLocale": "EN",
                "publisher": {
                    "publisherName": "Open Access Press",
                    "publisherShortname": "OAP",
                    "publisherUrl": "https://example.org/publisher",
                    "accessibilityStatement": null,
                    "contacts": []
                }
            },
            "issues": [
                {
                    "issueOrdinal": 7,
                    "issueNumber": 7,
                    "series": {
                        "seriesId": "00000000-0000-0000-0000-000000000401",
                        "seriesType": "JOURNAL",
                        "seriesName": "Open Access Series",
                        "issnPrint": null,
                        "issnDigital": null,
                        "seriesUrl": null,
                        "seriesDescription": null,
                        "seriesCfpUrl": null
                    }
                }
            ],
            "contributions": [
                {
                    "contributionType": "AUTHOR",
                    "firstName": "Ada",
                    "lastName": "Lovelace",
                    "fullName": "Ada Lovelace",
                    "mainContribution": true,
                    "biographies": [],
                    "contributionOrdinal": 1,
                    "contributor": {
                        "orcid": "https://orcid.org/0000-0002-0000-0001",
                        "website": null
                    },
                    "affiliations": [
                        {
                            "position": "Researcher",
                            "affiliationOrdinal": 1,
                            "institution": {
                                "institutionName": "Example University",
                                "institutionDoi": null,
                                "ror": "https://ror.org/02vxh6m30",
                                "countryCode": "GB"
                            }
                        }
                    ]
                },
                {
                    "contributionType": "EDITOR",
                    "firstName": "Grace",
                    "lastName": "Hopper",
                    "fullName": "Grace Hopper",
                    "mainContribution": false,
                    "biographies": [],
                    "contributionOrdinal": 2,
                    "contributor": {
                        "orcid": null,
                        "website": null
                    },
                    "affiliations": []
                }
            ],
            "languages": [
                {
                    "languageCode": "ENG",
                    "languageRelation": "ORIGINAL"
                },
                {
                    "languageCode": "DEU",
                    "languageRelation": "TRANSLATED_INTO"
                }
            ],
            "publications": [
                {
                    "publicationId": "00000000-0000-0000-0000-000000000501",
                    "publicationType": "PDF",
                    "isbn": "978-1-4028-9462-6",
                    "weightG": null,
                    "weightOz": null,
                    "widthMm": null,
                    "widthCm": null,
                    "widthIn": null,
                    "heightMm": null,
                    "heightCm": null,
                    "heightIn": null,
                    "depthMm": null,
                    "depthCm": null,
                    "depthIn": null,
                    "accessibilityStandard": null,
                    "accessibilityAdditionalStandard": null,
                    "accessibilityException": null,
                    "accessibilityReportUrl": null,
                    "prices": [],
                    "locations": [
                        {
                            "landingPage": "https://example.org/books/111",
                            "fullTextUrl": "https://example.org/books/111.pdf",
                            "locationPlatform": "OTHER",
                            "canonical": true
                        }
                    ]
                },
                {
                    "publicationId": "00000000-0000-0000-0000-000000000502",
                    "publicationType": "XML",
                    "isbn": "978-92-95055-02-5",
                    "weightG": null,
                    "weightOz": null,
                    "widthMm": null,
                    "widthCm": null,
                    "widthIn": null,
                    "heightMm": null,
                    "heightCm": null,
                    "heightIn": null,
                    "depthMm": null,
                    "depthCm": null,
                    "depthIn": null,
                    "accessibilityStandard": null,
                    "accessibilityAdditionalStandard": null,
                    "accessibilityException": null,
                    "accessibilityReportUrl": null,
                    "prices": [],
                    "locations": []
                }
            ],
            "subjects": [
                {
                    "subjectCode": "Open Access",
                    "subjectType": "KEYWORD",
                    "subjectOrdinal": 1
                },
                {
                    "subjectCode": "Scholarly Publishing",
                    "subjectType": "CUSTOM",
                    "subjectOrdinal": 2
                },
                {
                    "subjectCode": "LAN025000",
                    "subjectType": "BISAC",
                    "subjectOrdinal": 3
                },
                {
                    "subjectCode": "QRM",
                    "subjectType": "THEMA",
                    "subjectOrdinal": 4
                }
            ],
            "fundings": [
                {
                    "program": "Open Science",
                    "projectName": "Metadata Futures",
                    "projectShortname": "META-FUT",
                    "grantNumber": "GA-2024-0001",
                    "institution": {
                        "institutionName": "Research Council",
                        "institutionDoi": null,
                        "ror": "https://ror.org/03yrm5c26",
                        "countryCode": "GB"
                    }
                }
            ],
            "relations": [
                {
                    "relationType": "HAS_PART",
                    "relationOrdinal": 1,
                    "relatedWork": {
                        "edition": 1,
                        "doi": "https://doi.org/10.12345/example.related",
                        "publicationDate": "2023-11-01",
                        "withdrawnDate": null,
                        "workStatus": "ACTIVE",
                        "license": "http://creativecommons.org/licenses/by/4.0/",
                        "copyrightHolder": "Related Author",
                        "generalNote": null,
                        "place": "Leiden",
                        "firstPage": null,
                        "lastPage": null,
                        "pageCount": 120,
                        "pageInterval": null,
                        "landingPage": "https://example.org/books/related",
                        "titles": [
                            {
                                "titleId": "00000000-0000-0000-0000-000000000601",
                                "localeCode": "EN",
                                "fullTitle": "Related Work",
                                "title": "Related Work",
                                "subtitle": null,
                                "canonical": true
                            }
                        ],
                        "abstracts": [],
                        "imprint": {
                            "crossmarkDoi": null,
                            "publisher": {
                                "publisherName": "Open Access Press"
                            }
                        },
                        "contributions": [],
                        "languages": [
                            {
                                "languageCode": "ENG",
                                "languageRelation": "ORIGINAL"
                            }
                        ],
                        "publications": [
                            {
                                "publicationType": "PDF",
                                "isbn": "978-1-4028-9462-7",
                                "locations": []
                            }
                        ],
                        "fundings": [],
                        "references": []
                    }
                }
            ],
            "references": [
                {
                    "referenceOrdinal": 1,
                    "doi": null,
                    "unstructuredCitation": "Doe, J. (2020). The Open Knowledge Handbook.",
                    "issn": null,
                    "isbn": null,
                    "journalTitle": null,
                    "articleTitle": null,
                    "seriesTitle": null,
                    "volumeTitle": null,
                    "edition": null,
                    "author": null,
                    "volume": null,
                    "issue": null,
                    "firstPage": null,
                    "componentNumber": null,
                    "standardDesignator": null,
                    "standardsBodyName": null,
                    "standardsBodyAcronym": null,
                    "url": null,
                    "publicationDate": null,
                    "retrievalDate": null
                },
                {
                    "referenceOrdinal": 2,
                    "doi": "https://doi.org/10.55555/structured.2",
                    "unstructuredCitation": null,
                    "issn": "1234-5678",
                    "isbn": "978-0-12-345678-9",
                    "journalTitle": "Metadata Quarterly",
                    "articleTitle": "Structured Citation Patterns",
                    "seriesTitle": null,
                    "volumeTitle": null,
                    "edition": 1,
                    "author": "Smith, Jane",
                    "volume": "12",
                    "issue": "3",
                    "firstPage": "45",
                    "componentNumber": "2",
                    "standardDesignator": null,
                    "standardsBodyName": null,
                    "standardsBodyAcronym": null,
                    "url": "https://example.org/citation/2",
                    "publicationDate": "2022-06-01",
                    "retrievalDate": null
                }
            ]
        });
        serde_json::from_value(value).expect("valid Work fixture")
    }

    fn write_temp_file(prefix: &str, extension: &str, content: &str) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("{prefix}-{timestamp}.{extension}"));
        fs::write(&path, content).expect("write temp file");
        path
    }

    fn xmllint_available() -> bool {
        Command::new("xmllint")
            .arg("--version")
            .output()
            .is_ok_and(|output| output.status.success())
    }

    pub(crate) fn assert_valid_against_schema(xml: &str, schema_file_name: &str) {
        if !xmllint_available() {
            return;
        }
        let local_schema = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("xsd")
            .join(schema_file_name);
        let schema = if local_schema.exists() {
            local_schema
        } else {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../thoth-oai-server/tests/fixtures/xsd")
                .join(schema_file_name)
        };
        let xml_path = write_temp_file("oai-metadata", "xml", xml);
        let output = Command::new("xmllint")
            .arg("--noout")
            .arg("--schema")
            .arg(&schema)
            .arg(&xml_path)
            .output()
            .expect("run xmllint");
        let _ = fs::remove_file(&xml_path);
        assert!(
            output.status.success(),
            "schema validation failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::{assert_valid_against_schema, fixture_work};
    use super::*;

    #[test]
    fn xml_publication_type_maps_to_text_xml() {
        assert_eq!(publication_type_value(&PublicationType::XML), "text/xml");
    }

    #[test]
    fn dublin_core_mapping_is_exhaustive_for_titles_languages_descriptions_and_rights() {
        let work = fixture_work();
        let xml = map_dublin_core(&work).expect("map dublin_core");

        assert!(xml.contains("<dc:title>Canonical Title: A Story</dc:title>"));
        assert!(xml.contains("<dc:title>Alternativer Titel</dc:title>"));
        assert!(xml.contains("<dc:language>eng</dc:language>"));
        assert_eq!(xml.matches("<dc:language>").count(), 2);
        assert!(xml.contains("<dc:description>Short abstract text.</dc:description>"));
        assert!(xml.contains("<dc:description>Long abstract text.</dc:description>"));
        assert!(xml.contains("<dc:description>Part I; Part II</dc:description>"));
        assert!(xml.contains("<dc:description>General availability note.</dc:description>"));
        assert!(
            xml.contains("<dc:description>Includes bibliographical references.</dc:description>")
        );
        assert!(xml.contains("<dc:description>Front cover image.</dc:description>"));
        assert!(xml.contains("<dc:description>xii + 198 pages</dc:description>"));
        assert!(xml.contains("<dc:subject>Open Access</dc:subject>"));
        assert!(xml.contains("<dc:subject>BISAC:LAN025000</dc:subject>"));
        assert!(xml.contains("<dc:identifier>urn:lccn:2023123456</dc:identifier>"));
        assert!(xml.contains("<dc:identifier>urn:oclc:123456789</dc:identifier>"));
        assert!(xml.contains("<dc:rights>CC BY 4.0</dc:rights>"));
        assert!(xml.contains("<dc:rights>Copyright holder: Example Author</dc:rights>"));
        assert!(!xml.contains("<dc:coverage>"));

        assert_valid_against_schema(&xml, "oai_dc.xsd");
    }

    #[test]
    fn generator_returns_single_work_xml_with_declaration() {
        let xml = DublinCoreThoth {}
            .generate(&[fixture_work()])
            .expect("single dublin core");
        assert!(xml.starts_with(XML_DECLARATION));
        assert!(xml.contains("<oai_dc:dc "));
    }

    #[test]
    fn generator_rejects_multiple_works() {
        let work = fixture_work();
        let result = DublinCoreThoth {}.generate(&[work.clone(), work]);
        assert!(matches!(
            result,
            Err(ThothError::IncompleteMetadataRecord(spec, message))
                if spec == DUBLIN_CORE_ERROR && message == BY_WORK_ONLY_MESSAGE
        ));
    }
}
