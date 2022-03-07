use csv::{QuoteStyle, Writer, WriterBuilder};
use std::io::Write;
use thoth_client::Work;
use thoth_errors::{ThothError, ThothResult};

pub(crate) trait BibtexSpecification {
    fn generate(
        &self,
        works: &[Work],
        quote_style: QuoteStyle,
        delimiter: u8,
    ) -> ThothResult<String> {
        let mut writer = WriterBuilder::new()
            .quote_style(quote_style)
            .delimiter(delimiter)
            .from_writer(Vec::new());
        Self::handle_event(&mut writer, works)
            .map(|_| writer.into_inner().map_err(|e| e.error().into()))
            .and_then(|val| val)
            .and_then(|bibtex| {
                String::from_utf8(bibtex)
                    .map_err(|_| ThothError::InternalError("Could not parse BibTeX".to_string()))
            })
    }

    fn handle_event<W: Write>(w: &mut Writer<W>, works: &[Work]) -> ThothResult<()>;
}

pub(crate) trait BibtexEntry<T: BibtexSpecification> {
    fn bibtex_entry<W: Write>(&self, w: &mut Writer<W>) -> ThothResult<()>;
}

mod bibtex_crossref;
pub(crate) use bibtex_crossref::BibtexCrossref;
