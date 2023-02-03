use marc::{Record, RecordBuilder};
use std::io::Write;
use thoth_client::{Work, WorkType};
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
            1 => Marc21Entry::<Marc21RecordThoth>::marc21_entry(works.first().unwrap(), w),
            _ => {
                for work in works.iter() {
                    // Do not include Chapters in full publisher metadata record
                    // (assumes that a publisher will always have more than one work)
                    if work.work_type != WorkType::BOOK_CHAPTER {
                        Marc21Entry::<Marc21RecordThoth>::marc21_entry(work, w).ok();
                    }
                }
                Ok(())
            }
        }
    }
}

impl Marc21Entry<Marc21RecordThoth> for Work {
    fn marc21_entry(&self, w: &mut Vec<u8>) -> ThothResult<()> {
        w.write_all(self.marc21_record()?.as_ref())?;
        Ok(())
    }

    fn marc21_record(&self) -> ThothResult<Record> {
        let mut builder = RecordBuilder::new();
        let record = builder.add_field((b"001", self.title.clone().into_bytes()))?.get_record()?;
        Ok(record)
    }
}
