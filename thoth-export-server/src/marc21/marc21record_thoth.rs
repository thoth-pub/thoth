use crate::marc21::Marc21Field;
use cc_license::License;
use chrono::{Datelike, Utc};
use marc::{FieldRepr, Record, RecordBuilder};
use std::collections::HashMap;
use thoth_api::model::contribution::ContributionType;
use thoth_api::model::publication::PublicationType;
use thoth_client::{
    LanguageRelation, SubjectType, Work, WorkContributions, WorkLanguages, WorkPublications,
    WorkSubjects, WorkType,
};
use thoth_errors::{ThothError, ThothResult};

use super::{Marc21Entry, Marc21Specification};

#[derive(Copy, Clone)]
pub(crate) struct Marc21RecordThoth;

impl Marc21Specification for Marc21RecordThoth {
    fn handle_event(w: &mut Vec<u8>, works: &[Work]) -> ThothResult<()> {
        match works.len() {
            0 => Err(ThothError::IncompleteMetadataRecord(
                "marc21record::thoth".to_string(),
                "Not enough data".to_string(),
            )),
            1 => Marc21Entry::<Marc21RecordThoth>::marc21_record(works.first().unwrap(), w),
            _ => {
                for work in works.iter() {
                    // Do not include Chapters in full publisher metadata record
                    // (assumes that a publisher will always have more than one work)
                    if work.work_type != WorkType::BOOK_CHAPTER {
                        Marc21Entry::<Marc21RecordThoth>::marc21_record(work, w).ok();
                    }
                }
                Ok(())
            }
        }
    }
}

