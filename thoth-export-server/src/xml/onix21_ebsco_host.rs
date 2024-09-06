use chrono::Utc;
use std::collections::HashMap;
use std::io::Write;
use thoth_client::{
    ContributionType, LanguageRelation, PublicationType, SubjectType, Work, WorkContributions,
    WorkIssues, WorkLanguages, WorkPublications, WorkStatus, WorkSubjects, WorkType,
};
use xml::writer::{EventWriter, XmlEvent};

use super::{write_element_block, XmlElement, XmlSpecification};
use crate::xml::{write_full_element_block, XmlElementBlock};
use thoth_errors::{ThothError, ThothResult};

#[derive(Copy, Clone)]
pub struct Onix21EbscoHost {}

const ONIX_ERROR: &str = "onix_2.1::ebsco_host";

impl XmlSpecification for Onix21EbscoHost {
    fn handle_event<W: Write>(w: &mut EventWriter<W>, works: &[Work]) -> ThothResult<()> {
        write_full_element_block("ONIXMessage", None, w, |w| {
            write_element_block("Header", w, |w| {
                write_element_block("FromCompany", w, |w| {
                    w.write(XmlEvent::Characters("Thoth")).map_err(|e| e.into())
                })?;
                write_element_block("FromEmail", w, |w| {
                    w.write(XmlEvent::Characters("distribution@thoth.pub"))
                        .map_err(|e| e.into())
                })?;
                write_element_block("SentDate", w, |w| {
                    w.write(XmlEvent::Characters(
                        &Utc::now().format("%Y%m%d").to_string(),
                    ))
                    .map_err(|e| e.into())
                })
            })?;

            match works {
                [] => Err(ThothError::IncompleteMetadataRecord(
                    ONIX_ERROR.to_string(),
                    "Not enough data".to_string(),
                )),
                [work] => XmlElementBlock::<Onix21EbscoHost>::xml_element(work, w),
                _ => {
                    for work in works.iter() {
                        // Do not include Chapters in full publisher metadata record
                        // (assumes that a publisher will always have more than one work)
                        if work.work_type != WorkType::BOOK_CHAPTER {
                            XmlElementBlock::<Onix21EbscoHost>::xml_element(work, w).ok();
                        }
                    }
                    Ok(())
                }
            }
        })
    }
}

