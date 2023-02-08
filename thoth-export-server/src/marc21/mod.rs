use marc::{Record, RecordBuilder};
use std::io::Write;
use thoth_client::Work;
use thoth_errors::{ThothError, ThothResult};

pub(crate) trait Marc21Specification {
    fn generate(&self, works: &[Work]) -> ThothResult<String> {
        let mut buffer: Vec<u8> = Vec::new();
        Self::handle_event(&mut buffer, works)
            .map(|_| buffer)
            .and_then(|marc21| {
                String::from_utf8(marc21)
                    .map_err(|_| ThothError::InternalError("Could not parse MARC 21".to_string()))
            })
    }

    fn handle_event(w: &mut Vec<u8>, works: &[Work]) -> ThothResult<()>;
}

pub(crate) trait Marc21Entry<T: Marc21Specification> {
    fn marc21_record(&self, w: &mut Vec<u8>) -> ThothResult<()> {
        w.write_all(self.to_record()?.as_ref())?;
        Ok(())
    }

    fn to_record(&self) -> ThothResult<Record>;
}

pub(crate) trait Marc21Field<T: Marc21Specification> {
    fn to_field(&self, builder: &mut RecordBuilder) -> ThothResult<()>;
}

mod marc21record_thoth;
pub(crate) use marc21record_thoth::Marc21RecordThoth;
