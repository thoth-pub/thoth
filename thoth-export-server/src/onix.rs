use std::collections::HashMap;
use std::io::Write;

use chrono::prelude::Utc;
use thoth_api::errors::{ThothError, ThothResult};
use thoth_client::{PublicationType, Work, WorkPublications};
use xml::writer::{events::StartElementBuilder, EmitterConfig, EventWriter, Result, XmlEvent};

use crate::onix_types::{XmlElement, OnixSubjectScheme, OnixLanguageRole, OnixContributorRole};

pub fn generate_onix_3(work: Work) -> ThothResult<String> {
    let mut buffer = Vec::new();
    let mut writer = EmitterConfig::new()
        .perform_indent(true)
        .create_writer(&mut buffer);
    handle_event(&mut writer, &work)
        .map(|_| buffer)
        .map_err(|e| e.into())
        .and_then(|onix| {
            String::from_utf8(onix)
                .map_err(|_| ThothError::InternalError("Could not generate ONIX".to_string()))
        })
}

fn get_publications_data(publications: &[WorkPublications]) -> (String, String, Vec<String>) {
    let mut main_isbn = "".to_string();
    let mut pdf_url = "".to_string();
    let mut isbns: Vec<String> = Vec::new();

    for publication in publications {
        if publication.publication_type.eq(&PublicationType::PDF) {
            pdf_url = publication.publication_url.as_ref().unwrap().to_string();
        }

        if let Some(isbn) = &publication.isbn {
            isbns.push(isbn.replace("-", ""));
            // The default product ISBN is the PDF's
            if publication.publication_type.eq(&PublicationType::PDF) {
                main_isbn = isbn.replace("-", "");
            }
            // Books that don't have a PDF ISBN will use the paperback's
            if publication.publication_type.eq(&PublicationType::PAPERBACK) && main_isbn.is_empty()
            {
                main_isbn = isbn.replace("-", "");
            }
        }
    }

    (main_isbn, pdf_url, isbns)
}

pub(crate) fn write_element_block<W: Write, F: Fn(&mut EventWriter<W>)>(
    element: &str,
    ns: Option<HashMap<&str, &str>>,
    attr: Option<HashMap<&str, &str>>,
    w: &mut EventWriter<W>,
    f: F,
) -> Result<()> {
    let mut event_builder: StartElementBuilder = XmlEvent::start_element(element);

    if let Some(ns) = ns {
        for (k, v) in ns.iter() {
            event_builder = event_builder.ns(k.clone(), v.clone());
        }
    }

    if let Some(attr) = attr {
        for (k, v) in attr.iter() {
            event_builder = event_builder.attr(k.clone(), v.clone());
        }
    }

    let mut event: XmlEvent = event_builder.into();
    w.write(event)?;
    f(w);
    event = XmlEvent::end_element().into();
    w.write(event)
}

