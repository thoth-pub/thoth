use std::io::Write;
use thoth_client::Work;
use thoth_errors::{ThothError, ThothResult};

use super::{JsonEntry, JsonSpecification};

#[derive(Copy, Clone)]
pub(crate) struct JsonThoth;

impl JsonSpecification for JsonThoth {
    fn handle_event(w: &mut Vec<u8>, works: &[Work]) -> ThothResult<()> {
        match works.len() {
            0 => Err(ThothError::IncompleteMetadataRecord(
                "json::thoth".to_string(),
                "Not enough data".to_string(),
            )),
            1 => JsonEntry::<JsonThoth>::json_entry(works.first().unwrap(), w),
            // handler::by_publisher() prevents generation of output for multiple records
            _ => unreachable!(),
        }
    }
}

impl JsonEntry<JsonThoth> for Work {
    fn json_entry(&self, w: &mut Vec<u8>) -> ThothResult<()> {
        w.write_all(
            serde_json::to_string_pretty(self)
                .expect("Could not create JSON")
                .as_bytes(),
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // TODO
}