impl Marc21Entry<Marc21RecordThoth> for Work {
    fn to_record(&self) -> ThothResult<Record> {
        let publication_date = self.publication_date.ok_or_else(|| {
            ThothError::IncompleteMetadataRecord(
                "marc21record::thoth".to_string(),
                "Missing Publication Date".to_string(),
            )
        })?;

        let mut builder = RecordBuilder::new();

        // 001 - control number
        builder.add_field((b"001", self.work_id.to_string()))?;

        // 006 - media type
        builder.add_field((b"006", "m        d        "))?;

        // 007 - characteristics
        builder.add_field((b"007", "cr  n         "))?;

        // 008 - fixed-length data elements
        let date = Utc::now().format("%y%m%d").to_string();
        let pub_year = publication_date.year().to_string();
        let language = main_language(
            &self
                .languages
                .iter()
                .filter(|l| l.main_language)
                .cloned()
                .collect::<Vec<WorkLanguages>>(),
        )
        .ok_or_else(|| {
            ThothError::IncompleteMetadataRecord(
                "marc21record::thoth".to_string(),
                "Missing Main Language".to_string(),
            )
        })?;
        let data_field = format!("{date}t{pub_year}{pub_year}        sb    000 0 {language} d");
        builder.add_field((b"008", data_field.as_bytes()))?;

        // 010 - LCCN
        if let Some(lccn) = self.lccn.clone() {
            let mut lccn_field: FieldRepr = FieldRepr::from((b"010", "\\\\"));
            lccn_field = lccn_field.add_subfield(b"a", lccn.as_bytes())?;
            builder.add_field(lccn_field)?;
        }

        // 020 - ISBN
        if self.publications.iter().all(|p| p.isbn.is_none()) {
            return Err(ThothError::IncompleteMetadataRecord(
                "marc21record::thoth".to_string(),
                "Missing ISBN".to_string(),
            ));
        }
        for publication in &self.publications {
            Marc21Field::<Marc21RecordThoth>::to_field(publication, &mut builder)?;
        }

        // 024 - DOI
        if let Some(doi) = &self.doi {
            let mut doi_field: FieldRepr = FieldRepr::from((b"024", "7\\"));
            doi_field = doi_field.add_subfield(b"a", doi.to_string().as_bytes())?;
            doi_field = doi_field.add_subfield(b"2", "doi")?;
            builder.add_field(doi_field)?;
        }

        // 040 - cataloging source field \\$aStSaUL$beng$erda
        let mut cataloguing_field: FieldRepr = FieldRepr::from((b"040", "\\\\"));
        cataloguing_field = cataloguing_field.add_subfield(b"a", "Thoth")?;
        cataloguing_field = cataloguing_field.add_subfield(b"b", "eng")?;
        cataloguing_field = cataloguing_field.add_subfield(b"e", "rda")?;
        builder.add_field(cataloguing_field)?;

        // 041 - language
        if let Some(language_field) = language_field(&self.languages) {
            builder.add_field(language_field)?;
        }

        // 050 - LCC
        for subject in self
            .subjects
            .iter()
            .filter(|s| s.subject_type == SubjectType::LCC)
        {
            Marc21Field::<Marc21RecordThoth>::to_field(subject, &mut builder)?;
        }

        // 072 - BIC
        for subject in self
            .subjects
            .iter()
            .filter(|s| s.subject_type == SubjectType::BIC)
        {
            Marc21Field::<Marc21RecordThoth>::to_field(subject, &mut builder)?;
        }

        // 072 - BISAC
        for subject in self
            .subjects
            .iter()
            .filter(|s| s.subject_type == SubjectType::BISAC)
        {
            Marc21Field::<Marc21RecordThoth>::to_field(subject, &mut builder)?;
        }

        // 072 - Thema
        for subject in self
            .subjects
            .iter()
            .filter(|s| s.subject_type == SubjectType::THEMA)
        {
            Marc21Field::<Marc21RecordThoth>::to_field(subject, &mut builder)?;
        }

        // 245 – title
        let mut title_field: FieldRepr = FieldRepr::from((b"245", "00")); // no title added entry
        title_field = title_field.add_subfield(b"a", self.title.clone().into_bytes())?; // main title
        title_field = title_field.add_subfield(b"h", b"[electronic resource] :")?; // general material designation (GMD)
        if let Some(subtitle) = self.subtitle.clone() {
            title_field = title_field.add_subfield(b"b", subtitle.into_bytes())?;
            // subtitle
        }
        title_field =
            title_field.add_subfield(b"c", contributors_string(&self.contributions).as_bytes())?; // statement of responsibility
        builder.add_field(title_field)?;

        // 264 - publication
        let mut publication_field: FieldRepr = FieldRepr::from((b"264", "\\1"));
        if let Some(place) = self.place.clone() {
            publication_field = publication_field.add_subfield(b"a", place.into_bytes())?;
            // place of publication
        }
        publication_field = publication_field.add_subfield(
            b"b",
            self.imprint.publisher.publisher_name.clone().into_bytes(),
        )?; // publisher
            // year of publication is used in two 264 fields, let's do both
        let year = publication_date.year().to_string();
        publication_field = publication_field.add_subfield(b"c", year.clone().into_bytes())?;
        let mut copyright_year_field = FieldRepr::from((b"264", "\\4"));
        copyright_year_field =
            copyright_year_field.add_subfield(b"c", format!("©{}", year).into_bytes())?;
        builder.add_field(publication_field)?;
        builder.add_field(copyright_year_field)?;

        // 300 - extent and physical description
        let mut extent_field: FieldRepr = FieldRepr::from((b"300", "\\\\"));
        let (extent_str, resource_count) = description_string(self);
        extent_field = extent_field.add_subfield(b"a", extent_str)?;
        if let Some(resource_count_str) = resource_count {
            extent_field = extent_field.add_subfield(b"b", resource_count_str)?;
        }
        builder.add_field(extent_field)?;

        // 336 - content type
        let mut content_type_field: FieldRepr = FieldRepr::from((b"336", "\\\\"));
        content_type_field = content_type_field.add_subfield(b"a", "text")?;
        content_type_field = content_type_field.add_subfield(b"b", "txt")?;
        content_type_field = content_type_field.add_subfield(b"2", "rdacontent")?;
        builder.add_field(content_type_field)?;

        // 337 - type of material
        let mut material_field: FieldRepr = FieldRepr::from((b"337", "\\\\"));
        material_field = material_field.add_subfield(b"a", "computer")?;
        material_field = material_field.add_subfield(b"b", "c")?;
        material_field = material_field.add_subfield(b"2", "rdamedia")?;
        builder.add_field(material_field)?;

        // 338 - type of media
        let mut media_field: FieldRepr = FieldRepr::from((b"338", "\\\\"));
        media_field = media_field.add_subfield(b"a", "online resource")?;
        media_field = media_field.add_subfield(b"b", "cr")?;
        media_field = media_field.add_subfield(b"2", "rdacarrier")?;
        builder.add_field(media_field)?;

        // 500 - availability
        let mut availability_field: FieldRepr = FieldRepr::from((b"500", "\\\\"));
        availability_field = availability_field.add_subfield(
            b"a",
            format!(
                "Available through {}.",
                self.imprint.publisher.publisher_name.clone()
            )
            .into_bytes(),
        )?;
        builder.add_field(availability_field)?;

        // 504 - general note
        if let Some(general_note) = self.general_note.clone() {
            let mut note_field: FieldRepr = FieldRepr::from((b"504", "\\\\"));
            note_field = note_field.add_subfield(b"a", general_note.into_bytes())?;
            builder.add_field(note_field)?;
        }

        // 505 - contents note
        if let Some(toc) = self.toc.clone() {
            let mut toc_field: FieldRepr = FieldRepr::from((b"505", "0\\"));
            toc_field = toc_field.add_subfield(b"a", toc.into_bytes())?;
            builder.add_field(toc_field)?;
        }

        // 506 - restrictions on access
        let mut restrictions_field: FieldRepr = FieldRepr::from((b"506", "\\\\"));
        restrictions_field =
            restrictions_field.add_subfield(b"a", "Open access resource providing free access.")?;
        builder.add_field(restrictions_field)?;

        // 520 - abstract
        if let Some(long_abstract) = self.long_abstract.clone() {
            let mut abstract_field: FieldRepr = FieldRepr::from((b"520", "\\\\"));
            abstract_field = abstract_field.add_subfield(b"a", long_abstract.into_bytes())?;
            builder.add_field(abstract_field)?;
        }

        // 538 - mode of access
        let mut access_field: FieldRepr = FieldRepr::from((b"538", "\\\\"));
        access_field = access_field.add_subfield(b"a", "Mode of access: World Wide Web.")?;
        builder.add_field(access_field)?;

        // 540 - license
        if let Some(license_url) = self.license.clone() {
            let mut license_field: FieldRepr = FieldRepr::from((b"540", "\\\\"));
            match License::from_url(&license_url) {
                Ok(license) => license_field =
                    license_field.add_subfield(b"a", format!("The text of this book is licensed under a {} For more detailed information consult the publisher's website.", license.to_string()).into_bytes())?,
                Err(_) => license_field =
                    license_field.add_subfield(b"a", "The text of this book is licensed under a custom license. For more detailed information consult the publisher's website.")?,
            }
            license_field = license_field.add_subfield(b"u", license_url.into_bytes())?;
            builder.add_field(license_field)?;
        }

        // 700 - contributors
        if self.contributions.is_empty() {
            return Err(ThothError::IncompleteMetadataRecord(
                "marc21record::thoth".to_string(),
                "Missing Contributions".to_string(),
            ));
        }
        let mut contributions_by_name: HashMap<String, Vec<WorkContributions>> = HashMap::new();
        for contribution in &self.contributions {
            let name = contribution.full_name.clone();
            let contributions_for_name = contributions_by_name.entry(name).or_insert(Vec::new());
            contributions_for_name.push(contribution.clone());
        }
        for (name, contributions) in contributions_by_name.iter() {
            let roles = contributions
                .iter()
                .map(|c| ContributionType::from(c.contribution_type.clone()).to_string())
                .collect::<Vec<_>>()
                .join(", ");
            let mut contributor_field = FieldRepr::from((b"700", "1\\"));
            contributor_field = contributor_field.add_subfield(b"a", name)?;
            contributor_field = contributor_field.add_subfield(b"e", roles)?;
            builder.add_field(contributor_field)?;
        }

        // 710 - publisher
        let mut publisher_field: FieldRepr = FieldRepr::from((b"710", "2\\"));
        publisher_field = publisher_field.add_subfield(
            b"a",
            format!("{},", self.imprint.publisher.publisher_name.clone()).into_bytes(),
        )?;
        publisher_field = publisher_field.add_subfield(b"e", "publisher.")?;
        builder.add_field(publisher_field)?;

        // 856 - location
        if let Some(doi) = &self.doi {
            let mut cover_field: FieldRepr = FieldRepr::from((b"856", "40")); // version of resource
            cover_field = cover_field.add_subfield(b"u", doi.to_lowercase_string().into_bytes())?;
            cover_field = cover_field.add_subfield(b"z", b"Connect to e-book")?;
            builder.add_field(cover_field)?;
        }
        // 856 - cover
        if let Some(cover_url) = self.cover_url.clone() {
            let mut cover_field: FieldRepr = FieldRepr::from((b"856", "42")); // related resource
            cover_field = cover_field.add_subfield(b"u", cover_url.into_bytes())?;
            cover_field = cover_field.add_subfield(b"z", b"Connect to cover image")?;
            builder.add_field(cover_field)?;
        }

        Ok(builder.get_record()?)
    }
}

