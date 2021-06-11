use chrono::Utc;
use std::collections::HashMap;
use std::io::Write;
use thoth_client::{
    ContributionType, LanguageRelation, PublicationType, SubjectType, Work, WorkContributions,
    WorkLanguages, WorkPublications, WorkStatus,
};
use xml::writer::{EventWriter, Result, XmlEvent};

use super::{write_element_block, XmlElement, XmlSpecification};
use crate::xml::{write_full_element_block, XmlElementBlock};

pub struct Onix3ProjectMuse {}

impl XmlSpecification for Onix3ProjectMuse {
    fn handle_event<W: Write>(w: &mut EventWriter<W>, works: &[Work]) -> Result<()> {
        let mut attr_map: HashMap<&str, &str> = HashMap::new();

        attr_map.insert("release", "3.0");
        attr_map.insert("xmlns", "http://ns.editeur.org/onix/3.0/reference");

        write_full_element_block("ONIXMessage", None, Some(attr_map), w, |w| {
            write_element_block("Header", w, |w| {
                write_element_block("Sender", w, |w| {
                    write_element_block("SenderName", w, |w| {
                        w.write(XmlEvent::Characters("Thoth")).ok();
                    })
                    .ok();
                    write_element_block("EmailAddress", w, |w| {
                        w.write(XmlEvent::Characters("info@thoth.pub")).ok();
                    })
                    .ok();
                })
                .ok();
                write_element_block("SentDateTime", w, |w| {
                    w.write(XmlEvent::Characters(
                        &Utc::now().format("%Y%m%dT%H%M%S").to_string(),
                    ))
                    .ok();
                })
                .ok();
            })
            .ok();

            for work in works.iter() {
                XmlElementBlock::<Onix3ProjectMuse>::xml_element(work, w).ok();
            }
        })
    }
}

