use crate::marc21::Marc21Field;
use marc::{Record, RecordBuilder};
use thoth_client::{Work, WorkPublications, WorkType};
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
        builder.add_field((b"001", self.title.clone().into_bytes()))?;

        let publications: Vec<WorkPublications> = self
            .publications
            .clone()
            .into_iter()
            .filter(|p| p.isbn.is_some())
            .collect();
        if !publications.is_empty() {
            for publication in &publications {
                Marc21Field::<Marc21RecordThoth>::to_field(publication, &mut builder)?;
            }
        }

        if let Some(doi) = self.doi.clone() {
            builder.add_field((b"024", doi.to_lowercase_string().into_bytes()))?;
        }

        Ok(builder.get_record()?)
    }
}

impl Marc21Field<Marc21RecordThoth> for WorkPublications {
    fn to_field(&self, builder: &mut RecordBuilder) -> ThothResult<()> {
        if let Some(isbn) = self.isbn.clone() {
            let mut subfield_data: Vec<u8> = Vec::new();
            subfield_data.extend(b"a");
            subfield_data.extend(isbn.to_hyphenless_string().as_bytes());
            subfield_data.extend(b"q");
            subfield_data.extend(format!("({})", self.publication_type).into_bytes());

            builder.add_field((b"020", subfield_data))?;
        }
        Ok(())
    }
}
