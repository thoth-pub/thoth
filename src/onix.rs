use std::collections::HashMap;
use std::io::Write;

use chrono::prelude::*;
use xml::writer::events::StartElementBuilder;
use xml::writer::{EmitterConfig, EventWriter, Result, XmlEvent};

use crate::client::work_query::ContributionType;
use crate::client::work_query::LanguageRelation;
use crate::client::work_query::PublicationType;
use crate::client::work_query::SubjectType;
use crate::client::work_query::WorkQueryWork;
use crate::client::work_query::WorkStatus;
use crate::errors;

pub fn generate_onix_3(mut work: WorkQueryWork) -> errors::Result<Vec<u8>> {
    println!("{:#?}", work);

    let mut buffer = Vec::new();
    let mut writer = EmitterConfig::new()
        .perform_indent(true)
        .create_writer(&mut buffer);
    match handle_event(&mut writer, &mut work) {
        Ok(_) => Ok(buffer),
        Err(e) => Err(errors::ThothError::from(e).into()),
    }
}

fn string_to_static_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

fn stype_to_scheme(subject_type: &SubjectType) -> &str {
    match subject_type {
        SubjectType::BIC => "12",
        SubjectType::BISAC => "10",
        SubjectType::KEYWORD => "20",
        SubjectType::LCC => "04",
        SubjectType::THEMA => "93",
        SubjectType::CUSTOM => "B2", // B2 Keywords (not for display)
        _ => unreachable!(),
    }
}

fn langrel_to_role(language_relation: &LanguageRelation) -> &str {
    match language_relation {
        LanguageRelation::ORIGINAL => "01",        // Language of text
        LanguageRelation::TRANSLATED_FROM => "02", // Original language of a translated text
        LanguageRelation::TRANSLATED_INTO => "01",
        _ => unreachable!(),
    }
}

fn contribution_type_to_role(contribution_type: &ContributionType) -> &str {
    match contribution_type {
        ContributionType::AUTHOR => "A01",          // By (author)
        ContributionType::EDITOR => "B01",          // Edited by
        ContributionType::TRANSLATOR => "B06",      // Translated by
        ContributionType::PHOTOGRAPHER => "A13",    // Photographs by
        ContributionType::ILUSTRATOR => "A12",      // Illustrated by
        ContributionType::MUSIC_EDITOR => "B25",    // Arranged by (music)
        ContributionType::FOREWORD_BY => "A23",     // Foreword by
        ContributionType::INTRODUCTION_BY => "A24", // Introduction by
        ContributionType::AFTERWORD_BY => "A19",    // Afterword by
        ContributionType::PREFACE_BY => "A15",      // Preface by
        _ => unreachable!(),
    }
}

fn wstatus_to_status(work_status: &WorkStatus) -> &str {
    match work_status {
        WorkStatus::UNSPECIFIED => "00",
        WorkStatus::CANCELLED => "01",
        WorkStatus::FORTHCOMING => "02",
        WorkStatus::POSTPONED_INDEFINITELY => "03",
        WorkStatus::ACTIVE => "04",
        WorkStatus::NO_LONGER_OUR_PRODUCT => "05",
        WorkStatus::OUT_OF_STOCK_INDEFINITELY => "06",
        WorkStatus::OUT_OF_PRINT => "07",
        WorkStatus::INACTIVE => "08",
        WorkStatus::UNKNOWN => "09",
        WorkStatus::REMAINDERED => "10",
        WorkStatus::WITHDRAWN_FROM_SALE => "11",
        WorkStatus::RECALLED => "15",
        _ => unreachable!(),
    }
}

