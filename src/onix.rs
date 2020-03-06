use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

use chrono::prelude::*;
use xml::writer::events::StartElementBuilder;
use xml::writer::{EmitterConfig, EventWriter, Result, XmlEvent};

use crate::client::work_query::WorkQueryWork;
use crate::errors;

pub fn generate_onix_3(mut work: WorkQueryWork) -> errors::Result<()> {
    println!("{:#?}", work);

    let mut file = File::create("output.xml").unwrap();
    let mut writer = EmitterConfig::new()
        .perform_indent(true)
        .create_writer(&mut file);
    match handle_event(&mut writer, &mut work) {
        Ok(_) => Ok(()),
        Err(e) => Err(errors::ThothError::from(e).into()),
    }
}

fn string_to_static_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

fn write_element_block<W: Write, F: Fn(&mut EventWriter<W>)>(
    element: &str,
    ns: Option<HashMap<String, String>>,
    attr: Option<HashMap<String, String>>,
    w: &mut EventWriter<W>,
    f: F,
) -> Result<()> {
    let mut event_builder: StartElementBuilder = XmlEvent::start_element(element);

    if ns.is_some() {
        for (k, v) in ns.unwrap().iter() {
            event_builder = event_builder.ns(
                string_to_static_str(k.clone()),
                string_to_static_str(v.clone()),
            );
        }
    }

    if attr.is_some() {
        for (k, v) in attr.unwrap().iter() {
            event_builder = event_builder.attr(
                string_to_static_str(k.clone()),
                string_to_static_str(v.clone()),
            );
        }
    }

    let mut event: XmlEvent = event_builder.into();
    w.write(event)?;
    f(w);
    event = XmlEvent::end_element().into();
    return w.write(event);
}

fn handle_event<W: Write>(w: &mut EventWriter<W>, work: &mut WorkQueryWork) -> Result<()> {
    let mut ns_map: HashMap<String, String> = HashMap::new();
    let mut attr_map: HashMap<String, String> = HashMap::new();

    ns_map.insert("dpc".to_string(), "https://uba.uva.nl/dpc".to_string());

    attr_map.insert("xmlns".to_string(), "http://ns.editeur.org/onix/3.0/reference".to_string());
    attr_map.insert("release".to_string(), "3.0".to_string());

    let doi = &work.doi.as_ref().unwrap().replace("https://doi.org/", "");
    let work_id = &work.work_id.to_string();
    let subtitle = &work.subtitle.as_ref().unwrap();

    return write_element_block(
        "ONIXMessage",
        Some(ns_map),
        Some(attr_map),
        w,
        |w| {
            write_element_block("Header", None, None, w, |w| {
                write_element_block("Sender", None, None, w, |w| {
                    write_element_block("SenderName", None, None, w, |w| {
                        let event: XmlEvent = XmlEvent::Characters(&work.imprint.publisher.publisher_name).into();
                        w.write(event).ok();
                    }).ok();
                    write_element_block("EmailAddress", None, None, w, |w| {
                        let event: XmlEvent = XmlEvent::Characters("javi@openbookpublishers.com").into();
                        w.write(event).ok();
                    }).ok();
                }).ok();
                write_element_block("SentDateTime", None, None, w, |w| {
                    let utc = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
                    let event: XmlEvent = XmlEvent::Characters(&utc).into();
                    w.write(event).ok();
                }).ok();
            }).ok();

            write_element_block("Product", None, None, w, |w| {
                write_element_block("RecordReference", None, None, w, |w| {
                    let event: XmlEvent = XmlEvent::Characters(work_id).into();
                    w.write(event).ok();
                }).ok();
                // 03 Notification confirmed on publication
                write_element_block("NotificationType", None, None, w, |w| {
                    let event: XmlEvent = XmlEvent::Characters("03").into();
                    w.write(event).ok();
                }).ok();
                // 01 Publisher
                write_element_block("RecordSourceType", None, None, w, |w| {
                    let event: XmlEvent = XmlEvent::Characters("01").into();
                    w.write(event).ok();
                }).ok();
                write_element_block("ProductIdentifier", None, None, w, |w| {
                    // 01 Proprietary
                    write_element_block("ProductIDType", None, None, w, |w| {
                        let event: XmlEvent = XmlEvent::Characters("01").into();
                        w.write(event).ok();
                    }).ok();
                    write_element_block("IDValue", None, None, w, |w| {
                        let event: XmlEvent = XmlEvent::Characters(work_id).into();
                        w.write(event).ok();
                    }).ok();
                }).ok();
                if !doi.is_empty() {
                    write_element_block("ProductIdentifier", None, None, w, |w| {
                        write_element_block("ProductIDType", None, None, w, |w| {
                            let event: XmlEvent = XmlEvent::Characters("06").into();
                            w.write(event).ok();
                        }).ok();
                        write_element_block("IDValue", None, None, w, |w| {
                            let event: XmlEvent = XmlEvent::Characters(doi).into();
                            w.write(event).ok();
                        }).ok();
                    }).ok();
                }
                write_element_block("DescriptiveDetail", None, None, w, |w| {
                    write_element_block("TitleDetail", None, None, w, |w| {
                        // 01 Distinctive title (book)
                        write_element_block("TitleType", None, None, w, |w| {
                            let event: XmlEvent = XmlEvent::Characters("01").into();
                            w.write(event).ok();
                        }).ok();
                        write_element_block("TitleElement", None, None, w, |w| {
                            // 01 Product
                            write_element_block("TitleElementLevel", None, None, w, |w| {
                                let event: XmlEvent = XmlEvent::Characters("01").into();
                                w.write(event).ok();
                            }).ok();
                            if subtitle.is_empty() {
                                write_element_block("TitleText", None, None, w, |w| {
                                    let event: XmlEvent = XmlEvent::Characters(&work.full_title).into();
                                    w.write(event).ok();
                                }).ok();
                            } else {
                                write_element_block("TitleText", None, None, w, |w| {
                                    let event: XmlEvent = XmlEvent::Characters(&work.title).into();
                                    w.write(event).ok();
                                }).ok();
                                write_element_block("Subtitle", None, None, w, |w| {
                                    let event: XmlEvent = XmlEvent::Characters(subtitle).into();
                                    w.write(event).ok();
                                }).ok();
                            }
                        }).ok();
                    }).ok();
                }).ok();
            }).ok();
        },
    );
}
