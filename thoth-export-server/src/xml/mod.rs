use std::collections::HashMap;
use std::io::Write;
use thoth_client::Work;
use thoth_errors::{ThothError, ThothResult};
use xml::writer::events::StartElementBuilder;
use xml::writer::{EmitterConfig, EventWriter, XmlEvent};

pub(crate) fn write_element_block<W: Write, F: Fn(&mut EventWriter<W>) -> ThothResult<()>>(
    element: &str,
    w: &mut EventWriter<W>,
    f: F,
) -> ThothResult<()> {
    write_full_element_block(element, None, None, w, f)
}

pub(crate) fn write_full_element_block<W: Write, F: Fn(&mut EventWriter<W>) -> ThothResult<()>>(
    element: &str,
    ns: Option<HashMap<String, String>>,
    attr: Option<HashMap<&str, &str>>,
    w: &mut EventWriter<W>,
    f: F,
) -> ThothResult<()> {
    let mut event_builder: StartElementBuilder = XmlEvent::start_element(element);

    if let Some(ns) = ns {
        for (k, v) in ns.iter() {
            event_builder = event_builder.ns(k, v);
        }
    }

    if let Some(attr) = attr {
        for (k, v) in attr.iter() {
            event_builder = event_builder.attr(*k, *v);
        }
    }

    let mut event: XmlEvent = event_builder.into();
    w.write(event)?;
    f(w)?;
    event = XmlEvent::end_element().into();
    w.write(event).map_err(|e| e.into())
}

pub(crate) trait XmlSpecification {
    fn generate(&self, works: &[Work]) -> ThothResult<String> {
        let mut buffer = Vec::new();
        let mut writer = EmitterConfig::new()
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

mod onix3_project_muse;
pub(crate) use onix3_project_muse::Onix3ProjectMuse;
