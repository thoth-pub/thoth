use chrono::Utc;
use std::io::Write;
use thoth_client::{
    AbstractType, ContributionType, CurrencyCode, LanguageRelation, PublicationType, SubjectType,
    Work, WorkContributions, WorkIssues, WorkLanguages, WorkPublications, WorkStatus, WorkType,
};
use xml::writer::{EventWriter, XmlEvent};

use super::{write_element_block, XmlElement, XmlSpecification};
use crate::xml::{write_full_element_block, XmlElementBlock, ONIX3_NS};
use thoth_errors::{ThothError, ThothResult};

#[derive(Copy, Clone)]
pub struct Onix3GoogleBooks {}

const ONIX_ERROR: &str = "onix_3.0::google_books";

// Output format based on documentation at https://support.google.com/books/partner/answer/6374180.
impl XmlSpecification for Onix3GoogleBooks {
    fn handle_event<W: Write>(w: &mut EventWriter<W>, works: &[Work]) -> ThothResult<()> {
        write_full_element_block("ONIXMessage", Some(ONIX3_NS.to_vec()), w, |w| {
            write_element_block("Header", w, |w| {
                write_element_block("Sender", w, |w| {
                    write_element_block("SenderName", w, |w| {
                        w.write(XmlEvent::Characters("Thoth")).map_err(|e| e.into())
                    })?;
                    write_element_block("EmailAddress", w, |w| {
                        w.write(XmlEvent::Characters("distribution@thoth.pub"))
                            .map_err(|e| e.into())
                    })
                })?;
                write_element_block("Addressee", w, |w| {
                    write_element_block("AddresseeName", w, |w| {
                        w.write(XmlEvent::Characters("Google"))
                            .map_err(|e| e.into())
                    })
                })?;
                write_element_block("SentDateTime", w, |w| {
                    w.write(XmlEvent::Characters(
                        &Utc::now().format("%Y%m%dT%H%M%S").to_string(),
                    ))
                    .map_err(|e| e.into())
                })
            })?;

            match works {
                [] => Err(ThothError::IncompleteMetadataRecord(
                    ONIX_ERROR.to_string(),
                    "Not enough data".to_string(),
                )),
                [work] => XmlElementBlock::<Onix3GoogleBooks>::xml_element(work, w),
                _ => {
                    for work in works.iter() {
                        // Do not include Chapters in full publisher metadata record
                        // (assumes that a publisher will always have more than one work)
                        if work.work_type != WorkType::BOOK_CHAPTER {
                            XmlElementBlock::<Onix3GoogleBooks>::xml_element(work, w).ok();
                        }
                    }
                    Ok(())
                }
            }
        })
    }
}