fn handle_event<W: Write>(w: &mut EventWriter<W>, work: &Work) -> Result<()> {
    let ns_map: HashMap<&str, &str> = HashMap::new();
    let mut attr_map: HashMap<&str, &str> = HashMap::new();

    attr_map.insert("xmlns", "http://ns.editeur.org/onix/3.0/reference");
    attr_map.insert("release", "3.0");

    let work_id = format!("urn:uuid:{}", &work.work_id.to_string());
    let (main_isbn, pdf_url, isbns) = get_publications_data(&work.publications);

    write_element_block("ONIXMessage", Some(ns_map), Some(attr_map), w, |w| {
        write_element_block("Header", None, None, w, |w| {
            write_element_block("Sender", None, None, w, |w| {
                write_element_block("SenderName", None, None, w, |w| {
                    let event: XmlEvent =
                        XmlEvent::Characters(&work.imprint.publisher.publisher_name);
                    w.write(event).ok();
                })
                .ok();
                write_element_block("EmailAddress", None, None, w, |w| {
                    let event: XmlEvent = XmlEvent::Characters("info@thoth.pub");
                    w.write(event).ok();
                })
                .ok();
            })
            .ok();
            write_element_block("SentDateTime", None, None, w, |w| {
                let utc = Utc::now().format("%Y%m%dT%H%M%S").to_string();
                let event: XmlEvent = XmlEvent::Characters(&utc);
                w.write(event).ok();
            })
            .ok();
        })
        .ok();

        write_element_block("Product", None, None, w, |w| {
            write_element_block("RecordReference", None, None, w, |w| {
                let event: XmlEvent = XmlEvent::Characters(&work_id);
                w.write(event).ok();
            })
            .ok();
            // 03 Notification confirmed on publication
            write_element_block("NotificationType", None, None, w, |w| {
                let event: XmlEvent = XmlEvent::Characters("03");
                w.write(event).ok();
            })
            .ok();
            // 01 Publisher
            write_element_block("RecordSourceType", None, None, w, |w| {
                let event: XmlEvent = XmlEvent::Characters("01");
                w.write(event).ok();
            })
            .ok();
            write_element_block("ProductIdentifier", None, None, w, |w| {
                // 01 Proprietary
                write_element_block("ProductIDType", None, None, w, |w| {
                    let event: XmlEvent = XmlEvent::Characters("01");
                    w.write(event).ok();
                })
                .ok();
                write_element_block("IDValue", None, None, w, |w| {
                    let event: XmlEvent = XmlEvent::Characters(&work_id);
                    w.write(event).ok();
                })
                .ok();
            })
            .ok();
            write_element_block("ProductIdentifier", None, None, w, |w| {
                // 15 ISBN-13
                write_element_block("ProductIDType", None, None, w, |w| {
                    let event: XmlEvent = XmlEvent::Characters("15");
                    w.write(event).ok();
                })
                .ok();
                write_element_block("IDValue", None, None, w, |w| {
                    let event: XmlEvent = XmlEvent::Characters(&main_isbn);
                    w.write(event).ok();
                })
                .ok();
            })
            .ok();
            if let Some(doi) = &work.doi {
                write_element_block("ProductIdentifier", None, None, w, |w| {
                    write_element_block("ProductIDType", None, None, w, |w| {
                        let event: XmlEvent = XmlEvent::Characters("06");
                        w.write(event).ok();
                    })
                    .ok();
                    write_element_block("IDValue", None, None, w, |w| {
                        let sanitised_doi = doi.replace("https://doi.org/", "");
                        let event: XmlEvent = XmlEvent::Characters(&sanitised_doi);
                        w.write(event).ok();
                    })
                    .ok();
                })
                .ok();
            }
            write_element_block("DescriptiveDetail", None, None, w, |w| {
                // 00 Single-component retail product
                write_element_block("ProductComposition", None, None, w, |w| {
                    let event: XmlEvent = XmlEvent::Characters("00");
                    w.write(event).ok();
                })
                .ok();
                // EB Digital download and online
                write_element_block("ProductForm", None, None, w, |w| {
                    let event: XmlEvent = XmlEvent::Characters("EB");
                    w.write(event).ok();
                })
                .ok();
                // E107 PDF
                write_element_block("ProductFormDetail", None, None, w, |w| {
                    let event: XmlEvent = XmlEvent::Characters("E107");
                    w.write(event).ok();
                })
                .ok();
                // 10 Text (eye-readable)
                write_element_block("PrimaryContentType", None, None, w, |w| {
                    let event: XmlEvent = XmlEvent::Characters("10");
                    w.write(event).ok();
                })
                .ok();
                if let Some(license) = &work.license {
                    write_element_block("EpubLicense", None, None, w, |w| {
                        write_element_block("EpubLicenseName", None, None, w, |w| {
                            let event: XmlEvent = XmlEvent::Characters("Creative Commons License");
                            w.write(event).ok();
                        })
                        .ok();
                        write_element_block("EpubLicenseExpression", None, None, w, |w| {
                            write_element_block("EpubLicenseExpressionType", None, None, w, |w| {
                                let event: XmlEvent = XmlEvent::Characters("02");
                                w.write(event).ok();
                            })
                            .ok();
                            write_element_block("EpubLicenseExpressionLink", None, None, w, |w| {
                                let license_url = license.to_string();
                                let event: XmlEvent = XmlEvent::Characters(&license_url);
                                w.write(event).ok();
                            })
                            .ok();
                        })
                        .ok();
                    })
                    .ok();
                }
                write_element_block("TitleDetail", None, None, w, |w| {
                    // 01 Distinctive title (book)
                    write_element_block("TitleType", None, None, w, |w| {
                        let event: XmlEvent = XmlEvent::Characters("01");
                        w.write(event).ok();
                    })
                    .ok();
                    write_element_block("TitleElement", None, None, w, |w| {
                        // 01 Product
                        write_element_block("TitleElementLevel", None, None, w, |w| {
                            let event: XmlEvent = XmlEvent::Characters("01");
                            w.write(event).ok();
                        })
                        .ok();
                        if let Some(subtitle) = &work.subtitle {
                            write_element_block("TitleText", None, None, w, |w| {
                                let event: XmlEvent = XmlEvent::Characters(&work.title);
                                w.write(event).ok();
                            })
                            .ok();
                            write_element_block("Subtitle", None, None, w, |w| {
                                let event: XmlEvent = XmlEvent::Characters(&subtitle);
                                w.write(event).ok();
                            })
                            .ok();
                        } else {
                            write_element_block("TitleText", None, None, w, |w| {
                                let event: XmlEvent = XmlEvent::Characters(&work.full_title);
                                w.write(event).ok();
                            })
                            .ok();
                        }
                    })
                    .ok();
                })
                .ok();
                for (mut sequence_number, contribution) in work.contributions.iter().enumerate() {
                    sequence_number += 1;
                    write_element_block("Contributor", None, None, w, |w| {
                        write_element_block("SequenceNumber", None, None, w, |w| {
                            let seq = &sequence_number.to_string();
                            let event: XmlEvent = XmlEvent::Characters(seq);
                            w.write(event).ok();
                        })
                        .ok();
                        write_element_block("ContributorRole", None, None, w, |w| {
                            w.write(XmlEvent::Characters(
                                OnixContributorRole::from(contribution.contribution_type.clone())
                                    .to_string()
                                    .as_str(),
                            ))
                            .ok();
                        })
                        .ok();
                        if let Some(orcid) = &contribution.contributor.orcid {
                            write_element_block("NameIdentifier", None, None, w, |w| {
                                write_element_block("NameIDType", None, None, w, |w| {
                                    let event: XmlEvent = XmlEvent::Characters("21");
                                    w.write(event).ok();
                                })
                                .ok();
                                write_element_block("IDValue", None, None, w, |w| {
                                    let event: XmlEvent = XmlEvent::Characters(&orcid);
                                    w.write(event).ok();
                                })
                                .ok();
                            })
                            .ok();
                        }
                        if let Some(first_name) = &contribution.first_name {
                            write_element_block("NamesBeforeKey", None, None, w, |w| {
                                let event: XmlEvent = XmlEvent::Characters(&first_name);
                                w.write(event).ok();
                            })
                            .ok();
                            write_element_block("KeyNames", None, None, w, |w| {
                                let event: XmlEvent = XmlEvent::Characters(&contribution.last_name);
                                w.write(event).ok();
                            })
                            .ok();
                        } else {
                            write_element_block("PersonName", None, None, w, |w| {
                                let event: XmlEvent = XmlEvent::Characters(&contribution.full_name);
                                w.write(event).ok();
                            })
                            .ok();
                        }
                    })
                    .ok();
                }
                for language in &work.languages {
                    write_element_block("Language", None, None, w, |w| {
                        write_element_block("LanguageRole", None, None, w, |w| {
                            w.write(XmlEvent::Characters(
                                OnixLanguageRole::from(language.language_relation.clone())
                                    .to_string()
                                    .as_str(),
                            ))
                            .ok();
                        })
                        .ok();
                        write_element_block("LanguageCode", None, None, w, |w| {
                            let code = &language.language_code.to_string().to_lowercase();
                            let event: XmlEvent = XmlEvent::Characters(&code);
                            w.write(event).ok();
                        })
                        .ok();
                    })
                    .ok();
                }
                if let Some(page_count) = &work.page_count {
                    write_element_block("Extent", None, None, w, |w| {
                        // 00 Main content
                        write_element_block("ExtentType", None, None, w, |w| {
                            let event: XmlEvent = XmlEvent::Characters("00");
                            w.write(event).ok();
                        })
                        .ok();
                        write_element_block("ExtentValue", None, None, w, |w| {
                            let pcount = page_count.to_string();
                            let event: XmlEvent = XmlEvent::Characters(&pcount);
                            w.write(event).ok();
                        })
                        .ok();
                        // 03 Pages
                        write_element_block("ExtentUnit", None, None, w, |w| {
                            let event: XmlEvent = XmlEvent::Characters("03");
                            w.write(event).ok();
                        })
                        .ok();
                    })
                    .ok();
                }
                for subject in &work.subjects {
                    write_element_block("Subject", None, None, w, |w| {
                        write_element_block("SubjectSchemeIdentifier", None, None, w, |w| {
                            w.write(XmlEvent::Characters(
                                OnixSubjectScheme::from(subject.subject_type.clone())
                                    .to_string()
                                    .as_str(),
                            ))
                            .ok();
                        })
                        .ok();
                        write_element_block("SubjectCode", None, None, w, |w| {
                            let event: XmlEvent = XmlEvent::Characters(&subject.subject_code);
                            w.write(event).ok();
                        })
                        .ok();
                    })
                    .ok();
                }
            })
            .ok();
            if work.long_abstract.is_some() || work.toc.is_some() {
                write_element_block("CollateralDetail", None, None, w, |w| {
                    if let Some(labstract) = &work.long_abstract {
                        write_element_block("TextContent", None, None, w, |w| {
                            let mut lang_fmt: HashMap<&str, &str> = HashMap::new();
                            lang_fmt.insert("language", "eng");
                            // 03 Description ("30 Abstract" not implemented in OAPEN)
                            write_element_block("TextType", None, None, w, |w| {
                                let event: XmlEvent = XmlEvent::Characters("03");
                                w.write(event).ok();
                            })
                            .ok();
                            // 00 Unrestricted
                            write_element_block("ContentAudience", None, None, w, |w| {
                                let event: XmlEvent = XmlEvent::Characters("00");
                                w.write(event).ok();
                            })
                            .ok();
                            write_element_block("Text", None, Some(lang_fmt), w, |w| {
                                let event: XmlEvent = XmlEvent::Characters(&labstract);
                                w.write(event).ok();
                            })
                            .ok();
                        })
                        .ok();
                    }
                    if let Some(toc) = &work.toc {
                        write_element_block("TextContent", None, None, w, |w| {
                            // 04 Table of contents
                            write_element_block("TextType", None, None, w, |w| {
                                let event: XmlEvent = XmlEvent::Characters("04");
                                w.write(event).ok();
                            })
                            .ok();
                            // 00 Unrestricted
                            write_element_block("ContentAudience", None, None, w, |w| {
                                let event: XmlEvent = XmlEvent::Characters("00");
                                w.write(event).ok();
                            })
                            .ok();
                            write_element_block("Text", None, None, w, |w| {
                                let event: XmlEvent = XmlEvent::Characters(&toc);
                                w.write(event).ok();
                            })
                            .ok();
                        })
                        .ok();
                    }
                })
                .ok();
            }
            write_element_block("PublishingDetail", None, None, w, |w| {
                write_element_block("Imprint", None, None, w, |w| {
                    write_element_block("ImprintName", None, None, w, |w| {
                        let event: XmlEvent = XmlEvent::Characters(&work.imprint.imprint_name);
                        w.write(event).ok();
                    })
                    .ok();
                })
                .ok();
                write_element_block("Publisher", None, None, w, |w| {
                    // 01 Publisher
                    write_element_block("PublishingRole", None, None, w, |w| {
                        let event: XmlEvent = XmlEvent::Characters("01");
                        w.write(event).ok();
                    })
                    .ok();
                    write_element_block("PublisherName", None, None, w, |w| {
                        let event: XmlEvent =
                            XmlEvent::Characters(&work.imprint.publisher.publisher_name);
                        w.write(event).ok();
                    })
                    .ok();
                })
                .ok();
                if let Some(place) = &work.place {
                    write_element_block("CityOfPublication", None, None, w, |w| {
                        let event: XmlEvent = XmlEvent::Characters(&place);
                        w.write(event).ok();
                    })
                    .ok();
                }
                work.work_status.xml_element(w).ok();
                if let Some(date) = &work.publication_date {
                    write_element_block("PublishingDate", None, None, w, |w| {
                        let mut date_fmt: HashMap<&str, &str> = HashMap::new();
                        date_fmt.insert(
                            "dateformat",
                            "01", // 01 YYYYMM
                        );
                        // 19 Publication date of print counterpart
                        write_element_block("PublishingDateRole", None, None, w, |w| {
                            let event: XmlEvent = XmlEvent::Characters("19");
                            w.write(event).ok();
                        })
                        .ok();
                        // dateformat="01" YYYYMM
                        write_element_block("Date", None, Some(date_fmt), w, |w| {
                            let pub_date = date.format("%Y%m").to_string();
                            let event: XmlEvent = XmlEvent::Characters(&pub_date);
                            w.write(event).ok();
                        })
                        .ok();
                    })
                    .ok();
                }
            })
            .ok();
            if !isbns.is_empty() {
                write_element_block("RelatedMaterial", None, None, w, |w| {
                    for isbn in &isbns {
                        write_element_block("RelatedProduct", None, None, w, |w| {
                            // 06 Alternative format
                            write_element_block("ProductRelationCode", None, None, w, |w| {
                                let event: XmlEvent = XmlEvent::Characters("06");
                                w.write(event).ok();
                            })
                            .ok();
                            write_element_block("ProductIdentifier", None, None, w, |w| {
                                // 06 ISBN
                                write_element_block("ProductIDType", None, None, w, |w| {
                                    let event: XmlEvent = XmlEvent::Characters("06");
                                    w.write(event).ok();
                                })
                                .ok();
                                write_element_block("IDValue", None, None, w, |w| {
                                    let event: XmlEvent = XmlEvent::Characters(&isbn);
                                    w.write(event).ok();
                                })
                                .ok();
                            })
                            .ok();
                        })
                        .ok();
                    }
                })
                .ok();
            }
            write_element_block("ProductSupply", None, None, w, |w| {
                let mut supplies: HashMap<String, String> = HashMap::new();
                supplies.insert(
                    pdf_url.to_string(),
                    "Publisher's website: download the title".to_string(),
                );
                if let Some(landing_page) = &work.landing_page {
                    supplies.insert(
                        landing_page.to_string(),
                        "Publisher's website: web shop".to_string(),
                    );
                }
                for (url, description) in supplies.iter() {
                    write_element_block("SupplyDetail", None, None, w, |w| {
                        write_element_block("Supplier", None, None, w, |w| {
                            // 09 Publisher to end-customers
                            write_element_block("SupplierRole", None, None, w, |w| {
                                let event: XmlEvent = XmlEvent::Characters("11");
                                w.write(event).ok();
                            })
                            .ok();
                            write_element_block("SupplierName", None, None, w, |w| {
                                let event: XmlEvent =
                                    XmlEvent::Characters(&work.imprint.publisher.publisher_name);
                                w.write(event).ok();
                            })
                            .ok();
                            write_element_block("Website", None, None, w, |w| {
                                // 01 Publisher’s corporate website
                                write_element_block("WebsiteRole", None, None, w, |w| {
                                    let event: XmlEvent = XmlEvent::Characters("01");
                                    w.write(event).ok();
                                })
                                .ok();
                                write_element_block("WebsiteDescription", None, None, w, |w| {
                                    let event: XmlEvent = XmlEvent::Characters(&description);
                                    w.write(event).ok();
                                })
                                .ok();
                                write_element_block("WebsiteLink", None, None, w, |w| {
                                    let event: XmlEvent = XmlEvent::Characters(&url);
                                    w.write(event).ok();
                                })
                                .ok();
                            })
                            .ok();
                        })
                        .ok();
                        // 99 Contact supplier
                        write_element_block("ProductAvailability", None, None, w, |w| {
                            let event: XmlEvent = XmlEvent::Characters("99");
                            w.write(event).ok();
                        })
                        .ok();
                        // 04 Contact supplier
                        write_element_block("UnpricedItemType", None, None, w, |w| {
                            let event: XmlEvent = XmlEvent::Characters("04");
                            w.write(event).ok();
                        })
                        .ok();
                    })
                    .ok();
                }
            })
            .ok();
        })
        .ok();
    })
}
