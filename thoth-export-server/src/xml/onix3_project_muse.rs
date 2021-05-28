use chrono::Utc;
use std::collections::HashMap;
use std::io::Write;
use thoth_api::errors::{ThothError, ThothResult};
use thoth_client::{
    ContributionType, LanguageRelation, PublicationType, SubjectType, Work, WorkPublications,
    WorkStatus,
};
use xml::writer::{EmitterConfig, EventWriter, Result, XmlEvent};

use super::{write_element_block, XmlElement, XmlSpecification};
use crate::xml::write_full_element_block;

pub struct Onix3ProjectMuse {}

impl XmlSpecification for Onix3ProjectMuse {
    fn generate(self, work: Work) -> ThothResult<String> {
        let mut buffer = Vec::new();
        let mut writer = EmitterConfig::new()
            .perform_indent(true)
            .create_writer(&mut buffer);
        Self::handle_event(&mut writer, &work)
            .map(|_| buffer)
            .map_err(|e| e.into())
            .and_then(|onix| {
                String::from_utf8(onix)
                    .map_err(|_| ThothError::InternalError("Could not generate ONIX".to_string()))
            })
    }

    fn handle_event<W: Write>(w: &mut EventWriter<W>, work: &Work) -> Result<()> {
        let mut attr_map: HashMap<&str, &str> = HashMap::new();

        attr_map.insert("release", "3.0");
        attr_map.insert("xmlns", "http://ns.editeur.org/onix/3.0/reference");

        let work_id = format!("urn:uuid:{}", &work.work_id.to_string());
        let (main_isbn, pdf_url, isbns) = get_publications_data(&work.publications);

        write_full_element_block("ONIXMessage", None, Some(attr_map), w, |w| {
            write_element_block("Header", w, |w| {
                write_element_block("Sender", w, |w| {
                    write_element_block("SenderName", w, |w| {
                        let event: XmlEvent =
                            XmlEvent::Characters(&work.imprint.publisher.publisher_name);
                        w.write(event).ok();
                    })
                    .ok();
                    write_element_block("EmailAddress", w, |w| {
                        let event: XmlEvent = XmlEvent::Characters("info@thoth.pub");
                        w.write(event).ok();
                    })
                    .ok();
                })
                .ok();
                write_element_block("SentDateTime", w, |w| {
                    let utc = Utc::now().format("%Y%m%dT%H%M%S").to_string();
                    let event: XmlEvent = XmlEvent::Characters(&utc);
                    w.write(event).ok();
                })
                .ok();
            })
            .ok();

            write_element_block("Product", w, |w| {
                write_element_block("RecordReference", w, |w| {
                    let event: XmlEvent = XmlEvent::Characters(&work_id);
                    w.write(event).ok();
                })
                .ok();
                // 03 Notification confirmed on publication
                write_element_block("NotificationType", w, |w| {
                    let event: XmlEvent = XmlEvent::Characters("03");
                    w.write(event).ok();
                })
                .ok();
                // 01 Publisher
                write_element_block("RecordSourceType", w, |w| {
                    let event: XmlEvent = XmlEvent::Characters("01");
                    w.write(event).ok();
                })
                .ok();
                write_element_block("ProductIdentifier", w, |w| {
                    // 01 Proprietary
                    write_element_block("ProductIDType", w, |w| {
                        let event: XmlEvent = XmlEvent::Characters("01");
                        w.write(event).ok();
                    })
                    .ok();
                    write_element_block("IDValue", w, |w| {
                        let event: XmlEvent = XmlEvent::Characters(&work_id);
                        w.write(event).ok();
                    })
                    .ok();
                })
                .ok();
                write_element_block("ProductIdentifier", w, |w| {
                    // 15 ISBN-13
                    write_element_block("ProductIDType", w, |w| {
                        let event: XmlEvent = XmlEvent::Characters("15");
                        w.write(event).ok();
                    })
                    .ok();
                    write_element_block("IDValue", w, |w| {
                        let event: XmlEvent = XmlEvent::Characters(&main_isbn);
                        w.write(event).ok();
                    })
                    .ok();
                })
                .ok();
                if let Some(doi) = &work.doi {
                    write_element_block("ProductIdentifier", w, |w| {
                        write_element_block("ProductIDType", w, |w| {
                            let event: XmlEvent = XmlEvent::Characters("06");
                            w.write(event).ok();
                        })
                        .ok();
                        write_element_block("IDValue", w, |w| {
                            let sanitised_doi = doi.replace("https://doi.org/", "");
                            let event: XmlEvent = XmlEvent::Characters(&sanitised_doi);
                            w.write(event).ok();
                        })
                        .ok();
                    })
                    .ok();
                }
                write_element_block("DescriptiveDetail", w, |w| {
                    // 00 Single-component retail product
                    write_element_block("ProductComposition", w, |w| {
                        let event: XmlEvent = XmlEvent::Characters("00");
                        w.write(event).ok();
                    })
                    .ok();
                    // EB Digital download and online
                    write_element_block("ProductForm", w, |w| {
                        let event: XmlEvent = XmlEvent::Characters("EB");
                        w.write(event).ok();
                    })
                    .ok();
                    // E107 PDF
                    write_element_block("ProductFormDetail", w, |w| {
                        let event: XmlEvent = XmlEvent::Characters("E107");
                        w.write(event).ok();
                    })
                    .ok();
                    // 10 Text (eye-readable)
                    write_element_block("PrimaryContentType", w, |w| {
                        let event: XmlEvent = XmlEvent::Characters("10");
                        w.write(event).ok();
                    })
                    .ok();
                    if let Some(license) = &work.license {
                        write_element_block("EpubLicense", w, |w| {
                            write_element_block("EpubLicenseName", w, |w| {
                                let event: XmlEvent =
                                    XmlEvent::Characters("Creative Commons License");
                                w.write(event).ok();
                            })
                            .ok();
                            write_element_block("EpubLicenseExpression", w, |w| {
                                write_element_block("EpubLicenseExpressionType", w, |w| {
                                    let event: XmlEvent = XmlEvent::Characters("02");
                                    w.write(event).ok();
                                })
                                .ok();
                                write_element_block("EpubLicenseExpressionLink", w, |w| {
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
                    write_element_block("TitleDetail", w, |w| {
                        // 01 Distinctive title (book)
                        write_element_block("TitleType", w, |w| {
                            let event: XmlEvent = XmlEvent::Characters("01");
                            w.write(event).ok();
                        })
                        .ok();
                        write_element_block("TitleElement", w, |w| {
                            // 01 Product
                            write_element_block("TitleElementLevel", w, |w| {
                                let event: XmlEvent = XmlEvent::Characters("01");
                                w.write(event).ok();
                            })
                            .ok();
                            if let Some(subtitle) = &work.subtitle {
                                write_element_block("TitleText", w, |w| {
                                    let event: XmlEvent = XmlEvent::Characters(&work.title);
                                    w.write(event).ok();
                                })
                                .ok();
                                write_element_block("Subtitle", w, |w| {
                                    let event: XmlEvent = XmlEvent::Characters(&subtitle);
                                    w.write(event).ok();
                                })
                                .ok();
                            } else {
                                write_element_block("TitleText", w, |w| {
                                    let event: XmlEvent = XmlEvent::Characters(&work.full_title);
                                    w.write(event).ok();
                                })
                                .ok();
                            }
                        })
                        .ok();
                    })
                    .ok();
                    for (mut sequence_number, contribution) in work.contributions.iter().enumerate()
                    {
                        sequence_number += 1;
                        write_element_block("Contributor", w, |w| {
                            write_element_block("SequenceNumber", w, |w| {
                                let seq = &sequence_number.to_string();
                                let event: XmlEvent = XmlEvent::Characters(seq);
                                w.write(event).ok();
                            })
                            .ok();
                            XmlElement::<Self>::xml_element(&contribution.contribution_type, w)
                                .ok();

                            if let Some(orcid) = &contribution.contributor.orcid {
                                write_element_block("NameIdentifier", w, |w| {
                                    write_element_block("NameIDType", w, |w| {
                                        let event: XmlEvent = XmlEvent::Characters("21");
                                        w.write(event).ok();
                                    })
                                    .ok();
                                    write_element_block("IDValue", w, |w| {
                                        let event: XmlEvent = XmlEvent::Characters(&orcid);
                                        w.write(event).ok();
                                    })
                                    .ok();
                                })
                                .ok();
                            }
                            if let Some(first_name) = &contribution.first_name {
                                write_element_block("NamesBeforeKey", w, |w| {
                                    let event: XmlEvent = XmlEvent::Characters(&first_name);
                                    w.write(event).ok();
                                })
                                .ok();
                                write_element_block("KeyNames", w, |w| {
                                    let event: XmlEvent =
                                        XmlEvent::Characters(&contribution.last_name);
                                    w.write(event).ok();
                                })
                                .ok();
                            } else {
                                write_element_block("PersonName", w, |w| {
                                    let event: XmlEvent =
                                        XmlEvent::Characters(&contribution.full_name);
                                    w.write(event).ok();
                                })
                                .ok();
                            }
                        })
                        .ok();
                    }
                    for language in &work.languages {
                        write_element_block("Language", w, |w| {
                            XmlElement::<Self>::xml_element(&language.language_relation, w).ok();
                            write_element_block("LanguageCode", w, |w| {
                                let code = &language.language_code.to_string().to_lowercase();
                                let event: XmlEvent = XmlEvent::Characters(&code);
                                w.write(event).ok();
                            })
                            .ok();
                        })
                        .ok();
                    }
                    if let Some(page_count) = &work.page_count {
                        write_element_block("Extent", w, |w| {
                            // 00 Main content
                            write_element_block("ExtentType", w, |w| {
                                let event: XmlEvent = XmlEvent::Characters("00");
                                w.write(event).ok();
                            })
                            .ok();
                            write_element_block("ExtentValue", w, |w| {
                                let pcount = page_count.to_string();
                                let event: XmlEvent = XmlEvent::Characters(&pcount);
                                w.write(event).ok();
                            })
                            .ok();
                            // 03 Pages
                            write_element_block("ExtentUnit", w, |w| {
                                let event: XmlEvent = XmlEvent::Characters("03");
                                w.write(event).ok();
                            })
                            .ok();
                        })
                        .ok();
                    }
                    for subject in &work.subjects {
                        write_element_block("Subject", w, |w| {
                            XmlElement::<Self>::xml_element(&subject.subject_type, w).ok();
                            write_element_block("SubjectCode", w, |w| {
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
                    write_element_block("CollateralDetail", w, |w| {
                        if let Some(labstract) = &work.long_abstract {
                            write_element_block("TextContent", w, |w| {
                                let mut lang_fmt: HashMap<&str, &str> = HashMap::new();
                                lang_fmt.insert("language", "eng");
                                // 03 Description ("30 Abstract" not implemented in OAPEN)
                                write_element_block("TextType", w, |w| {
                                    let event: XmlEvent = XmlEvent::Characters("03");
                                    w.write(event).ok();
                                })
                                .ok();
                                // 00 Unrestricted
                                write_element_block("ContentAudience", w, |w| {
                                    let event: XmlEvent = XmlEvent::Characters("00");
                                    w.write(event).ok();
                                })
                                .ok();
                                write_full_element_block("Text", None, Some(lang_fmt), w, |w| {
                                    let event: XmlEvent = XmlEvent::Characters(&labstract);
                                    w.write(event).ok();
                                })
                                .ok();
                            })
                            .ok();
                        }
                        if let Some(toc) = &work.toc {
                            write_element_block("TextContent", w, |w| {
                                // 04 Table of contents
                                write_element_block("TextType", w, |w| {
                                    let event: XmlEvent = XmlEvent::Characters("04");
                                    w.write(event).ok();
                                })
                                .ok();
                                // 00 Unrestricted
                                write_element_block("ContentAudience", w, |w| {
                                    let event: XmlEvent = XmlEvent::Characters("00");
                                    w.write(event).ok();
                                })
                                .ok();
                                write_element_block("Text", w, |w| {
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
                write_element_block("PublishingDetail", w, |w| {
                    write_element_block("Imprint", w, |w| {
                        write_element_block("ImprintName", w, |w| {
                            let event: XmlEvent = XmlEvent::Characters(&work.imprint.imprint_name);
                            w.write(event).ok();
                        })
                        .ok();
                    })
                    .ok();
                    write_element_block("Publisher", w, |w| {
                        // 01 Publisher
                        write_element_block("PublishingRole", w, |w| {
                            let event: XmlEvent = XmlEvent::Characters("01");
                            w.write(event).ok();
                        })
                        .ok();
                        write_element_block("PublisherName", w, |w| {
                            let event: XmlEvent =
                                XmlEvent::Characters(&work.imprint.publisher.publisher_name);
                            w.write(event).ok();
                        })
                        .ok();
                    })
                    .ok();
                    if let Some(place) = &work.place {
                        write_element_block("CityOfPublication", w, |w| {
                            let event: XmlEvent = XmlEvent::Characters(&place);
                            w.write(event).ok();
                        })
                        .ok();
                    }
                    XmlElement::<Self>::xml_element(&work.work_status, w).ok();
                    if let Some(date) = &work.publication_date {
                        write_element_block("PublishingDate", w, |w| {
                            let mut date_fmt: HashMap<&str, &str> = HashMap::new();
                            date_fmt.insert(
                                "dateformat",
                                "01", // 01 YYYYMM
                            );
                            // 19 Publication date of print counterpart
                            write_element_block("PublishingDateRole", w, |w| {
                                let event: XmlEvent = XmlEvent::Characters("19");
                                w.write(event).ok();
                            })
                            .ok();
                            // dateformat="01" YYYYMM
                            write_full_element_block("Date", None, Some(date_fmt), w, |w| {
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
                    write_element_block("RelatedMaterial", w, |w| {
                        for isbn in &isbns {
                            write_element_block("RelatedProduct", w, |w| {
                                // 06 Alternative format
                                write_element_block("ProductRelationCode", w, |w| {
                                    let event: XmlEvent = XmlEvent::Characters("06");
                                    w.write(event).ok();
                                })
                                .ok();
                                write_element_block("ProductIdentifier", w, |w| {
                                    // 06 ISBN
                                    write_element_block("ProductIDType", w, |w| {
                                        let event: XmlEvent = XmlEvent::Characters("06");
                                        w.write(event).ok();
                                    })
                                    .ok();
                                    write_element_block("IDValue", w, |w| {
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
                write_element_block("ProductSupply", w, |w| {
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
                        write_element_block("SupplyDetail", w, |w| {
                            write_element_block("Supplier", w, |w| {
                                // 09 Publisher to end-customers
                                write_element_block("SupplierRole", w, |w| {
                                    let event: XmlEvent = XmlEvent::Characters("11");
                                    w.write(event).ok();
                                })
                                .ok();
                                write_element_block("SupplierName", w, |w| {
                                    let event: XmlEvent = XmlEvent::Characters(
                                        &work.imprint.publisher.publisher_name,
                                    );
                                    w.write(event).ok();
                                })
                                .ok();
                                write_element_block("Website", w, |w| {
                                    // 01 Publisher’s corporate website
                                    write_element_block("WebsiteRole", w, |w| {
                                        let event: XmlEvent = XmlEvent::Characters("01");
                                        w.write(event).ok();
                                    })
                                    .ok();
                                    write_element_block("WebsiteDescription", w, |w| {
                                        let event: XmlEvent = XmlEvent::Characters(&description);
                                        w.write(event).ok();
                                    })
                                    .ok();
                                    write_element_block("WebsiteLink", w, |w| {
                                        let event: XmlEvent = XmlEvent::Characters(&url);
                                        w.write(event).ok();
                                    })
                                    .ok();
                                })
                                .ok();
                            })
                            .ok();
                            // 99 Contact supplier
                            write_element_block("ProductAvailability", w, |w| {
                                let event: XmlEvent = XmlEvent::Characters("99");
                                w.write(event).ok();
                            })
                            .ok();
                            // 04 Contact supplier
                            write_element_block("UnpricedItemType", w, |w| {
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

impl XmlElement<Onix3ProjectMuse> for WorkStatus {
    const ELEMENT: &'static str = "PublishingStatus";

    fn value(&self) -> &'static str {
        match self {
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
            WorkStatus::Other(_) => unreachable!(),
        }
    }
}

impl XmlElement<Onix3ProjectMuse> for SubjectType {
    const ELEMENT: &'static str = "SubjectSchemeIdentifier";

    fn value(&self) -> &'static str {
        match self {
            SubjectType::BIC => "12",
            SubjectType::BISAC => "10",
            SubjectType::KEYWORD => "20",
            SubjectType::LCC => "04",
            SubjectType::THEMA => "93",
            SubjectType::CUSTOM => "B2",
            SubjectType::Other(_) => unreachable!(),
        }
    }
}

impl XmlElement<Onix3ProjectMuse> for LanguageRelation {
    const ELEMENT: &'static str = "LanguageRole";

    fn value(&self) -> &'static str {
        match self {
            LanguageRelation::ORIGINAL => "01",
            LanguageRelation::TRANSLATED_FROM => "02",
            LanguageRelation::TRANSLATED_INTO => "01",
            LanguageRelation::Other(_) => unreachable!(),
        }
    }
}

impl XmlElement<Onix3ProjectMuse> for ContributionType {
    const ELEMENT: &'static str = "ContributorRole";

    fn value(&self) -> &'static str {
        match self {
            ContributionType::AUTHOR => "A01",
            ContributionType::EDITOR => "B01",
            ContributionType::TRANSLATOR => "B06",
            ContributionType::PHOTOGRAPHER => "A13",
            ContributionType::ILUSTRATOR => "A12",
            ContributionType::MUSIC_EDITOR => "B25",
            ContributionType::FOREWORD_BY => "A23",
            ContributionType::INTRODUCTION_BY => "A24",
            ContributionType::AFTERWORD_BY => "A19",
            ContributionType::PREFACE_BY => "A15",
            ContributionType::Other(_) => unreachable!(),
        }
    }
}
