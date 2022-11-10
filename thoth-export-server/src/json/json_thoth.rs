use thoth_client::Work;
use thoth_errors::{ThothError, ThothResult};

use super::JsonSpecification;

#[derive(Copy, Clone)]
pub(crate) struct JsonThoth;

impl JsonSpecification for JsonThoth {
    fn handle_event(works: &[Work]) -> ThothResult<String> {
        match works.len() {
            0 => Err(ThothError::IncompleteMetadataRecord(
                "json::thoth".to_string(),
                "Not enough data".to_string(),
            )),
            1 => serde_json::to_string_pretty(works.first().unwrap())
                .map_err(|e| ThothError::InternalError(e.to_string())),
            // handler::by_publisher() prevents generation of output for multiple records
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    // TODO
}