fn main_language(languages: &[WorkLanguages]) -> Option<String> {
    match languages.len() {
        0 => None,
        1 => Some(languages[0].language_code.to_string().to_lowercase()),
        _ => languages
            .iter()
            .min_by_key(|language| match language.language_relation {
                LanguageRelation::TRANSLATED_INTO => 0,
                LanguageRelation::ORIGINAL => 1,
                LanguageRelation::TRANSLATED_FROM => 2,
                _ => 3,
            })
            .map(|language| language.language_code.to_string().to_lowercase()),
    }
}

pub fn language_field(languages: &[WorkLanguages]) -> Option<FieldRepr> {
    let (original_codes, into_codes, from_codes): (Vec<_>, Vec<_>, Vec<_>) = languages.iter().fold(
        (Vec::new(), Vec::new(), Vec::new()),
        |(mut orig, mut into, mut from), l| {
            match l.language_relation {
                LanguageRelation::ORIGINAL => orig.push(l.language_code.to_string().to_lowercase()),
                LanguageRelation::TRANSLATED_INTO => {
                    into.push(l.language_code.to_string().to_lowercase())
                }
                LanguageRelation::TRANSLATED_FROM => {
                    from.push(l.language_code.to_string().to_lowercase())
                }
                _ => {}
            }
            (orig, into, from)
        },
    );

    let has_original = !original_codes.is_empty();
    let has_translated_into = !into_codes.is_empty();
    let has_translated_from = !from_codes.is_empty();

    // $a is used for the language of text
    // $h is used for the language the text has been translated from
    // $k language of the text translated from if there's an ultimate original language, e.g. text in English translated from German and originally published in Swedish
    let (subfield_codes, subfield_language_codes): (Vec<_>, Vec<_>) =
        match (has_original, has_translated_into, has_translated_from) {
            (true, true, true) => (
                vec![b"a", b"h", b"k"],
                vec![into_codes, from_codes, original_codes],
            ),
            (true, true, false) => (vec![b"a", b"h"], vec![into_codes, original_codes]),
            (true, false, true) => (vec![b"a", b"h"], vec![original_codes, from_codes]),
            (true, false, false) => (vec![b"a"], vec![original_codes]),
            (false, true, true) => (vec![b"a", b"h"], vec![into_codes, from_codes]),
            (false, true, false) => (vec![b"a", b"h"], vec![into_codes, vec!["und".to_string()]]), // original language undertermined
            (false, false, true) => {
                return None;
            }
            (false, false, false) => {
                return None;
            }
        };

    let language_indicator = if subfield_codes.len() == 1 {
        "0\\" // original text
    } else {
        "1\\" // translation
    };
    let mut language_field: FieldRepr = FieldRepr::from((b"041", language_indicator));
    for (subfield_code, language_codes) in subfield_codes.iter().zip(subfield_language_codes) {
        for language_code in language_codes {
            language_field = language_field
                .add_subfield(*subfield_code, language_code)
                .ok()?;
        }
    }

    Some(language_field)
}