impl XmlElementBlock<Onix3ProjectMuse> for Work {
    fn xml_element<W: Write>(&self, w: &mut EventWriter<W>) -> Result<()> {
        let work_id = format!("urn:uuid:{}", self.work_id.to_string());
        let (main_isbn, isbns) = get_publications_data(&self.publications);
        // We can only generate the document if there's a PDF
        if let Some(pdf_url) = self
            .publications
            .iter()
            .find(|p| p.publication_type.eq(&PublicationType::PDF))
            .and_then(|p| p.publication_url.as_ref())
        {
            write_element_block("Product", w, |w| {
                write_element_block("RecordReference", w, |w| {
                    w.write(XmlEvent::Characters(&work_id)).ok();
                })
                .ok();
                // 03 Notification confirmed on publication
                write_element_block("NotificationType", w, |w| {
                    w.write(XmlEvent::Characters("03")).ok();
                })
                .ok();
                // 01 Publisher
                write_element_block("RecordSourceType", w, |w| {
                    w.write(XmlEvent::Characters("01")).ok();
                })
                .ok();
                write_element_block("ProductIdentifier", w, |w| {
                    // 01 Proprietary
                    write_element_block("ProductIDType", w, |w| {
                        w.write(XmlEvent::Characters("01")).ok();
                    })
                    .ok();
                    write_element_block("IDValue", w, |w| {
                        w.write(XmlEvent::Characters(&work_id)).ok();
                    })
                    .ok();
                })
                .ok();
                write_element_block("ProductIdentifier", w, |w| {
                    // 15 ISBN-13
                    write_element_block("ProductIDType", w, |w| {
                        w.write(XmlEvent::Characters("15")).ok();
                    })
                    .ok();
                    write_element_block("IDValue", w, |w| {
                        w.write(XmlEvent::Characters(&main_isbn)).ok();
                    })
                    .ok();
                })
                .ok();
                if let Some(doi) = &self.doi {
                    write_element_block("ProductIdentifier", w, |w| {
                        write_element_block("ProductIDType", w, |w| {
                            w.write(XmlEvent::Characters("06")).ok();
                        })
                        .ok();
                        write_element_block("IDValue", w, |w| {
                            w.write(XmlEvent::Characters(&doi.replace("https://doi.org/", "")))
                                .ok();
                        })
                        .ok();
                    })
                    .ok();
                }
                write_element_block("DescriptiveDetail", w, |w| {
                    // 00 Single-component retail product
                    write_element_block("ProductComposition", w, |w| {
                        w.write(XmlEvent::Characters("00")).ok();
                    })
                    .ok();
                    // EB Digital download and online
                    write_element_block("ProductForm", w, |w| {
                        w.write(XmlEvent::Characters("EB")).ok();
                    })
                    .ok();
                    // E107 PDF
                    write_element_block("ProductFormDetail", w, |w| {
                        w.write(XmlEvent::Characters("E107")).ok();
                    })
                    .ok();
                    // 10 Text (eye-readable)
                    write_element_block("PrimaryContentType", w, |w| {
                        w.write(XmlEvent::Characters("10")).ok();
                    })
                    .ok();
                    if let Some(license) = &self.license {
                        write_element_block("EpubLicense", w, |w| {
                            write_element_block("EpubLicenseName", w, |w| {
                                w.write(XmlEvent::Characters("Creative Commons License"))
                                    .ok();
                            })
                            .ok();
                            write_element_block("EpubLicenseExpression", w, |w| {
                                write_element_block("EpubLicenseExpressionType", w, |w| {
                                    w.write(XmlEvent::Characters("02")).ok();
                                })
                                .ok();
                                write_element_block("EpubLicenseExpressionLink", w, |w| {
                                    w.write(XmlEvent::Characters(&license)).ok();
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
                            w.write(XmlEvent::Characters("01")).ok();
                        })
                        .ok();
                        write_element_block("TitleElement", w, |w| {
                            // 01 Product
                            write_element_block("TitleElementLevel", w, |w| {
                                w.write(XmlEvent::Characters("01")).ok();
                            })
                            .ok();
                            if let Some(subtitle) = &self.subtitle {
                                write_element_block("TitleText", w, |w| {
                                    w.write(XmlEvent::Characters(&self.title)).ok();
                                })
                                .ok();
                                write_element_block("Subtitle", w, |w| {
                                    w.write(XmlEvent::Characters(&subtitle)).ok();
                                })
                                .ok();
                            } else {
                                write_element_block("TitleText", w, |w| {
                                    w.write(XmlEvent::Characters(&self.full_title)).ok();
                                })
                                .ok();
                            }
                        })
                        .ok();
                    })
                    .ok();
                    XmlElementBlock::<Onix3ProjectMuse>::xml_element(&self.contributions, w).ok();
                    for language in &self.languages {
                        XmlElementBlock::<Onix3ProjectMuse>::xml_element(language, w).ok();
                    }
                    if let Some(page_count) = self.page_count {
                        write_element_block("Extent", w, |w| {
                            // 00 Main content
                            write_element_block("ExtentType", w, |w| {
                                w.write(XmlEvent::Characters("00")).ok();
                            })
                            .ok();
                            write_element_block("ExtentValue", w, |w| {
                                w.write(XmlEvent::Characters(&page_count.to_string())).ok();
                            })
                            .ok();
                            // 03 Pages
                            write_element_block("ExtentUnit", w, |w| {
                                w.write(XmlEvent::Characters("03")).ok();
                            })
                            .ok();
                        })
                        .ok();
                    }
                    for subject in &self.subjects {
                        write_element_block("Subject", w, |w| {
                            XmlElement::<Onix3ProjectMuse>::xml_element(&subject.subject_type, w)
                                .ok();
                            write_element_block("SubjectCode", w, |w| {
                                w.write(XmlEvent::Characters(&subject.subject_code)).ok();
                            })
                            .ok();
                        })
                        .ok();
                    }
                })
                .ok();
                if self.long_abstract.is_some() || self.toc.is_some() {
                    write_element_block("CollateralDetail", w, |w| {
                        if let Some(labstract) = &self.long_abstract {
                            write_element_block("TextContent", w, |w| {
                                let mut lang_fmt: HashMap<&str, &str> = HashMap::new();
                                lang_fmt.insert("language", "eng");
                                // 03 Description ("30 Abstract" not implemented in OAPEN)
                                write_element_block("TextType", w, |w| {
                                    w.write(XmlEvent::Characters("03")).ok();
                                })
                                .ok();
                                // 00 Unrestricted
                                write_element_block("ContentAudience", w, |w| {
                                    w.write(XmlEvent::Characters("00")).ok();
                                })
                                .ok();
                                write_full_element_block("Text", None, Some(lang_fmt), w, |w| {
                                    w.write(XmlEvent::Characters(&labstract)).ok();
                                })
                                .ok();
                            })
                            .ok();
                        }
                        if let Some(toc) = &self.toc {
                            write_element_block("TextContent", w, |w| {
                                // 04 Table of contents
                                write_element_block("TextType", w, |w| {
                                    w.write(XmlEvent::Characters("04")).ok();
                                })
                                .ok();
                                // 00 Unrestricted
                                write_element_block("ContentAudience", w, |w| {
                                    w.write(XmlEvent::Characters("00")).ok();
                                })
                                .ok();
                                write_element_block("Text", w, |w| {
                                    w.write(XmlEvent::Characters(&toc)).ok();
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
                            w.write(XmlEvent::Characters(&self.imprint.imprint_name))
                                .ok();
                        })
                        .ok();
                    })
                    .ok();
                    write_element_block("Publisher", w, |w| {
                        // 01 Publisher
                        write_element_block("PublishingRole", w, |w| {
                            w.write(XmlEvent::Characters("01")).ok();
                        })
                        .ok();
                        write_element_block("PublisherName", w, |w| {
                            w.write(XmlEvent::Characters(&self.imprint.publisher.publisher_name))
                                .ok();
                        })
                        .ok();
                    })
                    .ok();
                    if let Some(place) = &self.place {
                        write_element_block("CityOfPublication", w, |w| {
                            w.write(XmlEvent::Characters(&place)).ok();
                        })
                        .ok();
                    }
                    XmlElement::<Onix3ProjectMuse>::xml_element(&self.work_status, w).ok();
                    if let Some(date) = self.publication_date {
                        write_element_block("PublishingDate", w, |w| {
                            let mut date_fmt: HashMap<&str, &str> = HashMap::new();
                            date_fmt.insert("dateformat", "01"); // 01 YYYYMM

                            write_element_block("PublishingDateRole", w, |w| {
                                // 19 Publication date of print counterpart
                                w.write(XmlEvent::Characters("19")).ok();
                            })
                            .ok();
                            // dateformat="01" YYYYMM
                            write_full_element_block("Date", None, Some(date_fmt), w, |w| {
                                w.write(XmlEvent::Characters(&date.format("%Y%m").to_string()))
                                    .ok();
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
                                    w.write(XmlEvent::Characters("06")).ok();
                                })
                                .ok();
                                write_element_block("ProductIdentifier", w, |w| {
                                    // 06 ISBN
                                    write_element_block("ProductIDType", w, |w| {
                                        w.write(XmlEvent::Characters("06")).ok();
                                    })
                                    .ok();
                                    write_element_block("IDValue", w, |w| {
                                        w.write(XmlEvent::Characters(&isbn)).ok();
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
                    if let Some(landing_page) = &self.landing_page {
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
                                    w.write(XmlEvent::Characters("11")).ok();
                                })
                                .ok();
                                write_element_block("SupplierName", w, |w| {
                                    w.write(XmlEvent::Characters(
                                        &self.imprint.publisher.publisher_name,
                                    ))
                                    .ok();
                                })
                                .ok();
                                write_element_block("Website", w, |w| {
                                    // 01 Publisher’s corporate website
                                    write_element_block("WebsiteRole", w, |w| {
                                        w.write(XmlEvent::Characters("01")).ok();
                                    })
                                    .ok();
                                    write_element_block("WebsiteDescription", w, |w| {
                                        w.write(XmlEvent::Characters(&description)).ok();
                                    })
                                    .ok();
                                    write_element_block("WebsiteLink", w, |w| {
                                        w.write(XmlEvent::Characters(&url)).ok();
                                    })
                                    .ok();
                                })
                                .ok();
                            })
                            .ok();
                            // 99 Contact supplier
                            write_element_block("ProductAvailability", w, |w| {
                                w.write(XmlEvent::Characters("99")).ok();
                            })
                            .ok();
                            // 04 Contact supplier
                            write_element_block("UnpricedItemType", w, |w| {
                                w.write(XmlEvent::Characters("04")).ok();
                            })
                            .ok();
                        })
                        .ok();
                    }
                })
                .ok();
            })
        } else {
            Ok(())
        }
    }
}

fn get_publications_data(publications: &[WorkPublications]) -> (String, Vec<String>) {
    let mut main_isbn = "".to_string();
    let mut isbns: Vec<String> = Vec::new();

    for publication in publications {
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

    (main_isbn, isbns)
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

// Replace with implementation for WorkContributions (without the vector)
// when we implement contribution ordering
impl XmlElementBlock<Onix3ProjectMuse> for Vec<WorkContributions> {
    fn xml_element<W: Write>(&self, w: &mut EventWriter<W>) -> Result<()> {
        for (mut sequence_number, contribution) in self.iter().enumerate() {
            sequence_number += 1;
            write_element_block("Contributor", w, |w| {
                write_element_block("SequenceNumber", w, |w| {
                    w.write(XmlEvent::Characters(&sequence_number.to_string()))
                        .ok();
                })
                .ok();
                XmlElement::<Onix3ProjectMuse>::xml_element(&contribution.contribution_type, w)
                    .ok();

                if let Some(orcid) = &contribution.contributor.orcid {
                    write_element_block("NameIdentifier", w, |w| {
                        write_element_block("NameIDType", w, |w| {
                            w.write(XmlEvent::Characters("21")).ok();
                        })
                        .ok();
                        write_element_block("IDValue", w, |w| {
                            w.write(XmlEvent::Characters(&orcid)).ok();
                        })
                        .ok();
                    })
                    .ok();
                }
                if let Some(first_name) = &contribution.first_name {
                    write_element_block("NamesBeforeKey", w, |w| {
                        w.write(XmlEvent::Characters(&first_name)).ok();
                    })
                    .ok();
                    write_element_block("KeyNames", w, |w| {
                        w.write(XmlEvent::Characters(&contribution.last_name)).ok();
                    })
                    .ok();
                } else {
                    write_element_block("PersonName", w, |w| {
                        w.write(XmlEvent::Characters(&contribution.full_name)).ok();
                    })
                    .ok();
                }
            })
            .ok();
        }
        Ok(())
    }
}

impl XmlElementBlock<Onix3ProjectMuse> for WorkLanguages {
    fn xml_element<W: Write>(&self, w: &mut EventWriter<W>) -> Result<()> {
        write_element_block("Language", w, |w| {
            XmlElement::<Onix3ProjectMuse>::xml_element(&self.language_relation, w).ok();
            // not worth implementing XmlElement for LanguageCode as all cases would
            // need to be exhaustively matched and the codes are equivalent anyway
            write_element_block("LanguageCode", w, |w| {
                w.write(XmlEvent::Characters(
                    &self.language_code.to_string().to_lowercase(),
                ))
                .ok();
            })
            .ok();
        })
    }
}
