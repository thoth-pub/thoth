use crate::marc21::Marc21Field;
use cc_license::License;
use chrono::Datelike;
use marc::{FieldRepr, Record, RecordBuilder};
use thoth_api::model::contribution::ContributionType;
use thoth_api::model::publication::PublicationType;
use thoth_client::{Work, WorkContributions, WorkPublications, WorkType};
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
        let mut builder = RecordBuilder::new();

        // 001 - control number
        builder.add_field((b"001", self.work_id.to_string()))?;

        // 006 - media type
        builder.add_field((b"006", "m        d        "))?;

        // 007 - characteristics
        builder.add_field((b"007", "cr  n         "))?;

        // 020 - ISBN
        for publication in &self.publications {
            Marc21Field::<Marc21RecordThoth>::to_field(publication, &mut builder)?;
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
        if let Some(publication_date) = self.publication_date {
            // year of publication is used in two 264 fields, let's do both
            let year = publication_date.year().to_string();
            publication_field = publication_field.add_subfield(b"c", year.clone().into_bytes())?;
            let mut copyright_year_field = FieldRepr::from((b"264", "\\4"));
            copyright_year_field =
                copyright_year_field.add_subfield(b"c", format!("©{}", year).into_bytes())?;
            builder.add_field(publication_field)?;
            builder.add_field(copyright_year_field)?;
        } else {
            builder.add_field(publication_field)?;
        }

        // 506 - restrictions on access
        let mut restrictions_field: FieldRepr = FieldRepr::from((b"506", "\\\\"));
        restrictions_field =
            restrictions_field.add_subfield(b"a", "Open access resource providing free access.")?;
        builder.add_field(restrictions_field)?;

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

fn contributors_string(contributions: &[WorkContributions]) -> String {
    // group main contributions by contribution type
    let mut contributions_by_type: std::collections::HashMap<
        ContributionType,
        Vec<&WorkContributions>,
    > = std::collections::HashMap::new();
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
