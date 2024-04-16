use chrono::Utc;
use std::collections::HashMap;
use std::io::Write;
use thoth_client::{
    ContributionType, LanguageRelation, PublicationType, SubjectType, Work, WorkContributions,
    WorkLanguages, WorkPublications, WorkStatus, WorkType,
};
use xml::writer::{EventWriter, XmlEvent};

use super::{write_element_block, XmlElement, XmlSpecification};
use crate::xml::{write_full_element_block, XmlElementBlock, ONIX3_NS};
use thoth_errors::{ThothError, ThothResult};

#[derive(Copy, Clone)]
pub struct Onix3Jstor {}

const ONIX_ERROR: &str = "onix_3.0::jstor";

impl XmlSpecification for Onix3Jstor {
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
                [work] => XmlElementBlock::<Onix3Jstor>::xml_element(work, w),
                _ => {
                    for work in works.iter() {
                        // Do not include Chapters in full publisher metadata record
                        // (assumes that a publisher will always have more than one work)
                        if work.work_type != WorkType::BOOK_CHAPTER {
                            XmlElementBlock::<Onix3Jstor>::xml_element(work, w).ok();
                        }
                    }
                    Ok(())
                }
            }
        })
    }
}

impl XmlElementBlock<Onix3Jstor> for Work {
    fn xml_element<W: Write>(&self, w: &mut EventWriter<W>) -> ThothResult<()> {
        // Don't output works with no BISAC subject code
        // JSTOR can only ingest works which have at least one
        if !self
            .subjects
            .iter()
            .any(|s| matches!(s.subject_type, SubjectType::BISAC))
        {
            return Err(ThothError::IncompleteMetadataRecord(
                ONIX_ERROR.to_string(),
                "No BISAC subject code".to_string(),
            ));
        }
        // We can only generate the document if there's a PDF
        if let Some(pdf_url) = self
            .publications
            .iter()
            .find(|p| p.publication_type.eq(&PublicationType::PDF) && !p.locations.is_empty())
            .and_then(|p| p.locations.iter().find(|l| l.canonical))
            .and_then(|l| l.full_text_url.as_ref())
        {
            let work_id = format!("urn:uuid:{}", self.work_id);
            let (main_isbn, print_isbn) = get_publications_data(&self.publications);
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
                            w.write(XmlEvent::Characters(&doi.to_string()))
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
                                    w.write(XmlEvent::Characters(license)).map_err(|e| e.into())
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
                        })
                    })?;
                    for contribution in &self.contributions {
                        XmlElementBlock::<Onix3Jstor>::xml_element(contribution, w).ok();
                    }
                    for language in &self.languages {
                        XmlElementBlock::<Onix3Jstor>::xml_element(language, w).ok();
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
                        write_element_block("Subject", w, |w| {
                            XmlElement::<Onix3Jstor>::xml_element(&subject.subject_type, w)?;
                            write_element_block("SubjectCode", w, |w| {
                                w.write(XmlEvent::Characters(&subject.subject_code))
                                    .map_err(|e| e.into())
                            })
                        })?;
                    }
                    Ok(())
                })?;
                write_element_block("CollateralDetail", w, |w| {
                    if let Some(labstract) = &self.long_abstract {
                        write_element_block("TextContent", w, |w| {
                            // 03 Description ("30 Abstract" not implemented in OAPEN)
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
                            write_element_block("Text", w, |w| {
                                w.write(XmlEvent::Characters(toc)).map_err(|e| e.into())
                            })
                        })?;
                    }
                    write_element_block("TextContent", w, |w| {
                        // 20 Open access statement
                        write_element_block("TextType", w, |w| {
                            w.write(XmlEvent::Characters("20")).map_err(|e| e.into())
                        })?;
                        // 00 Unrestricted
                        write_element_block("ContentAudience", w, |w| {
                            w.write(XmlEvent::Characters("00")).map_err(|e| e.into())
                        })?;
                        write_element_block("Text", w, |w| {
                            w.write(XmlEvent::Characters("Open Access"))
                                .map_err(|e| e.into())
                        })
                    })
                })?;
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
                    if let Some(place) = &self.place {
                        write_element_block("CityOfPublication", w, |w| {
                            w.write(XmlEvent::Characters(place)).map_err(|e| e.into())
                        })?;
                    }
                    XmlElement::<Onix3Jstor>::xml_element(&self.work_status, w)?;
                    if let Some(date) = self.publication_date {
                        write_element_block("PublishingDate", w, |w| {
                            write_element_block("PublishingDateRole", w, |w| {
                                // 01 Publication date
                                w.write(XmlEvent::Characters("01")).map_err(|e| e.into())
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
                    Ok(())
                })?;
                if !print_isbn.is_empty() {
                    write_element_block("RelatedMaterial", w, |w| {
                        // The only RelatedProduct supported by JSTOR is the print ISBN (if any).
                        write_element_block("RelatedProduct", w, |w| {
                            // 13 Epublication based on (print product)
                            write_element_block("ProductRelationCode", w, |w| {
                                w.write(XmlEvent::Characters("13")).map_err(|e| e.into())
                            })?;
                            write_element_block("ProductIdentifier", w, |w| {
                                // 15 ISBN-13
                                write_element_block("ProductIDType", w, |w| {
                                    w.write(XmlEvent::Characters("15")).map_err(|e| e.into())
                                })?;
                                write_element_block("IDValue", w, |w| {
                                    w.write(XmlEvent::Characters(&print_isbn))
                                        .map_err(|e| e.into())
                                })
                            })
                        })
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
                                })
                            })?;
                            // 99 Contact supplier
                            write_element_block("ProductAvailability", w, |w| {
                                w.write(XmlEvent::Characters("99")).map_err(|e| e.into())
                            })?;
                            // 01 Free of charge
                            write_element_block("UnpricedItemType", w, |w| {
                                w.write(XmlEvent::Characters("01")).map_err(|e| e.into())
                            })
                        })?;
                    }
                    Ok(())
                })
            })
        } else {
            Err(ThothError::IncompleteMetadataRecord(
                ONIX_ERROR.to_string(),
                "Missing PDF URL".to_string(),
            ))
        }
    }
}

