use marc::Record;
use thoth_client::Work;
use thoth_errors::{ThothError, ThothResult};

pub(crate) trait Marc21Specification {
    fn generate(&self, works: &[Work]) -> ThothResult<String> {
        let mut buffer: Vec<u8> = Vec::new();
        Self::handle_event(&mut buffer, works)
            .map(|_| buffer)
            .and_then(|bibtex| {
                String::from_utf8(bibtex)
                    .map_err(|_| ThothError::InternalError("Could not parse MARC 21".to_string()))
            })
    }

    fn handle_event(w: &mut Vec<u8>, works: &[Work]) -> ThothResult<()>;
}

pub(crate) trait Marc21Entry<T: Marc21Specification> {
    fn marc21_entry(&self, w: &mut Vec<u8>) -> ThothResult<()>;

    fn marc21_record(&self) -> ThothResult<Record>;
}

mod marc21record_thoth;
pub(crate) use marc21record_thoth::Marc21RecordThoth;