impl Marc21Field<Marc21RecordThoth> for WorkPublications {
    fn to_field(&self, builder: &mut RecordBuilder) -> ThothResult<()> {
        if let Some(isbn) = &self.isbn {
            let mut isbn_field: FieldRepr = FieldRepr::from((b"020", "\\\\")); // 2 backslashes represent that the subfield can appear multiple times within the field
            isbn_field = isbn_field.add_subfield(b"a", isbn.to_hyphenless_string().as_bytes())?;
            let publication_type: PublicationType = self.publication_type.clone().into();
            isbn_field =
                isbn_field.add_subfield(b"q", format!("({})", publication_type).into_bytes())?;

            builder.add_field(isbn_field)?;
        }
        Ok(())
    }
}

type SubjectField<'a, 'b> = (&'a [u8; 3], &'b str, Option<(&'a [u8; 1], &'b str)>);
impl Marc21Field<Marc21RecordThoth> for WorkSubjects {
    fn to_field(&self, builder: &mut RecordBuilder) -> ThothResult<()> {
        let (tag, ind, sub2): SubjectField = match self.subject_type {
            SubjectType::BIC => (b"072", " 7", Some((b"2", "bicssc"))),
            SubjectType::BISAC => (b"072", " 7", Some((b"2", "bisacsh"))),
            SubjectType::THEMA => (b"072", " 7", Some((b"2", "thema"))),
            SubjectType::LCC => (b"050", "00", None),
            _ => {
                return Ok(());
            }
        };
        let mut subject_field: FieldRepr = FieldRepr::from((tag, ind));
        subject_field = subject_field.add_subfield(b"a", self.subject_code.as_bytes())?;
        if let Some((subfield, value)) = sub2 {
            subject_field = subject_field.add_subfield(subfield, value)?;
        }
        builder.add_field(subject_field)?;
        Ok(())
    }
}