fn get_publications_data(publications: &[WorkPublications]) -> (String, String) {
    let pdf_isbn = publications
        .iter()
        .find(|p| p.publication_type.eq(&PublicationType::PDF))
        .and_then(|p| p.isbn.as_ref());
    let paperback_isbn = publications
        .iter()
        .find(|p| p.publication_type.eq(&PublicationType::PAPERBACK))
        .and_then(|p| p.isbn.as_ref());
    let hardback_isbn = publications
        .iter()
        .find(|p| p.publication_type.eq(&PublicationType::HARDBACK))
        .and_then(|p| p.isbn.as_ref());

    // The default product ISBN is the PDF's
    let main_isbn = pdf_isbn
        // Books that don't have a PDF ISBN will use the paperback's
        .or(paperback_isbn)
        .map_or_else(|| "".to_string(), |i| i.to_hyphenless_string());
    let print_isbn = hardback_isbn
        .or(paperback_isbn)
        .map_or_else(|| "".to_string(), |i| i.to_hyphenless_string());

    (main_isbn, print_isbn)
}

impl XmlElement<Onix3Jstor> for WorkStatus {
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

impl XmlElement<Onix3Jstor> for SubjectType {
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

impl XmlElement<Onix3Jstor> for LanguageRelation {
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

impl XmlElement<Onix3Jstor> for ContributionType {
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
            ContributionType::RESEARCH_BY => "A51",
            ContributionType::CONTRIBUTIONS_BY => "A32",
            ContributionType::INDEXER => "A34",
            ContributionType::Other(_) => unreachable!(),
        }
    }
}

impl XmlElementBlock<Onix3Jstor> for WorkContributions {
    fn xml_element<W: Write>(&self, w: &mut EventWriter<W>) -> ThothResult<()> {
        write_element_block("Contributor", w, |w| {
            write_element_block("SequenceNumber", w, |w| {
                w.write(XmlEvent::Characters(&self.contribution_ordinal.to_string()))
                    .map_err(|e| e.into())
            })?;
            XmlElement::<Onix3Jstor>::xml_element(&self.contribution_type, w)?;

            if let Some(orcid) = &self.contributor.orcid {
                write_element_block("NameIdentifier", w, |w| {
                    write_element_block("NameIDType", w, |w| {
                        w.write(XmlEvent::Characters("21")).map_err(|e| e.into())
                    })?;
                    write_element_block("IDValue", w, |w| {
                        w.write(XmlEvent::Characters(&orcid.to_string()))
                            .map_err(|e| e.into())
                    })
                })?;
            }
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
            Ok(())
        })
    }
}

impl XmlElementBlock<Onix3Jstor> for WorkLanguages {
    fn xml_element<W: Write>(&self, w: &mut EventWriter<W>) -> ThothResult<()> {
        write_element_block("Language", w, |w| {
            XmlElement::<Onix3Jstor>::xml_element(&self.language_relation, w).ok();
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
        ContributionType, LanguageCode, LanguageRelation, LocationPlatform, PublicationType,
        WorkContributionsContributor, WorkFundings, WorkImprint, WorkImprintPublisher, WorkIssues,
        WorkIssuesSeries, WorkPublicationsLocations, WorkStatus, WorkSubjects, WorkType,
    };
    use uuid::Uuid;

    fn generate_test_output(expect_ok: bool, input: &impl XmlElementBlock<Onix3Jstor>) -> String {
        // Helper function based on `XmlSpecification::generate`
        let mut buffer = Vec::new();
        let mut writer = xml::writer::EmitterConfig::new()
            .perform_indent(true)
            .create_writer(&mut buffer);
        let wrapped_output = XmlElementBlock::<Onix3Jstor>::xml_element(input, &mut writer)
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
    fn test_onix3_jstor_contributions() {
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
        assert!(output.contains(r#"  <NameIdentifier>"#));
        assert!(output.contains(r#"    <NameIDType>21</NameIDType>"#));
        assert!(output.contains(r#"    <IDValue>0000-0002-0000-0001</IDValue>"#));
        assert!(output.contains(r#"  </NameIdentifier>"#));
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
        assert!(!output.contains(r#"  <NameIdentifier>"#));
        assert!(!output.contains(r#"    <NameIDType>21</NameIDType>"#));
        assert!(!output.contains(r#"    <IDValue>0000-0002-0000-0001</IDValue>"#));
        assert!(!output.contains(r#"  </NameIdentifier>"#));
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
    }

    #[test]
    fn test_onix3_jstor_languages() {
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
    fn test_onix3_jstor_works() {
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
            toc: Some("1. Chapter 1".to_string()),
            lccn: None,
            oclc: None,
            cover_url: None,
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
            issues: vec![WorkIssues {
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
            }],
            contributions: vec![],
            languages: vec![],
            publications: vec![
                WorkPublications {
                    publication_id: Uuid::from_str("00000000-0000-0000-CCCC-000000000003").unwrap(),
                    publication_type: PublicationType::HARDBACK,
                    isbn: Some(Isbn::from_str("978-1-4028-9462-6").unwrap()),
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
                    locations: vec![],
                },
                WorkPublications {
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
                    prices: vec![],
                    locations: vec![WorkPublicationsLocations {
                        landing_page: Some("https://www.book.com/pdf_landing".to_string()),
                        full_text_url: Some("https://www.book.com/pdf_fulltext".to_string()),
                        location_platform: LocationPlatform::OTHER,
                        canonical: true,
                    }],
                },
            ],
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
            fundings: vec![WorkFundings {
                program: Some("Name of program".to_string()),
                project_name: Some("Name of project".to_string()),
                project_shortname: None,
                grant_number: Some("Number of grant".to_string()),
                jurisdiction: None,
                institution: thoth_client::FundingInstitution {
                    institution_name: "Name of institution".to_string(),
                    institution_doi: None,
                    ror: None,
                    country_code: None,
                },
            }],
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
        assert!(output.contains(r#"  <DescriptiveDetail>"#));
        assert!(output.contains(r#"    <ProductComposition>00</ProductComposition>"#));
        assert!(output.contains(r#"    <ProductForm>EB</ProductForm>"#));
        assert!(output.contains(r#"    <ProductFormDetail>E107</ProductFormDetail>"#));
        assert!(output.contains(r#"    <PrimaryContentType>10</PrimaryContentType>"#));
        assert!(output.contains(r#"    <EpubLicense>"#));
        assert!(
            output.contains(r#"      <EpubLicenseName>Creative Commons License</EpubLicenseName>"#)
        );
        assert!(output.contains(r#"      <EpubLicenseExpression>"#));
        assert!(
            output.contains(r#"        <EpubLicenseExpressionType>02</EpubLicenseExpressionType>"#)
        );
        assert!(output.contains(r#"        <EpubLicenseExpressionLink>https://creativecommons.org/licenses/by/4.0/</EpubLicenseExpressionLink>"#));
        assert!(output.contains(r#"    <TitleDetail>"#));
        assert!(output.contains(r#"      <TitleType>01</TitleType>"#));
        assert!(output.contains(r#"      <TitleElement>"#));
        assert!(output.contains(r#"        <TitleElementLevel>01</TitleElementLevel>"#));
        assert!(output.contains(r#"        <TitleText>Book Title</TitleText>"#));
        assert!(output.contains(r#"        <Subtitle>Book Subtitle</Subtitle>"#));
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
        assert!(output.contains(r#"      <SubjectSchemeIdentifier>93</SubjectSchemeIdentifier>"#));
        assert!(output.contains(r#"      <SubjectCode>JWA</SubjectCode>"#));
        assert!(output.contains(r#"      <SubjectSchemeIdentifier>20</SubjectSchemeIdentifier>"#));
        assert!(output.contains(r#"      <SubjectCode>keyword1</SubjectCode>"#));
        assert!(output.contains(r#"      <SubjectSchemeIdentifier>B2</SubjectSchemeIdentifier>"#));
        assert!(output.contains(r#"      <SubjectCode>custom1</SubjectCode>"#));
        assert!(output.contains(r#"  <CollateralDetail>"#));
        assert!(output.contains(r#"    <TextContent>"#));
        assert!(output.contains(r#"      <TextType>03</TextType>"#));
        assert!(output.contains(r#"      <ContentAudience>00</ContentAudience>"#));
        assert!(output.contains(r#"      <Text language="eng">Lorem ipsum dolor sit amet</Text>"#));
        assert!(output.contains(r#"    <TextContent>"#));
        assert!(output.contains(r#"      <TextType>04</TextType>"#));
        assert!(output.contains(r#"      <Text>1. Chapter 1</Text>"#));
        assert!(output.contains(r#"    <TextContent>"#));
        assert!(output.contains(r#"      <TextType>20</TextType>"#));
        assert!(output.contains(r#"      <Text>Open Access</Text>"#));
        assert!(output.contains(r#"  <PublishingDetail>"#));
        assert!(output.contains(r#"    <Imprint>"#));
        assert!(output.contains(r#"      <ImprintName>OA Editions Imprint</ImprintName>"#));
        assert!(output.contains(r#"    <Publisher>"#));
        assert!(output.contains(r#"      <PublishingRole>01</PublishingRole>"#));
        assert!(output.contains(r#"      <PublisherName>OA Editions</PublisherName>"#));
        assert!(output.contains(r#"    <CityOfPublication>León, Spain</CityOfPublication>"#));
        assert!(output.contains(r#"    <PublishingStatus>04</PublishingStatus>"#));
        assert!(output.contains(r#"    <PublishingDate>"#));
        assert!(output.contains(r#"      <PublishingDateRole>01</PublishingDateRole>"#));
        assert!(output.contains(r#"      <Date dateformat="00">19991231</Date>"#));
        assert!(output.contains(r#"    <RelatedProduct>"#));
        assert!(output.contains(r#"      <ProductRelationCode>13</ProductRelationCode>"#));
        assert!(output.contains(r#"      <ProductIdentifier>"#));
        assert!(output.contains(r#"        <ProductIDType>15</ProductIDType>"#));
        assert!(output.contains(r#"    <IDValue>9781402894626</IDValue>"#));
        assert!(output.contains(r#"  <ProductSupply>"#));
        assert!(output.contains(r#"    <SupplyDetail>"#));
        assert!(output.contains(r#"      <Supplier>"#));
        assert!(output.contains(r#"        <SupplierRole>09</SupplierRole>"#));
        assert!(output.contains(r#"        <SupplierName>OA Editions</SupplierName>"#));
        assert!(output.contains(r#"        <Website>"#));
        assert!(output.contains(r#"          <WebsiteRole>01</WebsiteRole>"#));
        assert!(output.contains(
            r#"          <WebsiteDescription>Publisher's website: web shop</WebsiteDescription>"#
        ));
        assert!(output.contains(r#"          <WebsiteLink>https://www.book.com</WebsiteLink>"#));
        assert!(output.contains(r#"      <ProductAvailability>99</ProductAvailability>"#));
        assert!(output.contains(r#"      <UnpricedItemType>01</UnpricedItemType>"#));
        assert!(output.contains(r#"        <SupplierRole>09</SupplierRole>"#));
        assert!(output.contains(r#"        <SupplierName>OA Editions</SupplierName>"#));
        assert!(output.contains(r#"          <WebsiteRole>29</WebsiteRole>"#));
        assert!(output.contains(r#"          <WebsiteDescription>Publisher's website: download the title</WebsiteDescription>"#));
        assert!(output
            .contains(r#"          <WebsiteLink>https://www.book.com/pdf_fulltext</WebsiteLink>"#));

        // Test that OAPEN-only blocks are not output in JSTOR format
        assert!(!output.contains(r#"    <Audience>"#));
        assert!(!output.contains(r#"      <AudienceCodeType>01</AudienceCodeType>"#));
        assert!(!output.contains(r#"      <AudienceCodeValue>06</AudienceCodeValue>"#));
        assert!(!output.contains(r#"      <PublishingRole>16</PublishingRole>"#));
        assert!(!output.contains(r#"      <PublisherName>Name of institution</PublisherName>"#));
        assert!(!output.contains(r#"      <Funding>"#));
        assert!(!output.contains(r#"        <FundingIdentifier>"#));
        assert!(!output.contains(r#"          <FundingIDType>01</FundingIDType>"#));
        assert!(!output.contains(r#"          <IDTypeName>programname</IDTypeName>"#));
        assert!(!output.contains(r#"          <IDValue>Name of program</IDValue>"#));
        assert!(!output.contains(r#"          <IDTypeName>projectname</IDTypeName>"#));
        assert!(!output.contains(r#"          <IDValue>Name of project</IDValue>"#));
        assert!(!output.contains(r#"          <IDTypeName>grantnumber</IDTypeName>"#));
        assert!(!output.contains(r#"          <IDValue>Number of grant</IDValue>"#));
        assert!(!output.contains(r#"    <Collection>"#));
        assert!(!output.contains(r#"      <CollectionType>10</CollectionType>"#));
        assert!(!output.contains(r#"          <TitleElementLevel>02</TitleElementLevel>"#));
        assert!(!output.contains(r#"          <PartNumber>1</PartNumber>"#));
        assert!(!output.contains(r#"          <TitleText>Name of series</TitleText>"#));

        // Remove some values to test non-output of optional blocks
        test_work.doi = None;
        test_work.license = None;
        test_work.subtitle = None;
        test_work.page_count = None;
        test_work.long_abstract = None;
        test_work.toc = None;
        test_work.place = None;
        test_work.publication_date = None;
        test_work.landing_page = None;
        test_work.publications[0].publication_type = PublicationType::XML;
        let output = generate_test_output(true, &test_work);
        // No DOI supplied
        assert!(!output.contains(r#"    <ProductIDType>06</ProductIDType>"#));
        assert!(!output.contains(r#"    <IDValue>10.00001/BOOK.0001</IDValue>"#));
        // No licence supplied
        assert!(!output.contains(r#"    <EpubLicense>"#));
        assert!(!output
            .contains(r#"      <EpubLicenseName>Creative Commons License</EpubLicenseName>"#));
        assert!(!output.contains(r#"      <EpubLicenseExpression>"#));
        assert!(!output
            .contains(r#"        <EpubLicenseExpressionType>02</EpubLicenseExpressionType>"#));
        assert!(!output.contains(r#"        <EpubLicenseExpressionLink>https://creativecommons.org/licenses/by/4.0/</EpubLicenseExpressionLink>"#));
        // No subtitle supplied (within Thoth UI this would automatically update full_title)
        assert!(!output.contains(r#"        <Subtitle>Book Subtitle</Subtitle>"#));
        // No page count supplied
        assert!(!output.contains(r#"    <Extent>"#));
        assert!(!output.contains(r#"      <ExtentType>00</ExtentType>"#));
        assert!(!output.contains(r#"      <ExtentValue>334</ExtentValue>"#));
        assert!(!output.contains(r#"      <ExtentUnit>03</ExtentUnit>"#));
        // No long abstract supplied
        assert!(!output.contains(r#"      <TextType>03</TextType>"#));
        assert!(!output.contains(r#"      <Text language="eng">Lorem ipsum dolor sit amet</Text>"#));
        // No TOC supplied
        assert!(!output.contains(r#"      <TextType>04</TextType>"#));
        assert!(!output.contains(r#"      <Text>1. Chapter 1</Text>"#));
        // CollateralDetail block is still present as it always contains Open Access statement
        assert!(output.contains(r#"  <CollateralDetail>"#));
        assert!(output.contains(r#"    <TextContent>"#));
        assert!(output.contains(r#"      <ContentAudience>00</ContentAudience>"#));
        // No place supplied
        assert!(!output.contains(r#"    <CityOfPublication>León, Spain</CityOfPublication>"#));
        // No publication date supplied
        assert!(!output.contains(r#"    <PublishingDate>"#));
        assert!(!output.contains(r#"      <PublishingDateRole>01</PublishingDateRole>"#));
        assert!(!output.contains(r#"      <Date dateformat="00">19991231</Date>"#));
        // No landing page supplied: only one SupplyDetail block, linking to PDF download
        assert!(!output.contains(r#"          <WebsiteRole>01</WebsiteRole>"#));
        assert!(!output.contains(
            r#"          <WebsiteDescription>Publisher's website: web shop</WebsiteDescription>"#
        ));
        assert!(!output.contains(r#"          <WebsiteLink>https://www.book.com</WebsiteLink>"#));
        // No print ISBN supplied
        assert!(!output.contains(r#"    <RelatedProduct>"#));
        assert!(!output.contains(r#"      <ProductRelationCode>13</ProductRelationCode>"#));
        assert!(!output.contains(r#"      <ProductIdentifier>"#));
        assert!(!output.contains(r#"        <ProductIDType>15</ProductIDType>"#));
        assert!(!output.contains(r#"    <IDValue>9781402894626</IDValue>"#));

        // Remove all subjects
        // Result: error (can't generate JSTOR ONIX without a BISAC subject)
        test_work.subjects.clear();
        let output = generate_test_output(false, &test_work);
        assert_eq!(
            output,
            "Could not generate onix_3.0::jstor: No BISAC subject code".to_string()
        );

        // Reinstate the BISAC subject but remove the only publication, which is the PDF
        // Result: error (can't generate JSTOR ONIX without PDF URL)
        test_work.subjects = vec![WorkSubjects {
            subject_code: "AAA000000".to_string(),
            subject_type: SubjectType::BISAC,
            subject_ordinal: 1,
        }];
        test_work.publications.clear();
        let output = generate_test_output(false, &test_work);
        assert_eq!(
            output,
            "Could not generate onix_3.0::jstor: Missing PDF URL".to_string()
        );
    }
}
