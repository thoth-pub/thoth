use std::io::Write;
use csv::{Writer, Result as CsvResult};
use thoth_client::Work;

use super::CsvSpecification;

pub(crate) struct CsvThoth;

impl CsvSpecification for CsvThoth {
    fn handle_event<W: Write>(w: &mut Writer<W>, works: Vec<Work>) -> CsvResult<()> {
        for work in works.iter() {
            w.serialize(work)?;
        }
        Ok(())
    }
}