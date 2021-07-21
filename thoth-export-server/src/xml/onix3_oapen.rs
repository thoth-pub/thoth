use chrono::Utc;
use std::collections::HashMap;
use std::io::Write;
use thoth_client::{
    ContributionType, LanguageRelation, PublicationType, SubjectType, Work, WorkContributions,
    WorkLanguages, WorkPublications, WorkStatus,
};
use xml::writer::{EventWriter, XmlEvent};

use super::{write_element_block, XmlElement, XmlSpecification};
use crate::xml::{write_full_element_block, XmlElementBlock};
use thoth_api::model::DOI_DOMAIN;
use thoth_errors::{ThothError, ThothResult};

pub struct Onix3Oapen {}

impl XmlSpecification for Onix3Oapen {
    fn handle_event<W: Write>(w: &mut EventWriter<W>, works: &[Work]) -> ThothResult<()> {
        let mut attr_map: HashMap<&str, &str> = HashMap::new();

        attr_map.insert("release", "3.0");
        attr_map.insert("xmlns", "http://ns.editeur.org/onix/3.0/reference");

        write_full_element_block("ONIXMessage", None, Some(attr_map), w, |w| {
            write_element_block("Header", w, |w| {
                write_element_block("Sender", w, |w| {
                    write_element_block("SenderName", w, |w| {
                        w.write(XmlEvent::Characters("Thoth")).map_err(|e| e.into())
                    })?;
                    write_element_block("EmailAddress", w, |w| {
                        w.write(XmlEvent::Characters("info@thoth.pub"))
                            .map_err(|e| e.into())
                    })
                })?;
                write_element_block("SentDateTime", w, |w| {
                    w.write(XmlEvent::Characters(
                        &Utc::now().format("%Y%m%d").to_string(),
                    ))
                    .map_err(|e| e.into())
                })
            })?;

            match works.len() {
                0 => Err(ThothError::IncompleteMetadataRecord(
                    "onix_3.0::oapen".to_string(),
                    "Not enough data".to_string(),
                )),
                1 => XmlElementBlock::<Onix3Oapen>::xml_element(works.first().unwrap(), w),
                _ => {
                    for work in works.iter() {
                        XmlElementBlock::<Onix3Oapen>::xml_element(work, w).ok();
                    }
                    Ok(())
                }
            }
        })
    }
}