impl XmlElementBlock<Onix3GoogleBooks> for Work {
    fn xml_element<W: Write>(&self, w: &mut EventWriter<W>) -> ThothResult<()> {
        // Don't output works with no BIC, BISAC or LCC subject code
        // Google Books can only ingest works which have at least one
        if !self.subjects.iter().any(|s| {
            matches!(
                s.subject_type,
                SubjectType::BISAC | SubjectType::BIC | SubjectType::LCC
            )
        }) {
            return Err(ThothError::IncompleteMetadataRecord(
                ONIX_ERROR.to_string(),
                "No BIC, BISAC or LCC subject code".to_string(),
            ));
        }
        // Don't output works with no publication date (mandatory in Google Books)
        if self.publication_date.is_none() {
            return Err(ThothError::IncompleteMetadataRecord(
                ONIX_ERROR.to_string(),
                "Missing Publication Date".to_string(),
            ));
        }
        // Don't output works with no contributors (at least one required for Google Books)
        if self.contributions.is_empty() {
            return Err(ThothError::IncompleteMetadataRecord(
                ONIX_ERROR.to_string(),
                "No contributors supplied".to_string(),
            ));
        }
        // We can only generate the document if there's an EPUB or PDF
        if let Some(main_publication) = self
            .publications
            .iter()
            // For preference, distribute the EPUB only
            .find(|p| {
                p.publication_type.eq(&PublicationType::EPUB)
                    && p.locations
                        .iter()
                        .any(|l| l.canonical && l.full_text_url.is_some())
            })
            // If no EPUB is found, distribute the PDF only
            .or_else(|| {
                self.publications.iter().find(|p| {
                    p.publication_type.eq(&PublicationType::PDF)
                        && p.locations
                            .iter()
                            .any(|l| l.canonical && l.full_text_url.is_some())
                })
            })
        {
            let (main_isbn, isbns) = get_publications_data(&self.publications, main_publication);
            if main_isbn.is_empty() {
                // Google Books requires at least one ProductIdentifier block with an ISBN type
                return Err(ThothError::IncompleteMetadataRecord(
                    ONIX_ERROR.to_string(),
                    "This work does not have a PDF, EPUB or paperback ISBN".to_string(),
                ));
            }
            write_element_block("Product", w, |w| {
                write_element_block("RecordReference", w, |w| {
                    w.write(XmlEvent::Characters(&format!("urn:uuid:{}", self.work_id)))
                        .map_err(|e| e.into())
                })?;
                // 03 Notification confirmed on publication
                write_element_block("NotificationType", w, |w| {
                    w.write(XmlEvent::Characters("03")).map_err(|e| e.into())
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
                write_element_block("DescriptiveDetail", w, |w| {
                    // 00 Single-component retail product
                    write_element_block("ProductComposition", w, |w| {
                        w.write(XmlEvent::Characters("00")).map_err(|e| e.into())
                    })?;
                    // EB Digital download and online
                    write_element_block("ProductForm", w, |w| {
                        w.write(XmlEvent::Characters("EB")).map_err(|e| e.into())
                    })?;
                    let digital_type = match main_publication.publication_type {
                        PublicationType::EPUB => "E101",
                        PublicationType::PDF => "E107",
                        _ => unreachable!(),
                    };
                    write_element_block("ProductFormDetail", w, |w| {
                        w.write(XmlEvent::Characters(digital_type))
                            .map_err(|e| e.into())
                    })?;
                    // 10 Text (eye-readable)
                    write_element_block("PrimaryContentType", w, |w| {
                        w.write(XmlEvent::Characters("10")).map_err(|e| e.into())
                    })?;
                    for issue in &self.issues {
                        XmlElementBlock::<Onix3GoogleBooks>::xml_element(issue, w).ok();
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
                            write_element_block("TitleText", w, |w| {
                                w.write(XmlEvent::Characters(&self.titles[0].title))
                                    .map_err(|e| e.into())
                            })?;
                            if let Some(subtitle) = &self.titles[0].subtitle {
                                write_element_block("Subtitle", w, |w| {
                                    w.write(XmlEvent::Characters(subtitle))
                                        .map_err(|e| e.into())
                                })?;
                            }
                            Ok(())
                        })
                    })?;
                    // Google Books requires at least one contributor coded as A01 (Author) -
                    // if this is e.g. a wholly edited book, code the first main contributor as an author.
                    let mut contributions = self.contributions.clone();
                    if !contributions
                        .iter()
                        .any(|c| c.contribution_type.eq(&ContributionType::AUTHOR))
                    {
                        // WorkQuery should already have retrieved these sorted by ordinal, but sort again for safety
                        contributions
                            .sort_by(|a, b| a.contribution_ordinal.cmp(&b.contribution_ordinal));
                        contributions.sort_by(|a, b| b.main_contribution.cmp(&a.main_contribution));
                        contributions[0].contribution_type = ContributionType::AUTHOR;
                    }
                    for contribution in &contributions {
                        // Google Books doesn't support B25, A30, A34 or A51 codes
                        // (or any appropriate "Other" code)
                        match contribution.contribution_type {
                            ContributionType::SOFTWARE_BY
                            | ContributionType::RESEARCH_BY
                            | ContributionType::INDEXER
                            | ContributionType::MUSIC_EDITOR => (),
                            _ => {
                                XmlElementBlock::<Onix3GoogleBooks>::xml_element(contribution, w)
                                    .ok();
                            }
                        }
                    }
                    for language in &self.languages {
                        XmlElementBlock::<Onix3GoogleBooks>::xml_element(language, w).ok();
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
                        // Google Books doesn't support Thema codes
                        if subject.subject_type != SubjectType::THEMA {
                            write_element_block("Subject", w, |w| {
                                XmlElement::<Onix3GoogleBooks>::xml_element(
                                    &subject.subject_type,
                                    w,
                                )?;
                                write_element_block("SubjectCode", w, |w| {
                                    w.write(XmlEvent::Characters(&subject.subject_code))
                                        .map_err(|e| e.into())
                                })
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
                    })
                })?;
                if self
                    .abstracts
                    .iter()
                    .any(|a| a.abstract_type == AbstractType::LONG && a.canonical)
                    || self.toc.is_some()
                {
                    write_element_block("CollateralDetail", w, |w| {
                        if let Some(labstract) = &self
                            .abstracts
                            .iter()
                            .find(|a| a.abstract_type == AbstractType::LONG && a.canonical)
                            .map(|a| a.content.clone())
                        {
                            write_element_block("TextContent", w, |w| {
                                // 03 Description ("30 Abstract" not implemented in Google Books)
                                write_element_block("TextType", w, |w| {
                                    w.write(XmlEvent::Characters("03")).map_err(|e| e.into())
                                })?;
                                // 00 Unrestricted
                                write_element_block("ContentAudience", w, |w| {
                                    w.write(XmlEvent::Characters("00")).map_err(|e| e.into())
                                })?;
                                write_full_element_block(
                                    "Text",
                                    Some(vec![("language", "eng")]),
                                    w,
                                    |w| {
                                        w.write(XmlEvent::Characters(labstract))
                                            .map_err(|e| e.into())
                                    },
                                )
                            })?;
                        }
                        if let Some(toc) = &self.toc {
                            write_element_block("TextContent", w, |w| {
                                // 04 Table of contents
                                write_element_block("TextType", w, |w| {
                                    w.write(XmlEvent::Characters("04")).map_err(|e| e.into())
                                })?;
                                // 00 Unrestricted
                                write_element_block("ContentAudience", w, |w| {
                                    w.write(XmlEvent::Characters("00")).map_err(|e| e.into())
                                })?;
                                write_full_element_block(
                                    "Text",
                                    Some(vec![("language", "eng")]),
                                    w,
                                    |w| w.write(XmlEvent::Characters(toc)).map_err(|e| e.into()),
                                )
                            })?;
                        }
                        Ok(())
                    })?;
                }
                // Google Books also supports <ContentDetail> blocks for chapter information.
                // Omitted at present but could be considered as a future enhancement.
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
                    XmlElement::<Onix3GoogleBooks>::xml_element(&self.work_status, w)?;
                    write_element_block("PublishingDate", w, |w| {
                        write_element_block("PublishingDateRole", w, |w| {
                            // 01 Publishing Date (19 not supported by Google Books)
                            w.write(XmlEvent::Characters("01")).map_err(|e| e.into())
                        })?;
                        // dateformat="00" YYYYMMDD
                        write_full_element_block("Date", Some(vec![("dateformat", "00")]), w, |w| {
                            w.write(XmlEvent::Characters(
                                &self.publication_date.unwrap().format("%Y%m%d").to_string(),
                            ))
                            .map_err(|e| e.into())
                        })
                    })?;
                    if let Some(date) = &self.withdrawn_date {
                        write_element_block("PublishingDate", w, |w| {
                            write_element_block("PublishingDateRole", w, |w| {
                                // 13 Out-of-print / permanently withdrawn date
                                w.write(XmlEvent::Characters("13")).map_err(|e| e.into())
                            })?;
                            // dateformat="00" YYYYMMDD
                            write_full_element_block(
                                "Date",
                                Some(vec![("dateformat", "00")]),
                                w,
                                |w| {
                                    w.write(XmlEvent::Characters(
                                        &date.format("%Y%m%d").to_string(),
                                    ))
                                    .map_err(|e| e.into())
                                },
                            )
                        })?;
                    }
                    write_element_block("SalesRights", w, |w| {
                        // 02 For sale with non-exclusive rights in the specified countries or territories
                        write_element_block("SalesRightsType", w, |w| {
                            w.write(XmlEvent::Characters("02")).map_err(|e| e.into())
                        })?;
                        write_element_block("Territory", w, |w| {
                            write_element_block("RegionsIncluded", w, |w| {
                                w.write(XmlEvent::Characters("WORLD")).map_err(|e| e.into())
                            })
                        })
                    })
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
                                        w.write(XmlEvent::Characters(isbn)).map_err(|e| e.into())
                                    })
                                })
                            })?;
                        }
                        Ok(())
                    })?;
                }
                write_element_block("ProductSupply", w, |w| {
                    write_element_block("Market", w, |w| {
                        write_element_block("Territory", w, |w| {
                            write_element_block("RegionsIncluded", w, |w| {
                                w.write(XmlEvent::Characters("WORLD")).map_err(|e| e.into())
                            })
                        })
                    })?;
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
                            })
                        })?;
                        // 20 Available from us (form of availability unspecified)
                        // (99 Contact supplier is not supported by Google Books)
                        write_element_block("ProductAvailability", w, |w| {
                            w.write(XmlEvent::Characters("20")).map_err(|e| e.into())
                        })?;
                        // Assume that the GBP price is the canonical one, currency conversion is
                        // turned on (a Google Books account setting which cannot be specified in the ONIX),
                        // and all other prices will be automatically derived from the GBP price.
                        if let Some(price) = main_publication
                            .prices
                            .iter()
                            .find(|pr| {
                                // Thoth database only accepts non-zero prices
                                pr.currency_code.eq(&CurrencyCode::GBP)
                            })
                            .map(|pr| pr.unit_price)
                        {
                            let formatted_price = format!("{price:.2}");
                            write_element_block("Price", w, |w| {
                                // 02 RRP including tax
                                write_element_block("PriceType", w, |w| {
                                    w.write(XmlEvent::Characters("02")).map_err(|e| e.into())
                                })?;
                                write_element_block("PriceAmount", w, |w| {
                                    w.write(XmlEvent::Characters(&formatted_price))
                                        .map_err(|e| e.into())
                                })?;
                                write_element_block("CurrencyCode", w, |w| {
                                    w.write(XmlEvent::Characters("GBP")).map_err(|e| e.into())
                                })?;
                                write_element_block("Territory", w, |w| {
                                    write_element_block("RegionsIncluded", w, |w| {
                                        w.write(XmlEvent::Characters("WORLD")).map_err(|e| e.into())
                                    })
                                })
                            })
                        } else {
                            // 01 Free of charge (this is the only UnpricedItemType code supported by Google Books)
                            write_element_block("UnpricedItemType", w, |w| {
                                w.write(XmlEvent::Characters("01")).map_err(|e| e.into())
                            })
                        }
                    })
                })
            })
        } else {
            Err(ThothError::IncompleteMetadataRecord(
                ONIX_ERROR.to_string(),
                "Missing EPUB or PDF URL".to_string(),
            ))
        }
    }
}