fn description_string(work: &Work) -> (String, Option<String>) {
    let description = match (work.page_breakdown.as_ref(), work.page_count) {
        (Some(breakdown), _) => format!("1 online resource ({} pages)", breakdown),
        (_, Some(count)) => format!("1 online resource ({} pages)", count),
        _ => "1 online resource".to_string(),
    };

    // other resource counts
    let counts = [
        (work.image_count, "illustration", "illustrations"),
        (work.table_count, "table", "tables"),
        (work.audio_count, "audio track", "audio tracks"),
        (work.video_count, "video", "videos"),
    ];
    let other_counts = counts
        .iter()
        .filter_map(|(count, singular, plural)| match count {
            Some(c) if *c > 0 => Some(format!("{} {}", c, if *c == 1 { singular } else { plural })),
            _ => None,
        })
        .collect::<Vec<_>>();

    match other_counts.is_empty() {
        true => (description + ".", None),
        false => (
            description + ": ",
            Some(format!("{}.", other_counts.join(", "))),
        ),
    }
}

fn contributors_string(contributions: &[WorkContributions]) -> String {
    // group main contributions by contribution type
    let mut contributions_by_type: HashMap<ContributionType, Vec<&WorkContributions>> =
        HashMap::new();
    for c in contributions.iter().filter(|c| c.main_contribution) {
        let entry = contributions_by_type
            .entry(ContributionType::from(c.contribution_type.clone()))
            .or_insert(vec![]);
        entry.push(c);
    }

    // build string for each contribution type
    let mut type_strings = vec![];
    for (contribution_type, contributions) in contributions_by_type.iter() {
        let names = contributions
            .iter()
            .map(|c| c.full_name.clone())
            .collect::<Vec<_>>()
            .join(", ");

        let type_string = match contribution_type {
            ContributionType::Author => names,
            ContributionType::Editor => format!("edited by {}", names),
            _ => format!("{} ({})", contribution_type, names),
        };
        type_strings.push(type_string);
    }

    // join type strings with appropriate separators
    let mut result = type_strings.join("; ");
    result.push('.');

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use thoth_api::model::{Doi, Isbn, Orcid, Ror};
    use thoth_client::{
        LanguageCode, SeriesType, WorkContributionsAffiliations,
        WorkContributionsAffiliationsInstitution, WorkContributionsContributor, WorkImprint,
        WorkImprintPublisher, WorkIssues, WorkIssuesSeries, WorkStatus,
    };
    use uuid::Uuid;

    fn test_work() -> Work {
        Work {
            work_id: Uuid::from_str("00000000-0000-0000-AAAA-000000000001").unwrap(),
            work_status: WorkStatus::ACTIVE,
            full_title: "Book Title: Book Subtitle".to_string(),
            title: "Book Title".to_string(),
            subtitle: Some("Book Subtitle".to_string()),
            work_type: WorkType::MONOGRAPH,
            reference: None,
            edition: Some(1),
            doi: Some(Doi::from_str("https://doi.org/10.00001/BOOK.0001").unwrap()),
            publication_date: chrono::NaiveDate::from_ymd_opt(2010, 2, 1),
            license: Some("https://creativecommons.org/licenses/by/4.0/".to_string()),
            copyright_holder: None,
            short_abstract: None,
            long_abstract: Some("Lorem ipsum dolor sit amet".to_string()),
            general_note: Some("Includes bibliography (pages 165-170) and index.".to_string()),
            place: Some("León, Spain".to_string()),
            page_count: None,
            page_breakdown: None,
            first_page: None,
            last_page: None,
            page_interval: None,
            image_count: None,
            table_count: None,
            audio_count: None,
            video_count: None,
            landing_page: None,
            toc: None,
            lccn: None,
            oclc: None,
            cover_url: Some("https://www.book.com/cover.jpg".to_string()),
            cover_caption: None,
            imprint: WorkImprint {
                imprint_name: "OA Editions Imprint".to_string(),
                imprint_url: None,
                publisher: WorkImprintPublisher {
                    publisher_name: "OA Editions".to_string(),
                    publisher_shortname: None,
                    publisher_url: None,
                },
            },
            issues: vec![WorkIssues {
                issue_ordinal: 11,
                series: WorkIssuesSeries {
                    series_type: SeriesType::BOOK_SERIES,
                    series_name: "Name of series".to_string(),
                    issn_print: "1234-5678".to_string(),
                    issn_digital: "8765-4321".to_string(),
                    series_url: None,
                    series_description: None,
                    series_cfp_url: None,
                },
            }],
            contributions: vec![
                WorkContributions {
                    contribution_type: thoth_client::ContributionType::AUTHOR,
                    first_name: Some("Sole".to_string()),
                    last_name: "Author".to_string(),
                    full_name: "Sole Author".to_string(),
                    main_contribution: true,
                    biography: None,
                    contribution_ordinal: 1,
                    contributor: WorkContributionsContributor {
                        orcid: Some(
                            Orcid::from_str("https://orcid.org/0000-0002-0000-0001").unwrap(),
                        ),
                        website: None,
                    },
                    affiliations: vec![WorkContributionsAffiliations {
                        position: None,
                        affiliation_ordinal: 1,
                        institution: WorkContributionsAffiliationsInstitution {
                            institution_name: "Thoth University".to_string(),
                            institution_doi: None,
                            ror: Some(Ror::from_str("https://ror.org/0abcdef12").unwrap()),
                            country_code: None,
                        },
                    }],
                },
                WorkContributions {
                    contribution_type: thoth_client::ContributionType::EDITOR,
                    first_name: Some("Only".to_string()),
                    last_name: "Editor".to_string(),
                    full_name: "Only Editor".to_string(),
                    main_contribution: true,
                    biography: None,
                    contribution_ordinal: 2,
                    contributor: WorkContributionsContributor {
                        orcid: Some(
                            Orcid::from_str("https://orcid.org/0000-0002-0000-0002").unwrap(),
                        ),
                        website: None,
                    },
                    affiliations: vec![],
                },
                WorkContributions {
                    contribution_type: thoth_client::ContributionType::TRANSLATOR,
                    first_name: None,
                    last_name: "Translator".to_string(),
                    full_name: "Translator".to_string(),
                    main_contribution: true,
                    biography: None,
                    contribution_ordinal: 3,
                    contributor: WorkContributionsContributor {
                        orcid: None,
                        website: None,
                    },
                    affiliations: vec![WorkContributionsAffiliations {
                        position: None,
                        affiliation_ordinal: 1,
                        institution: WorkContributionsAffiliationsInstitution {
                            institution_name: "COPIM".to_string(),
                            institution_doi: None,
                            ror: None,
                            country_code: None,
                        },
                    }],
                },
            ],
            languages: vec![
                WorkLanguages {
                    language_code: LanguageCode::ENG,
                    language_relation: LanguageRelation::TRANSLATED_INTO,
                    main_language: true,
                },
                WorkLanguages {
                    language_code: LanguageCode::SPA,
                    language_relation: LanguageRelation::TRANSLATED_FROM,
                    main_language: true,
                },
            ],
            publications: vec![
                WorkPublications {
                    publication_id: Uuid::from_str("00000000-0000-0000-DDDD-000000000004").unwrap(),
                    publication_type: thoth_client::PublicationType::PDF,
                    isbn: Some(Isbn::from_str("978-3-16-148410-0").unwrap()),
                    width_mm: None,
                    width_cm: None,
                    width_in: None,
                    height_mm: None,
                    height_cm: None,
                    height_in: None,
                    depth_mm: None,
                    depth_cm: None,
                    depth_in: None,
                    weight_g: None,
                    weight_oz: None,
                    prices: vec![],
                    locations: vec![],
                },
                WorkPublications {
                    publication_id: Uuid::from_str("00000000-0000-0000-FFFF-000000000006").unwrap(),
                    publication_type: thoth_client::PublicationType::XML,
                    isbn: Some(Isbn::from_str("978-92-95055-02-5").unwrap()),
                    width_mm: None,
                    width_cm: None,
                    width_in: None,
                    height_mm: None,
                    height_cm: None,
                    height_in: None,
                    depth_mm: None,
                    depth_cm: None,
                    depth_in: None,
                    weight_g: None,
                    weight_oz: None,
                    prices: vec![],
                    locations: vec![],
                },
                WorkPublications {
                    publication_id: Uuid::from_str("00000000-0000-0000-CCCC-000000000003").unwrap(),
                    publication_type: thoth_client::PublicationType::HARDBACK,
                    isbn: Some(Isbn::from_str("978-1-4028-9462-6").unwrap()),
                    width_mm: None,
                    width_cm: None,
                    width_in: None,
                    height_mm: None,
                    height_cm: None,
                    height_in: None,
                    depth_mm: None,
                    depth_cm: None,
                    depth_in: None,
                    weight_g: None,
                    weight_oz: None,
                    prices: vec![],
                    locations: vec![],
                },
            ],
            subjects: vec![
                WorkSubjects {
                    subject_code: "AAB".to_string(),
                    subject_type: SubjectType::BIC,
                    subject_ordinal: 1,
                },
                WorkSubjects {
                    subject_code: "AAA000000".to_string(),
                    subject_type: SubjectType::BISAC,
                    subject_ordinal: 2,
                },
                WorkSubjects {
                    subject_code: "JA85".to_string(),
                    subject_type: SubjectType::LCC,
                    subject_ordinal: 3,
                },
                WorkSubjects {
                    subject_code: "JWA".to_string(),
                    subject_type: SubjectType::THEMA,
                    subject_ordinal: 4,
                },
            ],
            fundings: vec![],
            relations: vec![],
            references: vec![],
        }
    }

    #[test]
    fn test_language_field_original_only() {
        let languages = vec![WorkLanguages {
            language_code: LanguageCode::ENG,
            language_relation: LanguageRelation::ORIGINAL,
            main_language: true,
        }];
        assert_eq!(
            language_field(&languages).unwrap().get_data(),
            b"0\\\x1faeng"
        );
    }

    #[test]
    fn test_language_field_translated_into_only() {
        let languages = vec![
            WorkLanguages {
                language_code: LanguageCode::FRE,
                language_relation: LanguageRelation::TRANSLATED_INTO,
                main_language: true,
            },
            WorkLanguages {
                language_code: LanguageCode::SPA,
                language_relation: LanguageRelation::TRANSLATED_INTO,
                main_language: true,
            },
        ];
        assert_eq!(
            language_field(&languages).unwrap().get_data(),
            b"1\\\x1fafre\x1faspa\x1fhund"
        );
    }

    #[test]
    fn test_language_field_translated_from_only() {
        let languages = vec![
            WorkLanguages {
                language_code: LanguageCode::GER,
                language_relation: LanguageRelation::TRANSLATED_FROM,
                main_language: true,
            },
            WorkLanguages {
                language_code: LanguageCode::ITA,
                language_relation: LanguageRelation::TRANSLATED_FROM,
                main_language: true,
            },
        ];
        assert_eq!(language_field(&languages), None);
    }

    #[test]
    fn test_language_field_original_and_double_translated_into() {
        let languages = vec![
            WorkLanguages {
                language_code: LanguageCode::ENG,
                language_relation: LanguageRelation::ORIGINAL,
                main_language: true,
            },
            WorkLanguages {
                language_code: LanguageCode::FRE,
                language_relation: LanguageRelation::TRANSLATED_INTO,
                main_language: true,
            },
            WorkLanguages {
                language_code: LanguageCode::SPA,
                language_relation: LanguageRelation::TRANSLATED_INTO,
                main_language: true,
            },
        ];
        assert_eq!(
            language_field(&languages).unwrap().get_data(),
            b"1\\\x1fafre\x1faspa\x1fheng"
        );
    }

    #[test]
    fn test_language_field_original_and_double_translated_from() {
        let languages = vec![
            WorkLanguages {
                language_code: LanguageCode::ENG,
                language_relation: LanguageRelation::ORIGINAL,
                main_language: true,
            },
            WorkLanguages {
                language_code: LanguageCode::GER,
                language_relation: LanguageRelation::TRANSLATED_FROM,
                main_language: true,
            },
            WorkLanguages {
                language_code: LanguageCode::ITA,
                language_relation: LanguageRelation::TRANSLATED_FROM,
                main_language: true,
            },
        ];
        assert_eq!(
            language_field(&languages).unwrap().get_data(),
            b"1\\\x1faeng\x1fhger\x1fhita"
        );
    }

    #[test]
    fn test_language_field_no_languages() {
        let languages: [WorkLanguages; 0] = [];
        assert_eq!(language_field(&languages), None);
    }

    #[test]
    fn test_language_field_original_and_translated_into() {
        let languages = [
            WorkLanguages {
                language_relation: LanguageRelation::ORIGINAL,
                language_code: LanguageCode::ENG,
                main_language: true,
            },
            WorkLanguages {
                language_relation: LanguageRelation::TRANSLATED_INTO,
                language_code: LanguageCode::FRE,
                main_language: true,
            },
        ];
        assert_eq!(
            language_field(&languages).unwrap().get_data(),
            b"1\\\x1fafre\x1fheng"
        );
    }

    #[test]
    fn test_language_field_original_and_translated_from() {
        let languages = [
            WorkLanguages {
                language_relation: LanguageRelation::ORIGINAL,
                language_code: LanguageCode::ENG,
                main_language: true,
            },
            WorkLanguages {
                language_relation: LanguageRelation::TRANSLATED_FROM,
                language_code: LanguageCode::FRE,
                main_language: true,
            },
        ];
        assert_eq!(
            language_field(&languages).unwrap().get_data(),
            b"1\\\x1faeng\x1fhfre"
        );
    }

    #[test]
    fn test_language_field_translated_into_and_translated_from() {
        let languages = [
            WorkLanguages {
                language_relation: LanguageRelation::TRANSLATED_INTO,
                language_code: LanguageCode::FRE,
                main_language: true,
            },
            WorkLanguages {
                language_relation: LanguageRelation::TRANSLATED_FROM,
                language_code: LanguageCode::GER,
                main_language: true,
            },
        ];
        assert_eq!(
            language_field(&languages).unwrap().get_data(),
            b"1\\\x1fafre\x1fhger"
        );
    }

    #[test]
    fn test_language_field_original_translated_into_and_translated_from() {
        let languages = [
            WorkLanguages {
                language_relation: LanguageRelation::ORIGINAL,
                language_code: LanguageCode::ENG,
                main_language: true,
            },
            WorkLanguages {
                language_relation: LanguageRelation::TRANSLATED_INTO,
                language_code: LanguageCode::FRE,
                main_language: true,
            },
            WorkLanguages {
                language_relation: LanguageRelation::TRANSLATED_FROM,

                language_code: LanguageCode::GER,
                main_language: true,
            },
        ];
        assert_eq!(
            language_field(&languages).unwrap().get_data(),
            b"1\\\x1fafre\x1fhger\x1fkeng"
        );
    }

    #[test]
    fn test_description_string_no_counts() {
        let work = test_work();

        let expected = ("1 online resource.".to_string(), None);
        assert_eq!(description_string(&work), expected);
    }

    #[test]
    fn test_description_string_page_count_only() {
        let mut work = test_work();
        work.page_count = Some(100);

        let expected = ("1 online resource (100 pages).".to_string(), None);
        assert_eq!(description_string(&work), expected);
    }

    #[test]
    fn test_description_string_page_breakdown_only() {
        let mut work = test_work();
        work.page_breakdown = Some("x+238".to_string());

        let expected = ("1 online resource (x+238 pages).".to_string(), None);
        assert_eq!(description_string(&work), expected);
    }

    #[test]
    fn test_description_string_page_count_and_breakdown() {
        let mut work = test_work();
        work.page_count = Some(248);
        work.page_breakdown = Some("x+238".to_string());

        let expected = ("1 online resource (x+238 pages).".to_string(), None);
        assert_eq!(description_string(&work), expected);
    }

    #[test]
    fn test_description_string_other_counts_only() {
        let mut work = test_work();
        work.image_count = Some(1);
        work.table_count = Some(2);
        work.audio_count = Some(3);
        work.video_count = Some(4);

        let expected = (
            "1 online resource: ".to_string(),
            Some("1 illustration, 2 tables, 3 audio tracks, 4 videos.".to_string()),
        );
        assert_eq!(description_string(&work), expected);
    }

    #[test]
    fn test_description_string_all_counts() {
        let mut work = test_work();
        work.page_count = Some(248);
        work.page_breakdown = Some("x+238".to_string());
        work.image_count = Some(1);
        work.table_count = Some(2);
        work.audio_count = Some(3);
        work.video_count = Some(4);

        let expected = (
            "1 online resource (x+238 pages): ".to_string(),
            Some("1 illustration, 2 tables, 3 audio tracks, 4 videos.".to_string()),
        );
        assert_eq!(description_string(&work), expected);
    }

    #[test]
    fn test_description_string_some_counts() {
        let mut work = test_work();
        work.page_count = Some(248);
        work.image_count = Some(9);
        work.table_count = Some(1);

        let expected = (
            "1 online resource (248 pages): ".to_string(),
            Some("9 illustrations, 1 table.".to_string()),
        );
        assert_eq!(description_string(&work), expected);
    }
}
