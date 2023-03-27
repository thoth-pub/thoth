use crate::marc21::Marc21Field;
use marc::{FieldRepr, Record, RecordBuilder};
use thoth_api::model::contribution::ContributionType;
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

        // 245 – title
        let mut title_field: FieldRepr = FieldRepr::from((b"245", "00")); // no title added entry
        title_field = title_field.add_subfield(b'a', self.title.clone().into_bytes())?; // main title
        title_field = title_field.add_subfield(b'h', b"[electronic resource] :")?; // general material designation (GMD)
        if let Some(subtitle) = self.subtitle.clone() {
            title_field = title_field.add_subfield(b'b', subtitle.into_bytes())?;
            // subtitle
        }
        title_field =
            title_field.add_subfield(b'c', contributors_string(&self.contributions).as_bytes())?; // statement of responsibility
        builder.add_field(title_field)?;

        // 020 - ISBN
        for publication in &self.publications {
            Marc21Field::<Marc21RecordThoth>::to_field(publication, &mut builder)?;
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
            let mut isbn_field: FieldRepr = FieldRepr::from((b"020", ""));
            isbn_field = isbn_field.add_subfield(b'a', isbn.to_hyphenless_string().as_bytes())?;
            isbn_field = isbn_field
                .add_subfield(b'q', format!("({})", self.publication_type).into_bytes())?;

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