fn get_publications_data(
    publications: &[WorkPublications],
    main_publication: &WorkPublications,
) -> (String, Vec<String>) {
    let mut main_isbn = "".to_string();
    let mut isbns: Vec<String> = Vec::new();

    for publication in publications {
        if let Some(isbn) = &publication.isbn.as_ref() {
            isbns.push(isbn.to_hyphenless_string());
            // The default product ISBN is the main publication's (EPUB or PDF)
            if publication
                .publication_id
                .eq(&main_publication.publication_id)
            {
                main_isbn = isbn.to_hyphenless_string();
            }
            // If the main publication has no ISBN, use either the PDF's or the paperback's
            // (no guarantee as to which will be chosen)
            if (publication.publication_type.eq(&PublicationType::PDF)
                || publication.publication_type.eq(&PublicationType::PAPERBACK))
                && main_isbn.is_empty()
            {
                main_isbn = isbn.to_hyphenless_string();
            }
        }
    }

    (main_isbn, isbns)
}

impl XmlElement<Onix3GoogleBooks> for WorkStatus {
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

impl XmlElement<Onix3GoogleBooks> for SubjectType {
    const ELEMENT: &'static str = "SubjectSchemeIdentifier";

    fn value(&self) -> &'static str {
        match self {
            SubjectType::BIC => "12",
            SubjectType::BISAC => "10",
            SubjectType::LCC => "04",
            // 23 Publisher's own category code
            SubjectType::KEYWORD | SubjectType::CUSTOM => "23",
            // Thema codes are not output for Google Books
            SubjectType::THEMA | SubjectType::Other(_) => unreachable!(),
        }
    }
}