impl XmlElementBlock<Onix21EbscoHost> for Work {
    fn xml_element<W: Write>(&self, w: &mut EventWriter<W>) -> ThothResult<()> {
        // EBSCO Host can only accept PDFs and EPUBs, and can only
        // process them as Open Access if they are unpriced
        let pdf_url = self
            .publications
            .iter()
            .find(|p| {
                p.publication_type.eq(&PublicationType::PDF)
                    && !p.locations.is_empty()
                    // Thoth database only accepts non-zero prices
                    && p.prices.is_empty()
            })
            .and_then(|p| p.locations.iter().find(|l| l.canonical))
            .and_then(|l| l.full_text_url.as_ref());
        let epub_url = self
            .publications
            .iter()
            .find(|p| {
                p.publication_type.eq(&PublicationType::EPUB)
                    && !p.locations.is_empty()
                    // Thoth database only accepts non-zero prices
                    && p.prices.is_empty()
            })
            .and_then(|p| p.locations.iter().find(|l| l.canonical))
            .and_then(|l| l.full_text_url.as_ref());
        if pdf_url.is_some() || epub_url.is_some() {
            let work_id = format!("urn:uuid:{}", self.work_id);
            let (main_isbn, isbns) = get_publications_data(&self.publications);
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
                if let Some(isbn) = &main_isbn {
                    write_element_block("ProductIdentifier", w, |w| {
                        // 15 ISBN-13
                        write_element_block("ProductIDType", w, |w| {
                            w.write(XmlEvent::Characters("15")).map_err(|e| e.into())
                        })?;
                        write_element_block("IDValue", w, |w| {
                            w.write(XmlEvent::Characters(isbn)).map_err(|e| e.into())
                        })
                    })?;
                }
                if let Some(doi) = &self.doi {
                    write_element_block("ProductIdentifier", w, |w| {
                        write_element_block("ProductIDType", w, |w| {
                            w.write(XmlEvent::Characters("06")).map_err(|e| e.into())
                        })?;
                        write_element_block("IDValue", w, |w| {
                            w.write(XmlEvent::Characters(&doi.to_string()))
                                .map_err(|e| e.into())
                        })
                    })?;
                }
                // DG Electronic book text in proprietary or open standard format
                write_element_block("ProductForm", w, |w| {
                    w.write(XmlEvent::Characters("DG")).map_err(|e| e.into())
                })?;
                write_element_block("EpubType", w, |w| {
                    // 002 PDF
                    let mut epub_type = "002";
                    // We definitely have either a PDF URL or an EPUB URL (or both)
                    if pdf_url.is_none() {
                        // 029 EPUB
                        epub_type = "029";
                    }
                    w.write(XmlEvent::Characters(epub_type))
                        .map_err(|e| e.into())
                })?;
                for issue in &self.issues {
                    XmlElementBlock::<Onix21EbscoHost>::xml_element(issue, w).ok();
                }
                write_element_block("Title", w, |w| {
                    // 01 Distinctive title (book)
                    write_element_block("TitleType", w, |w| {
                        w.write(XmlEvent::Characters("01")).map_err(|e| e.into())
                    })?;
                    write_element_block("TitleText", w, |w| {
                        w.write(XmlEvent::Characters(&self.title))
                            .map_err(|e| e.into())
                    })?;
                    if let Some(subtitle) = &self.subtitle {
                        write_element_block("Subtitle", w, |w| {
                            w.write(XmlEvent::Characters(subtitle))
                                .map_err(|e| e.into())
                        })?;
                    }
                    Ok(())
                })?;
                write_element_block("WorkIdentifier", w, |w| {
                    // 01 Proprietary
                    write_element_block("WorkIDType", w, |w| {
                        w.write(XmlEvent::Characters("01")).map_err(|e| e.into())
                    })?;
                    write_element_block("IDTypeName", w, |w| {
                        w.write(XmlEvent::Characters("Thoth WorkID"))
                            .map_err(|e| e.into())
                    })?;
                    write_element_block("IDValue", w, |w| {
                        w.write(XmlEvent::Characters(&work_id))
                            .map_err(|e| e.into())
                    })
                })?;
                let mut websites: HashMap<String, (String, String)> = HashMap::new();
                if let Some(pdf) = pdf_url {
                    websites.insert(
                        pdf.to_string(),
                        (
                            "29".to_string(),
                            "Publisher's website: download the title".to_string(),
                        ),
                    );
                }
                if let Some(epub) = epub_url {
                    websites.insert(
                        epub.to_string(),
                        (
                            "29".to_string(),
                            "Publisher's website: download the title".to_string(),
                        ),
                    );
                }
                if let Some(landing_page) = &self.landing_page {
                    websites.insert(
                        landing_page.to_string(),
                        (
                            "02".to_string(),
                            "Publisher's website: web shop".to_string(),
                        ),
                    );
                }
                for (url, description) in websites.iter() {
                    write_element_block("Website", w, |w| {
                        write_element_block("WebsiteRole", w, |w| {
                            w.write(XmlEvent::Characters(&description.0))
                                .map_err(|e| e.into())
                        })?;
                        write_element_block("WebsiteDescription", w, |w| {
                            w.write(XmlEvent::Characters(&description.1))
                                .map_err(|e| e.into())
                        })?;
                        write_element_block("WebsiteLink", w, |w| {
                            w.write(XmlEvent::Characters(url)).map_err(|e| e.into())
                        })
                    })?;
                }
                for contribution in &self.contributions {
                    // A51 Research by is not supported in ONIX 2
                    if contribution.contribution_type != ContributionType::RESEARCH_BY {
                        XmlElementBlock::<Onix21EbscoHost>::xml_element(contribution, w).ok();
                    }
                }
                for language in &self.languages {
                    XmlElementBlock::<Onix21EbscoHost>::xml_element(language, w).ok();
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
                    XmlElementBlock::<Onix21EbscoHost>::xml_element(subject, w).ok();
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
                write_element_block("OtherText", w, |w| {
                    // 47 Open access statement
                    // "Should always be accompanied by a link to the complete license (see code 46)"
                    // (not specified as required by EBSCO Host themselves)
                    write_element_block("TextTypeCode", w, |w| {
                        w.write(XmlEvent::Characters("47")).map_err(|e| e.into())
                    })?;
                    write_element_block("Text", w, |w| {
                        w.write(XmlEvent::Characters("Open access - no commercial use"))
                            .map_err(|e| e.into())
                    })
                })?;
                if let Some(license) = &self.license {
                    write_element_block("OtherText", w, |w| {
                        // 46 License
                        write_element_block("TextTypeCode", w, |w| {
                            w.write(XmlEvent::Characters("46")).map_err(|e| e.into())
                        })?;
                        write_element_block("Text", w, |w| {
                            w.write(XmlEvent::Characters(license)).map_err(|e| e.into())
                        })
                    })?;
                }
                if let Some(labstract) = &self.long_abstract {
                    write_element_block("OtherText", w, |w| {
                        // 03 Long description
                        write_element_block("TextTypeCode", w, |w| {
                            w.write(XmlEvent::Characters("03")).map_err(|e| e.into())
                        })?;
                        // 06 Default text format
                        write_element_block("TextFormat", w, |w| {
                            w.write(XmlEvent::Characters("06")).map_err(|e| e.into())
                        })?;
                        write_element_block("Text", w, |w| {
                            w.write(XmlEvent::Characters(labstract))
                                .map_err(|e| e.into())
                        })
                    })?;
                }
                if let Some(cover_url) = &self.cover_url {
                    write_element_block("MediaFile", w, |w| {
                        // 04 Image: front cover
                        write_element_block("MediaFileTypeCode", w, |w| {
                            w.write(XmlEvent::Characters("04")).map_err(|e| e.into())
                        })?;
                        // 01 URL
                        write_element_block("MediaFileLinkTypeCode", w, |w| {
                            w.write(XmlEvent::Characters("01")).map_err(|e| e.into())
                        })?;
                        write_element_block("MediaFileLink", w, |w| {
                            w.write(XmlEvent::Characters(cover_url))
                                .map_err(|e| e.into())
                        })
                    })?;
                }
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
                    })?;
                    if let Some(publisher_url) = &self.imprint.publisher.publisher_url {
                        write_element_block("Website", w, |w| {
                            write_element_block("WebsiteLink", w, |w| {
                                w.write(XmlEvent::Characters(publisher_url))
                                    .map_err(|e| e.into())
                            })
                        })?;
                    }
                    Ok(())
                })?;
                if let Some(place) = &self.place {
                    write_element_block("CityOfPublication", w, |w| {
                        w.write(XmlEvent::Characters(place)).map_err(|e| e.into())
                    })?;
                }
                XmlElement::<Onix21EbscoHost>::xml_element(&self.work_status, w)?;
                if let Some(date) = self.publication_date {
                    write_element_block("PublicationDate", w, |w| {
                        w.write(XmlEvent::Characters(&date.format("%Y%m%d").to_string()))
                            .map_err(|e| e.into())
                    })?;
                    write_element_block("CopyrightYear", w, |w| {
                        w.write(XmlEvent::Characters(&date.format("%Y").to_string()))
                            .map_err(|e| e.into())
                    })?;
                }
                write_element_block("SalesRights", w, |w| {
                    // 02 For sale with non-exclusive rights in the specified countries or territories
                    write_element_block("SalesRightsType", w, |w| {
                        w.write(XmlEvent::Characters("02")).map_err(|e| e.into())
                    })?;
                    write_element_block("RightsTerritory", w, |w| {
                        w.write(XmlEvent::Characters("WORLD")).map_err(|e| e.into())
                    })
                })?;
                if !isbns.is_empty() {
                    for (publication_type, isbn) in &isbns {
                        let relation_code = match publication_type {
                            PublicationType::PAPERBACK | PublicationType::HARDBACK => "13", // Epublication based on (print product)
                            _ => "06", // Alternative format
                        };

                        write_element_block("RelatedProduct", w, |w| {
                            write_element_block("RelationCode", w, |w| {
                                w.write(XmlEvent::Characters(relation_code))
                                    .map_err(|e| e.into())
                            })?;
                            write_element_block("ProductIdentifier", w, |w| {
                                // 15 ISBN-13
                                write_element_block("ProductIDType", w, |w| {
                                    w.write(XmlEvent::Characters("15")).map_err(|e| e.into())
                                })?;
                                write_element_block("IDValue", w, |w| {
                                    w.write(XmlEvent::Characters(isbn)).map_err(|e| e.into())
                                })
                            })
                        })?;
                    }
                }
                if let Some(date) = self.withdrawn_date {
                    write_element_block("OutofPrintDate", w, |w| {
                        w.write(XmlEvent::Characters(&date.format("%Y%m%d").to_string()))
                            .map_err(|e| e.into())
                    })?;
                }
                write_element_block("SupplyDetail", w, |w| {
                    write_element_block("SupplierName", w, |w| {
                        w.write(XmlEvent::Characters(&self.imprint.publisher.publisher_name))
                            .map_err(|e| e.into())
                    })?;
                    // 09 Publisher to end-customers
                    write_element_block("SupplierRole", w, |w| {
                        w.write(XmlEvent::Characters("09")).map_err(|e| e.into())
                    })?;
                    // 99 Contact supplier
                    write_element_block("ProductAvailability", w, |w| {
                        w.write(XmlEvent::Characters("99")).map_err(|e| e.into())
                    })?;
                    // R Restrictions apply, see note
                    write_element_block("AudienceRestrictionFlag", w, |w| {
                        w.write(XmlEvent::Characters("R")).map_err(|e| e.into())
                    })?;
                    write_element_block("AudienceRestrictionNote", w, |w| {
                        w.write(XmlEvent::Characters("Open access"))
                            .map_err(|e| e.into())
                    })?;
                    // EBSCO Host require the price point for Open Access titles to be listed as "0.01 USD".
                    write_element_block("Price", w, |w| {
                        // 01 RRP excluding tax (price code requested by EBSCO)
                        write_element_block("PriceTypeCode", w, |w| {
                            w.write(XmlEvent::Characters("01")).map_err(|e| e.into())
                        })?;
                        write_element_block("PriceAmount", w, |w| {
                            w.write(XmlEvent::Characters("0.01")).map_err(|e| e.into())
                        })?;
                        write_element_block("CurrencyCode", w, |w| {
                            w.write(XmlEvent::Characters("USD")).map_err(|e| e.into())
                        })
                    })
                })
            })
        } else {
            Err(ThothError::IncompleteMetadataRecord(
                ONIX_ERROR.to_string(),
                "No unpriced PDF or EPUB URL".to_string(),
            ))
        }
    }
}

