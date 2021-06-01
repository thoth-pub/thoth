use csv::{Writer, Result as CsvResult};
use std::io::Write;
use thoth_api::errors::{ThothError, ThothResult};
use thoth_client::Work;

pub(crate) trait CsvSpecification {
    fn generate(&self, works: Vec<Work>) -> ThothResult<String> {
        let mut buffer = Vec::new();
        let mut writer = Writer::from_writer(buffer);
        Self::handle_event(&mut writer, works)
            .map(|_| buffer)
            .map_err(|_| ThothError::InternalError("CSV Error".to_string())) // todo: impl From<csv::Error> for ThothError
            .and_then(|csv| {
                String::from_utf8(csv)
                    .map_err(|_| ThothError::InternalError("Could not parse CSV".to_string()))
            })
    }

    fn handle_event<W: Write>(w: &mut Writer<W>, works: Vec<Work>) -> CsvResult<()>;
}

pub(crate) trait CsvRow<T: CsvSpecification> {}

mod csv_thoth;
pub(crate) use csv_thoth::CsvThoth;