impl XmlElement<Onix3GoogleBooks> for LanguageRelation {
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

impl XmlElement<Onix3GoogleBooks> for ContributionType {
    const ELEMENT: &'static str = "ContributorRole";

    fn value(&self) -> &'static str {
        match self {
            ContributionType::AUTHOR => "A01",
            ContributionType::EDITOR => "B01",
            ContributionType::TRANSLATOR => "B06",
            ContributionType::PHOTOGRAPHER => "A13",
            ContributionType::ILLUSTRATOR => "A12",
            ContributionType::FOREWORD_BY => "A23",
            ContributionType::INTRODUCTION_BY => "A24",
            ContributionType::AFTERWORD_BY => "A19",
            ContributionType::PREFACE_BY => "A15",
            ContributionType::CONTRIBUTIONS_BY => "A32",
            // B25, A30, A34, and A51 are not output for Google Books
            ContributionType::SOFTWARE_BY
            | ContributionType::RESEARCH_BY
            | ContributionType::INDEXER
            | ContributionType::MUSIC_EDITOR
            | ContributionType::Other(_) => unreachable!(),
        }
    }
}

impl XmlElementBlock<Onix3GoogleBooks> for WorkContributions {
    fn xml_element<W: Write>(&self, w: &mut EventWriter<W>) -> ThothResult<()> {
        write_element_block("Contributor", w, |w| {
            write_element_block("SequenceNumber", w, |w| {
                w.write(XmlEvent::Characters(&self.contribution_ordinal.to_string()))
                    .map_err(|e| e.into())
            })?;
            XmlElement::<Onix3GoogleBooks>::xml_element(&self.contribution_type, w)?;
            write_element_block("PersonName", w, |w| {
                w.write(XmlEvent::Characters(&self.full_name))
                    .map_err(|e| e.into())
            })?;
            if !&self.biographies.is_empty() {
                let biography = &self.biographies[0].content.clone();
                write_element_block("BiographicalNote", w, |w| {
                    w.write(XmlEvent::Characters(biography))
                        .map_err(|e| e.into())
                })?;
            }
            Ok(())
        })
    }
}

