use thoth_client::Work;
use thoth_errors::ThothResult;

pub(crate) trait JsonSpecification {
    fn generate(&self, works: &[Work]) -> ThothResult<String> {
        Self::handle_event(works)
    }

    fn handle_event(works: &[Work]) -> ThothResult<String>;
}

mod json_thoth;
pub(crate) use json_thoth::JsonThoth;
