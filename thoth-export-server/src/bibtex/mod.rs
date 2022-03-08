use thoth_client::Work;
use thoth_errors::{ThothError, ThothResult};

pub(crate) trait BibtexSpecification {
    fn generate(&self, works: &[Work]) -> ThothResult<String> {
        let mut buffer: Vec<u8> = Vec::new();
        Self::handle_event(&mut buffer, works)
            .map(|_| buffer)
            .and_then(|bibtex| {
                String::from_utf8(bibtex)
                    .map_err(|_| ThothError::InternalError("Could not parse BibTeX".to_string()))
            })
    }

    fn handle_event(w: &mut Vec<u8>, works: &[Work]) -> ThothResult<()>;
}

pub(crate) trait BibtexEntry<T: BibtexSpecification> {
    fn bibtex_entry(&self, w: &mut Vec<u8>) -> ThothResult<()>;
}

mod bibtex_crossref;
pub(crate) use bibtex_crossref::BibtexCrossref;