impl XmlElementBlock<Onix3GoogleBooks> for WorkLanguages {
    fn xml_element<W: Write>(&self, w: &mut EventWriter<W>) -> ThothResult<()> {
        write_element_block("Language", w, |w| {
            XmlElement::<Onix3GoogleBooks>::xml_element(&self.language_relation, w).ok();
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

impl XmlElementBlock<Onix3GoogleBooks> for WorkIssues {
    fn xml_element<W: Write>(&self, w: &mut EventWriter<W>) -> ThothResult<()> {
        write_element_block("Collection", w, |w| {
            // 10 Publisher collection (e.g. series)
            write_element_block("CollectionType", w, |w| {
                w.write(XmlEvent::Characters("10")).map_err(|e| e.into())
            })?;
            if let Some(issn_digital) = &self.series.issn_digital {
                write_element_block("CollectionIdentifier", w, |w| {
                    // 02 ISSN
                    write_element_block("CollectionIDType", w, |w| {
                        w.write(XmlEvent::Characters("02")).map_err(|e| e.into())
                    })?;
                    write_element_block("IDValue", w, |w| {
                        w.write(XmlEvent::Characters(
                            &issn_digital.as_str().replace('-', ""),
                        ))
                        .map_err(|e| e.into())
                    })
                })?;
            }
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
                        w.write(XmlEvent::Characters(&self.issue_ordinal.to_string()))
                            .map_err(|e| e.into())
                    })?;
                    write_element_block("TitleText", w, |w| {
                        w.write(XmlEvent::Characters(&self.series.series_name))
                            .map_err(|e| e.into())
                    })
                })
            })
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
    use thoth_client::WorkContributionsBiographies;
    use thoth_client::{
        ContributionType, LanguageCode, LanguageRelation, LocationPlatform, PublicationType,
        WorkContributionsContributor, WorkImprint, WorkImprintPublisher, WorkIssuesSeries,
        WorkPublicationsLocations, WorkPublicationsPrices, WorkStatus, WorkSubjects, WorkType,
    };
    use uuid::Uuid;

    fn generate_test_output(
        expect_ok: bool,
        input: &impl XmlElementBlock<Onix3GoogleBooks>,
    ) -> String {
        // Helper function based on `XmlSpecification::generate`
        let mut buffer = Vec::new();
        let mut writer = xml::writer::EmitterConfig::new()
            .perform_indent(true)
            .create_writer(&mut buffer);
        let wrapped_output = XmlElementBlock::<Onix3GoogleBooks>::xml_element(input, &mut writer)
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
    fn test_onix3_google_books_contributions() {
        let mut test_contribution = WorkContributions {
            contribution_type: ContributionType::AUTHOR,
            first_name: Some("Author".to_string()),
            last_name: "1".to_string(),
            full_name: "Author 1".to_string(),
            main_contribution: true,
            biographies: vec![WorkContributionsBiographies {
                biography_id: Uuid::parse_str("00000000-0000-0000-AAAA-000000000001").unwrap(),
                contribution_id: Uuid::parse_str("00000000-0000-0000-CCCC-000000000003").unwrap(),
                content: "Author 1 is an author of books".to_string(),
                canonical: true,
                locale_code: thoth_client::LocaleCode::EN,
            }],
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
        assert!(output.contains(r#"  <PersonName>Author 1</PersonName>"#));
        assert!(output
            .contains(r#"  <BiographicalNote>Author 1 is an author of books</BiographicalNote>"#));

        // Change all possible values to test that output is updated
        test_contribution.contribution_type = ContributionType::EDITOR;
        test_contribution.contribution_ordinal = 2;
        test_contribution.biographies = vec![];
        test_contribution.first_name = None;
        let output = generate_test_output(true, &test_contribution);
        assert!(output.contains(r#"  <SequenceNumber>2</SequenceNumber>"#));
        assert!(output.contains(r#"  <ContributorRole>B01</ContributorRole>"#));
        // PersonName is always output regardless of whether given name is supplied
        assert!(output.contains(r#"  <PersonName>Author 1</PersonName>"#));
        // No biography supplied
        assert!(!output
            .contains(r#"  <BiographicalNote>Author 1 is an author of books</BiographicalNote>"#));

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
        // Don't test Music Editor as the Work logic prevents us from entering
        // the Contributions routine for them - see Work test instead
    }

    #[test]
    fn test_onix3_google_books_languages() {
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
    fn test_onix3_google_books_issues() {
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
        assert!(output.contains(r#"<Collection>"#));
        assert!(output.contains(r#"  <CollectionType>10</CollectionType>"#));
        assert!(output.contains(r#"  <CollectionIdentifier>"#));
        assert!(output.contains(r#"    <CollectionIDType>02</CollectionIDType>"#));
        assert!(output.contains(r#"    <IDValue>87654321</IDValue>"#));
        assert!(output.contains(r#"  <TitleDetail>"#));
        assert!(output.contains(r#"    <TitleType>01</TitleType>"#));
        assert!(output.contains(r#"    <TitleElement>"#));
        assert!(output.contains(r#"      <TitleElementLevel>02</TitleElementLevel>"#));
        assert!(output.contains(r#"      <PartNumber>1</PartNumber>"#));
        assert!(output.contains(r#"      <TitleText>Name of series</TitleText>"#));

        // Change all possible values to test that output is updated
        test_issue.issue_ordinal = 2;
        test_issue.series.series_name = "Different series".to_string();
        test_issue.series.issn_digital = Some("1111-2222".to_string());
        let output = generate_test_output(true, &test_issue);
        assert!(output.contains(r#"<Collection>"#));
        assert!(output.contains(r#"  <CollectionType>10</CollectionType>"#));
        assert!(output.contains(r#"  <CollectionIdentifier>"#));
        assert!(output.contains(r#"    <CollectionIDType>02</CollectionIDType>"#));
        assert!(output.contains(r#"    <IDValue>11112222</IDValue>"#));
        assert!(output.contains(r#"  <TitleDetail>"#));
        assert!(output.contains(r#"    <TitleType>01</TitleType>"#));
        assert!(output.contains(r#"    <TitleElement>"#));
        assert!(output.contains(r#"      <TitleElementLevel>02</TitleElementLevel>"#));
        assert!(output.contains(r#"      <PartNumber>2</PartNumber>"#));
        assert!(output.contains(r#"      <TitleText>Different series</TitleText>"#));
    }

    #[test]
    fn test_onix3_google_books_works() {
        let mut test_work = Work {
            work_id: Uuid::from_str("00000000-0000-0000-AAAA-000000000001").unwrap(),
            work_status: WorkStatus::ACTIVE,
            titles: vec![thoth_client::WorkTitles {
                title_id: Uuid::from_str("00000000-0000-0000-CCCC-000000000001").unwrap(),
                locale_code: thoth_client::LocaleCode::EN,
                full_title: "Book Title: Book Subtitle".to_string(),
                title: "Book Title".to_string(),
                subtitle: Some("Book Subtitle".to_string()),
                canonical: true,
            }],
            abstracts: vec![thoth_client::WorkAbstracts {
                abstract_id: Uuid::from_str("00000000-0000-0000-AAAA-000000000001").unwrap(),
                work_id: Uuid::from_str("00000000-0000-0000-AAAA-000000000001").unwrap(),
                content: "Lorem ipsum dolor sit amet, consectetur adipiscing elit".to_string(),
                locale_code: thoth_client::LocaleCode::EN,
                abstract_type: thoth_client::AbstractType::LONG,
                canonical: true,
            }],
            work_type: WorkType::MONOGRAPH,
            reference: None,
            edition: Some(1),
            doi: Some(Doi::from_str("https://doi.org/10.00001/BOOK.0001").unwrap()),
            publication_date: chrono::NaiveDate::from_ymd_opt(1999, 12, 31),
            withdrawn_date: None,
            license: Some("https://creativecommons.org/licenses/by/4.0/".to_string()),
            copyright_holder: Some("Author 1; Author 2".to_string()),
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
            toc: Some("1. Chapter 1".to_string()),
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
                    publisher_shortname: Some("OAE".to_string()),
                    publisher_url: None,
                },
            },
            issues: vec![],
            contributions: vec![
                WorkContributions {
                    contribution_type: ContributionType::MUSIC_EDITOR,
                    first_name: Some("Music".to_string()),
                    last_name: "Editor".to_string(),
                    full_name: "Music Editor".to_string(),
                    main_contribution: false,
                    biographies: vec![],
                    contribution_ordinal: 1,
                    contributor: WorkContributionsContributor {
                        orcid: None,
                        website: None,
                    },
                    affiliations: vec![],
                },
                WorkContributions {
                    contribution_type: ContributionType::EDITOR,
                    first_name: Some("Volume".to_string()),
                    last_name: "Editor".to_string(),
                    full_name: "Volume Editor".to_string(),
                    main_contribution: true,
                    biographies: vec![],
                    contribution_ordinal: 2,
                    contributor: WorkContributionsContributor {
                        orcid: None,
                        website: None,
                    },
                    affiliations: vec![],
                },
            ],
            languages: vec![],
            publications: vec![WorkPublications {
                publication_id: Uuid::from_str("00000000-0000-0000-DDDD-000000000004").unwrap(),
                publication_type: PublicationType::PDF,
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
                        currency_code: CurrencyCode::USD,
                        unit_price: 7.99,
                    },
                    WorkPublicationsPrices {
                        currency_code: CurrencyCode::GBP,
                        unit_price: 5.0,
                    },
                ],
                locations: vec![WorkPublicationsLocations {
                    landing_page: Some("https://www.book.com/ebook_landing".to_string()),
                    full_text_url: Some("https://www.book.com/ebook_fulltext".to_string()),
                    location_platform: LocationPlatform::OTHER,
                    canonical: true,
                }],
            }],
            subjects: vec![
                WorkSubjects {
                    subject_code: "AAB".to_string(),
                    subject_type: SubjectType::BIC,
                    subject_ordinal: 1,
                },
                WorkSubjects {
                    subject_code: "AAA000000".to_string(),
                    subject_type: SubjectType::BISAC,
                    subject_ordinal: 2,
                },
                WorkSubjects {
                    subject_code: "JA85".to_string(),
                    subject_type: SubjectType::LCC,
                    subject_ordinal: 3,
                },
                WorkSubjects {
                    subject_code: "JWA".to_string(),
                    subject_type: SubjectType::THEMA,
                    subject_ordinal: 4,
                },
                WorkSubjects {
                    subject_code: "keyword1".to_string(),
                    subject_type: SubjectType::KEYWORD,
                    subject_ordinal: 5,
                },
                WorkSubjects {
                    subject_code: "custom1".to_string(),
                    subject_type: SubjectType::CUSTOM,
                    subject_ordinal: 6,
                },
            ],
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
        assert!(output.contains(r#"  <ProductIdentifier>"#));
        assert!(output.contains(r#"    <ProductIDType>15</ProductIDType>"#));
        assert!(output.contains(r#"    <IDValue>9783161484100</IDValue>"#));
        assert!(output.contains(r#"  <DescriptiveDetail>"#));
        assert!(output.contains(r#"    <ProductComposition>00</ProductComposition>"#));
        assert!(output.contains(r#"    <ProductForm>EB</ProductForm>"#));
        assert!(output.contains(r#"    <ProductFormDetail>E107</ProductFormDetail>"#));
        assert!(output.contains(r#"    <PrimaryContentType>10</PrimaryContentType>"#));
        assert!(output.contains(r#"    <TitleDetail>"#));
        assert!(output.contains(r#"      <TitleType>01</TitleType>"#));
        assert!(output.contains(r#"      <TitleElement>"#));
        assert!(output.contains(r#"        <TitleElementLevel>01</TitleElementLevel>"#));
        assert!(output.contains(r#"        <TitleText>Book Title</TitleText>"#));
        assert!(output.contains(r#"        <Subtitle>Book Subtitle</Subtitle>"#));
        // If a book has no Authors, the first main contributor will be marked as an Author
        assert!(output.contains(r#"    <Contributor>"#));
        assert!(output.contains(r#"      <SequenceNumber>2</SequenceNumber>"#));
        assert!(output.contains(r#"      <ContributorRole>A01</ContributorRole>"#));
        assert!(output.contains(r#"      <PersonName>Volume Editor</PersonName>"#));
        // Music Editors are omitted (unless required to be marked as an Author as above)
        assert!(!output.contains(r#"      <SequenceNumber>1</SequenceNumber>"#));
        assert!(!output.contains(r#"      <ContributorRole>B25</ContributorRole>"#));
        assert!(!output.contains(r#"      <PersonName>Music Editor</PersonName>"#));
        assert!(output.contains(r#"    <Extent>"#));
        assert!(output.contains(r#"      <ExtentType>00</ExtentType>"#));
        assert!(output.contains(r#"      <ExtentValue>334</ExtentValue>"#));
        assert!(output.contains(r#"      <ExtentUnit>03</ExtentUnit>"#));
        assert!(output.contains(r#"    <Subject>"#));
        assert!(output.contains(r#"      <SubjectSchemeIdentifier>12</SubjectSchemeIdentifier>"#));
        assert!(output.contains(r#"      <SubjectCode>AAB</SubjectCode>"#));
        assert!(output.contains(r#"      <SubjectSchemeIdentifier>10</SubjectSchemeIdentifier>"#));
        assert!(output.contains(r#"      <SubjectCode>AAA000000</SubjectCode>"#));
        assert!(output.contains(r#"      <SubjectSchemeIdentifier>04</SubjectSchemeIdentifier>"#));
        assert!(output.contains(r#"      <SubjectCode>JA85</SubjectCode>"#));
        assert!(output.contains(r#"      <SubjectSchemeIdentifier>23</SubjectSchemeIdentifier>"#));
        assert!(output.contains(r#"      <SubjectCode>keyword1</SubjectCode>"#));
        // Both Keywords and Custom codes are output with code 23
        assert!(output.contains(r#"      <SubjectCode>custom1</SubjectCode>"#));
        // Thema codes are not output for Google Books
        assert!(!output.contains(r#"      <SubjectSchemeIdentifier>93</SubjectSchemeIdentifier>"#));
        assert!(!output.contains(r#"      <SubjectCode>JWA</SubjectCode>"#));
        assert!(output.contains(r#"    <Audience>"#));
        assert!(output.contains(r#"      <AudienceCodeType>01</AudienceCodeType>"#));
        assert!(output.contains(r#"      <AudienceCodeValue>06</AudienceCodeValue>"#));
        assert!(output.contains(r#"  <CollateralDetail>"#));
        assert!(output.contains(r#"    <TextContent>"#));
        assert!(output.contains(r#"      <TextType>03</TextType>"#));
        assert!(output.contains(r#"      <ContentAudience>00</ContentAudience>"#));
        assert!(output.contains(
            r#"      <Text language="eng">Lorem ipsum dolor sit amet, consectetur adipiscing elit</Text>"#
        ));
        assert!(output.contains(r#"      <TextType>04</TextType>"#));
        assert!(output.contains(r#"      <Text language="eng">1. Chapter 1</Text>"#));
        assert!(output.contains(r#"  <PublishingDetail>"#));
        assert!(output.contains(r#"    <Imprint>"#));
        assert!(output.contains(r#"      <ImprintName>OA Editions Imprint</ImprintName>"#));
        assert!(output.contains(r#"    <Publisher>"#));
        assert!(output.contains(r#"      <PublishingRole>01</PublishingRole>"#));
        assert!(output.contains(r#"      <PublisherName>OA Editions</PublisherName>"#));
        assert!(output.contains(r#"    <PublishingStatus>04</PublishingStatus>"#));
        assert!(output.contains(r#"    <PublishingDate>"#));
        assert!(output.contains(r#"      <PublishingDateRole>01</PublishingDateRole>"#));
        assert!(output.contains(r#"      <Date dateformat="00">19991231</Date>"#));
        assert!(output.contains(r#"    <SalesRights>"#));
        assert!(output.contains(r#"      <SalesRightsType>02</SalesRightsType>"#));
        assert!(output.contains(r#"      <Territory>"#));
        assert!(output.contains(r#"         <RegionsIncluded>WORLD</RegionsIncluded>"#));
        assert!(output.contains(r#"    <RelatedProduct>"#));
        assert!(output.contains(r#"      <ProductRelationCode>06</ProductRelationCode>"#));
        assert!(output.contains(r#"      <ProductIdentifier>"#));
        assert!(output.contains(r#"        <ProductIDType>15</ProductIDType>"#));
        assert!(output.contains(r#"        <IDValue>9783161484100</IDValue>"#));
        assert!(output.contains(r#"  <ProductSupply>"#));
        assert!(output.contains(r#"    <Market>"#));
        assert!(output.contains(r#"      <Territory>"#));
        assert!(output.contains(r#"         <RegionsIncluded>WORLD</RegionsIncluded>"#));
        assert!(output.contains(r#"    <SupplyDetail>"#));
        assert!(output.contains(r#"      <Supplier>"#));
        assert!(output.contains(r#"        <SupplierRole>09</SupplierRole>"#));
        assert!(output.contains(r#"        <SupplierName>OA Editions</SupplierName>"#));
        assert!(output.contains(r#"      <ProductAvailability>20</ProductAvailability>"#));
        assert!(output.contains(r#"      <Price>"#));
        assert!(output.contains(r#"        <PriceType>02</PriceType>"#));
        assert!(output.contains(r#"        <PriceAmount>5.00</PriceAmount>"#));
        assert!(output.contains(r#"        <CurrencyCode>GBP</CurrencyCode>"#));
        assert!(output.contains(r#"        <Territory>"#));
        assert!(output.contains(r#"          <RegionsIncluded>WORLD</RegionsIncluded>"#));

        // Remove/change some values to test (non-)output of optional blocks
        test_work.titles[0].subtitle = None;
        test_work.page_count = None;
        test_work.abstracts.clear();
        test_work.publications[0].prices.pop();
        test_work.publications[0].publication_type = PublicationType::EPUB;
        let output = generate_test_output(true, &test_work);
        // Ebook type changed
        assert!(!output.contains(r#"    <ProductFormDetail>E107</ProductFormDetail>"#));
        assert!(output.contains(r#"    <ProductFormDetail>E101</ProductFormDetail>"#));
        // No subtitle supplied (within Thoth UI this would automatically update full_title)
        assert!(!output.contains(r#"        <Subtitle>Book Subtitle</Subtitle>"#));
        // No page count supplied
        assert!(!output.contains(r#"    <Extent>"#));
        assert!(!output.contains(r#"      <ExtentType>00</ExtentType>"#));
        assert!(!output.contains(r#"      <ExtentValue>334</ExtentValue>"#));
        assert!(!output.contains(r#"      <ExtentUnit>03</ExtentUnit>"#));
        // No long abstract supplied: CollateralDetail block only contains table of contents
        assert!(output.contains(r#"  <CollateralDetail>"#));
        assert!(output.contains(r#"    <TextContent>"#));
        assert!(output.contains(r#"      <TextType>04</TextType>"#));
        assert!(output.contains(r#"      <Text language="eng">1. Chapter 1</Text>"#));
        assert!(!output.contains(r#"      <TextType>03</TextType>"#));
        assert!(!output.contains(
            r#"      <Text language="eng">Lorem ipsum dolor sit amet, consectetur adipiscing elit</Text>"#
        ));
        // No GBP price supplied
        assert!(!output.contains(r#"      <Price>"#));
        assert!(!output.contains(r#"        <PriceType>02</PriceType>"#));
        assert!(!output.contains(r#"        <PriceAmount>5.00</PriceAmount>"#));
        assert!(!output.contains(r#"        <CurrencyCode>GBP</CurrencyCode>"#));
        assert!(!output.contains(r#"        <Territory>"#));
        assert!(!output.contains(r#"          <RegionsIncluded>WORLD</RegionsIncluded>"#));
        assert!(output.contains(r#"      <UnpricedItemType>01</UnpricedItemType>"#));

        // Replace long abstract but remove table of contents
        // Result: CollateralDetail block still present, but now only contains long abstract
        test_work.abstracts = vec![thoth_client::WorkAbstracts {
            abstract_id: Uuid::from_str("00000000-0000-0000-AAAA-000000000001").unwrap(),
            work_id: Uuid::from_str("00000000-0000-0000-AAAA-000000000001").unwrap(),
            content: "Lorem ipsum dolor sit amet, consectetur adipiscing elit".to_string(),
            locale_code: thoth_client::LocaleCode::EN,
            abstract_type: thoth_client::AbstractType::LONG,
            canonical: true,
        }];
        test_work.toc = None;
        let output = generate_test_output(true, &test_work);
        assert!(output.contains(r#"  <CollateralDetail>"#));
        assert!(output.contains(r#"    <TextContent>"#));
        assert!(output.contains(r#"      <TextType>03</TextType>"#));
        assert!(output.contains(r#"      <ContentAudience>00</ContentAudience>"#));
        assert!(output.contains(
            r#"      <Text language="eng">Lorem ipsum dolor sit amet, consectetur adipiscing elit</Text>"#
        ));
        assert!(!output.contains(r#"      <TextType>04</TextType>"#));
        assert!(!output.contains(r#"      <Text language="eng">1. Chapter 1</Text>"#));

        // Remove both table of contents and long abstract
        // Result: No CollateralDetail block present at all
        test_work.abstracts.clear();
        let output = generate_test_output(true, &test_work);
        assert!(!output.contains(r#"  <CollateralDetail>"#));
        assert!(!output.contains(r#"    <TextContent>"#));
        assert!(!output.contains(r#"      <TextType>03</TextType>"#));
        assert!(!output.contains(r#"      <ContentAudience>00</ContentAudience>"#));
        assert!(!output.contains(
            r#"      <Text language="eng">Lorem ipsum dolor sit amet, consectetur adipiscing elit</Text>"#
        ));
        assert!(!output.contains(r#"      <TextType>04</TextType>"#));
        assert!(!output.contains(r#"      <Text language="eng">1. Chapter 1</Text>"#));

        // Remove all subjects
        // Result: error (can't generate Google Books ONIX without a BIC, BISAC, or LCC subject)
        test_work.subjects.clear();
        let output = generate_test_output(false, &test_work);
        assert_eq!(
            output,
            "Could not generate onix_3.0::google_books: No BIC, BISAC or LCC subject code"
                .to_string()
        );

        // Reinstate the BIC subject but remove publication date: result is error
        test_work.subjects = vec![WorkSubjects {
            subject_code: "AAB".to_string(),
            subject_type: SubjectType::BIC,
            subject_ordinal: 1,
        }];
        test_work.publication_date = None;
        let output = generate_test_output(false, &test_work);
        assert_eq!(
            output,
            "Could not generate onix_3.0::google_books: Missing Publication Date".to_string()
        );

        // Replace publication date, add withdrawn date
        test_work.publication_date = chrono::NaiveDate::from_ymd_opt(1999, 12, 31);
        test_work.withdrawn_date = chrono::NaiveDate::from_ymd_opt(2020, 12, 31);
        let output = generate_test_output(true, &test_work);
        assert!(output.contains(
            r#"
    <PublishingDate>
      <PublishingDateRole>13</PublishingDateRole>
      <Date dateformat="00">20201231</Date>
    </PublishingDate>"#
        ));

        // Remove the only (PDF) publication's only location
        // Result: error (can't generate Google Books ONIX without EPUB or PDF URL)
        test_work.publications[0].locations.clear();
        let output = generate_test_output(false, &test_work);
        assert_eq!(
            output,
            "Could not generate onix_3.0::google_books: Missing EPUB or PDF URL".to_string()
        );

        // Replace location but remove the only ISBN: result is error
        test_work.publications[0].locations = vec![WorkPublicationsLocations {
            landing_page: Some("https://www.book.com/pdf_landing".to_string()),
            full_text_url: Some("https://www.book.com/pdf_fulltext".to_string()),
            location_platform: LocationPlatform::OTHER,
            canonical: true,
        }];
        test_work.publications[0].isbn = None;
        let output = generate_test_output(false, &test_work);
        assert_eq!(
            output,
            "Could not generate onix_3.0::google_books: This work does not have a PDF, EPUB or paperback ISBN".to_string()
        );

        // Replace ISBN but remove all contributors: result is error
        test_work.publications[0].isbn = Some(Isbn::from_str("978-3-16-148410-0").unwrap());
        test_work.contributions.clear();
        let output = generate_test_output(false, &test_work);
        assert_eq!(
            output,
            "Could not generate onix_3.0::google_books: No contributors supplied".to_string()
        );
    }
}
