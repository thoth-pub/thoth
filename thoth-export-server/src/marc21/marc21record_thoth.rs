use crate::marc21::{Marc21Field, MARC_ORGANIZATION_CODE};
use cc_license::License;
use chrono::{Datelike, Utc};
use marc::{DescriptiveCatalogingForm, EncodingLevel, FieldRepr, Record, RecordBuilder};
use thoth_api::model::contribution::ContributionType;
use thoth_api::model::publication::PublicationType;
use thoth_api::model::IdentifierWithDomain;
use thoth_client::{
    LanguageRelation, RelationType, SubjectType, Work, WorkContributions, WorkFundings, WorkIssues,
    WorkLanguages, WorkPublications, WorkRelations, WorkSubjects, WorkType,
};
use thoth_errors::{ThothError, ThothResult};

use super::{Marc21Entry, Marc21Specification};

#[derive(Copy, Clone)]
pub(crate) struct Marc21RecordThoth;

const MARC_ERROR: &str = "marc21record::thoth";

impl Marc21Specification for Marc21RecordThoth {
    fn handle_event(w: &mut Vec<u8>, works: &[Work]) -> ThothResult<()> {
        match works {
            [] => Err(ThothError::IncompleteMetadataRecord(
                MARC_ERROR.to_string(),
                "Not enough data".to_string(),
            )),
            [work] => Marc21Entry::<Marc21RecordThoth>::marc21_record(work, w),
            _ => works.iter().try_for_each(|work| {
                Marc21Entry::<Marc21RecordThoth>::marc21_record(work, w).ok();
                Ok(())
            }),
        }
    }
}

