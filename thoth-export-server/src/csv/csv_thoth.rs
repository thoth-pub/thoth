use csv::Writer;
use serde::Serialize;
use std::io::Write;
use thoth_api::errors::ThothResult;
use thoth_client::Work;

use super::{CsvRow, CsvSpecification};

pub(crate) struct CsvThoth;

#[derive(Debug, Serialize)]
struct CsvThothRow {
    publisher: String,
    imprint: String,
    work_type: String,
    title: String,
    subtitle: Option<String>,
    doi: Option<String>,
    publication_date: Option<String>,
}

impl CsvSpecification for CsvThoth {
    fn handle_event<W: Write>(w: &mut Writer<W>, works: &[Work]) -> ThothResult<()> {
        for work in works.iter() {
            CsvRow::<CsvThoth>::csv_row(work, w)?;
        }
        Ok(())
    }
}

impl CsvRow<CsvThoth> for Work {
    fn csv_row<W: Write>(&self, w: &mut Writer<W>) -> ThothResult<()> {
        w.serialize(CsvThothRow::from(self.clone()))
            .map_err(|e| e.into())
    }
}

impl From<Work> for CsvThothRow {
    fn from(work: Work) -> Self {
        CsvThothRow {
            publisher: work.imprint.publisher.publisher_name,
            imprint: work.imprint.imprint_name,
            work_type: format!("{:?}", work.work_type),
            title: work.title,
            subtitle: work.subtitle,
            doi: work.doi,
            publication_date: work.publication_date.map(|d| d.to_string()),
        }
    }
}
