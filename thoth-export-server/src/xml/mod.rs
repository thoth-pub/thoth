use crate::record::XML_DECLARATION;
use std::io::Write;
use thoth_client::Work;
use thoth_errors::{ThothError, ThothResult};
use xml::writer::events::StartElementBuilder;
use xml::writer::{EmitterConfig, EventWriter, XmlEvent};

const ONIX3_NS: &[(&str, &str)] = &[
    ("release", "3.0"),
    ("xmlns", "http://ns.editeur.org/onix/3.0/reference"),
];

fn write_element_block<W: Write, F: Fn(&mut EventWriter<W>) -> ThothResult<()>>(
    element: &str,
    w: &mut EventWriter<W>,
    f: F,
) -> ThothResult<()> {
    write_full_element_block(element, None, w, f)
}

fn write_full_element_block<W: Write, F: Fn(&mut EventWriter<W>) -> ThothResult<()>>(
    element: &str,
    attr: Option<Vec<(&str, &str)>>,
    w: &mut EventWriter<W>,
    f: F,
) -> ThothResult<()> {
    let mut event_builder: StartElementBuilder = XmlEvent::start_element(element);

    if let Some(attr) = attr {
        for &(k, v) in attr.iter() {
            event_builder = event_builder.attr(k, v);
        }
    }

    let mut event: XmlEvent = event_builder.into();
    w.write(event)?;
    f(w)?;
    event = XmlEvent::end_element().into();
    w.write(event).map_err(|e| e.into())
}

pub(crate) trait XmlSpecification {
    fn generate(&self, works: &[Work], doctype: Option<&str>) -> ThothResult<String> {
        let mut buffer = format!("{}{}", XML_DECLARATION, doctype.unwrap_or_default())
            .as_bytes()
            .to_vec();
        let mut writer = EmitterConfig::new()
            .write_document_declaration(false)
            .perform_indent(true)
            .create_writer(&mut buffer);
        Self::handle_event(&mut writer, works)
            .map(|_| buffer)
            .and_then(|onix| {
                String::from_utf8(onix)
                    .map_err(|_| ThothError::InternalError("Could not parse XML".to_string()))
            })
    }

    fn handle_event<W: Write>(w: &mut EventWriter<W>, works: &[Work]) -> ThothResult<()>;
}

pub(crate) trait XmlElement<T: XmlSpecification> {
    const ELEMENT: &'static str = "";

    fn value(&self) -> &'static str;

    fn xml_element<W: Write>(&self, w: &mut EventWriter<W>) -> ThothResult<()> {
        write_element_block(Self::ELEMENT, w, |w| {
            w.write(XmlEvent::Characters(self.value()))
                .map_err(|e| e.into())
        })
    }
}

pub(crate) trait XmlElementBlock<T: XmlSpecification> {
    fn xml_element<W: Write>(&self, w: &mut EventWriter<W>) -> ThothResult<()>;
}

mod onix3_thoth;
pub(crate) use onix3_thoth::Onix3Thoth;
mod onix3_project_muse;
pub(crate) use onix3_project_muse::Onix3ProjectMuse;
mod onix3_oapen;
pub(crate) use onix3_oapen::Onix3Oapen;
mod onix3_jstor;
pub(crate) use onix3_jstor::Onix3Jstor;
mod onix3_google_books;
pub(crate) use onix3_google_books::Onix3GoogleBooks;
mod onix3_overdrive;
pub(crate) use onix3_overdrive::Onix3Overdrive;
mod onix21_ebsco_host;
pub(crate) use onix21_ebsco_host::Onix21EbscoHost;
mod doideposit_crossref;
pub(crate) use doideposit_crossref::DoiDepositCrossref;
mod marc21xml_thoth;
pub(crate) use marc21xml_thoth::Marc21XmlThoth;
mod onix21_proquest_ebrary;
pub(crate) use onix21_proquest_ebrary::Onix21ProquestEbrary;