impl Marc21Entry<Marc21RecordThoth> for Work {
    fn to_record(&self) -> ThothResult<Record> {
        if self.work_type == WorkType::BOOK_CHAPTER {
            return Err(ThothError::IncompleteMetadataRecord(
                MARC_ERROR.to_string(),
                "MARC records are not available for chapters".to_string(),
            ));
        }

        if self.publications.iter().all(|p| p.isbn.is_none()) {
            return Err(ThothError::IncompleteMetadataRecord(
                MARC_ERROR.to_string(),
                "Missing ISBN".to_string(),
            ));
        }

        if self.contributions.is_empty() {
            return Err(ThothError::IncompleteMetadataRecord(
                MARC_ERROR.to_string(),
                "Missing Contributions".to_string(),
            ));
        }

        let publication_date = self.publication_date.ok_or_else(|| {
            ThothError::IncompleteMetadataRecord(
                MARC_ERROR.to_string(),
                "Missing Publication Date".to_string(),
            )
        })?;

        let mut builder = RecordBuilder::new();

        // Leader values - can't guarantee record is complete or formatting is standard
        builder.set_encoding_level(EncodingLevel::LessThanFullLevelMaterialNotExamined);
        builder.set_descriptive_cataloging_form(DescriptiveCatalogingForm::NonIsbd);

        // 001 - control number
        builder.add_field((b"001", self.work_id.to_string()))?;

        // 006 - media type
        builder.add_field((b"006", "m     o  d        "))?;

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
                MARC_ERROR.to_string(),
                "Missing Main Language".to_string(),
            )
        })?;
        let mut has_index = 0;
        if let Some(bibliography_note) = &self.bibliography_note {
            if bibliography_note.contains("index") {
                has_index = 1;
            }
        }
        let data_field =
            format!("{date}t{pub_year}{pub_year}        ob    00{has_index} 0 {language} d");
        builder.add_field((b"008", data_field.as_bytes()))?;

        // 010 - LCCN
        if let Some(lccn) = self.lccn.clone() {
            FieldRepr::from((b"010", "\\\\"))
                .add_subfield(b"a", lccn.into_bytes())
                .and_then(|f| builder.add_field(f))?;
        }

        // 020 - ISBN
        for publication in &self.publications {
            Marc21Field::<Marc21RecordThoth>::to_field(publication, &mut builder)?;
        }

        // 024 - standard identifiers (DOI, OCLC)
        if let Some(doi) = &self.doi {
            FieldRepr::from((b"024", "7\\"))
                .add_subfield(b"a", doi.to_string().as_bytes())
                .and_then(|f| f.add_subfield(b"2", "doi"))
                .and_then(|f| builder.add_field(f))?;
        }
        if let Some(oclc) = &self.oclc {
            FieldRepr::from((b"024", "7\\"))
                .add_subfield(b"a", oclc.clone().as_bytes())
                .and_then(|f| f.add_subfield(b"2", "worldcat"))
                .and_then(|f| builder.add_field(f))?;
        }

        // 040 - cataloging source field
        FieldRepr::from((b"040", "\\\\"))
            .add_subfield(b"a", MARC_ORGANIZATION_CODE)
            .and_then(|f| f.add_subfield(b"b", "eng"))
            .and_then(|f| f.add_subfield(b"e", "local"))
            .and_then(|f| builder.add_field(f))?;

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

        // 100 and 700 - contributors
        let contributor_fields = contributor_fields(&self.contributions)?;
        for contributor_field in &contributor_fields {
            builder.add_field(contributor_field.clone())?;
        }

        // 245 – title
        let added_entry = match contributor_fields.iter().any(|c| c.get_tag() == b"100") {
            true => "1",
            false => "0",
        };
        let nonfiling_char_count = nonfiling_char_count(&self.titles[0].title, &language);
        let mut title_field =
            FieldRepr::from((b"245", format!("{}{}", added_entry, nonfiling_char_count)));
        if let Some(subtitle) = self.titles[0].subtitle.clone() {
            title_field = title_field
                .add_subfield(
                    b"a",
                    format!("{} :", self.titles[0].title.clone()).as_bytes(),
                )
                .and_then(|f| f.add_subfield(b"b", format!("{} /", subtitle).as_bytes()))?;
        } else {
            title_field = title_field.add_subfield(
                b"a",
                format!("{} /", self.titles[0].title.clone()).as_bytes(),
            )?;
        }
        title_field
            .add_subfield(b"c", contributors_string(&self.contributions).as_bytes())
            .and_then(|f| builder.add_field(f))?;

        // 250 - edition statement
        if let Some(edition) = &self.edition {
            if let Some(edition_statement) = match edition {
                // Edition statement not required for first edition
                1 => None,
                // RDA best practice is to spell out ordinals rather than abbreviate
                // Implement this for the commonest ones, as a compromise
                2 => Some("Second edition.".to_string()),
                3 => Some("Third edition.".to_string()),
                _ => {
                    let suffix = match edition % 10 {
                        1 if edition % 100 != 11 => "st",
                        2 if edition % 100 != 12 => "nd",
                        3 if edition % 100 != 13 => "rd",
                        _ => "th",
                    };
                    Some(format!("{}{} edition.", edition, suffix))
                }
            } {
                FieldRepr::from((b"250", "\\\\"))
                    .add_subfield(b"a", edition_statement)
                    .and_then(|f| builder.add_field(f))?;
            }
        }

        // 264 - publication
        let year = publication_date.year().to_string();
        if let Some(place) = self.place.clone() {
            FieldRepr::from((b"264", "\\1"))
                .add_subfield(b"a", format!("{} :", place).into_bytes())
                .and_then(|f| {
                    f.add_subfield(
                        b"b",
                        format!("{},", self.imprint.publisher.publisher_name.clone()).into_bytes(),
                    )
                })
                .and_then(|f| f.add_subfield(b"c", format!("{}.", year.clone()).into_bytes()))
                .and_then(|f| builder.add_field(f))?;
        }
        FieldRepr::from((b"264", "\\4"))
            .add_subfield(b"c", format!("©{}", year).into_bytes())
            .and_then(|f| builder.add_field(f))?;

        // 300 - extent and physical description
        let (extent_str, resource_count) = description_string(self);
        FieldRepr::from((b"300", "\\\\"))
            .add_subfield(b"a", extent_str)
            .and_then(|f| {
                if let Some(resource_count_str) = resource_count {
                    f.add_subfield(b"b", resource_count_str)
                } else {
                    Ok(f)
                }
            })
            .and_then(|f| builder.add_field(f))?;

        // 336 - content type
        FieldRepr::from((b"336", "\\\\"))
            .add_subfield(b"a", "text")
            .and_then(|f| f.add_subfield(b"b", "txt"))
            .and_then(|f| f.add_subfield(b"2", "rdacontent"))
            .and_then(|f| builder.add_field(f))?;

        // 337 - type of material
        FieldRepr::from((b"337", "\\\\"))
            .add_subfield(b"a", "computer")
            .and_then(|f| f.add_subfield(b"b", "c"))
            .and_then(|f| f.add_subfield(b"2", "rdamedia"))
            .and_then(|f| builder.add_field(f))?;

        // 338 - type of media
        FieldRepr::from((b"338", "\\\\"))
            .add_subfield(b"a", "online resource")
            .and_then(|f| f.add_subfield(b"b", "cr"))
            .and_then(|f| f.add_subfield(b"2", "rdacarrier"))
            .and_then(|f| builder.add_field(f))?;

        // 409 and 830 - series
        for issue in &self.issues {
            Marc21Field::<Marc21RecordThoth>::to_field(issue, &mut builder)?;
        }

        // 500 - general note
        let note_field = match &self.general_note {
            Some(general_note) => general_note.clone().into_bytes(),
            None => format!(
                "Available through {}.",
                self.imprint.publisher.publisher_name.clone()
            )
            .into_bytes(),
        };
        FieldRepr::from((b"500", "\\\\"))
            .add_subfield(b"a", note_field)
            .and_then(|f| builder.add_field(f))?;

        // 504 - bibliography note
        if let Some(bibliography_note) = &self.bibliography_note {
            FieldRepr::from((b"504", "\\\\"))
                .add_subfield(b"a", bibliography_note.clone().into_bytes())
                .and_then(|f| builder.add_field(f))?;
        }

        // 505 - contents note
        if let Ok(toc_field) = toc_field(&self.relations) {
            builder.add_field(toc_field)?;
        } else if let Some(mut toc) = self.toc.clone() {
            // Strip out formatting marks as these may stop records loading successfully
            toc.retain(|c| c != '\n' && c != '\r' && c != '\t');
            FieldRepr::from((b"505", "0\\"))
                .add_subfield(b"a", toc.into_bytes())
                .and_then(|f| builder.add_field(f))?;
        }

        // Assume omission of licence means work is non-OA
        if self.license.is_some() {
            // 506 - restrictions on access
            FieldRepr::from((b"506", "0\\"))
                .add_subfield(b"a", "Open Access")
                .and_then(|f| f.add_subfield(b"f", "Unrestricted online access"))
                .and_then(|f| f.add_subfield(b"2", "star"))
                .and_then(|f| builder.add_field(f))?;
        }

        // 520 - abstract
        if let Some(mut long_abstract) = self.long_abstract.clone() {
            // Strip out formatting marks as these may stop records loading successfully
            long_abstract.retain(|c| c != '\n' && c != '\r' && c != '\t');
            FieldRepr::from((b"520", "\\\\"))
                .add_subfield(b"a", long_abstract.into_bytes())
                .and_then(|f| builder.add_field(f))?;
        }

        // 536 - funding
        for funding in &self.fundings {
            Marc21Field::<Marc21RecordThoth>::to_field(funding, &mut builder)?;
        }

        // 538 - mode of access
        FieldRepr::from((b"538", "\\\\"))
            .add_subfield(b"a", "Mode of access: World Wide Web.")
            .and_then(|f| builder.add_field(f))?;

        // 540 - license
        if let Some(license_url) = self.license.clone() {
            let license_text = match License::from_url(&license_url) {
                Ok(license) => format!("The text of this book is licensed under a {} For more detailed information consult the publisher's website.", license.to_string()),
                Err(_) => "The text of this book is licensed under a custom license. For more detailed information consult the publisher's website.".to_string(),
            };
            FieldRepr::from((b"540", "\\\\"))
                .add_subfield(b"a", license_text.as_bytes())
                .and_then(|f| f.add_subfield(b"u", license_url.into_bytes()))
                .and_then(|f| builder.add_field(f))?;
        }

        // 588 - source of description note
        FieldRepr::from((b"588", "0\\"))
            .add_subfield(
                b"a",
                "Metadata licensed under CC0 Public Domain Dedication.",
            )
            .and_then(|f| builder.add_field(f))?;

        // 653 - Uncontrolled Subject Heading
        for subject in self
            .subjects
            .iter()
            .filter(|s| s.subject_type == SubjectType::KEYWORD)
        {
            Marc21Field::<Marc21RecordThoth>::to_field(subject, &mut builder)?;
        }

        // 710 - publisher
        FieldRepr::from((b"710", "2\\"))
            .add_subfield(
                b"a",
                format!("{},", self.imprint.publisher.publisher_name.clone()).into_bytes(),
            )
            .and_then(|f| f.add_subfield(b"e", "publisher."))
            .and_then(|f| builder.add_field(f))?;

        // 856 - location
        if let Some(doi) = &self.doi {
            FieldRepr::from((b"856", "40"))
                .add_subfield(b"u", doi.to_lowercase_string().into_bytes())
                .and_then(|f| f.add_subfield(b"z", "Connect to e-book"))
                .and_then(|f| builder.add_field(f))?;
        }
        // 856 - cover
        if let Some(cover_url) = self.cover_url.clone() {
            FieldRepr::from((b"856", "42"))
                .add_subfield(b"u", cover_url.into_bytes())
                .and_then(|f| f.add_subfield(b"z", "Connect to cover image"))
                .and_then(|f| builder.add_field(f))?;
        }
        // 856 - metadata license
        FieldRepr::from((b"856", "42"))
            .add_subfield(b"u", "https://creativecommons.org/publicdomain/zero/1.0/")
            .and_then(|f| f.add_subfield(b"z", "CC0 Metadata License"))
            .and_then(|f| builder.add_field(f))?;

        Ok(builder.get_record()?)
    }
}

