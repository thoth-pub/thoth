use std::io::Write;
use csv::{Writer, Result as CsvResult};
use thoth_client::Work;
use serde::Serialize;

use super::{CsvSpecification, CsvRow};

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
    fn handle_event<W: Write>(w: &mut Writer<W>, works: Vec<Work>) -> CsvResult<()> {
        for work in works.iter() {
            CsvRow::<CsvThoth>::csv_row(work, w)?;
        }
        Ok(())
    }
}

impl CsvRow<CsvThoth> for Work {
    fn csv_row<W: Write>(&self, w: &mut Writer<W>) -> CsvResult<()> {
        w.serialize(CsvThothRow::from(self.clone()))
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