fn write_element_block<W: Write, F: Fn(&mut EventWriter<W>)>(
    element: &str,
    ns: Option<HashMap<String, String>>,
    attr: Option<HashMap<String, String>>,
    w: &mut EventWriter<W>,
    f: F,
) -> Result<()> {
    let mut event_builder: StartElementBuilder = XmlEvent::start_element(element);

    if let Some(ns) = ns {
        for (k, v) in ns.iter() {
            event_builder = event_builder.ns(
                string_to_static_str(k.clone()),
                string_to_static_str(v.clone()),
            );
        }
    }

    if let Some(attr) = attr {
        for (k, v) in attr.iter() {
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
    w.write(event)
}

fn handle_event<W: Write>(w: &mut EventWriter<W>, work: &mut WorkQueryWork) -> Result<()> {
    let ns_map: HashMap<String, String> = HashMap::new();
    let mut attr_map: HashMap<String, String> = HashMap::new();

    attr_map.insert(
        "xmlns".to_string(),
        "http://ns.editeur.org/onix/3.0/reference".to_string(),
    );
    attr_map.insert("release".to_string(), "3.0".to_string());

    let work_id = format!("urn:uuid:{}", &work.work_id.to_string());
    let mut isbn = "".to_string();
    let mut pdf_url = "".to_string();
    for publication in &work.publications {
        if publication.publication_type.eq(&PublicationType::PDF) {
            isbn = match &publication.isbn.as_ref() {
                Some(isbn) => isbn.replace("-", ""),
                None => "".to_string(),
            };
            pdf_url = match &publication.publication_url {
                Some(pdf_url) => pdf_url.to_string(),
                None => "".to_string(),
            };
            break;
        }
    }

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
                    let event: XmlEvent = XmlEvent::Characters("javi@openbookpublishers.com");
                    w.write(event).ok();
                })
                .ok();
            })
            .ok();
            write_element_block("SentDateTime", None, None, w, |w| {
                let utc = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
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
                    let event: XmlEvent = XmlEvent::Characters(&isbn);
                    w.write(event).ok();
                })
                .ok();
            })
            .ok();
            if let Some(doi) = &work.doi.as_ref() {
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
                        if let Some(subtitle) = &work.subtitle.as_ref() {
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
                for contribution in &work.contributions {
                    write_element_block("Contributor", None, None, w, |w| {
                        write_element_block("SequenceNumber", None, None, w, |w| {
                            let event: XmlEvent = XmlEvent::Characters("");
                            w.write(event).ok();
                        })
                        .ok();
                        write_element_block("ContributorRole", None, None, w, |w| {
                            let role = contribution_type_to_role(&contribution.contribution_type);
                            let event: XmlEvent = XmlEvent::Characters(role);
                            w.write(event).ok();
                        })
                        .ok();
                        if let Some(orcid) = &contribution.contributor.orcid.as_ref() {
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
                        if let Some(first_name) = &contribution.contributor.first_name.as_ref() {
                            write_element_block("NamesBeforeKey", None, None, w, |w| {
                                let event: XmlEvent = XmlEvent::Characters(&first_name);
                                w.write(event).ok();
                            })
                            .ok();
                            write_element_block("KeyNames", None, None, w, |w| {
                                let event: XmlEvent =
                                    XmlEvent::Characters(&contribution.contributor.last_name);
                                w.write(event).ok();
                            })
                            .ok();
                        } else {
                            write_element_block("PersonName", None, None, w, |w| {
                                let event: XmlEvent =
                                    XmlEvent::Characters(&contribution.contributor.full_name);
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
                            let role = langrel_to_role(&language.language_relation);
                            let event: XmlEvent = XmlEvent::Characters(role);
                            w.write(event).ok();
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
                if let Some(page_count) = &work.page_count.as_ref() {
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
                            let scheme = stype_to_scheme(&subject.subject_type);
                            let event: XmlEvent = XmlEvent::Characters(scheme);
                            w.write(event).ok();
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
                        let mut lang_fmt: HashMap<String, String> = HashMap::new();
                        lang_fmt.insert("language".to_string(), "eng".to_string());
                        write_element_block("TextContent", None, None, w, |w| {
                            // 30 Abstract
                            write_element_block("TextType", None, None, w, |w| {
                                let event: XmlEvent = XmlEvent::Characters("30");
                                w.write(event).ok();
                            })
                            .ok();
                            // 00 Unrestricted
                            write_element_block("ContentAudience", None, None, w, |w| {
                                let event: XmlEvent = XmlEvent::Characters("00");
                                w.write(event).ok();
                            })
                            .ok();
                            write_element_block("Text", None, Some(lang_fmt.to_owned()), w, |w| {
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
                write_element_block("PublishingStatus", None, None, w, |w| {
                    let status = wstatus_to_status(&work.work_status);
                    let event: XmlEvent = XmlEvent::Characters(status);
                    w.write(event).ok();
                })
                .ok();
                if let Some(date) = &work.publication_date.as_ref() {
                    let mut date_fmt: HashMap<String, String> = HashMap::new();
                    date_fmt.insert(
                        "dateformat".to_string(),
                        "01".to_string(), // 01 YYYYMM
                    );
                    write_element_block("PublishingDate", None, None, w, |w| {
                        // 19 Publication date of print counterpart
                        write_element_block("PublishingDateRole", None, None, w, |w| {
                            let event: XmlEvent = XmlEvent::Characters("19");
                            w.write(event).ok();
                        })
                        .ok();
                        // dateformat="01" YYYYMM
                        write_element_block("Date", None, Some(date_fmt.to_owned()), w, |w| {
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
            write_element_block("RelatedMaterial", None, None, w, |w| {
                for publication in &work.publications {
                    if !publication.publication_type.eq(&PublicationType::PDF) {
                        if let Some(isbn) = &publication.isbn.as_ref() {
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
                                        let nohyphen_isbn = isbn.replace("-", "");
                                        let event: XmlEvent = XmlEvent::Characters(&nohyphen_isbn);
                                        w.write(event).ok();
                                    })
                                    .ok();
                                })
                                .ok();
                            })
                            .ok();
                        }
                    }
                }
            })
            .ok();
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