fn main_language(languages: &[WorkLanguages]) -> Option<String> {
    match languages {
        [] => None,
        [language] => Some(language.language_code.to_string().to_lowercase()),
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

fn contributor_fields(contributions: &[WorkContributions]) -> ThothResult<Vec<FieldRepr>> {
    // 100 and 700 - contributors
    let mut contributions_by_name: Vec<(String, Vec<&WorkContributions>, String)> = vec![];
    for c in contributions {
        let mut key = c.full_name.clone();
        let mut name_indicator = "0\\".to_string();
        if let Some(first_name) = &c.first_name {
            key = format!("{}, {}", &c.last_name, first_name);
            name_indicator = "1\\".to_string();
        }
        match contributions_by_name.iter_mut().find(|(k, _, _)| *k == key) {
            Some(entry) => entry.1.push(c),
            None => contributions_by_name.push((key, vec![c], name_indicator)),
        }
    }

    let mut contributor_fields: Vec<FieldRepr> = vec![];
    // only one 100 field is allowed, first-come first-served
    let mut is_main_author_defined = false;
    for (name, contributions, indicator) in contributions_by_name.iter() {
        let is_main = contributions
            .iter()
            .any(|c| c.contribution_type == thoth_client::ContributionType::AUTHOR);
        let mut field_code = b"700";
        if is_main && !is_main_author_defined {
            field_code = b"100";
            is_main_author_defined = true;
        }
        let roles = contributions
            .iter()
            .map(|c| {
                ContributionType::from(c.contribution_type.clone())
                    .to_string()
                    .to_lowercase()
            })
            .collect::<Vec<_>>()
            .join(", ");

        let mut contributor_field = FieldRepr::from((field_code, indicator.as_str()));
        contributor_field =
            contributor_field.add_subfield(b"a", format!("{},", name).as_bytes())?;
        contributor_field =
            contributor_field.add_subfield(b"e", format!("{}.", roles).as_bytes())?;
        if let Some(affiliation) = &contributions.first().unwrap().affiliations.first() {
            contributor_field = contributor_field.add_subfield(
                b"u",
                format!("{}.", affiliation.institution.institution_name.clone()).as_bytes(),
            )?;
        }
        if let Some(orcid) = &contributions.first().unwrap().contributor.orcid {
            // $0 Authority Record Control Number (ORCID is a permitted source)
            contributor_field = contributor_field.add_subfield(
                b"0",
                format!("(orcid){}", orcid.to_string().replace('-', "")),
            )?;
            // $1 Real World Object URI
            contributor_field =
                contributor_field.add_subfield(b"1", orcid.with_domain().as_bytes())?;
        }
        contributor_fields.push(contributor_field);
    }
    Ok(contributor_fields)
}

fn language_field(languages: &[WorkLanguages]) -> Option<FieldRepr> {
    if languages.len() == 1 {
        // 041 language fields are not needed when there is only one language
        // (already represented in field 008 positions 35-37)
        return None;
    }
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
            (false, true, false) => (vec![b"a", b"h"], vec![into_codes, vec!["und".to_string()]]), // original language undetermined
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

fn toc_field(relations: &[WorkRelations]) -> ThothResult<FieldRepr> {
    let mut chapters = relations
        .iter()
        .filter(|r| r.relation_type == RelationType::HAS_CHILD)
        .collect::<Vec<_>>();
    if chapters.is_empty() {
        return Err(ThothError::IncompleteMetadataRecord(
            MARC_ERROR.to_string(),
            "No chapter data available for constructing contents note".to_string(),
        ));
    }
    // WorkQuery should already have retrieved these sorted by ordinal, but sort again for safety
    chapters.sort_by(|a, b| a.relation_ordinal.cmp(&b.relation_ordinal));
    let mut chapter_list = chapters.iter().peekable();
    let mut toc_field: FieldRepr = FieldRepr::from((b"505", "00"));
    let mut separator = " --";
    while let Some(chapter) = chapter_list.next() {
        let chapter_title = &chapter.related_work.titles[0].full_title;
        let chapter_pages = &chapter.related_work.page_interval;
        let chapter_authors = chapter
            .related_work
            .contributions
            .iter()
            .map(|c| c.full_name.clone())
            .collect::<Vec<_>>()
            .join(", ");
        if chapter_list.peek().is_none() {
            // End list of chapters with a full stop
            separator = ".";
        }
        if !chapter_authors.is_empty() {
            toc_field = toc_field.add_subfield(b"t", format!("{} /", chapter_title))?;
            if chapter_pages.is_some() {
                toc_field = toc_field.add_subfield(b"r", chapter_authors)?;
                toc_field = toc_field.add_subfield(
                    b"g",
                    format!("(pp{}){}", chapter_pages.as_ref().unwrap(), separator),
                )?;
            } else {
                toc_field =
                    toc_field.add_subfield(b"r", format!("{}{}", chapter_authors, separator))?;
            }
        } else if chapter_pages.is_some() {
            toc_field = toc_field.add_subfield(b"t", chapter_title)?;
            toc_field = toc_field.add_subfield(
                b"g",
                format!("(pp{}){}", chapter_pages.as_ref().unwrap(), separator),
            )?;
        } else {
            toc_field = toc_field.add_subfield(b"t", format!("{}{}", chapter_title, separator))?;
        }
    }
    Ok(toc_field)
}

impl Marc21Field<Marc21RecordThoth> for WorkPublications {
    fn to_field(&self, builder: &mut RecordBuilder) -> ThothResult<()> {
        if let Some(isbn) = &self.isbn {
            let publication_type: PublicationType = self.publication_type.clone().into();
            let identifier = match publication_type {
                PublicationType::Paperback | PublicationType::Hardback => b"z",
                _ => b"a",
            };
            FieldRepr::from((b"020", "\\\\"))
                .add_subfield(identifier, isbn.to_hyphenless_string().as_bytes())
                .and_then(|f| f.add_subfield(b"q", format!("({})", publication_type)))
                .and_then(|f| builder.add_field(f))?;
        }
        Ok(())
    }
}

impl Marc21Field<Marc21RecordThoth> for WorkIssues {
    fn to_field(&self, builder: &mut RecordBuilder) -> ThothResult<()> {
        let fields = [(b"490", "1\\"), (b"830", "\\0")];
        for (field, indicator) in fields {
            FieldRepr::from((field, indicator))
                .add_subfield(b"a", format!("{} ;", self.series.series_name).as_bytes())
                .and_then(|f| {
                    f.add_subfield(b"v", format!("vol. {}.", self.issue_ordinal).as_bytes())
                })
                .and_then(|f| {
                    self.series
                        .issn_digital
                        .as_ref()
                        .map(|issn_digital| f.add_subfield(b"x", issn_digital.as_bytes()))
                        .unwrap_or(Ok(f))
                })
                .and_then(|f| {
                    self.series
                        .issn_print
                        .as_ref()
                        .map(|issn_print| f.add_subfield(b"x", issn_print.as_bytes()))
                        .unwrap_or(Ok(f))
                })
                .and_then(|f| builder.add_field(f))?;
        }
        Ok(())
    }
}

impl Marc21Field<Marc21RecordThoth> for WorkFundings {
    fn to_field(&self, builder: &mut RecordBuilder) -> ThothResult<()> {
        let mut funding_field: FieldRepr = FieldRepr::from((b"536", "\\\\"))
            .add_subfield(b"a", self.institution.institution_name.clone().as_bytes())?;
        if let Some(grant_number) = &self.grant_number {
            funding_field = funding_field.add_subfield(b"c", grant_number.clone().as_bytes())?;
        }
        if let Some(program) = &self.program {
            funding_field = funding_field.add_subfield(b"e", program.clone().as_bytes())?;
        }
        if let Some(project_name) = &self.project_name {
            funding_field = funding_field.add_subfield(b"f", project_name.clone().as_bytes())?;
        }
        builder.add_field(funding_field)?;
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
            SubjectType::KEYWORD => (b"653", "\\\\", None),
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
    let mut contributions_by_type: Vec<(ContributionType, Vec<&WorkContributions>)> = vec![];
    for c in contributions.iter().filter(|c| c.main_contribution) {
        let key = ContributionType::from(c.contribution_type.clone());
        match contributions_by_type.iter_mut().find(|(k, _)| *k == key) {
            Some(entry) => entry.1.push(c),
            None => contributions_by_type.push((key, vec![c])),
        }
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
            ContributionType::Translator => format!("translated by {}", names),
            ContributionType::Photographer => format!("photography by {}", names),
            ContributionType::Illustrator => format!("illustrations by {}", names),
            ContributionType::MusicEditor => format!("music edited by {}", names),
            _ => format!("{} {}", contribution_type.to_string().to_lowercase(), names),
        };
        type_strings.push(type_string);
    }

    // join type strings with appropriate separators
    let mut result = type_strings.join("; ");
    result.push('.');

    result
}

fn nonfiling_char_count(title: &str, language: &str) -> String {
    let mut nonfiling_char_count = "0".to_string();
    if language == "eng" {
        // Ideally we would also do this for other languages
        if let Some(prefix) = ["a ", "an ", "the "]
            .iter()
            .find(|p| title.to_lowercase().starts_with(*p))
        {
            nonfiling_char_count = prefix.len().to_string()
        }
    }
    nonfiling_char_count
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::str::FromStr;
    use thoth_api::model::{Doi, Isbn, Orcid};
    use thoth_client::{
        FundingInstitution, LanguageCode, SeriesType, WorkContributionsAffiliations,
        WorkContributionsAffiliationsInstitution, WorkContributionsContributor, WorkImprint,
        WorkImprintPublisher, WorkIssues, WorkIssuesSeries, WorkRelationsRelatedWork,
        WorkRelationsRelatedWorkContributions, WorkRelationsRelatedWorkContributionsContributor,
        WorkRelationsRelatedWorkImprint, WorkRelationsRelatedWorkImprintPublisher, WorkStatus,
    };
    use uuid::Uuid;

    pub(crate) fn test_work() -> Work {
        Work {
            work_id: Uuid::from_str("00000000-0000-0000-AAAA-000000000001").unwrap(),
            work_status: WorkStatus::ACTIVE,
            titles: vec![thoth_client::WorkTitles {
                title_id: Uuid::from_str("00000000-0000-0000-CCCC-000000000001").unwrap(),
                locale_code: thoth_client::LocaleCode::EN,
                full_title: "Book Title: Book Subtitle".to_string(),
                title: "Book Title".to_string(),
                subtitle: Some("Book Subtitle".to_string()),
                canonical: true,
            }],
            work_type: WorkType::MONOGRAPH,
            reference: None,
            edition: Some(2),
            doi: Some(Doi::from_str("https://doi.org/10.00001/BOOK.0001").unwrap()),
            publication_date: chrono::NaiveDate::from_ymd_opt(2010, 2, 1),
            withdrawn_date: None,
            license: Some("https://creativecommons.org/licenses/by/4.0/".to_string()),
            copyright_holder: None,
            short_abstract: None,
            long_abstract: Some("Lorem\tipsum\r\ndolor sit amet".to_string()),
            general_note: Some(
                "Please note that in this book the mathematical formulas are encoded in MathML."
                    .to_string(),
            ),
            bibliography_note: Some("Includes bibliography (pages 165-170) and index.".to_string()),
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
            toc: Some("Introduction;\tChapter 1;\rChapter 2;\nBibliography; Index".to_string()),
            lccn: Some("LCCN010101".to_string()),
            oclc: Some("OCLC010101".to_string()),
            cover_url: Some("https://www.book.com/cover.jpg".to_string()),
            cover_caption: None,
            imprint: WorkImprint {
                imprint_name: "OA Editions Imprint".to_string(),
                imprint_url: None,
                crossmark_doi: None,
                publisher: WorkImprintPublisher {
                    publisher_name: "OA Editions".to_string(),
                    publisher_shortname: None,
                    publisher_url: None,
                },
            },
            issues: vec![WorkIssues {
                issue_ordinal: 11,
                series: WorkIssuesSeries {
                    series_id: Uuid::parse_str("00000000-0000-0000-BBBB-000000000002").unwrap(),
                    series_type: SeriesType::BOOK_SERIES,
                    series_name: "Name of series".to_string(),
                    issn_print: Some("1234-5678".to_string()),
                    issn_digital: Some("8765-4321".to_string()),
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
                            ror: None,
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
                            Orcid::from_str("https://orcid.org/0000-0002-0000-0004").unwrap(),
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
                    publication_id: Default::default(),
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
                    publication_id: Default::default(),
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
                    publication_id: Default::default(),
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
                WorkSubjects {
                    subject_code: "custom1".to_string(),
                    subject_type: SubjectType::CUSTOM,
                    subject_ordinal: 5,
                },
                WorkSubjects {
                    subject_code: "keyword1".to_string(),
                    subject_type: SubjectType::KEYWORD,
                    subject_ordinal: 6,
                },
                WorkSubjects {
                    subject_code: "keyword2".to_string(),
                    subject_type: SubjectType::KEYWORD,
                    subject_ordinal: 7,
                },
            ],
            fundings: vec![WorkFundings {
                program: Some("Funding Programme".to_string()),
                project_name: Some("Funding Project".to_string()),
                project_shortname: None,
                grant_number: Some("JA0001".to_string()),
                jurisdiction: None,
                institution: FundingInstitution {
                    institution_name: "Funding Institution".to_string(),
                    institution_doi: None,
                    ror: None,
                    country_code: None,
                },
            }],
            relations: vec![],
            references: vec![],
        }
    }

    fn test_contribution() -> WorkContributions {
        WorkContributions {
            contribution_type: thoth_client::ContributionType::AUTHOR,
            first_name: None,
            last_name: "".to_string(),
            full_name: "".to_string(),
            main_contribution: true,
            biography: None,
            contribution_ordinal: 1,
            contributor: WorkContributionsContributor {
                orcid: None,
                website: None,
            },
            affiliations: vec![],
        }
    }

    fn test_relation() -> WorkRelations {
        WorkRelations {
            relation_type: RelationType::HAS_CHILD,
            relation_ordinal: 1,
            related_work: WorkRelationsRelatedWork {
                work_status: WorkStatus::ACTIVE,
                titles: vec![thoth_client::WorkRelationsRelatedWorkTitles {
                    title_id: Uuid::from_str("00000000-0000-0000-CCCC-000000000001").unwrap(),
                    locale_code: thoth_client::LocaleCode::EN,
                    full_title: "Chapter One".to_string(),
                    title: "N/A".to_string(),
                    subtitle: None,
                    canonical: true,
                }],
                edition: None,
                doi: None,
                publication_date: None,
                withdrawn_date: None,
                license: None,
                copyright_holder: None,
                short_abstract: None,
                long_abstract: None,
                general_note: None,
                place: None,
                first_page: None,
                last_page: None,
                page_count: None,
                page_interval: None,
                landing_page: None,
                imprint: WorkRelationsRelatedWorkImprint {
                    crossmark_doi: None,
                    publisher: WorkRelationsRelatedWorkImprintPublisher {
                        publisher_name: "N/A".to_string(),
                    },
                },
                contributions: vec![WorkRelationsRelatedWorkContributions {
                    contribution_type: thoth_client::ContributionType::AUTHOR,
                    first_name: Some("Chapter-One".to_string()),
                    last_name: "Author".to_string(),
                    full_name: "Chapter-One Author".to_string(),
                    biography: None,
                    contribution_ordinal: 1,
                    contributor: WorkRelationsRelatedWorkContributionsContributor {
                        orcid: None,
                        website: None,
                    },
                    affiliations: vec![],
                }],
                publications: vec![],
                references: vec![],
                fundings: vec![],
                languages: vec![],
            },
        }
    }

    #[test]
    fn test_contributor_fields_empty_slice() {
        let contributions: [WorkContributions; 0] = [];
        let expected: ThothResult<Vec<FieldRepr>> = Ok(vec![]);
        let result = contributor_fields(&contributions);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_contributor_fields_single_author() {
        let mut contribution = test_contribution();
        contribution.first_name = Some("Jane".to_string());
        contribution.last_name = "Doe".to_string();
        contribution.full_name = "Jane Doe".to_string();
        let contributions = [contribution];

        let expected = Ok(vec![FieldRepr::from((b"100", "1\\"))
            .add_subfield(b"a", "Doe, Jane,".as_bytes())
            .and_then(|f| f.add_subfield(b"e", "author.".as_bytes()))
            .unwrap()]);
        assert_eq!(contributor_fields(&contributions), expected);
    }

    #[test]
    fn test_contributor_fields_multiple_contributors_no_author() {
        let mut first_contribution = test_contribution();
        first_contribution.first_name = Some("Jane".to_string());
        first_contribution.last_name = "Doe".to_string();
        first_contribution.full_name = "Jane Doe".to_string();
        first_contribution.contribution_type = thoth_client::ContributionType::EDITOR;
        let mut second_contribution = test_contribution();
        second_contribution.first_name = Some("John".to_string());
        second_contribution.last_name = "Smith".to_string();
        second_contribution.full_name = "John Smith".to_string();
        second_contribution.contribution_type = thoth_client::ContributionType::TRANSLATOR;
        let contributions = [first_contribution, second_contribution];

        let expected = Ok(vec![
            FieldRepr::from((b"700", "1\\"))
                .add_subfield(b"a", "Doe, Jane,".as_bytes())
                .and_then(|f| f.add_subfield(b"e", "editor.".as_bytes()))
                .unwrap(),
            FieldRepr::from((b"700", "1\\"))
                .add_subfield(b"a", "Smith, John,".as_bytes())
                .and_then(|f| f.add_subfield(b"e", "translator.".as_bytes()))
                .unwrap(),
        ]);
        assert_eq!(contributor_fields(&contributions), expected);
    }

    #[test]
    fn test_contributor_fields_multiple_contributions_one_author() {
        let mut first_contribution = test_contribution();
        first_contribution.first_name = Some("John".to_string());
        first_contribution.last_name = "Smith".to_string();
        first_contribution.full_name = "John Smith".to_string();
        let mut second_contribution = test_contribution();
        second_contribution.first_name = Some("Jane".to_string());
        second_contribution.last_name = "Doe".to_string();
        second_contribution.full_name = "Jane Doe".to_string();
        second_contribution.contribution_type = thoth_client::ContributionType::EDITOR;
        let mut third_contribution = test_contribution();
        third_contribution.first_name = Some("Jane".to_string());
        third_contribution.last_name = "Doe".to_string();
        third_contribution.full_name = "Jane Doe".to_string();
        third_contribution.contribution_type = thoth_client::ContributionType::TRANSLATOR;
        let contributions = [first_contribution, second_contribution, third_contribution];

        let expected = Ok(vec![
            FieldRepr::from((b"100", "1\\"))
                .add_subfield(b"a", "Smith, John,".as_bytes())
                .and_then(|f| f.add_subfield(b"e", "author.".as_bytes()))
                .unwrap(),
            FieldRepr::from((b"700", "1\\"))
                .add_subfield(b"a", "Doe, Jane,".as_bytes())
                .and_then(|f| f.add_subfield(b"e", "editor, translator.".as_bytes()))
                .unwrap(),
        ]);
        assert_eq!(contributor_fields(&contributions), expected);
    }

    #[test]
    fn test_contributor_fields_multiple_contributors_multiple_authors() {
        let mut first_contribution = test_contribution();
        first_contribution.first_name = Some("John".to_string());
        first_contribution.last_name = "Smith".to_string();
        first_contribution.full_name = "John Smith".to_string();
        let mut second_contribution = test_contribution();
        second_contribution.first_name = Some("Jane".to_string());
        second_contribution.last_name = "Doe".to_string();
        second_contribution.full_name = "Jane Doe".to_string();
        let mut third_contribution = test_contribution();
        third_contribution.first_name = Some("Billy Bob".to_string());
        third_contribution.last_name = "Johnson".to_string();
        third_contribution.full_name = "Billy Bob Johnson".to_string();
        third_contribution.contribution_type = thoth_client::ContributionType::INTRODUCTION_BY;
        let mut fourth_contribution = test_contribution();
        fourth_contribution.first_name = Some("Juan".to_string());
        fourth_contribution.last_name = "García Sánchez".to_string();
        fourth_contribution.full_name = "Juan García Sánchez".to_string();
        fourth_contribution.contribution_type = thoth_client::ContributionType::TRANSLATOR;
        let contributions = [
            first_contribution,
            second_contribution,
            third_contribution,
            fourth_contribution,
        ];

        let expected = Ok(vec![
            FieldRepr::from((b"100", "1\\"))
                .add_subfield(b"a", "Smith, John,".as_bytes())
                .and_then(|f| f.add_subfield(b"e", "author.".as_bytes()))
                .unwrap(),
            FieldRepr::from((b"700", "1\\"))
                .add_subfield(b"a", "Doe, Jane,".as_bytes())
                .and_then(|f| f.add_subfield(b"e", "author.".as_bytes()))
                .unwrap(),
            FieldRepr::from((b"700", "1\\"))
                .add_subfield(b"a", "Johnson, Billy Bob,".as_bytes())
                .and_then(|f| f.add_subfield(b"e", "introduction by.".as_bytes()))
                .unwrap(),
            FieldRepr::from((b"700", "1\\"))
                .add_subfield(b"a", "García Sánchez, Juan,".as_bytes())
                .and_then(|f| f.add_subfield(b"e", "translator.".as_bytes()))
                .unwrap(),
        ]);
        assert_eq!(contributor_fields(&contributions), expected);
    }

    #[test]
    fn test_contributor_fields_single_author_single_affiliation() {
        let mut contribution = test_contribution();
        contribution.first_name = Some("Jane".to_string());
        contribution.last_name = "Doe".to_string();
        contribution.full_name = "Jane Doe".to_string();
        contribution.affiliations = vec![WorkContributionsAffiliations {
            position: None,
            affiliation_ordinal: 1,
            institution: WorkContributionsAffiliationsInstitution {
                institution_name: "Thoth University".to_string(),
                institution_doi: None,
                ror: None,
                country_code: None,
            },
        }];
        let contributions = [contribution];

        let expected = Ok(vec![FieldRepr::from((b"100", "1\\"))
            .add_subfield(b"a", "Doe, Jane,".as_bytes())
            .and_then(|f| f.add_subfield(b"e", "author.".as_bytes()))
            .and_then(|f| f.add_subfield(b"u", "Thoth University.".as_bytes()))
            .unwrap()]);
        assert_eq!(contributor_fields(&contributions), expected);
    }

    #[test]
    fn test_contributor_fields_single_author_multiple_affiliations() {
        let mut contribution = test_contribution();
        contribution.first_name = Some("Jane".to_string());
        contribution.last_name = "Doe".to_string();
        contribution.full_name = "Jane Doe".to_string();
        contribution.affiliations = vec![
            WorkContributionsAffiliations {
                position: None,
                affiliation_ordinal: 1,
                institution: WorkContributionsAffiliationsInstitution {
                    institution_name: "Thoth University".to_string(),
                    institution_doi: None,
                    ror: None,
                    country_code: None,
                },
            },
            WorkContributionsAffiliations {
                position: None,
                affiliation_ordinal: 2,
                institution: WorkContributionsAffiliationsInstitution {
                    institution_name: "COPIM".to_string(),
                    institution_doi: None,
                    ror: None,
                    country_code: None,
                },
            },
        ];
        let contributions = [contribution];

        let expected = Ok(vec![FieldRepr::from((b"100", "1\\"))
            .add_subfield(b"a", "Doe, Jane,".as_bytes())
            .and_then(|f| f.add_subfield(b"e", "author.".as_bytes()))
            .and_then(|f| f.add_subfield(b"u", "Thoth University.".as_bytes()))
            .unwrap()]);
        assert_eq!(contributor_fields(&contributions), expected);
    }

    #[test]
    fn test_contributor_fields_single_author_with_orcid() {
        let mut contribution = test_contribution();
        contribution.first_name = Some("Jane".to_string());
        contribution.last_name = "Doe".to_string();
        contribution.full_name = "Jane Doe".to_string();
        contribution.contributor = WorkContributionsContributor {
            orcid: Some(Orcid::from_str("https://orcid.org/0000-0002-0000-0011").unwrap()),
            website: None,
        };
        let contributions = [contribution];

        let expected = Ok(vec![FieldRepr::from((b"100", "1\\"))
            .add_subfield(b"a", "Doe, Jane,".as_bytes())
            .and_then(|f| f.add_subfield(b"e", "author.".as_bytes()))
            .and_then(|f| f.add_subfield(b"0", "(orcid)0000000200000011".as_bytes()))
            .and_then(|f| f.add_subfield(b"1", "https://orcid.org/0000-0002-0000-0011".as_bytes()))
            .unwrap()]);
        assert_eq!(contributor_fields(&contributions), expected);
    }

    #[test]
    fn test_contributor_fields_multiple_contributors_no_first_names() {
        let mut first_contribution = test_contribution();
        first_contribution.last_name = "Thoth Collective".to_string();
        first_contribution.full_name = "Thoth Collective".to_string();
        first_contribution.contribution_type = thoth_client::ContributionType::EDITOR;
        let mut second_contribution = test_contribution();
        second_contribution.last_name = "Anonymous".to_string();
        second_contribution.full_name = "Anonymous".to_string();
        let mut third_contribution = test_contribution();
        third_contribution.last_name = "Thoth Collective".to_string();
        third_contribution.full_name = "Thoth Collective".to_string();
        third_contribution.contribution_type = thoth_client::ContributionType::INTRODUCTION_BY;
        let mut fourth_contribution = test_contribution();
        fourth_contribution.first_name = Some("Thoth".to_string());
        fourth_contribution.last_name = "Collective".to_string();
        fourth_contribution.full_name = "Thoth Collective".to_string();
        fourth_contribution.contribution_type = thoth_client::ContributionType::RESEARCH_BY;
        let contributions = [
            first_contribution,
            second_contribution,
            third_contribution,
            fourth_contribution,
        ];

        let expected = Ok(vec![
            FieldRepr::from((b"700", "0\\"))
                .add_subfield(b"a", "Thoth Collective,".as_bytes())
                .and_then(|f| f.add_subfield(b"e", "editor, introduction by.".as_bytes()))
                .unwrap(),
            FieldRepr::from((b"100", "0\\"))
                .add_subfield(b"a", "Anonymous,".as_bytes())
                .and_then(|f| f.add_subfield(b"e", "author.".as_bytes()))
                .unwrap(),
            FieldRepr::from((b"700", "1\\"))
                .add_subfield(b"a", "Collective, Thoth,".as_bytes())
                .and_then(|f| f.add_subfield(b"e", "research by.".as_bytes()))
                .unwrap(),
        ]);
        assert_eq!(contributor_fields(&contributions), expected);
    }

    #[test]
    fn test_language_field_original_only() {
        let languages = vec![WorkLanguages {
            language_code: LanguageCode::ENG,
            language_relation: LanguageRelation::ORIGINAL,
            main_language: true,
        }];
        assert_eq!(language_field(&languages), None);
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
    fn test_toc_field_one_chapter_one_author() {
        let relations = vec![test_relation()];
        let expected = Ok(FieldRepr::from((b"505", "00"))
            .add_subfield(b"t", "Chapter One /".as_bytes())
            .and_then(|f| f.add_subfield(b"r", "Chapter-One Author.".as_bytes()))
            .unwrap());
        assert_eq!(toc_field(&relations), expected);
    }

    #[test]
    fn test_toc_field_one_chapter_no_author() {
        let mut relation = test_relation();
        relation.related_work.contributions.clear();
        let relations = vec![relation];
        let expected = Ok(FieldRepr::from((b"505", "00"))
            .add_subfield(b"t", "Chapter One.".as_bytes())
            .unwrap());
        assert_eq!(toc_field(&relations), expected);
    }

    #[test]
    fn test_toc_field_one_chapter_one_author_with_pages() {
        let mut relation = test_relation();
        relation.related_work.page_interval = Some("10–20".to_string());
        let relations = vec![relation];
        let expected = Ok(FieldRepr::from((b"505", "00"))
            .add_subfield(b"t", "Chapter One /".as_bytes())
            .and_then(|f| f.add_subfield(b"r", "Chapter-One Author".as_bytes()))
            .and_then(|f| f.add_subfield(b"g", "(pp10–20).".as_bytes()))
            .unwrap());
        assert_eq!(toc_field(&relations), expected);
    }

    #[test]
    fn test_toc_field_one_chapter_no_author_with_pages() {
        let mut relation = test_relation();
        relation.related_work.contributions.clear();
        relation.related_work.page_interval = Some("10–20".to_string());
        let relations = vec![relation];
        let expected = Ok(FieldRepr::from((b"505", "00"))
            .add_subfield(b"t", "Chapter One".as_bytes())
            .and_then(|f| f.add_subfield(b"g", "(pp10–20).".as_bytes()))
            .unwrap());
        assert_eq!(toc_field(&relations), expected);
    }

    #[test]
    fn test_toc_field_one_chapter_two_authors() {
        let mut relation = test_relation();
        relation
            .related_work
            .contributions
            .append(&mut vec![relation.related_work.contributions[0].clone()]);
        relation.related_work.contributions[1].full_name = "Chapter-One Second-Author".to_string();
        // Not strictly required, but let's keep things tidy
        relation.related_work.contributions[1].contribution_ordinal = 2;
        let relations = vec![relation];
        let expected = Ok(FieldRepr::from((b"505", "00"))
            .add_subfield(b"t", "Chapter One /".as_bytes())
            .and_then(|f| {
                f.add_subfield(
                    b"r",
                    "Chapter-One Author, Chapter-One Second-Author.".as_bytes(),
                )
            })
            .unwrap());
        assert_eq!(toc_field(&relations), expected);
    }

    #[test]
    fn test_toc_field_two_chapters_one_author_each() {
        let mut second_relation = test_relation();
        second_relation.relation_ordinal = 2;
        second_relation.related_work.titles[0].full_title = "Chapter Two".to_string();
        second_relation.related_work.contributions[0].full_name = "Chapter-Two Author".to_string();
        // Place in reverse order and test correct re-ordering
        let relations = vec![second_relation, test_relation()];
        let expected = Ok(FieldRepr::from((b"505", "00"))
            .add_subfield(b"t", "Chapter One /".as_bytes())
            .and_then(|f| f.add_subfield(b"r", "Chapter-One Author --".as_bytes()))
            .and_then(|f| f.add_subfield(b"t", "Chapter Two /".as_bytes()))
            .and_then(|f| f.add_subfield(b"r", "Chapter-Two Author.".as_bytes()))
            .unwrap());
        assert_eq!(toc_field(&relations), expected);
    }

    #[test]
    fn test_toc_field_two_chapters_no_authors() {
        let mut first_relation = test_relation();
        first_relation.related_work.contributions.clear();
        let mut second_relation = test_relation();
        second_relation.relation_ordinal = 2;
        second_relation.related_work.titles[0].full_title = "Chapter Two".to_string();
        second_relation.related_work.contributions.clear();
        let relations = vec![first_relation, second_relation];
        let expected = Ok(FieldRepr::from((b"505", "00"))
            .add_subfield(b"t", "Chapter One --".as_bytes())
            .and_then(|f| f.add_subfield(b"t", "Chapter Two.".as_bytes()))
            .unwrap());
        assert_eq!(toc_field(&relations), expected);
    }

    #[test]
    fn test_toc_field_two_chapters_no_authors_with_pages() {
        let mut first_relation = test_relation();
        first_relation.related_work.page_interval = Some("10–20".to_string());
        first_relation.related_work.contributions.clear();
        let mut second_relation = test_relation();
        second_relation.relation_ordinal = 2;
        second_relation.related_work.titles[0].full_title = "Chapter Two".to_string();
        second_relation.related_work.page_interval = Some("20–30".to_string());
        second_relation.related_work.contributions.clear();
        let relations = vec![first_relation, second_relation];
        let expected = Ok(FieldRepr::from((b"505", "00"))
            .add_subfield(b"t", "Chapter One".as_bytes())
            .and_then(|f| f.add_subfield(b"g", "(pp10–20) --".as_bytes()))
            .and_then(|f| f.add_subfield(b"t", "Chapter Two".as_bytes()))
            .and_then(|f| f.add_subfield(b"g", "(pp20–30).".as_bytes()))
            .unwrap());
        assert_eq!(toc_field(&relations), expected);
    }

    #[test]
    fn test_toc_field_one_relation_is_chapter() {
        let mut parent_relation = test_relation();
        parent_relation.relation_type = RelationType::IS_CHILD_OF;
        let relations = vec![test_relation(), parent_relation];
        let expected = Ok(FieldRepr::from((b"505", "00"))
            .add_subfield(b"t", "Chapter One /".as_bytes())
            .and_then(|f| f.add_subfield(b"r", "Chapter-One Author.".as_bytes()))
            .unwrap());
        assert_eq!(toc_field(&relations), expected);
    }

    #[test]
    fn test_toc_field_no_relations_are_chapters() {
        let mut relation = test_relation();
        relation.relation_type = RelationType::IS_TRANSLATION_OF;
        let relations = vec![relation];
        assert!(toc_field(&relations).is_err());
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

    #[test]
    fn test_contributors_string_single_author() {
        let mut contribution = test_contribution();
        contribution.full_name = "John Doe".to_string();
        let contributions = [contribution];

        let expected = "John Doe.";
        assert_eq!(contributors_string(&contributions), expected);
    }

    #[test]
    fn test_contributors_string_single_editor() {
        let mut contribution = test_contribution();
        contribution.full_name = "Jane Smith".to_string();
        contribution.contribution_type = thoth_client::ContributionType::EDITOR;
        let contributions = [contribution];

        let expected = "edited by Jane Smith.";
        assert_eq!(contributors_string(&contributions), expected);
    }

    #[test]
    fn test_contributors_string_single_translator() {
        let mut contribution = test_contribution();
        contribution.full_name = "Juan García Sánchez".to_string();
        contribution.contribution_type = thoth_client::ContributionType::TRANSLATOR;
        let contributions = [contribution];

        let expected = "translated by Juan García Sánchez.";
        assert_eq!(contributors_string(&contributions), expected);
    }

    #[test]
    fn test_contributors_string_multiple_authors() {
        let mut first_contribution = test_contribution();
        first_contribution.full_name = "John Doe".to_string();
        let mut second_contribution = test_contribution();
        second_contribution.full_name = "Jane Smith".to_string();
        let mut third_contribution = test_contribution();
        third_contribution.full_name = "Billy Bob Johnson".to_string();
        let contributions = [first_contribution, second_contribution, third_contribution];

        let expected = "John Doe, Jane Smith, Billy Bob Johnson.";
        assert_eq!(contributors_string(&contributions), expected);
    }

    #[test]
    fn test_contributors_string_multiple_editors() {
        let mut first_contribution = test_contribution();
        first_contribution.full_name = "Jane Smith".to_string();
        first_contribution.contribution_type = thoth_client::ContributionType::EDITOR;
        let mut second_contribution = test_contribution();
        second_contribution.full_name = "Billy Bob Johnson".to_string();
        second_contribution.contribution_type = thoth_client::ContributionType::EDITOR;
        let contributions = [first_contribution, second_contribution];

        let expected = "edited by Jane Smith, Billy Bob Johnson.";
        assert_eq!(contributors_string(&contributions), expected);
    }

    #[test]
    fn test_contributors_string_multiple_types() {
        let mut first_contribution = test_contribution();
        first_contribution.full_name = "John Doe".to_string();
        let mut second_contribution = test_contribution();
        second_contribution.full_name = "Jane Smith".to_string();
        second_contribution.contribution_type = thoth_client::ContributionType::EDITOR;
        let mut third_contribution = test_contribution();
        third_contribution.full_name = "Billy Bob Johnson".to_string();
        third_contribution.contribution_type = thoth_client::ContributionType::TRANSLATOR;
        let mut fourth_contribution = test_contribution();
        fourth_contribution.full_name = "Alice Brown".to_string();
        fourth_contribution.contribution_type = thoth_client::ContributionType::INTRODUCTION_BY;
        let contributions = [
            first_contribution,
            second_contribution,
            third_contribution,
            fourth_contribution,
        ];

        let expected = "John Doe; edited by Jane Smith; translated by Billy Bob Johnson; introduction by Alice Brown.";
        assert_eq!(contributors_string(&contributions), expected);
    }

    #[test]
    fn test_contributors_string_multiple_authors_and_editors() {
        let mut first_contribution = test_contribution();
        first_contribution.full_name = "John Doe".to_string();
        let mut second_contribution = test_contribution();
        second_contribution.full_name = "Jane Smith".to_string();
        second_contribution.contribution_type = thoth_client::ContributionType::EDITOR;
        let mut third_contribution = test_contribution();
        third_contribution.full_name = "Billy Bob Johnson".to_string();
        third_contribution.contribution_type = thoth_client::ContributionType::EDITOR;
        let mut fourth_contribution = test_contribution();
        fourth_contribution.full_name = "Alice Brown".to_string();
        let contributions = [
            first_contribution,
            second_contribution,
            third_contribution,
            fourth_contribution,
        ];

        let expected = "John Doe, Alice Brown; edited by Jane Smith, Billy Bob Johnson.";
        assert_eq!(contributors_string(&contributions), expected);
    }

    #[test]
    fn test_nonfiling_char_count() {
        for (language, title, expected) in [
            ("eng", "A sample book", "2"),
            ("eng", "An example book", "3"),
            ("eng", "The next example book", "4"),
            ("eng", "a lowercase book", "2"),
            ("eng", "AAAAAA BOOK", "0"),
            ("eng", "There's a book", "0"),
            ("fre", "The et cafe", "0"),
            ("fre", "A la gare", "0"),
        ] {
            assert_eq!(nonfiling_char_count(title, language), expected);
        }
    }

    #[test]
    fn test_generate_marc() {
        let work = test_work();
        let current_date = Utc::now().format("%y%m%d").to_string();
        let expected = format!("02443nam  22005532  4500001003700000006001900037007001500056008004100071010001500112020002500127020002500152020003000177024002800207024002500235040002400260041001300284050000900297072001600306072002300322072001500345100011000360245009700470250002000567264004000587264001100627300002300638336002600661337002600687338003600713490005300749500008300802504005300885505005700938506005000995520002901045536006801074538003601142540022301178588005801401653001301459653001301472700009101485700003701576710002901613830005301642856005801695856005901753856007701812\u{1e}00000000-0000-0000-aaaa-000000000001\u{1e}m     o  d        \u{1e}cr  n         \u{1e}{current_date}t20102010        ob    001 0 eng d\u{1e}\\\\\u{1f}aLCCN010101\u{1e}\\\\\u{1f}a9783161484100\u{1f}q(PDF)\u{1e}\\\\\u{1f}a9789295055025\u{1f}q(XML)\u{1e}\\\\\u{1f}z9781402894626\u{1f}q(Hardback)\u{1e}7\\\u{1f}a10.00001/BOOK.0001\u{1f}2doi\u{1e}7\\\u{1f}aOCLC010101\u{1f}2worldcat\u{1e}\\\\\u{1f}aUkCbTOM\u{1f}beng\u{1f}elocal\u{1e}1\\\u{1f}aeng\u{1f}hspa\u{1e}00\u{1f}aJA85\u{1e} 7\u{1f}aAAB\u{1f}2bicssc\u{1e} 7\u{1f}aAAA000000\u{1f}2bisacsh\u{1e} 7\u{1f}aJWA\u{1f}2thema\u{1e}1\\\u{1f}aAuthor, Sole,\u{1f}eauthor.\u{1f}uThoth University.\u{1f}0(orcid)0000000200000001\u{1f}1https://orcid.org/0000-0002-0000-0001\u{1e}10\u{1f}aBook Title :\u{1f}bBook Subtitle /\u{1f}cSole Author; edited by Only Editor; translated by Translator.\u{1e}\\\\\u{1f}aSecond edition.\u{1e}\\1\u{1f}aLeón, Spain :\u{1f}bOA Editions,\u{1f}c2010.\u{1e}\\4\u{1f}c©2010\u{1e}\\\\\u{1f}a1 online resource.\u{1e}\\\\\u{1f}atext\u{1f}btxt\u{1f}2rdacontent\u{1e}\\\\\u{1f}acomputer\u{1f}bc\u{1f}2rdamedia\u{1e}\\\\\u{1f}aonline resource\u{1f}bcr\u{1f}2rdacarrier\u{1e}1\\\u{1f}aName of series ;\u{1f}vvol. 11.\u{1f}x8765-4321\u{1f}x1234-5678\u{1e}\\\\\u{1f}aPlease note that in this book the mathematical formulas are encoded in MathML.\u{1e}\\\\\u{1f}aIncludes bibliography (pages 165-170) and index.\u{1e}0\\\u{1f}aIntroduction;Chapter 1;Chapter 2;Bibliography; Index\u{1e}0\\\u{1f}aOpen Access\u{1f}fUnrestricted online access\u{1f}2star\u{1e}\\\\\u{1f}aLoremipsumdolor sit amet\u{1e}\\\\\u{1f}aFunding Institution\u{1f}cJA0001\u{1f}eFunding Programme\u{1f}fFunding Project\u{1e}\\\\\u{1f}aMode of access: World Wide Web.\u{1e}\\\\\u{1f}aThe text of this book is licensed under a Creative Commons Attribution 4.0 International license (CC BY 4.0). For more detailed information consult the publisher's website.\u{1f}uhttps://creativecommons.org/licenses/by/4.0/\u{1e}0\\\u{1f}aMetadata licensed under CC0 Public Domain Dedication.\u{1e}\\\\\u{1f}akeyword1\u{1e}\\\\\u{1f}akeyword2\u{1e}1\\\u{1f}aEditor, Only,\u{1f}eeditor.\u{1f}0(orcid)0000000200000004\u{1f}1https://orcid.org/0000-0002-0000-0004\u{1e}0\\\u{1f}aTranslator,\u{1f}etranslator.\u{1f}uCOPIM.\u{1e}2\\\u{1f}aOA Editions,\u{1f}epublisher.\u{1e}\\0\u{1f}aName of series ;\u{1f}vvol. 11.\u{1f}x8765-4321\u{1f}x1234-5678\u{1e}40\u{1f}uhttps://doi.org/10.00001/book.0001\u{1f}zConnect to e-book\u{1e}42\u{1f}uhttps://www.book.com/cover.jpg\u{1f}zConnect to cover image\u{1e}42\u{1f}uhttps://creativecommons.org/publicdomain/zero/1.0/\u{1f}zCC0 Metadata License\u{1e}\u{1d}");

        assert_eq!(Marc21RecordThoth {}.generate(&[work]), Ok(expected))
    }

    #[test]
    fn test_generate_no_work_error() {
        assert!(Marc21RecordThoth {}.generate(&[]).is_err())
    }

    #[test]
    fn test_generate_chapter_error() {
        let mut work = test_work();
        work.work_type = WorkType::BOOK_CHAPTER;
        assert!(Marc21RecordThoth {}.generate(&[work]).is_err())
    }

    #[test]
    fn test_generate_no_publications_error() {
        let mut work = test_work();
        work.publications = vec![];
        assert!(Marc21RecordThoth {}.generate(&[work]).is_err())
    }

    #[test]
    fn test_generate_no_contributions_error() {
        let mut work = test_work();
        work.contributions = vec![];
        assert!(Marc21RecordThoth {}.generate(&[work]).is_err())
    }

    #[test]
    fn test_generate_no_publication_date_error() {
        let mut work = test_work();
        work.publication_date = None;
        assert!(Marc21RecordThoth {}.generate(&[work]).is_err())
    }
}
