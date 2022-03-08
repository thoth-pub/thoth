use std::io::Write;
use thoth_client::Work;
use thoth_errors::{ThothError, ThothResult};

pub struct BibtexWriter<W: Write> {
    writer: W,
}

pub(crate) trait BibtexSpecification {
    fn generate(&self, works: &[Work]) -> ThothResult<String> {
        let mut writer = BibtexWriter { writer: vec![] };
        Self::handle_event(&mut writer, works)
            .map(|_| writer)
            .and_then(|bibtex| {
                String::from_utf8(bibtex.writer)
                    .map_err(|_| ThothError::InternalError("Could not parse BibTeX".to_string()))
            })
    }

    fn handle_event<W: Write>(w: &mut BibtexWriter<W>, works: &[Work]) -> ThothResult<()>;
}

pub(crate) trait BibtexEntry<T: BibtexSpecification> {
    fn bibtex_entry<W: Write>(&self, w: &mut BibtexWriter<W>) -> ThothResult<()>;
}

mod bibtex_crossref;
pub(crate) use bibtex_crossref::BibtexCrossref;