impl XmlElementBlock<Onix3Oapen> for Work {
    fn xml_element<W: Write>(&self, w: &mut EventWriter<W>) -> ThothResult<()> {
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
                    w.write(XmlEvent::Characters(&work_id))
                        .map_err(|e| e.into())
                })?;
                // 03 Notification confirmed on publication
                write_element_block("NotificationType", w, |w| {
                    w.write(XmlEvent::Characters("03")).map_err(|e| e.into())
                })?;
                // 01 Publisher
                write_element_block("RecordSourceType", w, |w| {
                    w.write(XmlEvent::Characters("01")).map_err(|e| e.into())
                })?;
                write_element_block("ProductIdentifier", w, |w| {
                    // 01 Proprietary
                    write_element_block("ProductIDType", w, |w| {
                        w.write(XmlEvent::Characters("01")).map_err(|e| e.into())
                    })?;
                    write_element_block("IDValue", w, |w| {
                        w.write(XmlEvent::Characters(&work_id))
                            .map_err(|e| e.into())
                    })
                })?;
                write_element_block("ProductIdentifier", w, |w| {
                    // 15 ISBN-13
                    write_element_block("ProductIDType", w, |w| {
                        w.write(XmlEvent::Characters("15")).map_err(|e| e.into())
                    })?;
                    write_element_block("IDValue", w, |w| {
                        w.write(XmlEvent::Characters(&main_isbn))
                            .map_err(|e| e.into())
                    })
                })?;
                if let Some(doi) = &self.doi {
                    write_element_block("ProductIdentifier", w, |w| {
                        write_element_block("ProductIDType", w, |w| {
                            w.write(XmlEvent::Characters("06")).map_err(|e| e.into())
                        })?;
                        write_element_block("IDValue", w, |w| {
                            w.write(XmlEvent::Characters(&doi.replace(DOI_DOMAIN, "")))
                                .map_err(|e| e.into())
                        })
                    })?;
                }
                write_element_block("DescriptiveDetail", w, |w| {
                    // 00 Single-component retail product
                    write_element_block("ProductComposition", w, |w| {
                        w.write(XmlEvent::Characters("00")).map_err(|e| e.into())
                    })?;
                    // EB Digital download and online
                    write_element_block("ProductForm", w, |w| {
                        w.write(XmlEvent::Characters("EB")).map_err(|e| e.into())
                    })?;
                    // E107 PDF
                    write_element_block("ProductFormDetail", w, |w| {
                        w.write(XmlEvent::Characters("E107")).map_err(|e| e.into())
                    })?;
                    // 10 Text (eye-readable)
                    write_element_block("PrimaryContentType", w, |w| {
                        w.write(XmlEvent::Characters("10")).map_err(|e| e.into())
                    })?;
                    if let Some(license) = &self.license {
                        write_element_block("EpubLicense", w, |w| {
                            write_element_block("EpubLicenseName", w, |w| {
                                w.write(XmlEvent::Characters("Creative Commons License"))
                                    .map_err(|e| e.into())
                            })?;
                            write_element_block("EpubLicenseExpression", w, |w| {
                                write_element_block("EpubLicenseExpressionType", w, |w| {
                                    w.write(XmlEvent::Characters("02")).map_err(|e| e.into())
                                })?;
                                write_element_block("EpubLicenseExpressionLink", w, |w| {
                                    w.write(XmlEvent::Characters(&license))
                                        .map_err(|e| e.into())
                                })
                            })
                        })?;
                    }
                    for issue in &self.issues {
                        write_element_block("Collection", w, |w| {
                            // 10 Publisher collection (e.g. series)
                            write_element_block("CollectionType", w, |w| {
                                w.write(XmlEvent::Characters("10")).map_err(|e| e.into())
                            })?;
                            write_element_block("TitleDetail", w, |w| {
                                // 01 Cover title (serial)
                                write_element_block("TitleType", w, |w| {
                                    w.write(XmlEvent::Characters("01")).map_err(|e| e.into())
                                })?;
                                write_element_block("TitleElement", w, |w| {
                                    // 02 Collection level
                                    write_element_block("TitleElementLevel", w, |w| {
                                        w.write(XmlEvent::Characters("02")).map_err(|e| e.into())
                                    })?;
                                    write_element_block("PartNumber", w, |w| {
                                        w.write(XmlEvent::Characters(
                                            &issue.issue_ordinal.to_string(),
                                        ))
                                        .map_err(|e| e.into())
                                    })?;
                                    write_element_block("TitleText", w, |w| {
                                        w.write(XmlEvent::Characters(&issue.series.series_name))
                                            .map_err(|e| e.into())
                                    })
                                })
                            })
                        })?;
                    }
                    write_element_block("TitleDetail", w, |w| {
                        // 01 Distinctive title (book)
                        write_element_block("TitleType", w, |w| {
                            w.write(XmlEvent::Characters("01")).map_err(|e| e.into())
                        })?;
                        write_element_block("TitleElement", w, |w| {
                            // 01 Product
                            write_element_block("TitleElementLevel", w, |w| {
                                w.write(XmlEvent::Characters("01")).map_err(|e| e.into())
                            })?;
                            if let Some(subtitle) = &self.subtitle {
                                write_element_block("TitleText", w, |w| {
                                    w.write(XmlEvent::Characters(&self.title))
                                        .map_err(|e| e.into())
                                })?;
                                write_element_block("Subtitle", w, |w| {
                                    w.write(XmlEvent::Characters(&subtitle))
                                        .map_err(|e| e.into())
                                })
                            } else {
                                write_element_block("TitleText", w, |w| {
                                    w.write(XmlEvent::Characters(&self.full_title))
                                        .map_err(|e| e.into())
                                })
                            }
                        })
                    })?;
                    XmlElementBlock::<Onix3Oapen>::xml_element(&self.contributions, w).ok();
                    for language in &self.languages {
                        XmlElementBlock::<Onix3Oapen>::xml_element(language, w).ok();
                    }
                    if let Some(page_count) = self.page_count {
                        write_element_block("Extent", w, |w| {
                            // 00 Main content
                            write_element_block("ExtentType", w, |w| {
                                w.write(XmlEvent::Characters("00")).map_err(|e| e.into())
                            })?;
                            write_element_block("ExtentValue", w, |w| {
                                w.write(XmlEvent::Characters(&page_count.to_string()))
                                    .map_err(|e| e.into())
                            })?;
                            // 03 Pages
                            write_element_block("ExtentUnit", w, |w| {
                                w.write(XmlEvent::Characters("03")).map_err(|e| e.into())
                            })
                        })?;
                    }
                    for subject in &self.subjects {
                        // Don't output Custom codes, as these are not imported by OAPEN,
                        // and only used for internal purposes
                        if subject.subject_type != SubjectType::CUSTOM {
                            write_element_block("Subject", w, |w| {
                                XmlElement::<Onix3Oapen>::xml_element(&subject.subject_type, w)?;
                                match subject.subject_type {
                                    SubjectType::KEYWORD => {
                                        write_element_block("SubjectHeadingText", w, |w| {
                                            w.write(XmlEvent::Characters(&subject.subject_code))
                                                .map_err(|e| e.into())
                                        })
                                    }
                                    _ => write_element_block("SubjectCode", w, |w| {
                                        w.write(XmlEvent::Characters(&subject.subject_code))
                                            .map_err(|e| e.into())
                                    }),
                                }
                            })?;
                        }
                    }
                    write_element_block("Audience", w, |w| {
                        // 01 ONIX audience codes
                        write_element_block("AudienceCodeType", w, |w| {
                            w.write(XmlEvent::Characters("01")).map_err(|e| e.into())
                        })?;
                        // 06 Professional and scholarly
                        write_element_block("AudienceCodeValue", w, |w| {
                            w.write(XmlEvent::Characters("06")).map_err(|e| e.into())
                        })
                    })?;
                    Ok(())
                })?;
                if self.long_abstract.is_some() || self.toc.is_some() {
                    write_element_block("CollateralDetail", w, |w| {
                        if let Some(labstract) = &self.long_abstract {
                            write_element_block("TextContent", w, |w| {
                                let mut lang_fmt: HashMap<&str, &str> = HashMap::new();
                                lang_fmt.insert("language", "eng");
                                // 03 Description ("30 Abstract" not implemented in OAPEN)
                                write_element_block("TextType", w, |w| {
                                    w.write(XmlEvent::Characters("03")).map_err(|e| e.into())
                                })?;
                                // 00 Unrestricted
                                write_element_block("ContentAudience", w, |w| {
                                    w.write(XmlEvent::Characters("00")).map_err(|e| e.into())
                                })?;
                                write_full_element_block("Text", None, Some(lang_fmt), w, |w| {
                                    w.write(XmlEvent::Characters(&labstract))
                                        .map_err(|e| e.into())
                                })
                            })?;
                        }
                        if let Some(cover_url) = &self.cover_url {
                            write_element_block("SupportingResource", w, |w| {
                                // 01 Front cover
                                write_element_block("ResourceContentType", w, |w| {
                                    w.write(XmlEvent::Characters("01")).map_err(|e| e.into())
                                })?;
                                // 00 Unrestricted
                                write_element_block("ContentAudience", w, |w| {
                                    w.write(XmlEvent::Characters("00")).map_err(|e| e.into())
                                })?;
                                // 03 Image
                                write_element_block("ResourceMode", w, |w| {
                                    w.write(XmlEvent::Characters("03")).map_err(|e| e.into())
                                })?;
                                write_element_block("ResourceVersion", w, |w| {
                                    // 02 Downloadable file
                                    write_element_block("ResourceForm", w, |w| {
                                        w.write(XmlEvent::Characters("02")).map_err(|e| e.into())
                                    })?;
                                    write_element_block("ResourceLink", w, |w| {
                                        w.write(XmlEvent::Characters(cover_url))
                                            .map_err(|e| e.into())
                                    })
                                })
                            })?;
                        }
                        Ok(())
                    })?;
                }
                write_element_block("PublishingDetail", w, |w| {
                    write_element_block("Imprint", w, |w| {
                        write_element_block("ImprintName", w, |w| {
                            w.write(XmlEvent::Characters(&self.imprint.imprint_name))
                                .map_err(|e| e.into())
                        })
                    })?;
                    write_element_block("Publisher", w, |w| {
                        // 01 Publisher
                        write_element_block("PublishingRole", w, |w| {
                            w.write(XmlEvent::Characters("01")).map_err(|e| e.into())
                        })?;
                        write_element_block("PublisherName", w, |w| {
                            w.write(XmlEvent::Characters(&self.imprint.publisher.publisher_name))
                                .map_err(|e| e.into())
                        })
                    })?;
                    for funding in &self.fundings {
                        write_element_block("Publisher", w, |w| {
                            // 16 Funding body
                            write_element_block("PublishingRole", w, |w| {
                                w.write(XmlEvent::Characters("16")).map_err(|e| e.into())
                            })?;
                            write_element_block("PublisherName", w, |w| {
                                w.write(XmlEvent::Characters(&funding.funder.funder_name))
                                    .map_err(|e| e.into())
                            })?;
                            let mut identifiers: HashMap<String, String> = HashMap::new();
                            if let Some(program) = &funding.program {
                                identifiers.insert("programname".to_string(), program.to_string());
                            }
                            if let Some(project_name) = &funding.project_name {
                                identifiers
                                    .insert("projectname".to_string(), project_name.to_string());
                            }
                            if let Some(grant_number) = &funding.grant_number {
                                identifiers
                                    .insert("grantnumber".to_string(), grant_number.to_string());
                            }
                            if !identifiers.is_empty() {
                                write_element_block("Funding", w, |w| {
                                    for (typename, value) in &identifiers {
                                        write_element_block("FundingIdentifier", w, |w| {
                                            // 01 Proprietary
                                            write_element_block("FundingIDType", w, |w| {
                                                w.write(XmlEvent::Characters("01"))
                                                    .map_err(|e| e.into())
                                            })?;
                                            write_element_block("IDTypeName", w, |w| {
                                                w.write(XmlEvent::Characters(&typename))
                                                    .map_err(|e| e.into())
                                            })?;
                                            write_element_block("IDValue", w, |w| {
                                                w.write(XmlEvent::Characters(&value))
                                                    .map_err(|e| e.into())
                                            })
                                        })?;
                                    }
                                    Ok(())
                                })?;
                            }
                            Ok(())
                        })?;
                    }
                    if let Some(place) = &self.place {
                        write_element_block("CityOfPublication", w, |w| {
                            w.write(XmlEvent::Characters(&place)).map_err(|e| e.into())
                        })?;
                    }
                    XmlElement::<Onix3Oapen>::xml_element(&self.work_status, w)?;
                    if let Some(date) = self.publication_date {
                        write_element_block("PublishingDate", w, |w| {
                            let mut date_fmt: HashMap<&str, &str> = HashMap::new();
                            date_fmt.insert("dateformat", "05"); // 01 YYYY

                            write_element_block("PublishingDateRole", w, |w| {
                                // 19 Publication date of print counterpart
                                w.write(XmlEvent::Characters("19")).map_err(|e| e.into())
                            })?;
                            // dateformat="05" YYYY
                            write_full_element_block("Date", None, Some(date_fmt), w, |w| {
                                w.write(XmlEvent::Characters(&date.format("%Y").to_string()))
                                    .map_err(|e| e.into())
                            })
                        })?;
                    }
                    Ok(())
                })?;
                if !isbns.is_empty() {
                    write_element_block("RelatedMaterial", w, |w| {
                        for isbn in &isbns {
                            write_element_block("RelatedProduct", w, |w| {
                                // 06 Alternative format
                                write_element_block("ProductRelationCode", w, |w| {
                                    w.write(XmlEvent::Characters("06")).map_err(|e| e.into())
                                })?;
                                write_element_block("ProductIdentifier", w, |w| {
                                    // 15 ISBN-13
                                    write_element_block("ProductIDType", w, |w| {
                                        w.write(XmlEvent::Characters("15")).map_err(|e| e.into())
                                    })?;
                                    write_element_block("IDValue", w, |w| {
                                        w.write(XmlEvent::Characters(&isbn)).map_err(|e| e.into())
                                    })
                                })
                            })?;
                        }
                        Ok(())
                    })?;
                }
                write_element_block("ProductSupply", w, |w| {
                    let mut supplies: HashMap<String, (String, String)> = HashMap::new();
                    supplies.insert(
                        pdf_url.to_string(),
                        (
                            "29".to_string(),
                            "Publisher's website: download the title".to_string(),
                        ),
                    );
                    if let Some(landing_page) = &self.landing_page {
                        supplies.insert(
                            landing_page.to_string(),
                            (
                                "01".to_string(),
                                "Publisher's website: web shop".to_string(),
                            ),
                        );
                    }
                    for (url, description) in supplies.iter() {
                        write_element_block("SupplyDetail", w, |w| {
                            write_element_block("Supplier", w, |w| {
                                // 09 Publisher to end-customers
                                write_element_block("SupplierRole", w, |w| {
                                    w.write(XmlEvent::Characters("09")).map_err(|e| e.into())
                                })?;
                                write_element_block("SupplierName", w, |w| {
                                    w.write(XmlEvent::Characters(
                                        &self.imprint.publisher.publisher_name,
                                    ))
                                    .map_err(|e| e.into())
                                })?;
                                write_element_block("Website", w, |w| {
                                    // 01 Publisher’s corporate website
                                    write_element_block("WebsiteRole", w, |w| {
                                        w.write(XmlEvent::Characters(&description.0))
                                            .map_err(|e| e.into())
                                    })?;
                                    write_element_block("WebsiteDescription", w, |w| {
                                        w.write(XmlEvent::Characters(&description.1))
                                            .map_err(|e| e.into())
                                    })?;
                                    write_element_block("WebsiteLink", w, |w| {
                                        w.write(XmlEvent::Characters(&url)).map_err(|e| e.into())
                                    })
                                })
                            })?;
                            // 99 Contact supplier
                            write_element_block("ProductAvailability", w, |w| {
                                w.write(XmlEvent::Characters("99")).map_err(|e| e.into())
                            })?;
                            // 04 Contact supplier
                            write_element_block("UnpricedItemType", w, |w| {
                                w.write(XmlEvent::Characters("04")).map_err(|e| e.into())
                            })
                        })?;
                    }
                    Ok(())
                })
            })
        } else {
            Err(ThothError::IncompleteMetadataRecord(
                "onix_3.0::oapen".to_string(),
                "Missing PDF URL".to_string(),
            ))
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

impl XmlElement<Onix3Oapen> for WorkStatus {
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

impl XmlElement<Onix3Oapen> for SubjectType {
    const ELEMENT: &'static str = "SubjectSchemeIdentifier";

    fn value(&self) -> &'static str {
        match self {
            SubjectType::BIC => "12",
            SubjectType::BISAC => "10",
            SubjectType::KEYWORD => "20",
            SubjectType::LCC => "04",
            SubjectType::THEMA => "93",
            // Custom codes are not output for OAPEN
            SubjectType::CUSTOM | SubjectType::Other(_) => unreachable!(),
        }
    }
}

impl XmlElement<Onix3Oapen> for LanguageRelation {
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

impl XmlElement<Onix3Oapen> for ContributionType {
    const ELEMENT: &'static str = "ContributorRole";

    fn value(&self) -> &'static str {
        match self {
            ContributionType::AUTHOR => "A01",
            ContributionType::EDITOR => "B01",
            ContributionType::TRANSLATOR
            | ContributionType::PHOTOGRAPHER
            | ContributionType::ILUSTRATOR
            | ContributionType::MUSIC_EDITOR
            | ContributionType::FOREWORD_BY
            | ContributionType::INTRODUCTION_BY
            | ContributionType::AFTERWORD_BY
            | ContributionType::PREFACE_BY => "Z01",
            ContributionType::Other(_) => unreachable!(),
        }
    }
}

// Replace with implementation for WorkContributions (without the vector)
// when we implement contribution ordering
impl XmlElementBlock<Onix3Oapen> for Vec<WorkContributions> {
    fn xml_element<W: Write>(&self, w: &mut EventWriter<W>) -> ThothResult<()> {
        for (mut sequence_number, contribution) in self.iter().enumerate() {
            sequence_number += 1;
            write_element_block("Contributor", w, |w| {
                write_element_block("SequenceNumber", w, |w| {
                    w.write(XmlEvent::Characters(&sequence_number.to_string()))
                        .map_err(|e| e.into())
                })?;
                XmlElement::<Onix3Oapen>::xml_element(&contribution.contribution_type, w)?;

                if let Some(orcid) = &contribution.contributor.orcid {
                    write_element_block("NameIdentifier", w, |w| {
                        write_element_block("NameIDType", w, |w| {
                            w.write(XmlEvent::Characters("21")).map_err(|e| e.into())
                        })?;
                        write_element_block("IDValue", w, |w| {
                            w.write(XmlEvent::Characters(&orcid)).map_err(|e| e.into())
                        })
                    })?;
                }
                if let Some(first_name) = &contribution.first_name {
                    write_element_block("NamesBeforeKey", w, |w| {
                        w.write(XmlEvent::Characters(&first_name))
                            .map_err(|e| e.into())
                    })?;
                    write_element_block("KeyNames", w, |w| {
                        w.write(XmlEvent::Characters(&contribution.last_name))
                            .map_err(|e| e.into())
                    })?;
                } else {
                    write_element_block("PersonName", w, |w| {
                        w.write(XmlEvent::Characters(&contribution.full_name))
                            .map_err(|e| e.into())
                    })?;
                }
                Ok(())
            })?;
        }
        Ok(())
    }
}

impl XmlElementBlock<Onix3Oapen> for WorkLanguages {
    fn xml_element<W: Write>(&self, w: &mut EventWriter<W>) -> ThothResult<()> {
        write_element_block("Language", w, |w| {
            XmlElement::<Onix3Oapen>::xml_element(&self.language_relation, w).ok();
            // not worth implementing XmlElement for LanguageCode as all cases would
            // need to be exhaustively matched and the codes are equivalent anyway
            write_element_block("LanguageCode", w, |w| {
                w.write(XmlEvent::Characters(
                    &self.language_code.to_string().to_lowercase(),
                ))
                .map_err(|e| e.into())
            })
        })
    }
}