fn get_publications_data(
    publications: &[WorkPublications],
) -> (Option<String>, Vec<(PublicationType, String)>) {
    let isbns: Vec<(PublicationType, String)> = publications
        .iter()
        .filter_map(|publication| {
            publication.isbn.as_ref().map(|isbn| {
                (
                    publication.publication_type.clone(),
                    isbn.to_hyphenless_string(),
                )
            })
        })
        .collect();

    // The default product ISBN is the PDF's, and fallback to a paperback if not found
    let main_isbn = isbns
        .iter()
        .find(|(publication_type, _)| publication_type == &PublicationType::PDF)
        .or_else(|| {
            isbns
                .iter()
                .find(|(publication_type, _)| publication_type == &PublicationType::PAPERBACK)
        })
        .map(|(_, isbn)| isbn.clone());

    (main_isbn, isbns)
}

impl XmlElement<Onix21EbscoHost> for WorkStatus {
    const ELEMENT: &'static str = "PublishingStatus";

    fn value(&self) -> &'static str {
        match self {
            WorkStatus::CANCELLED => "01",
            WorkStatus::FORTHCOMING => "02",
            WorkStatus::POSTPONED_INDEFINITELY => "03",
            WorkStatus::ACTIVE => "04",
            WorkStatus::SUPERSEDED => "08",
            WorkStatus::WITHDRAWN => "11",
            WorkStatus::Other(_) => unreachable!(),
        }
    }
}

impl XmlElement<Onix21EbscoHost> for SubjectType {
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

impl XmlElement<Onix21EbscoHost> for LanguageRelation {
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

impl XmlElement<Onix21EbscoHost> for ContributionType {
    const ELEMENT: &'static str = "ContributorRole";

