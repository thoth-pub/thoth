use csv::Writer;
use std::io::Write;
use thoth_api::errors::{ThothError, ThothResult};
use thoth_client::Work;

pub(crate) trait CsvSpecification {
    fn generate(&self, works: &[Work]) -> ThothResult<String> {
        let mut writer = Writer::from_writer(Vec::new());
        Self::handle_event(&mut writer, works)
            .map(|_| writer.into_inner().map_err(|e| e.error().into()))
            .and_then(|val| val)
            .and_then(|csv| {
                String::from_utf8(csv)
                    .map_err(|_| ThothError::InternalError("Could not parse CSV".to_string()))
            })
    }

    fn handle_event<W: Write>(w: &mut Writer<W>, works: &[Work]) -> ThothResult<()>;
}

pub(crate) trait CsvRow<T: CsvSpecification> {
    fn csv_row<W: Write>(&self, w: &mut Writer<W>) -> ThothResult<()>;
}

mod csv_thoth;
pub(crate) use csv_thoth::CsvThoth;