    fn value(&self) -> &'static str {
        match self {
            ContributionType::AUTHOR => "A01",
            ContributionType::EDITOR => "B01",
            ContributionType::TRANSLATOR => "B06",
            ContributionType::PHOTOGRAPHER => "A13",
            ContributionType::ILLUSTRATOR => "A12",
            ContributionType::MUSIC_EDITOR => "B25",
            ContributionType::FOREWORD_BY => "A23",
            ContributionType::INTRODUCTION_BY => "A24",
            ContributionType::AFTERWORD_BY => "A19",
            ContributionType::PREFACE_BY => "A15",
            ContributionType::SOFTWARE_BY => "A30",
            ContributionType::CONTRIBUTIONS_BY => "A32",
            ContributionType::INDEXER => "A34",
            ContributionType::RESEARCH_BY | ContributionType::Other(_) => unreachable!(),
        }
    }
}

impl XmlElementBlock<Onix21EbscoHost> for WorkContributions {
    fn xml_element<W: Write>(&self, w: &mut EventWriter<W>) -> ThothResult<()> {
        write_element_block("Contributor", w, |w| {
            write_element_block("SequenceNumber", w, |w| {
                w.write(XmlEvent::Characters(&self.contribution_ordinal.to_string()))
                    .map_err(|e| e.into())
            })?;
            XmlElement::<Onix21EbscoHost>::xml_element(&self.contribution_type, w)?;

            if let Some(first_name) = &self.first_name {
                write_element_block("NamesBeforeKey", w, |w| {
                    w.write(XmlEvent::Characters(first_name))
                        .map_err(|e| e.into())
                })?;
                write_element_block("KeyNames", w, |w| {
                    w.write(XmlEvent::Characters(&self.last_name))
                        .map_err(|e| e.into())
                })?;
            } else {
                write_element_block("PersonName", w, |w| {
                    w.write(XmlEvent::Characters(&self.full_name))
                        .map_err(|e| e.into())
                })?;
            }
            if let Some(orcid) = &self.contributor.orcid {
                write_element_block("PersonNameIdentifier", w, |w| {
                    // 01 Proprietary
                    write_element_block("PersonNameIDType", w, |w| {
                        w.write(XmlEvent::Characters("01")).map_err(|e| e.into())
                    })?;
                    write_element_block("IDTypeName", w, |w| {
                        w.write(XmlEvent::Characters("ORCID")).map_err(|e| e.into())
                    })?;
                    write_element_block("IDValue", w, |w| {
                        w.write(XmlEvent::Characters(&orcid.to_string()))
                            .map_err(|e| e.into())
                    })
                })?;
            }
            Ok(())
        })
    }
}

impl XmlElementBlock<Onix21EbscoHost> for WorkLanguages {
    fn xml_element<W: Write>(&self, w: &mut EventWriter<W>) -> ThothResult<()> {
        write_element_block("Language", w, |w| {
            XmlElement::<Onix21EbscoHost>::xml_element(&self.language_relation, w).ok();
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

impl XmlElementBlock<Onix21EbscoHost> for WorkIssues {
    fn xml_element<W: Write>(&self, w: &mut EventWriter<W>) -> ThothResult<()> {
        write_element_block("Series", w, |w| {
            write_element_block("TitleOfSeries", w, |w| {
                w.write(XmlEvent::Characters(&self.series.series_name))
                    .map_err(|e| e.into())
            })?;
            write_element_block("NumberWithinSeries", w, |w| {
                w.write(XmlEvent::Characters(&self.issue_ordinal.to_string()))
                    .map_err(|e| e.into())
            })
        })
    }
}

impl XmlElementBlock<Onix21EbscoHost> for WorkSubjects {
    fn xml_element<W: Write>(&self, w: &mut EventWriter<W>) -> ThothResult<()> {
        write_element_block("Subject", w, |w| {
            XmlElement::<Onix21EbscoHost>::xml_element(&self.subject_type, w)?;
            match self.subject_type {
                SubjectType::KEYWORD | SubjectType::CUSTOM => {
                    write_element_block("SubjectHeadingText", w, |w| {
                        w.write(XmlEvent::Characters(&self.subject_code))
                            .map_err(|e| e.into())
                    })
                }
                _ => write_element_block("SubjectCode", w, |w| {
                    w.write(XmlEvent::Characters(&self.subject_code))
                        .map_err(|e| e.into())
                }),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    // Testing note: XML nodes cannot be guaranteed to be output in the same order every time
    // We therefore rely on `assert!(contains)` rather than `assert_eq!`
    use super::*;
    use std::str::FromStr;
    use thoth_api::model::Doi;
    use thoth_api::model::Isbn;
    use thoth_api::model::Orcid;
    use thoth_client::{
        ContributionType, CurrencyCode, LanguageCode, LanguageRelation, LocationPlatform,
        PublicationType, WorkContributionsContributor, WorkImprint, WorkImprintPublisher,
        WorkIssuesSeries, WorkPublicationsLocations, WorkPublicationsPrices, WorkStatus, WorkType,
    };
    use uuid::Uuid;

    fn generate_test_output(
        expect_ok: bool,
        input: &impl XmlElementBlock<Onix21EbscoHost>,
    ) -> String {
        // Helper function based on `XmlSpecification::generate`
        let mut buffer = Vec::new();
        let mut writer = xml::writer::EmitterConfig::new()
            .perform_indent(true)
            .create_writer(&mut buffer);
        let wrapped_output = XmlElementBlock::<Onix21EbscoHost>::xml_element(input, &mut writer)
            .map(|_| buffer)
            .and_then(|xml| {
                String::from_utf8(xml)
                    .map_err(|_| ThothError::InternalError("Could not parse XML".to_string()))
            });
        if expect_ok {
            assert!(wrapped_output.is_ok());
            wrapped_output.unwrap()
        } else {
            assert!(wrapped_output.is_err());
            wrapped_output.unwrap_err().to_string()
        }
    }

    #[test]
    fn test_onix21_ebsco_host_contributions() {
        let mut test_contribution = WorkContributions {
            contribution_type: ContributionType::AUTHOR,
            first_name: Some("Author".to_string()),
            last_name: "1".to_string(),
            full_name: "Author 1".to_string(),
            main_contribution: true,
            biography: None,
            contribution_ordinal: 1,
            contributor: WorkContributionsContributor {
                orcid: Some(Orcid::from_str("https://orcid.org/0000-0002-0000-0001").unwrap()),
                website: None,
            },
            affiliations: vec![],
        };

        // Test standard output
        let output = generate_test_output(true, &test_contribution);
        assert!(output.contains(r#"  <SequenceNumber>1</SequenceNumber>"#));
        assert!(output.contains(r#"  <ContributorRole>A01</ContributorRole>"#));
        assert!(output.contains(r#"  <PersonNameIdentifier>"#));
        assert!(output.contains(r#"    <PersonNameIDType>01</PersonNameIDType>"#));
        assert!(output.contains(r#"    <IDTypeName>ORCID</IDTypeName>"#));
        assert!(output.contains(r#"    <IDValue>0000-0002-0000-0001</IDValue>"#));
        assert!(output.contains(r#"  </PersonNameIdentifier>"#));
        // Given name is output as NamesBeforeKey and family name as KeyNames
        assert!(output.contains(r#"  <NamesBeforeKey>Author</NamesBeforeKey>"#));
        assert!(output.contains(r#"  <KeyNames>1</KeyNames>"#));
        // PersonName is not output when given name is supplied
        assert!(!output.contains(r#"  <PersonName>Author 1</PersonName>"#));

        // Change all possible values to test that output is updated
        test_contribution.contribution_type = ContributionType::EDITOR;
        test_contribution.contribution_ordinal = 2;
        test_contribution.contributor.orcid = None;
        test_contribution.first_name = None;
        let output = generate_test_output(true, &test_contribution);
        assert!(output.contains(r#"  <SequenceNumber>2</SequenceNumber>"#));
        assert!(output.contains(r#"  <ContributorRole>B01</ContributorRole>"#));
        // No ORCID supplied
        assert!(!output.contains(r#"  <PersonNameIdentifier>"#));
        assert!(!output.contains(r#"    <PersonNameIDType>01</PersonNameIDType>"#));
        assert!(!output.contains(r#"    <IDTypeName>ORCID</IDTypeName>"#));
        assert!(!output.contains(r#"    <IDValue>0000-0002-0000-0001</IDValue>"#));
        assert!(!output.contains(r#"  </PersonNameIdentifier>"#));
        // No given name supplied, so PersonName is output instead of KeyNames and NamesBeforeKey
        assert!(!output.contains(r#"  <NamesBeforeKey>Author</NamesBeforeKey>"#));
        assert!(!output.contains(r#"  <KeyNames>1</KeyNames>"#));
        assert!(output.contains(r#"  <PersonName>Author 1</PersonName>"#));

        // Test all remaining contributor roles
        test_contribution.contribution_type = ContributionType::TRANSLATOR;
        let output = generate_test_output(true, &test_contribution);
        assert!(output.contains(r#"  <ContributorRole>B06</ContributorRole>"#));
        test_contribution.contribution_type = ContributionType::PHOTOGRAPHER;
        let output = generate_test_output(true, &test_contribution);
        assert!(output.contains(r#"  <ContributorRole>A13</ContributorRole>"#));
        test_contribution.contribution_type = ContributionType::ILLUSTRATOR;
        let output = generate_test_output(true, &test_contribution);
        assert!(output.contains(r#"  <ContributorRole>A12</ContributorRole>"#));
        test_contribution.contribution_type = ContributionType::MUSIC_EDITOR;
        let output = generate_test_output(true, &test_contribution);
        assert!(output.contains(r#"  <ContributorRole>B25</ContributorRole>"#));
        test_contribution.contribution_type = ContributionType::FOREWORD_BY;
        let output = generate_test_output(true, &test_contribution);
        assert!(output.contains(r#"  <ContributorRole>A23</ContributorRole>"#));
        test_contribution.contribution_type = ContributionType::INTRODUCTION_BY;
        let output = generate_test_output(true, &test_contribution);
        assert!(output.contains(r#"  <ContributorRole>A24</ContributorRole>"#));
        test_contribution.contribution_type = ContributionType::AFTERWORD_BY;
        let output = generate_test_output(true, &test_contribution);
        assert!(output.contains(r#"  <ContributorRole>A19</ContributorRole>"#));
        test_contribution.contribution_type = ContributionType::PREFACE_BY;
        let output = generate_test_output(true, &test_contribution);
        assert!(output.contains(r#"  <ContributorRole>A15</ContributorRole>"#));
        test_contribution.contribution_type = ContributionType::SOFTWARE_BY;
        let output = generate_test_output(true, &test_contribution);
        assert!(output.contains(r#"  <ContributorRole>A30</ContributorRole>"#));
        test_contribution.contribution_type = ContributionType::CONTRIBUTIONS_BY;
        let output = generate_test_output(true, &test_contribution);
        assert!(output.contains(r#"  <ContributorRole>A32</ContributorRole>"#));
        test_contribution.contribution_type = ContributionType::INDEXER;
        let output = generate_test_output(true, &test_contribution);
        assert!(output.contains(r#"  <ContributorRole>A34</ContributorRole>"#));
    }

    #[test]
    fn test_onix21_ebsco_host_languages() {
        let mut test_language = WorkLanguages {
            language_code: LanguageCode::SPA,
            language_relation: LanguageRelation::TRANSLATED_FROM,
            main_language: true,
        };

        // Test standard output
        let output = generate_test_output(true, &test_language);
        assert!(output.contains(r#"<Language>"#));
        assert!(output.contains(r#"  <LanguageRole>02</LanguageRole>"#));
        assert!(output.contains(r#"  <LanguageCode>spa</LanguageCode>"#));

        // Change all possible values to test that output is updated
        test_language.language_code = LanguageCode::WEL;
        for language_relation in [
            LanguageRelation::ORIGINAL,
            LanguageRelation::TRANSLATED_INTO,
        ] {
            test_language.language_relation = language_relation;
            let output = generate_test_output(true, &test_language);
            assert!(output.contains(r#"<Language>"#));
            assert!(output.contains(r#"  <LanguageRole>01</LanguageRole>"#));
            assert!(output.contains(r#"  <LanguageCode>wel</LanguageCode>"#));
        }
    }

    #[test]
    fn test_onix21_ebsco_host_issues() {
        let mut test_issue = WorkIssues {
            issue_ordinal: 1,
            series: WorkIssuesSeries {
                series_id: Uuid::parse_str("00000000-0000-0000-BBBB-000000000002").unwrap(),
                series_type: thoth_client::SeriesType::JOURNAL,
                series_name: "Name of series".to_string(),
                issn_print: Some("1234-5678".to_string()),
                issn_digital: Some("8765-4321".to_string()),
                series_url: None,
                series_description: None,
                series_cfp_url: None,
            },
        };

        // Test standard output
        let output = generate_test_output(true, &test_issue);
        assert!(output.contains(r#"<Series>"#));
        assert!(output.contains(r#"  <TitleOfSeries>Name of series</TitleOfSeries>"#));
        assert!(output.contains(r#"  <NumberWithinSeries>1</NumberWithinSeries>"#));

        // Change all possible values to test that output is updated
        test_issue.issue_ordinal = 2;
        test_issue.series.series_name = "Different series".to_string();
        let output = generate_test_output(true, &test_issue);
        assert!(output.contains(r#"<Series>"#));
        assert!(output.contains(r#"  <TitleOfSeries>Different series</TitleOfSeries>"#));
        assert!(output.contains(r#"  <NumberWithinSeries>2</NumberWithinSeries>"#));
    }

    #[test]
    fn test_onix21_ebsco_host_subjects() {
        let mut test_subject = WorkSubjects {
            subject_code: "AAB".to_string(),
            subject_type: SubjectType::BIC,
            subject_ordinal: 1,
        };

        // Test BIC output
        let output = generate_test_output(true, &test_subject);
        assert!(output.contains(r#"<Subject>"#));
        assert!(output.contains(r#"  <SubjectSchemeIdentifier>12</SubjectSchemeIdentifier>"#));
        assert!(output.contains(r#"  <SubjectCode>AAB</SubjectCode>"#));

        // Test BISAC output
        test_subject.subject_code = "AAA000000".to_string();
        test_subject.subject_type = SubjectType::BISAC;
        let output = generate_test_output(true, &test_subject);
        assert!(output.contains(r#"<Subject>"#));
        assert!(output.contains(r#"  <SubjectSchemeIdentifier>10</SubjectSchemeIdentifier>"#));
        assert!(output.contains(r#"  <SubjectCode>AAA000000</SubjectCode>"#));

        // Test LCC output
        test_subject.subject_code = "JA85".to_string();
        test_subject.subject_type = SubjectType::LCC;
        let output = generate_test_output(true, &test_subject);
        assert!(output.contains(r#"<Subject>"#));
        assert!(output.contains(r#"  <SubjectSchemeIdentifier>04</SubjectSchemeIdentifier>"#));
        assert!(output.contains(r#"  <SubjectCode>JA85</SubjectCode>"#));

        // Test Thema output
        test_subject.subject_code = "JWA".to_string();
        test_subject.subject_type = SubjectType::THEMA;
        let output = generate_test_output(true, &test_subject);
        assert!(output.contains(r#"<Subject>"#));
        assert!(output.contains(r#"  <SubjectSchemeIdentifier>93</SubjectSchemeIdentifier>"#));
        assert!(output.contains(r#"  <SubjectCode>JWA</SubjectCode>"#));

        // Test keyword output
        test_subject.subject_code = "keyword1".to_string();
        test_subject.subject_type = SubjectType::KEYWORD;
        let output = generate_test_output(true, &test_subject);
        assert!(output.contains(r#"<Subject>"#));
        assert!(output.contains(r#"  <SubjectSchemeIdentifier>20</SubjectSchemeIdentifier>"#));
        assert!(output.contains(r#"  <SubjectHeadingText>keyword1</SubjectHeadingText>"#));

        // Test custom output
        test_subject.subject_code = "custom1".to_string();
        test_subject.subject_type = SubjectType::CUSTOM;
        let output = generate_test_output(true, &test_subject);
        assert!(output.contains(r#"  <SubjectSchemeIdentifier>B2</SubjectSchemeIdentifier>"#));
        assert!(output.contains(r#"  <SubjectHeadingText>custom1</SubjectHeadingText>"#));
    }

    #[test]
    fn test_onix21_ebsco_host_works() {
        let mut test_work = Work {
            work_id: Uuid::from_str("00000000-0000-0000-AAAA-000000000001").unwrap(),
            work_status: WorkStatus::ACTIVE,
            full_title: "Book Title: Book Subtitle".to_string(),
            title: "Book Title".to_string(),
            subtitle: Some("Book Subtitle".to_string()),
            work_type: WorkType::MONOGRAPH,
            reference: None,
            edition: Some(1),
            doi: Some(Doi::from_str("https://doi.org/10.00001/BOOK.0001").unwrap()),
            publication_date: chrono::NaiveDate::from_ymd_opt(1999, 12, 31),
            withdrawn_date: None,
            license: Some("https://creativecommons.org/licenses/by/4.0/".to_string()),
            copyright_holder: Some("Author 1; Author 2".to_string()),
            short_abstract: None,
            long_abstract: Some("Lorem ipsum dolor sit amet".to_string()),
            general_note: None,
            bibliography_note: None,
            place: Some("León, Spain".to_string()),
            page_count: Some(334),
            page_breakdown: None,
            first_page: None,
            last_page: None,
            page_interval: None,
            image_count: None,
            table_count: None,
            audio_count: None,
            video_count: None,
            landing_page: Some("https://www.book.com".to_string()),
            toc: None,
            lccn: None,
            oclc: None,
            cover_url: Some("https://www.book.com/cover".to_string()),
            cover_caption: None,
            imprint: WorkImprint {
                imprint_name: "OA Editions Imprint".to_string(),
                imprint_url: None,
                crossmark_doi: None,
                publisher: WorkImprintPublisher {
                    publisher_name: "OA Editions".to_string(),
                    publisher_shortname: Some(("OAE").to_string()),
                    publisher_url: Some("https://www.publisher.com".to_string()),
                },
            },
            issues: vec![],
            contributions: vec![],
            languages: vec![],
            publications: vec![
                WorkPublications {
                    publication_id: Uuid::from_str("00000000-0000-0000-AAAA-000000000001").unwrap(),
                    publication_type: PublicationType::EPUB,
                    isbn: Some(Isbn::from_str("978-92-95055-02-5").unwrap()),
                    width_mm: None,
                    width_cm: None,
                    width_in: None,
                    height_mm: None,
                    height_cm: None,
                    height_in: None,
                    depth_mm: None,
                    depth_cm: None,
                    depth_in: None,
                    weight_g: None,
                    weight_oz: None,
                    prices: vec![],
                    locations: vec![WorkPublicationsLocations {
                        landing_page: Some("https://www.book.com/epub_landing".to_string()),
                        full_text_url: Some("https://www.book.com/epub_fulltext".to_string()),
                        location_platform: LocationPlatform::OTHER,
                        canonical: true,
                    }],
                },
                WorkPublications {
                    publication_id: Uuid::from_str("00000000-0000-0000-DDDD-000000000004").unwrap(),
                    publication_type: PublicationType::PDF,
                    isbn: Some(Isbn::from_str("978-1-56619-909-4").unwrap()),
                    width_mm: None,
                    width_cm: None,
                    width_in: None,
                    height_mm: None,
                    height_cm: None,
                    height_in: None,
                    depth_mm: None,
                    depth_cm: None,
                    depth_in: None,
                    weight_g: None,
                    weight_oz: None,
                    prices: vec![],
                    locations: vec![WorkPublicationsLocations {
                        landing_page: Some("https://www.book.com/pdf_landing".to_string()),
                        full_text_url: Some("https://www.book.com/pdf_fulltext".to_string()),
                        location_platform: LocationPlatform::OTHER,
                        canonical: true,
                    }],
                },
                WorkPublications {
                    publication_id: Uuid::from_str("00000000-0000-0000-AAAA-000000000001").unwrap(),
                    publication_type: PublicationType::PAPERBACK,
                    isbn: Some(Isbn::from_str("978-3-16-148410-0").unwrap()),
                    width_mm: None,
                    width_cm: None,
                    width_in: None,
                    height_mm: None,
                    height_cm: None,
                    height_in: None,
                    depth_mm: None,
                    depth_cm: None,
                    depth_in: None,
                    weight_g: None,
                    weight_oz: None,
                    prices: vec![
                        WorkPublicationsPrices {
                            currency_code: CurrencyCode::EUR,
                            unit_price: 5.95,
                        },
                        WorkPublicationsPrices {
                            currency_code: CurrencyCode::GBP,
                            unit_price: 4.95,
                        },
                        WorkPublicationsPrices {
                            currency_code: CurrencyCode::USD,
                            unit_price: 7.99,
                        },
                    ],
                    locations: vec![],
                },
            ],
            subjects: vec![],
            fundings: vec![],
            relations: vec![],
            references: vec![],
        };

        // Test standard output
        let output = generate_test_output(true, &test_work);
        assert!(output.contains(r#"<Product>"#));
        assert!(output.contains(
            r#"  <RecordReference>urn:uuid:00000000-0000-0000-aaaa-000000000001</RecordReference>"#
        ));
        assert!(output.contains(r#"  <NotificationType>03</NotificationType>"#));
        assert!(output.contains(r#"  <RecordSourceType>01</RecordSourceType>"#));
        assert!(output.contains(r#"  <ProductIdentifier>"#));
        assert!(output.contains(r#"    <ProductIDType>01</ProductIDType>"#));
        assert!(output
            .contains(r#"    <IDValue>urn:uuid:00000000-0000-0000-aaaa-000000000001</IDValue>"#));
        assert!(output.contains(r#"    <ProductIDType>15</ProductIDType>"#));
        assert!(output.contains(r#"    <IDValue>9783161484100</IDValue>"#));
        assert!(output.contains(r#"    <ProductIDType>06</ProductIDType>"#));
        assert!(output.contains(r#"    <IDValue>10.00001/BOOK.0001</IDValue>"#));
        assert!(output.contains(r#"  <ProductForm>DG</ProductForm>"#));
        assert!(output.contains(r#"  <EpubType>002</EpubType>"#));
        assert!(output.contains(r#"  <Title>"#));
        assert!(output.contains(r#"    <TitleType>01</TitleType>"#));
        assert!(output.contains(r#"    <TitleText>Book Title</TitleText>"#));
        assert!(output.contains(r#"    <Subtitle>Book Subtitle</Subtitle>"#));
        assert!(output.contains(r#"  <WorkIdentifier>"#));
        assert!(output.contains(r#"    <WorkIDType>01</WorkIDType>"#));
        assert!(output.contains(r#"    <IDTypeName>Thoth WorkID</IDTypeName>"#));
        assert!(output
            .contains(r#"    <IDValue>urn:uuid:00000000-0000-0000-aaaa-000000000001</IDValue>"#));
        assert!(output.contains(r#"  <Website>"#));
        assert!(output.contains(r#"    <WebsiteRole>02</WebsiteRole>"#));
        assert!(output.contains(
            r#"    <WebsiteDescription>Publisher's website: web shop</WebsiteDescription>"#
        ));
        assert!(output.contains(r#"    <WebsiteLink>https://www.book.com</WebsiteLink>"#));
        assert!(output.contains(r#"    <WebsiteRole>29</WebsiteRole>"#));
        assert!(output.contains(r#"    <WebsiteDescription>Publisher's website: download the title</WebsiteDescription>"#));
        assert!(
            output.contains(r#"    <WebsiteLink>https://www.book.com/epub_fulltext</WebsiteLink>"#)
        );
        assert!(
            output.contains(r#"    <WebsiteLink>https://www.book.com/pdf_fulltext</WebsiteLink>"#)
        );
        assert!(output.contains(r#"  <Extent>"#));
        assert!(output.contains(r#"    <ExtentType>00</ExtentType>"#));
        assert!(output.contains(r#"    <ExtentValue>334</ExtentValue>"#));
        assert!(output.contains(r#"    <ExtentUnit>03</ExtentUnit>"#));
        assert!(output.contains(r#"  <Audience>"#));
        assert!(output.contains(r#"    <AudienceCodeType>01</AudienceCodeType>"#));
        assert!(output.contains(r#"    <AudienceCodeValue>06</AudienceCodeValue>"#));
        assert!(output.contains(r#"  <OtherText>"#));
        assert!(output.contains(r#"    <TextTypeCode>47</TextTypeCode>"#));
        assert!(output.contains(r#"    <Text>Open access - no commercial use</Text>"#));
        assert!(output.contains(r#"    <TextTypeCode>46</TextTypeCode>"#));
        assert!(output.contains(r#"    <Text>https://creativecommons.org/licenses/by/4.0/</Text>"#));
        assert!(output.contains(r#"    <TextTypeCode>03</TextTypeCode>"#));
        assert!(output.contains(r#"    <TextFormat>06</TextFormat>"#));
        assert!(output.contains(r#"    <Text>Lorem ipsum dolor sit amet</Text>"#));
        assert!(output.contains(r#"  <MediaFile>"#));
        assert!(output.contains(r#"    <MediaFileTypeCode>04</MediaFileTypeCode>"#));
        assert!(output.contains(r#"    <MediaFileLinkTypeCode>01</MediaFileLinkTypeCode>"#));
        assert!(output.contains(r#"    <MediaFileLink>https://www.book.com/cover</MediaFileLink>"#));
        assert!(output.contains(r#"  <Imprint>"#));
        assert!(output.contains(r#"    <ImprintName>OA Editions Imprint</ImprintName>"#));
        assert!(output.contains(r#"  <Publisher>"#));
        assert!(output.contains(r#"    <PublishingRole>01</PublishingRole>"#));
        assert!(output.contains(r#"    <PublisherName>OA Editions</PublisherName>"#));
        assert!(output.contains(r#"    <Website>"#));
        assert!(output.contains(r#"      <WebsiteLink>https://www.publisher.com</WebsiteLink>"#));
        assert!(output.contains(r#"  <CityOfPublication>León, Spain</CityOfPublication>"#));
        assert!(output.contains(r#"  <PublishingStatus>04</PublishingStatus>"#));
        assert!(output.contains(r#"  <PublicationDate>19991231</PublicationDate>"#));
        assert!(output.contains(r#"  <CopyrightYear>1999</CopyrightYear>"#));
        assert!(output.contains(r#"  <SalesRights>"#));
        assert!(output.contains(r#"    <SalesRightsType>02</SalesRightsType>"#));
        assert!(output.contains(r#"    <RightsTerritory>WORLD</RightsTerritory>"#));
        assert!(output.contains(r#"  <RelatedProduct>"#));
        assert!(output.contains(r#"    <RelationCode>06</RelationCode>"#));
        assert!(output.contains(r#"    <RelationCode>13</RelationCode>"#));
        assert!(output.contains(r#"    <ProductIdentifier>"#));
        assert!(output.contains(r#"      <ProductIDType>15</ProductIDType>"#));
        assert!(output.contains(r#"      <IDValue>9783161484100</IDValue>"#));
        assert!(output.contains(r#"      <IDValue>9781566199094</IDValue>"#));
        assert!(output.contains(r#"      <IDValue>9789295055025</IDValue>"#));
        assert!(output.contains(r#"  <SupplyDetail>"#));
        assert!(output.contains(r#"    <SupplierName>OA Editions</SupplierName>"#));
        assert!(output.contains(r#"    <SupplierRole>09</SupplierRole>"#));
        assert!(output.contains(r#"    <ProductAvailability>99</ProductAvailability>"#));
        assert!(output.contains(r#"    <AudienceRestrictionFlag>R</AudienceRestrictionFlag>"#));
        assert!(output
            .contains(r#"    <AudienceRestrictionNote>Open access</AudienceRestrictionNote>"#));
        assert!(output.contains(r#"    <Price>"#));
        assert!(output.contains(r#"      <PriceTypeCode>01</PriceTypeCode>"#));
        assert!(output.contains(r#"      <PriceAmount>0.01</PriceAmount>"#));
        assert!(output.contains(r#"      <CurrencyCode>USD</CurrencyCode>"#));

        // Add withdrawn date
        test_work.withdrawn_date = chrono::NaiveDate::from_ymd_opt(2020, 12, 31);
        let output = generate_test_output(true, &test_work);
        assert!(output.contains(r#"  <OutofPrintDate>20201231</OutofPrintDate>"#));

        // Remove some values to test non-output of optional blocks
        test_work.doi = None;
        test_work.license = None;
        test_work.subtitle = None;
        test_work.page_count = None;
        test_work.long_abstract = None;
        test_work.place = None;
        test_work.publication_date = None;
        test_work.license = None;
        test_work.landing_page = None;
        test_work.cover_url = None;
        test_work.imprint.publisher.publisher_url = None;
        // Remove third (paperback) publication
        test_work.publications.pop();
        // Give PDF publication a positive price point
        test_work.publications[1].prices = vec![WorkPublicationsPrices {
            currency_code: CurrencyCode::USD,
            unit_price: 7.99,
        }];
        let output = generate_test_output(true, &test_work);
        // Paperback publication removed, so its ISBN no longer appears
        // (either as the main ISBN or in RelatedProducts)
        assert!(!output.contains(r#"    <IDValue>9783161484100</IDValue>"#));
        // PDF ISBN is used as main ISBN instead
        assert!(output.contains(r#"    <IDValue>9781566199094</IDValue>"#));
        // No DOI supplied
        assert!(!output.contains(r#"    <ProductIDType>06</ProductIDType>"#));
        assert!(!output.contains(r#"    <IDValue>10.00001/BOOK.0001</IDValue>"#));
        // No subtitle supplied (within Thoth UI this would automatically update full_title)
        assert!(!output.contains(r#"    <Subtitle>Book Subtitle</Subtitle>"#));
        // No landing page supplied
        assert!(!output.contains(r#"    <WebsiteRole>02</WebsiteRole>"#));
        assert!(!output.contains(
            r#"    <WebsiteDescription>Publisher's website: web shop</WebsiteDescription>"#
        ));
        assert!(!output.contains(r#"    <WebsiteLink>https://www.book.com</WebsiteLink>"#));
        // PDF publication is no longer unpriced, hence no PDF URL, and EpubType changes
        assert!(
            !output.contains(r#"    <WebsiteLink>https://www.book.com/pdf_fulltext</WebsiteLink>"#)
        );
        assert!(!output.contains(r#"  <EpubType>002</EpubType>"#));
        assert!(output.contains(r#"  <EpubType>029</EpubType>"#));
        // No page count supplied
        assert!(!output.contains(r#"  <Extent>"#));
        assert!(!output.contains(r#"    <ExtentType>00</ExtentType>"#));
        assert!(!output.contains(r#"    <ExtentValue>334</ExtentValue>"#));
        assert!(!output.contains(r#"    <ExtentUnit>03</ExtentUnit>"#));
        // No long abstract supplied
        assert!(!output.contains(r#"    <TextTypeCode>03</TextTypeCode>"#));
        assert!(!output.contains(r#"    <TextFormat>06</TextFormat>"#));
        assert!(!output.contains(r#"    <Text>Lorem ipsum dolor sit amet</Text>"#));
        // No licence supplied
        assert!(!output.contains(r#"    <TextTypeCode>46</TextTypeCode>"#));
        assert!(
            !output.contains(r#"    <Text>https://creativecommons.org/licenses/by/4.0/</Text>"#)
        );
        // No cover URL supplied
        assert!(!output.contains(r#"  <MediaFile>"#));
        assert!(!output.contains(r#"    <MediaFileTypeCode>04</MediaFileTypeCode>"#));
        assert!(!output.contains(r#"    <MediaFileLinkTypeCode>01</MediaFileLinkTypeCode>"#));
        assert!(
            !output.contains(r#"    <MediaFileLink>https://www.book.com/cover</MediaFileLink>"#)
        );
        // No publisher website supplied (and no other <Website> blocks remain)
        assert!(!output.contains(r#"    <Website>"#));
        assert!(!output.contains(r#"      <WebsiteLink>https://www.publisher.com</WebsiteLink>"#));
        // No place supplied
        assert!(!output.contains(r#"  <CityOfPublication>León, Spain</CityOfPublication>"#));
        // No publication date supplied
        assert!(!output.contains(r#"  <PublicationDate>19991231</PublicationDate>"#));
        assert!(!output.contains(r#"  <CopyrightYear>1999</CopyrightYear>"#));

        // Remove the EPUB publication's only location: error
        test_work.publications[0].locations.clear();
        let output = generate_test_output(false, &test_work);
        assert_eq!(
            output,
            "Could not generate onix_2.1::ebsco_host: No unpriced PDF or EPUB URL".to_string()
        );
    }
}
