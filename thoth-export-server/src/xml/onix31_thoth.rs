use cc_license::License;
use chrono::Utc;
use std::io::Write;
use thoth_client::{
    AbstractType, ContributionType, LanguageRelation, LocationPlatform, PublicationType,
    RelationType, SubjectType, Work, WorkContributions, WorkFundings, WorkIssues, WorkLanguages,
    WorkPublicationsLocations, WorkReferences, WorkRelations, WorkRelationsRelatedWork,
    WorkRelationsRelatedWorkContributions, WorkRelationsRelatedWorkLanguages, WorkStatus, WorkType,
};
use xml::writer::{EventWriter, XmlEvent};

use super::{write_element_block, XmlElement, XmlSpecification};
use crate::xml::{write_full_element_block, XmlElementBlock, ONIX31_NS};
use thoth_errors::{ThothError, ThothResult};

#[derive(Copy, Clone)]
pub struct Onix31Thoth {}

struct Measure {
    measure_type: &'static str,
    measurement: f64,
    measure_unit_code: &'static str,
}

const ONIX_ERROR: &str = "onix_3.1::thoth";

// Based on ONIX for Books Release 3.1.2 Specification accessed January 2025
// Download link: https://www.editeur.org/files/ONIX%203/ONIX_for_Books_Release_3-1_pdf_docs+codes_Issue_67.zip
// Retrieved from: https://www.editeur.org/93/Release-3.0-Downloads/#Specifications
impl XmlSpecification for Onix31Thoth {
    fn handle_event<W: Write>(w: &mut EventWriter<W>, works: &[Work]) -> ThothResult<()> {
        write_full_element_block("ONIXMessage", Some(ONIX31_NS.to_vec()), w, |w| {
            write_element_block("Header", w, |w| {
                write_element_block("Sender", w, |w| {
                    write_element_block("SenderName", w, |w| {
                        w.write(XmlEvent::Characters("Thoth")).map_err(Into::into)
                    })?;
                    write_element_block("EmailAddress", w, |w| {
                        w.write(XmlEvent::Characters("distribution@thoth.pub"))
                            .map_err(Into::into)
                    })
                })?;
                write_element_block("SentDateTime", w, |w| {
                    w.write(XmlEvent::Characters(
                        &Utc::now().format("%Y%m%dT%H%M%S").to_string(),
                    ))
                    .map_err(Into::into)
                })
            })?;

            match works {
                [] => Err(ThothError::IncompleteMetadataRecord(
                    ONIX_ERROR.to_string(),
                    "Not enough data".to_string(),
                )),
                [work] => XmlElementBlock::<Onix31Thoth>::xml_element(work, w),
                _ => {
                    for work in works.iter() {
                        // Do not include Chapters in full publisher metadata record
                        // (assumes that a publisher will always have more than one work)
                        if work.work_type != WorkType::BOOK_CHAPTER {
                            XmlElementBlock::<Onix31Thoth>::xml_element(work, w).ok();
                        }
                    }
                    Ok(())
                }
            }
        })
    }
}

impl XmlElementBlock<Onix31Thoth> for Work {
    fn xml_element<W: Write>(&self, w: &mut EventWriter<W>) -> ThothResult<()> {
        // Format is one record per Publication
        if self.publications.is_empty() {
            return Err(ThothError::IncompleteMetadataRecord(
                ONIX_ERROR.to_string(),
                "No publications supplied".to_string(),
            ));
        }
        let work_id = format!("urn:uuid:{}", self.work_id);
        let isbns: Vec<String> = self
            .publications
            .iter()
            .filter_map(|p| p.isbn.clone())
            .map(|i| i.to_hyphenless_string())
            .collect();
        for publication in &self.publications {
            let publication_id = format!("urn:uuid:{}", publication.publication_id);
            let current_isbn = &publication.isbn.as_ref().map(|p| p.to_hyphenless_string());
            write_element_block("Product", w, |w| {
                write_element_block("RecordReference", w, |w| {
                    // Note that most existing Thoth ONIX outputs use the Work ID, not Publication ID,
                    // as they output one record per Work rather than one record per Publication
                    w.write(XmlEvent::Characters(&publication_id))
                        .map_err(Into::into)
                })?;
                // 03 Notification confirmed on publication
                write_element_block("NotificationType", w, |w| {
                    w.write(XmlEvent::Characters("03")).map_err(Into::into)
                })?;
                // 01 Publisher
                write_element_block("RecordSourceType", w, |w| {
                    w.write(XmlEvent::Characters("01")).map_err(Into::into)
                })?;
                write_element_block("ProductIdentifier", w, |w| {
                    // 01 Proprietary
                    write_element_block("ProductIDType", w, |w| {
                        w.write(XmlEvent::Characters("01")).map_err(Into::into)
                    })?;
                    write_element_block("IDTypeName", w, |w| {
                        w.write(XmlEvent::Characters("thoth-work-id"))
                            .map_err(Into::into)
                    })?;
                    write_element_block("IDValue", w, |w| {
                        w.write(XmlEvent::Characters(&work_id)).map_err(Into::into)
                    })
                })?;
                write_element_block("ProductIdentifier", w, |w| {
                    // 01 Proprietary
                    write_element_block("ProductIDType", w, |w| {
                        w.write(XmlEvent::Characters("01")).map_err(Into::into)
                    })?;
                    write_element_block("IDTypeName", w, |w| {
                        w.write(XmlEvent::Characters("thoth-publication-id"))
                            .map_err(Into::into)
                    })?;
                    write_element_block("IDValue", w, |w| {
                        w.write(XmlEvent::Characters(&publication_id))
                            .map_err(Into::into)
                    })
                })?;
                if let Some(isbn) = current_isbn {
                    write_element_block("ProductIdentifier", w, |w| {
                        // 15 ISBN-13
                        write_element_block("ProductIDType", w, |w| {
                            w.write(XmlEvent::Characters("15")).map_err(Into::into)
                        })?;
                        write_element_block("IDValue", w, |w| {
                            w.write(XmlEvent::Characters(isbn)).map_err(Into::into)
                        })
                    })?;
                }
                if let Some(doi) = &self.doi {
                    write_element_block("ProductIdentifier", w, |w| {
                        write_element_block("ProductIDType", w, |w| {
                            w.write(XmlEvent::Characters("06")).map_err(Into::into)
                        })?;
                        write_element_block("IDValue", w, |w| {
                            w.write(XmlEvent::Characters(&doi.to_string()))
                                .map_err(Into::into)
                        })
                    })?;
                }
                if let Some(lccn) = &self.lccn {
                    write_element_block("ProductIdentifier", w, |w| {
                        write_element_block("ProductIDType", w, |w| {
                            w.write(XmlEvent::Characters("13")).map_err(Into::into)
                        })?;
                        write_element_block("IDValue", w, |w| {
                            w.write(XmlEvent::Characters(&lccn.to_string()))
                                .map_err(Into::into)
                        })
                    })?;
                }
                if let Some(oclc) = &self.oclc {
                    write_element_block("ProductIdentifier", w, |w| {
                        write_element_block("ProductIDType", w, |w| {
                            w.write(XmlEvent::Characters("23")).map_err(Into::into)
                        })?;
                        write_element_block("IDValue", w, |w| {
                            w.write(XmlEvent::Characters(&oclc.to_string()))
                                .map_err(Into::into)
                        })
                    })?;
                }
                if let Some(reference) = &self.reference {
                    write_element_block("ProductIdentifier", w, |w| {
                        write_element_block("ProductIDType", w, |w| {
                            w.write(XmlEvent::Characters("01")).map_err(Into::into)
                        })?;
                        write_element_block("IDTypeName", w, |w| {
                            w.write(XmlEvent::Characters("internal-reference"))
                                .map_err(Into::into)
                        })?;
                        write_element_block("IDValue", w, |w| {
                            w.write(XmlEvent::Characters(&reference.to_string()))
                                .map_err(Into::into)
                        })
                    })?;
                }
                write_element_block("DescriptiveDetail", w, |w| {
                    // 00 Single-component retail product
                    write_element_block("ProductComposition", w, |w| {
                        w.write(XmlEvent::Characters("00")).map_err(Into::into)
                    })?;
                    let (form, form_detail) = get_product_form_codes(&publication.publication_type);
                    write_element_block("ProductForm", w, |w| {
                        w.write(XmlEvent::Characters(form)).map_err(Into::into)
                    })?;
                    if let Some(code) = form_detail {
                        write_element_block("ProductFormDetail", w, |w| {
                            w.write(XmlEvent::Characters(code)).map_err(Into::into)
                        })?;
                    }
                    // 10 Text (eye-readable)
                    write_element_block("PrimaryContentType", w, |w| {
                        w.write(XmlEvent::Characters("10")).map_err(Into::into)
                    })?;
                    for &(measurement_opt, measure_type, measure_unit_code) in &[
                        // 01 height
                        (publication.height_mm, "01", "mm"),
                        (publication.height_cm, "01", "cm"),
                        (publication.height_in, "01", "in"),
                        // 02 width
                        (publication.width_mm, "02", "mm"),
                        (publication.width_cm, "02", "cm"),
                        (publication.width_in, "02", "in"),
                        // 03 thickness
                        (publication.depth_mm, "03", "mm"),
                        (publication.depth_cm, "03", "cm"),
                        (publication.depth_in, "03", "in"),
                        // 08 unit weight
                        (publication.weight_g, "08", "gr"),
                        (publication.weight_oz, "08", "oz"),
                    ] {
                        if let Some(measurement) = measurement_opt {
                            XmlElementBlock::<Onix31Thoth>::xml_element(
                                &Measure {
                                    measure_type,
                                    measurement,
                                    measure_unit_code,
                                },
                                w,
                            )
                            .ok();
                        }
                    }
                    if let Some(license_url) = &self.license {
                        write_license(license_url.to_string(), w)?;
                    }
                    for issue in &self.issues {
                        XmlElementBlock::<Onix31Thoth>::xml_element(issue, w).ok();
                    }
                    write_title(
                        self.titles[0].title.clone(),
                        self.titles[0].subtitle.clone(),
                        w,
                    )?;
                    for contribution in &self.contributions {
                        XmlElementBlock::<Onix31Thoth>::xml_element(contribution, w).ok();
                    }
                    if let Some(edition) = &self.edition {
                        // "Normally sent only for the second and subsequent editions"
                        if edition > &1 {
                            write_element_block("Edition", w, |w| {
                                write_element_block("EditionNumber", w, |w| {
                                    w.write(XmlEvent::Characters(&edition.to_string()))
                                        .map_err(Into::into)
                                })
                            })?;
                        }
                    }
                    for language in &self.languages {
                        XmlElementBlock::<Onix31Thoth>::xml_element(language, w).ok();
                    }
                    if let Some(page_count) = self.page_count {
                        write_element_block("Extent", w, |w| {
                            // 00 Main content
                            write_element_block("ExtentType", w, |w| {
                                w.write(XmlEvent::Characters("00")).map_err(Into::into)
                            })?;
                            write_element_block("ExtentValue", w, |w| {
                                w.write(XmlEvent::Characters(&page_count.to_string()))
                                    .map_err(Into::into)
                            })?;
                            // 03 Pages
                            write_element_block("ExtentUnit", w, |w| {
                                w.write(XmlEvent::Characters("03")).map_err(Into::into)
                            })
                        })?;
                    }
                    if let Some(bibliography_note) = &self.bibliography_note {
                        // "This data element carries text stating the number and type of
                        // illustrations. The text may also include other content items,
                        // eg maps, bibliography, tables, index etc."
                        write_element_block("IllustrationsNote", w, |w| {
                            w.write(XmlEvent::Characters(&bibliography_note.to_string()))
                                .map_err(Into::into)
                        })?;
                    }
                    if let Some(image_count) = self.image_count {
                        write_element_block("AncillaryContent", w, |w| {
                            // 09 Illustrations, unspecified
                            // (note that there are separate codes for e.g. "halftones" - we don't distinguish)
                            write_element_block("AncillaryContentType", w, |w| {
                                w.write(XmlEvent::Characters("09")).map_err(Into::into)
                            })?;
                            write_element_block("Number", w, |w| {
                                w.write(XmlEvent::Characters(&image_count.to_string()))
                                    .map_err(Into::into)
                            })
                        })?;
                    }
                    if let Some(table_count) = self.table_count {
                        write_element_block("AncillaryContent", w, |w| {
                            // 11 Tables, unspecified
                            write_element_block("AncillaryContentType", w, |w| {
                                w.write(XmlEvent::Characters("11")).map_err(Into::into)
                            })?;
                            write_element_block("Number", w, |w| {
                                w.write(XmlEvent::Characters(&table_count.to_string()))
                                    .map_err(Into::into)
                            })
                        })?;
                    }
                    if let Some(audio_count) = self.audio_count {
                        write_element_block("AncillaryContent", w, |w| {
                            // 19 Recorded music items
                            // (closest equivalent - audio might not always be music)
                            write_element_block("AncillaryContentType", w, |w| {
                                w.write(XmlEvent::Characters("19")).map_err(Into::into)
                            })?;
                            write_element_block("Number", w, |w| {
                                w.write(XmlEvent::Characters(&audio_count.to_string()))
                                    .map_err(Into::into)
                            })
                        })?;
                    }
                    if let Some(video_count) = self.video_count {
                        write_element_block("AncillaryContent", w, |w| {
                            // 00 Unspecified, see description
                            // (there is no code for "videos")
                            write_element_block("AncillaryContentType", w, |w| {
                                w.write(XmlEvent::Characters("00")).map_err(Into::into)
                            })?;
                            write_element_block("AncillaryContentDescription", w, |w| {
                                w.write(XmlEvent::Characters("Videos")).map_err(Into::into)
                            })?;
                            write_element_block("Number", w, |w| {
                                w.write(XmlEvent::Characters(&video_count.to_string()))
                                    .map_err(Into::into)
                            })
                        })?;
                    }
                    let mut main_subject_found = vec![];
                    for subject in &self.subjects {
                        // One subject within every subject type can/should be marked as Main
                        // Use first one found with ordinal 1 (there may be multiple)
                        let is_main_subject = subject.subject_ordinal == 1
                            && !main_subject_found.contains(&subject.subject_type);
                        write_element_block("Subject", w, |w| {
                            if is_main_subject {
                                // MainSubject is an empty element
                                write_element_block("MainSubject", w, |_w| Ok(()))?;
                            }
                            XmlElement::<Onix31Thoth>::xml_element(&subject.subject_type, w)?;
                            match subject.subject_type {
                                SubjectType::KEYWORD | SubjectType::CUSTOM => {
                                    write_element_block("SubjectHeadingText", w, |w| {
                                        w.write(XmlEvent::Characters(&subject.subject_code))
                                            .map_err(Into::into)
                                    })
                                }
                                _ => write_element_block("SubjectCode", w, |w| {
                                    w.write(XmlEvent::Characters(&subject.subject_code))
                                        .map_err(Into::into)
                                }),
                            }
                        })?;
                        if is_main_subject {
                            main_subject_found.push(subject.subject_type.clone());
                        }
                    }
                    write_element_block("Audience", w, |w| {
                        // 01 ONIX audience codes
                        write_element_block("AudienceCodeType", w, |w| {
                            w.write(XmlEvent::Characters("01")).map_err(Into::into)
                        })?;
                        // 06 Professional and scholarly
                        write_element_block("AudienceCodeValue", w, |w| {
                            w.write(XmlEvent::Characters("06")).map_err(Into::into)
                        })
                    })
                })?;
                if self
                    .abstracts
                    .iter()
                    .any(|a| a.abstract_type == AbstractType::SHORT)
                    || self
                        .abstracts
                        .iter()
                        .any(|a| a.abstract_type == AbstractType::LONG)
                    || self.toc.is_some()
                    || self.general_note.is_some()
                    || self.cover_url.is_some()
                    || self.license.is_some()
                {
                    write_element_block("CollateralDetail", w, |w| {
                        write_work_short_abstract(self, w)?;
                        write_work_long_abstract(self, w)?;
                        if let Some(toc) = &self.toc {
                            write_element_block("TextContent", w, |w| {
                                // 04 Table of contents
                                write_element_block("TextType", w, |w| {
                                    w.write(XmlEvent::Characters("04")).map_err(Into::into)
                                })?;
                                // 00 Unrestricted
                                write_element_block("ContentAudience", w, |w| {
                                    w.write(XmlEvent::Characters("00")).map_err(Into::into)
                                })?;
                                write_element_block("Text", w, |w| {
                                    w.write(XmlEvent::Characters(toc)).map_err(Into::into)
                                })
                            })?;
                        }
                        write_work_open_access_statement(self, w)?;
                        write_work_general_note(self, w)?;
                        if let Some(cover_url) = &self.cover_url {
                            write_element_block("SupportingResource", w, |w| {
                                // 01 Front cover
                                write_element_block("ResourceContentType", w, |w| {
                                    w.write(XmlEvent::Characters("01")).map_err(Into::into)
                                })?;
                                // 00 Unrestricted
                                write_element_block("ContentAudience", w, |w| {
                                    w.write(XmlEvent::Characters("00")).map_err(Into::into)
                                })?;
                                // 03 Image
                                write_element_block("ResourceMode", w, |w| {
                                    w.write(XmlEvent::Characters("03")).map_err(Into::into)
                                })?;
                                if let Some(cover_caption) = &self.cover_caption {
                                    write_element_block("ResourceFeature", w, |w| {
                                        // 02 Caption
                                        write_element_block("ResourceFeatureType", w, |w| {
                                            w.write(XmlEvent::Characters("02")).map_err(Into::into)
                                        })?;
                                        write_element_block("FeatureNote", w, |w| {
                                            w.write(XmlEvent::Characters(cover_caption))
                                                .map_err(Into::into)
                                        })
                                    })?;
                                }
                                write_element_block("ResourceVersion", w, |w| {
                                    // 02 Downloadable file
                                    write_element_block("ResourceForm", w, |w| {
                                        w.write(XmlEvent::Characters("02")).map_err(Into::into)
                                    })?;
                                    write_element_block("ResourceLink", w, |w| {
                                        w.write(XmlEvent::Characters(cover_url)).map_err(Into::into)
                                    })
                                })
                            })?;
                        }
                        Ok(())
                    })?;
                }
                let chapter_relations: Vec<WorkRelations> = self
                    .relations
                    .clone()
                    .into_iter()
                    .filter(|r| {
                        r.relation_type == RelationType::HAS_CHILD && r.related_work.doi.is_some()
                    })
                    .collect();
                if !chapter_relations.is_empty() {
                    write_element_block("ContentDetail", w, |w| {
                        for relation in &chapter_relations {
                            let chapter = &relation.related_work;
                            write_element_block("ContentItem", w, |w| {
                                write_element_block("LevelSequenceNumber", w, |w| {
                                    w.write(XmlEvent::Characters(
                                        &relation.relation_ordinal.to_string(),
                                    ))
                                    .map_err(Into::into)
                                })?;
                                write_element_block("TextItem", w, |w| {
                                    // 03 Body matter
                                    write_element_block("TextItemType", w, |w| {
                                        w.write(XmlEvent::Characters("03")).map_err(Into::into)
                                    })?;
                                    write_element_block("TextItemIdentifier", w, |w| {
                                        // 06 DOI
                                        write_element_block("TextItemIDType", w, |w| {
                                            w.write(XmlEvent::Characters("06")).map_err(Into::into)
                                        })?;
                                        write_element_block("IDValue", w, |w| {
                                            w.write(XmlEvent::Characters(
                                                &chapter.doi.as_ref().unwrap().to_string(),
                                            ))
                                            .map_err(Into::into)
                                        })
                                    })?;
                                    if let Some(first_page) = &chapter.first_page {
                                        write_element_block("PageRun", w, |w| {
                                            write_element_block("FirstPageNumber", w, |w| {
                                                w.write(XmlEvent::Characters(first_page))
                                                    .map_err(Into::into)
                                            })?;
                                            if let Some(last_page) = &chapter.last_page {
                                                write_element_block("LastPageNumber", w, |w| {
                                                    w.write(XmlEvent::Characters(last_page))
                                                        .map_err(Into::into)
                                                })?;
                                            }
                                            Ok(())
                                        })?;
                                    }
                                    if let Some(page_count) = &chapter.page_count {
                                        write_element_block("NumberOfPages", w, |w| {
                                            w.write(XmlEvent::Characters(&page_count.to_string()))
                                                .map_err(Into::into)
                                        })?;
                                    }
                                    Ok(())
                                })?;
                                if let Some(license_url) = &chapter.license {
                                    write_license(license_url.to_string(), w)?;
                                }
                                write_element_block("ComponentTypeName", w, |w| {
                                    w.write(XmlEvent::Characters("Chapter")).map_err(Into::into)
                                })?;
                                write_title(
                                    chapter.titles[0].title.clone(),
                                    chapter.titles[0].subtitle.clone(),
                                    w,
                                )?;
                                for contribution in &chapter.contributions {
                                    XmlElementBlock::<Onix31Thoth>::xml_element(contribution, w)
                                        .ok();
                                }
                                for language in &chapter.languages {
                                    XmlElementBlock::<Onix31Thoth>::xml_element(language, w).ok();
                                }
                                if chapter
                                    .abstracts
                                    .iter()
                                    .any(|a| a.abstract_type == AbstractType::SHORT)
                                {
                                    write_chapter_short_abstract(chapter, w)?;
                                }
                                if chapter
                                    .abstracts
                                    .iter()
                                    .any(|a| a.abstract_type == AbstractType::LONG)
                                {
                                    write_chapter_long_abstract(chapter, w)?;
                                }
                                if chapter.license.is_some() {
                                    write_chapter_open_access_statement(chapter, w)?;
                                }
                                if chapter.general_note.is_some() {
                                    write_chapter_general_note(chapter, w)?;
                                }
                                write_chapter_copyright(chapter, w)?;
                                for reference in &chapter.references {
                                    XmlElementBlock::<Onix31Thoth>::xml_element(reference, w).ok();
                                }
                                Ok(())
                            })?;
                        }
                        Ok(())
                    })?;
                }
                write_element_block("PublishingDetail", w, |w| {
                    write_element_block("Imprint", w, |w| {
                        write_element_block("ImprintName", w, |w| {
                            w.write(XmlEvent::Characters(&self.imprint.imprint_name))
                                .map_err(Into::into)
                        })?;
                        if let Some(url) = &self.imprint.imprint_url {
                            write_element_block("ImprintIdentifier", w, |w| {
                                // 01 Proprietary
                                write_element_block("ImprintIDType", w, |w| {
                                    w.write(XmlEvent::Characters("01")).map_err(Into::into)
                                })?;
                                write_element_block("IDTypeName", w, |w| {
                                    w.write(XmlEvent::Characters("URL")).map_err(Into::into)
                                })?;
                                write_element_block("IDValue", w, |w| {
                                    w.write(XmlEvent::Characters(url)).map_err(Into::into)
                                })
                            })?;
                        }
                        Ok(())
                    })?;
                    write_element_block("Publisher", w, |w| {
                        // 01 Publisher
                        write_element_block("PublishingRole", w, |w| {
                            w.write(XmlEvent::Characters("01")).map_err(Into::into)
                        })?;
                        write_element_block("PublisherName", w, |w| {
                            w.write(XmlEvent::Characters(&self.imprint.publisher.publisher_name))
                                .map_err(Into::into)
                        })?;
                        if let Some(url) = &self.imprint.publisher.publisher_url {
                            write_element_block("Website", w, |w| {
                                // 01 Publisher’s corporate website
                                write_element_block("WebsiteRole", w, |w| {
                                    w.write(XmlEvent::Characters("01")).map_err(Into::into)
                                })?;
                                write_element_block("WebsiteDescription", w, |w| {
                                    w.write(XmlEvent::Characters("Publisher's website: home page"))
                                        .map_err(Into::into)
                                })?;
                                write_element_block("WebsiteLink", w, |w| {
                                    w.write(XmlEvent::Characters(url)).map_err(Into::into)
                                })
                            })?;
                        }
                        if let Some(url) = &self.landing_page {
                            write_element_block("Website", w, |w| {
                                // 02 Publisher's website for a specified work
                                write_element_block("WebsiteRole", w, |w| {
                                    w.write(XmlEvent::Characters("02")).map_err(Into::into)
                                })?;
                                write_element_block("WebsiteDescription", w, |w| {
                                    w.write(XmlEvent::Characters(
                                        "Publisher's website: webpage for this title",
                                    ))
                                    .map_err(Into::into)
                                })?;
                                write_element_block("WebsiteLink", w, |w| {
                                    w.write(XmlEvent::Characters(url)).map_err(Into::into)
                                })
                            })?;
                        }
                        Ok(())
                    })?;
                    for funding in &self.fundings {
                        XmlElementBlock::<Onix31Thoth>::xml_element(funding, w).ok();
                    }
                    if let Some(place) = &self.place {
                        write_element_block("CityOfPublication", w, |w| {
                            w.write(XmlEvent::Characters(place)).map_err(Into::into)
                        })?;
                    }
                    XmlElement::<Onix31Thoth>::xml_element(&self.work_status, w)?;
                    if let Some(date) = &self.publication_date {
                        write_element_block("PublishingDate", w, |w| {
                            write_element_block("PublishingDateRole", w, |w| {
                                // 01 Publication date
                                w.write(XmlEvent::Characters("01")).map_err(Into::into)
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
                                    .map_err(Into::into)
                                },
                            )
                        })?;
                    }
                    if let Some(date) = &self.withdrawn_date {
                        write_element_block("PublishingDate", w, |w| {
                            write_element_block("PublishingDateRole", w, |w| {
                                // 13 Out-of-print / permanently withdrawn date
                                w.write(XmlEvent::Characters("13")).map_err(Into::into)
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
                                    .map_err(Into::into)
                                },
                            )
                        })?;
                    }
                    write_work_copyright(self, w)?;
                    write_element_block("SalesRights", w, |w| {
                        // 02 For sale with non-exclusive rights in the specified countries or territories
                        write_element_block("SalesRightsType", w, |w| {
                            w.write(XmlEvent::Characters("02")).map_err(Into::into)
                        })?;
                        write_element_block("Territory", w, |w| {
                            write_element_block("RegionsIncluded", w, |w| {
                                w.write(XmlEvent::Characters("WORLD")).map_err(Into::into)
                            })
                        })
                    })
                })?;
                let non_child_relations: Vec<WorkRelations> = self
                    .relations
                    .iter()
                    .filter(|r| {
                        r.relation_type != RelationType::HAS_CHILD
                            && r.relation_type != RelationType::IS_CHILD_OF
                            && r.related_work.doi.is_some()
                    })
                    .cloned()
                    .collect();
                // Only output ISBNs in RelatedMaterial if at least one is present which
                // doesn't relate to the current publication
                if (!isbns.is_empty() && !isbns.eq(&vec![current_isbn.clone().unwrap_or_default()]))
                    || !non_child_relations.is_empty()
                    || !self.references.is_empty()
                {
                    write_element_block("RelatedMaterial", w, |w| {
                        // RelatedWorks should be listed before RelatedProducts
                        for relation in &non_child_relations {
                            if relation.relation_type == RelationType::HAS_TRANSLATION
                                || relation.relation_type == RelationType::IS_TRANSLATION_OF
                            {
                                XmlElementBlock::<Onix31Thoth>::xml_element(relation, w).ok();
                            }
                        }
                        for relation in &non_child_relations {
                            if relation.relation_type != RelationType::HAS_TRANSLATION
                                && relation.relation_type != RelationType::IS_TRANSLATION_OF
                            {
                                XmlElementBlock::<Onix31Thoth>::xml_element(relation, w).ok();
                            }
                        }
                        for isbn in &isbns {
                            if !current_isbn.eq(&Some(isbn.clone())) {
                                write_element_block("RelatedProduct", w, |w| {
                                    // 06 Alternative format
                                    write_element_block("ProductRelationCode", w, |w| {
                                        w.write(XmlEvent::Characters("06")).map_err(Into::into)
                                    })?;
                                    write_element_block("ProductIdentifier", w, |w| {
                                        // 15 ISBN-13
                                        write_element_block("ProductIDType", w, |w| {
                                            w.write(XmlEvent::Characters("15")).map_err(Into::into)
                                        })?;
                                        write_element_block("IDValue", w, |w| {
                                            w.write(XmlEvent::Characters(isbn)).map_err(Into::into)
                                        })
                                    })
                                })?;
                            }
                        }
                        for reference in &self.references {
                            XmlElementBlock::<Onix31Thoth>::xml_element(reference, w).ok();
                        }
                        Ok(())
                    })?;
                }
                write_element_block("ProductSupply", w, |w| {
                    write_element_block("Market", w, |w| {
                        write_element_block("Territory", w, |w| {
                            write_element_block("RegionsIncluded", w, |w| {
                                w.write(XmlEvent::Characters("WORLD")).map_err(Into::into)
                            })
                        })
                    })?;
                    let mut locations: Vec<WorkPublicationsLocations> =
                        publication.locations.clone();
                    if locations.is_empty() {
                        // Create single Supplier based on Work Landing Page
                        // so that we can output a SupplyDetail block (inc. Price info)
                        // (if Landing Page is None, Supplier will be complete apart from Website)
                        locations = vec![WorkPublicationsLocations {
                            landing_page: self.landing_page.clone(),
                            full_text_url: None,
                            location_platform: LocationPlatform::PUBLISHER_WEBSITE,
                            canonical: true,
                        }];
                    }
                    for location in locations {
                        let (supplier_name, description_string, supplier_role, landing_page_role) =
                            match location.location_platform {
                                LocationPlatform::PUBLISHER_WEBSITE => (
                                    self.imprint.publisher.publisher_name.clone(),
                                    "Publisher's website".to_string(),
                                    // 09 Publisher to end-customers
                                    "09",
                                    // 02 Publisher's website for a specified work
                                    "02",
                                ),
                                LocationPlatform::OTHER => (
                                    "Unknown".to_string(),
                                    "Unspecified hosting platform".to_string(),
                                    // 11 Non-exclusive distributor to end-customers
                                    "11",
                                    // 36 Supplier’s website for a specified work
                                    "36",
                                ),
                                _ => (
                                    location.location_platform.to_string(),
                                    location.location_platform.to_string(),
                                    // 11 Non-exclusive distributor to end-customers
                                    "11",
                                    // 36 Supplier’s website for a specified work
                                    "36",
                                ),
                            };
                        write_element_block("SupplyDetail", w, |w| {
                            write_element_block("Supplier", w, |w| {
                                write_element_block("SupplierRole", w, |w| {
                                    w.write(XmlEvent::Characters(supplier_role))
                                        .map_err(Into::into)
                                })?;
                                write_element_block("SupplierName", w, |w| {
                                    w.write(XmlEvent::Characters(&supplier_name))
                                        .map_err(Into::into)
                                })?;
                                if let Some(url) = &location.landing_page {
                                    write_element_block("Website", w, |w| {
                                        write_element_block("WebsiteRole", w, |w| {
                                            w.write(XmlEvent::Characters(landing_page_role))
                                                .map_err(Into::into)
                                        })?;
                                        write_element_block("WebsiteDescription", w, |w| {
                                            w.write(XmlEvent::Characters(&format!(
                                                "{description_string}: webpage for this product"
                                            )))
                                            .map_err(Into::into)
                                        })?;
                                        write_element_block("WebsiteLink", w, |w| {
                                            w.write(XmlEvent::Characters(url)).map_err(Into::into)
                                        })
                                    })?;
                                }
                                if let Some(url) = &location.full_text_url {
                                    write_element_block("Website", w, |w| {
                                        write_element_block("WebsiteRole", w, |w| {
                                            // 29 Web page for full content
                                            w.write(XmlEvent::Characters("29")).map_err(Into::into)
                                        })?;
                                        write_element_block("WebsiteDescription", w, |w| {
                                            w.write(XmlEvent::Characters(&format!(
                                                "{description_string}: download the title"
                                            )))
                                            .map_err(Into::into)
                                        })?;
                                        write_element_block("WebsiteLink", w, |w| {
                                            w.write(XmlEvent::Characters(url)).map_err(Into::into)
                                        })
                                    })?;
                                }
                                Ok(())
                            })?;
                            write_element_block("ProductAvailability", w, |w| {
                                let availability = match self.work_status {
                                    WorkStatus::CANCELLED => "01",              // 01 – Cancelled
                                    WorkStatus::FORTHCOMING => "10", // 10 – Not yet available
                                    WorkStatus::POSTPONED_INDEFINITELY => "09", // 09 – Not yet available, postponed indefinitely
                                    WorkStatus::ACTIVE => "20",                 // 20 – Available
                                    WorkStatus::SUPERSEDED => "41", // 41 – Not available, replaced by new product
                                    WorkStatus::WITHDRAWN => "49",  // 49 – Recalled
                                    WorkStatus::Other(_) => unreachable!(),
                                };
                                w.write(XmlEvent::Characters(availability))
                                    .map_err(Into::into)
                            })?;
                            if let Some(date) = &self.publication_date {
                                write_element_block("SupplyDate", w, |w| {
                                    write_element_block("SupplyDateRole", w, |w| {
                                        // 02 Expected availability date
                                        w.write(XmlEvent::Characters("08")).map_err(Into::into)
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
                                            .map_err(Into::into)
                                        },
                                    )
                                })?;
                            }
                            if publication.prices.is_empty() {
                                // 01 Free of charge
                                write_element_block("UnpricedItemType", w, |w| {
                                    w.write(XmlEvent::Characters("01")).map_err(Into::into)
                                })
                            } else {
                                for price in &publication.prices {
                                    let unit_price = price.unit_price;
                                    let formatted_price = format!("{unit_price:.2}");
                                    write_element_block("Price", w, |w| {
                                        // 02 RRP including tax
                                        write_element_block("PriceType", w, |w| {
                                            w.write(XmlEvent::Characters("02")).map_err(Into::into)
                                        })?;
                                        write_element_block("PriceAmount", w, |w| {
                                            w.write(XmlEvent::Characters(&formatted_price))
                                                .map_err(Into::into)
                                        })?;
                                        write_element_block("CurrencyCode", w, |w| {
                                            w.write(XmlEvent::Characters(
                                                &price.currency_code.to_string(),
                                            ))
                                            .map_err(Into::into)
                                        })?;
                                        write_element_block("Territory", w, |w| {
                                            write_element_block("RegionsIncluded", w, |w| {
                                                w.write(XmlEvent::Characters("WORLD"))
                                                    .map_err(Into::into)
                                            })
                                        })
                                    })?;
                                }
                                Ok(())
                            }
                        })?;
                    }
                    Ok(())
                })
            })?;
        }
        Ok(())
    }
}

fn write_license<W: Write>(license: String, w: &mut EventWriter<W>) -> ThothResult<()> {
    let license_text = match License::from_url(&license) {
        Ok(license) => license.to_string(),
        Err(_) => "Unspecified".to_string(),
    };
    write_element_block("EpubLicense", w, |w| {
        write_element_block("EpubLicenseName", w, |w| {
            w.write(XmlEvent::Characters(&license_text))
                .map_err(Into::into)
        })?;
        write_element_block("EpubLicenseExpression", w, |w| {
            write_element_block("EpubLicenseExpressionType", w, |w| {
                w.write(XmlEvent::Characters("02")).map_err(Into::into)
            })?;
            write_element_block("EpubLicenseExpressionLink", w, |w| {
                w.write(XmlEvent::Characters(&license)).map_err(Into::into)
            })
        })
    })?;
    Ok(())
}

fn write_title<W: Write>(
    title: String,
    subtitle: Option<String>,
    w: &mut EventWriter<W>,
) -> ThothResult<()> {
    write_element_block("TitleDetail", w, |w| {
        // 01 Distinctive title (book)
        write_element_block("TitleType", w, |w| {
            w.write(XmlEvent::Characters("01")).map_err(Into::into)
        })?;
        write_element_block("TitleElement", w, |w| {
            // 01 Product
            write_element_block("TitleElementLevel", w, |w| {
                w.write(XmlEvent::Characters("01")).map_err(Into::into)
            })?;
            write_element_block("TitleText", w, |w| {
                w.write(XmlEvent::Characters(&title)).map_err(Into::into)
            })?;
            if let Some(subtitle) = &subtitle {
                write_element_block("Subtitle", w, |w| {
                    w.write(XmlEvent::Characters(subtitle)).map_err(Into::into)
                })?;
            }
            Ok(())
        })
    })?;
    Ok(())
}

fn write_work_copyright<W: Write>(work: &Work, w: &mut EventWriter<W>) -> ThothResult<()> {
    write_copyright_content(work.copyright_holder.clone(), w)
}

fn write_chapter_copyright<W: Write>(
    chapter: &WorkRelationsRelatedWork,
    w: &mut EventWriter<W>,
) -> ThothResult<()> {
    write_copyright_content(chapter.copyright_holder.clone(), w)
}

fn write_copyright_content<W: Write>(
    copyright_holder: Option<String>,
    w: &mut EventWriter<W>,
) -> ThothResult<()> {
    if let Some(copyright_holder) = &copyright_holder {
        write_element_block("CopyrightStatement", w, |w| {
            write_element_block("CopyrightOwner", w, |w| {
                // This might be a CorporateName rather than PersonName, but we can't tell
                write_element_block("PersonName", w, |w| {
                    w.write(XmlEvent::Characters(copyright_holder))
                        .map_err(Into::into)
                })
            })
        })?;
    }
    Ok(())
}

fn write_work_short_abstract<W: Write>(work: &Work, w: &mut EventWriter<W>) -> ThothResult<()> {
    let short_abstract = work
        .abstracts
        .iter()
        .find(|a| a.abstract_type == AbstractType::SHORT)
        .map(|a| a.content.clone())
        .unwrap_or_default();
    write_short_abstract_content(short_abstract, w)?;
    Ok(())
}

fn write_chapter_short_abstract<W: Write>(
    chapter: &WorkRelationsRelatedWork,
    w: &mut EventWriter<W>,
) -> ThothResult<()> {
    let short_abstract = chapter
        .abstracts
        .iter()
        .find(|a| a.abstract_type == AbstractType::LONG)
        .map(|a| a.content.clone())
        .unwrap_or_default();
    write_short_abstract_content(short_abstract, w)?;
    Ok(())
}

fn write_short_abstract_content<W: Write>(
    mut short_abstract: String,
    w: &mut EventWriter<W>,
) -> ThothResult<()> {
    // Short description field may not exceed 350 characters.
    // Ensure that the string is truncated at a valid UTF-8 boundary
    // by finding the byte index of the 350th character and then truncating
    // the string at that index, to avoid creating invalid UTF-8 sequences.
    if let Some((byte_index, _)) = short_abstract.char_indices().nth(350) {
        short_abstract.truncate(byte_index);
    }
    write_element_block("TextContent", w, |w| {
        // 02 Short description
        write_element_block("TextType", w, |w| {
            w.write(XmlEvent::Characters("02")).map_err(Into::into)
        })?;
        // 00 Unrestricted
        write_element_block("ContentAudience", w, |w| {
            w.write(XmlEvent::Characters("00")).map_err(Into::into)
        })?;
        write_element_block("Text", w, |w| {
            w.write(XmlEvent::Characters(&short_abstract))
                .map_err(Into::into)
        })
    })?;
    Ok(())
}

fn write_work_long_abstract<W: Write>(work: &Work, w: &mut EventWriter<W>) -> ThothResult<()> {
    if let Some(long_abstract) = work
        .abstracts
        .iter()
        .find(|a| a.abstract_type == AbstractType::LONG)
        .map(|a| a.content.clone())
    {
        write_long_abstract_content(long_abstract, w)?;
    }
    Ok(())
}

fn write_chapter_long_abstract<W: Write>(
    chapter: &WorkRelationsRelatedWork,
    w: &mut EventWriter<W>,
) -> ThothResult<()> {
    if let Some(long_abstract) = chapter
        .abstracts
        .iter()
        .find(|a| a.abstract_type == AbstractType::LONG)
        .map(|a| a.content.clone())
    {
        write_long_abstract_content(long_abstract, w)?;
    }
    Ok(())
}

fn write_long_abstract_content<W: Write>(
    long_abstract: String,
    w: &mut EventWriter<W>,
) -> ThothResult<()> {
    // 03 Description, 30 Abstract
    for text_type in ["03", "30"] {
        write_element_block("TextContent", w, |w| {
            write_element_block("TextType", w, |w| {
                w.write(XmlEvent::Characters(text_type)).map_err(Into::into)
            })?;
            // 00 Unrestricted
            write_element_block("ContentAudience", w, |w| {
                w.write(XmlEvent::Characters("00")).map_err(Into::into)
            })?;
            write_element_block("Text", w, |w| {
                w.write(XmlEvent::Characters(&long_abstract))
                    .map_err(Into::into)
            })
        })?;
    }
    Ok(())
}

fn write_work_general_note<W: Write>(work: &Work, w: &mut EventWriter<W>) -> ThothResult<()> {
    if let Some(general_note) = work.general_note.clone() {
        write_general_note_content(general_note, w)?;
    }
    Ok(())
}

fn write_chapter_general_note<W: Write>(
    chapter: &WorkRelationsRelatedWork,
    w: &mut EventWriter<W>,
) -> ThothResult<()> {
    if let Some(general_note) = chapter.general_note.clone() {
        write_general_note_content(general_note, w)?;
    }
    Ok(())
}

fn write_general_note_content<W: Write>(
    general_note: String,
    w: &mut EventWriter<W>,
) -> ThothResult<()> {
    write_element_block("TextContent", w, |w| {
        // 13 Publisher's notice
        // "A statement included by a publisher in fulfillment of contractual obligations"
        // Used in many different ways - closest approximation
        write_element_block("TextType", w, |w| {
            w.write(XmlEvent::Characters("13")).map_err(Into::into)
        })?;
        // 00 Unrestricted
        write_element_block("ContentAudience", w, |w| {
            w.write(XmlEvent::Characters("00")).map_err(Into::into)
        })?;
        write_element_block("Text", w, |w| {
            w.write(XmlEvent::Characters(&general_note))
                .map_err(Into::into)
        })
    })?;
    Ok(())
}

fn write_work_open_access_statement<W: Write>(
    work: &Work,
    w: &mut EventWriter<W>,
) -> ThothResult<()> {
    if work.license.is_some() {
        write_open_access_statement_content(w)?;
    }
    Ok(())
}

fn write_chapter_open_access_statement<W: Write>(
    chapter: &WorkRelationsRelatedWork,
    w: &mut EventWriter<W>,
) -> ThothResult<()> {
    if chapter.license.is_some() {
        write_open_access_statement_content(w)?;
    }
    Ok(())
}

fn write_open_access_statement_content<W: Write>(w: &mut EventWriter<W>) -> ThothResult<()> {
    write_element_block("TextContent", w, |w| {
        // 20 Open access statement
        write_element_block("TextType", w, |w| {
            w.write(XmlEvent::Characters("20")).map_err(Into::into)
        })?;
        // 00 Unrestricted
        write_element_block("ContentAudience", w, |w| {
            w.write(XmlEvent::Characters("00")).map_err(Into::into)
        })?;
        write_full_element_block("Text", Some(vec![("language", "eng")]), w, |w| {
            w.write(XmlEvent::Characters("Open Access"))
                .map_err(Into::into)
        })
    })?;
    Ok(())
}

fn get_product_form_codes(publication_type: &PublicationType) -> (&str, Option<&str>) {
    match publication_type {
        PublicationType::PAPERBACK => ("BC", None),
        PublicationType::HARDBACK => ("BB", None),
        // EB Digital download and online
        PublicationType::PDF => ("EB", Some("E107")),
        PublicationType::HTML => ("EB", Some("E105")),
        PublicationType::XML => ("EB", Some("E113")),
        PublicationType::EPUB => ("EB", Some("E101")),
        PublicationType::MOBI => ("EB", Some("E127")),
        PublicationType::AZW3 => ("EB", Some("E116")),
        PublicationType::DOCX => ("EB", Some("E104")),
        // E100 "not yet allocated" - no codelist entry for .fb2, .fb3, .fbz
        PublicationType::FICTION_BOOK => ("EB", Some("E100")),
        // AN Downloadable and online audio file
        PublicationType::MP3 => ("AN", Some("A103")),
        PublicationType::WAV => ("AN", Some("A104")),
        PublicationType::Other(_) => unreachable!(),
    }
}

impl XmlElement<Onix31Thoth> for WorkStatus {
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

impl XmlElement<Onix31Thoth> for SubjectType {
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

impl XmlElement<Onix31Thoth> for LanguageRelation {
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

impl XmlElement<Onix31Thoth> for ContributionType {
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

impl XmlElementBlock<Onix31Thoth> for WorkContributions {
    fn xml_element<W: Write>(&self, w: &mut EventWriter<W>) -> ThothResult<()> {
        write_element_block("Contributor", w, |w| {
            write_element_block("SequenceNumber", w, |w| {
                w.write(XmlEvent::Characters(&self.contribution_ordinal.to_string()))
                    .map_err(Into::into)
            })?;
            XmlElement::<Onix31Thoth>::xml_element(&self.contribution_type, w)?;

            if let Some(orcid) = &self.contributor.orcid {
                write_element_block("NameIdentifier", w, |w| {
                    write_element_block("NameIDType", w, |w| {
                        w.write(XmlEvent::Characters("21")).map_err(Into::into)
                    })?;
                    write_element_block("IDValue", w, |w| {
                        w.write(XmlEvent::Characters(&orcid.to_string()))
                            .map_err(Into::into)
                    })
                })?;
            }
            write_element_block("PersonName", w, |w| {
                w.write(XmlEvent::Characters(&self.full_name))
                    .map_err(Into::into)
            })?;
            if let Some(first_name) = &self.first_name {
                write_element_block("NamesBeforeKey", w, |w| {
                    w.write(XmlEvent::Characters(first_name))
                        .map_err(Into::into)
                })?;
            }
            write_element_block("KeyNames", w, |w| {
                w.write(XmlEvent::Characters(&self.last_name))
                    .map_err(Into::into)
            })?;
            for affiliation in &self.affiliations {
                write_element_block("ProfessionalAffiliation", w, |w| {
                    if let Some(position) = &affiliation.position {
                        write_element_block("ProfessionalPosition", w, |w| {
                            w.write(XmlEvent::Characters(position)).map_err(Into::into)
                        })?;
                    }
                    if let Some(ror) = &affiliation.institution.ror {
                        write_element_block("AffiliationIdentifier", w, |w| {
                            // 40: ROR ID
                            write_element_block("AffiliationIDType", w, |w| {
                                w.write(XmlEvent::Characters("40")).map_err(Into::into)
                            })?;
                            write_element_block("IDValue", w, |w| {
                                w.write(XmlEvent::Characters(&ror.to_string()))
                                    .map_err(Into::into)
                            })
                        })?;
                    }
                    write_element_block("Affiliation", w, |w| {
                        w.write(XmlEvent::Characters(
                            &affiliation.institution.institution_name,
                        ))
                        .map_err(Into::into)
                    })
                })?;
            }
            if let Some(biography) = &self.biography {
                write_element_block("BiographicalNote", w, |w| {
                    w.write(XmlEvent::Characters(biography)).map_err(Into::into)
                })?;
            }
            if let Some(website) = &self.contributor.website {
                write_element_block("Website", w, |w| {
                    write_element_block("WebsiteRole", w, |w| {
                        w.write(XmlEvent::Characters("06")).map_err(Into::into)
                    })?;
                    write_element_block("WebsiteDescription", w, |w| {
                        w.write(XmlEvent::Characters("Own website"))
                            .map_err(Into::into)
                    })?;
                    write_element_block("WebsiteLink", w, |w| {
                        w.write(XmlEvent::Characters(website)).map_err(Into::into)
                    })
                })?;
            }
            Ok(())
        })
    }
}

impl XmlElementBlock<Onix31Thoth> for WorkRelationsRelatedWorkContributions {
    fn xml_element<W: Write>(&self, w: &mut EventWriter<W>) -> ThothResult<()> {
        write_element_block("Contributor", w, |w| {
            write_element_block("SequenceNumber", w, |w| {
                w.write(XmlEvent::Characters(&self.contribution_ordinal.to_string()))
                    .map_err(Into::into)
            })?;
            XmlElement::<Onix31Thoth>::xml_element(&self.contribution_type, w)?;

            if let Some(orcid) = &self.contributor.orcid {
                write_element_block("NameIdentifier", w, |w| {
                    write_element_block("NameIDType", w, |w| {
                        w.write(XmlEvent::Characters("21")).map_err(Into::into)
                    })?;
                    write_element_block("IDValue", w, |w| {
                        w.write(XmlEvent::Characters(&orcid.to_string()))
                            .map_err(Into::into)
                    })
                })?;
            }
            write_element_block("PersonName", w, |w| {
                w.write(XmlEvent::Characters(&self.full_name))
                    .map_err(Into::into)
            })?;
            if let Some(first_name) = &self.first_name {
                write_element_block("NamesBeforeKey", w, |w| {
                    w.write(XmlEvent::Characters(first_name))
                        .map_err(Into::into)
                })?;
            }
            write_element_block("KeyNames", w, |w| {
                w.write(XmlEvent::Characters(&self.last_name))
                    .map_err(Into::into)
            })?;
            for affiliation in &self.affiliations {
                write_element_block("ProfessionalAffiliation", w, |w| {
                    if let Some(position) = &affiliation.position {
                        write_element_block("ProfessionalPosition", w, |w| {
                            w.write(XmlEvent::Characters(position)).map_err(Into::into)
                        })?;
                    }
                    if let Some(ror) = &affiliation.institution.ror {
                        write_element_block("AffiliationIdentifier", w, |w| {
                            write_element_block("AffiliationIDType", w, |w| {
                                w.write(XmlEvent::Characters("40")).map_err(Into::into)
                            })?;
                            write_element_block("IDValue", w, |w| {
                                w.write(XmlEvent::Characters(&ror.to_string()))
                                    .map_err(Into::into)
                            })
                        })?;
                    }
                    write_element_block("Affiliation", w, |w| {
                        w.write(XmlEvent::Characters(
                            &affiliation.institution.institution_name,
                        ))
                        .map_err(Into::into)
                    })
                })?;
            }
            if let Some(biography) = &self.biography {
                write_element_block("BiographicalNote", w, |w| {
                    w.write(XmlEvent::Characters(biography)).map_err(Into::into)
                })?;
            }
            if let Some(website) = &self.contributor.website {
                write_element_block("Website", w, |w| {
                    write_element_block("WebsiteRole", w, |w| {
                        w.write(XmlEvent::Characters("06")).map_err(Into::into)
                    })?;
                    write_element_block("WebsiteDescription", w, |w| {
                        w.write(XmlEvent::Characters("Own website"))
                            .map_err(Into::into)
                    })?;
                    write_element_block("WebsiteLink", w, |w| {
                        w.write(XmlEvent::Characters(website)).map_err(Into::into)
                    })
                })?;
            }
            Ok(())
        })
    }
}

impl XmlElementBlock<Onix31Thoth> for WorkLanguages {
    fn xml_element<W: Write>(&self, w: &mut EventWriter<W>) -> ThothResult<()> {
        write_element_block("Language", w, |w| {
            XmlElement::<Onix31Thoth>::xml_element(&self.language_relation, w).ok();
            // not worth implementing XmlElement for LanguageCode as all cases would
            // need to be exhaustively matched and the codes are equivalent anyway
            write_element_block("LanguageCode", w, |w| {
                w.write(XmlEvent::Characters(
                    &self.language_code.to_string().to_lowercase(),
                ))
                .map_err(Into::into)
            })
        })
    }
}

impl XmlElementBlock<Onix31Thoth> for WorkRelationsRelatedWorkLanguages {
    fn xml_element<W: Write>(&self, w: &mut EventWriter<W>) -> ThothResult<()> {
        write_element_block("Language", w, |w| {
            XmlElement::<Onix31Thoth>::xml_element(&self.language_relation, w).ok();
            // not worth implementing XmlElement for LanguageCode as all cases would
            // need to be exhaustively matched and the codes are equivalent anyway
            write_element_block("LanguageCode", w, |w| {
                w.write(XmlEvent::Characters(
                    &self.language_code.to_string().to_lowercase(),
                ))
                .map_err(Into::into)
            })
        })
    }
}

impl XmlElementBlock<Onix31Thoth> for WorkIssues {
    fn xml_element<W: Write>(&self, w: &mut EventWriter<W>) -> ThothResult<()> {
        write_element_block("Collection", w, |w| {
            // 10 Publisher collection (e.g. series)
            write_element_block("CollectionType", w, |w| {
                w.write(XmlEvent::Characters("10")).map_err(Into::into)
            })?;
            write_element_block("CollectionIdentifier", w, |w| {
                // 01 Proprietary
                write_element_block("CollectionIDType", w, |w| {
                    w.write(XmlEvent::Characters("01")).map_err(Into::into)
                })?;
                write_element_block("IDTypeName", w, |w| {
                    w.write(XmlEvent::Characters("Series ID"))
                        .map_err(Into::into)
                })?;
                write_element_block("IDValue", w, |w| {
                    w.write(XmlEvent::Characters(&self.series.series_id.to_string()))
                        .map_err(Into::into)
                })
            })?;
            if let Some(issn_digital) = &self.series.issn_digital {
                write_element_block("CollectionIdentifier", w, |w| {
                    // 02 ISSN
                    write_element_block("CollectionIDType", w, |w| {
                        w.write(XmlEvent::Characters("02")).map_err(Into::into)
                    })?;
                    write_element_block("IDValue", w, |w| {
                        w.write(XmlEvent::Characters(
                            &issn_digital.as_str().replace('-', ""),
                        ))
                        .map_err(Into::into)
                    })
                })?;
            }
            if let Some(url) = &self.series.series_url {
                write_element_block("CollectionIdentifier", w, |w| {
                    // 01 Proprietary
                    write_element_block("CollectionIDType", w, |w| {
                        w.write(XmlEvent::Characters("01")).map_err(Into::into)
                    })?;
                    write_element_block("IDTypeName", w, |w| {
                        w.write(XmlEvent::Characters("Series URL"))
                            .map_err(Into::into)
                    })?;
                    write_element_block("IDValue", w, |w| {
                        w.write(XmlEvent::Characters(url)).map_err(Into::into)
                    })
                })?;
            }
            if let Some(url) = &self.series.series_cfp_url {
                write_element_block("CollectionIdentifier", w, |w| {
                    // 01 Proprietary
                    write_element_block("CollectionIDType", w, |w| {
                        w.write(XmlEvent::Characters("01")).map_err(Into::into)
                    })?;
                    write_element_block("IDTypeName", w, |w| {
                        w.write(XmlEvent::Characters("Series Call for Proposals URL"))
                            .map_err(Into::into)
                    })?;
                    write_element_block("IDValue", w, |w| {
                        w.write(XmlEvent::Characters(url)).map_err(Into::into)
                    })
                })?;
            }
            write_element_block("CollectionSequence", w, |w| {
                // 03 Publication order
                write_element_block("CollectionSequenceType", w, |w| {
                    w.write(XmlEvent::Characters("03")).map_err(Into::into)
                })?;
                write_element_block("CollectionSequenceNumber", w, |w| {
                    w.write(XmlEvent::Characters(&self.issue_ordinal.to_string()))
                        .map_err(Into::into)
                })
            })?;
            write_element_block("TitleDetail", w, |w| {
                // 01 Cover title (serial)
                write_element_block("TitleType", w, |w| {
                    w.write(XmlEvent::Characters("01")).map_err(Into::into)
                })?;
                write_element_block("TitleElement", w, |w| {
                    // 02 Collection level
                    write_element_block("TitleElementLevel", w, |w| {
                        w.write(XmlEvent::Characters("02")).map_err(Into::into)
                    })?;
                    write_element_block("PartNumber", w, |w| {
                        w.write(XmlEvent::Characters(&self.issue_ordinal.to_string()))
                            .map_err(Into::into)
                    })?;
                    write_element_block("TitleText", w, |w| {
                        w.write(XmlEvent::Characters(&self.series.series_name))
                            .map_err(Into::into)
                    })
                })
            })
        })
    }
}

impl XmlElementBlock<Onix31Thoth> for WorkFundings {
    fn xml_element<W: Write>(&self, w: &mut EventWriter<W>) -> ThothResult<()> {
        write_element_block("Publisher", w, |w| {
            // 16 Funding body
            write_element_block("PublishingRole", w, |w| {
                w.write(XmlEvent::Characters("16")).map_err(Into::into)
            })?;
            if let Some(ror) = &self.institution.ror {
                write_element_block("PublisherIdentifier", w, |w| {
                    // 40 ROR
                    write_element_block("PublisherIDType", w, |w| {
                        w.write(XmlEvent::Characters("40")).map_err(Into::into)
                    })?;
                    write_element_block("IDValue", w, |w| {
                        w.write(XmlEvent::Characters(&ror.to_string()))
                            .map_err(Into::into)
                    })
                })?;
            }
            if let Some(doi) = &self.institution.institution_doi {
                write_element_block("PublisherIdentifier", w, |w| {
                    // 32 FundRef DOI
                    write_element_block("PublisherIDType", w, |w| {
                        w.write(XmlEvent::Characters("32")).map_err(Into::into)
                    })?;
                    write_element_block("IDValue", w, |w| {
                        w.write(XmlEvent::Characters(&doi.to_string()))
                            .map_err(Into::into)
                    })
                })?;
            }
            write_element_block("PublisherName", w, |w| {
                w.write(XmlEvent::Characters(&self.institution.institution_name))
                    .map_err(Into::into)
            })?;
            let identifiers: Vec<(&str, Option<&str>)> = vec![
                ("programname", self.program.as_deref()),
                ("projectname", self.project_name.as_deref()),
                ("projectshortname", self.project_shortname.as_deref()),
                ("grantnumber", self.grant_number.as_deref()),
                ("jurisdiction", self.jurisdiction.as_deref()),
            ];
            if identifiers.iter().any(|(_, i)| i.is_some()) {
                write_element_block("Funding", w, |w| {
                    for (typename, value_opt) in &identifiers {
                        if let Some(value) = *value_opt {
                            write_element_block("FundingIdentifier", w, |w| {
                                // 01 Proprietary
                                write_element_block("FundingIDType", w, |w| {
                                    w.write(XmlEvent::Characters("01")).map_err(Into::into)
                                })?;
                                write_element_block("IDTypeName", w, |w| {
                                    w.write(XmlEvent::Characters(typename)).map_err(Into::into)
                                })?;
                                write_element_block("IDValue", w, |w| {
                                    w.write(XmlEvent::Characters(value)).map_err(Into::into)
                                })
                            })?;
                        }
                    }
                    Ok(())
                })?;
            }
            Ok(())
        })
    }
}

impl XmlElementBlock<Onix31Thoth> for WorkReferences {
    fn xml_element<W: Write>(&self, w: &mut EventWriter<W>) -> ThothResult<()> {
        write_element_block("RelatedProduct", w, |w| {
            // 34 Cites
            write_element_block("ProductRelationCode", w, |w| {
                w.write(XmlEvent::Characters("34")).map_err(Into::into)
            })?;
            let (product_id_type, id_value) = self.doi.as_ref().map_or_else(
                || {
                    (
                        // 01 Proprietary
                        "01",
                        // Unstructured citation is mandatory in Thoth if DOI is missing
                        self.unstructured_citation.as_ref().unwrap().to_owned(),
                    )
                },
                // 06 DOI
                |doi| ("06", doi.to_string()),
            );
            write_element_block("ProductIdentifier", w, |w| {
                write_element_block("ProductIDType", w, |w| {
                    w.write(XmlEvent::Characters(product_id_type))
                        .map_err(Into::into)
                })?;
                if product_id_type == "01" {
                    write_element_block("IDTypeName", w, |w| {
                        w.write(XmlEvent::Characters("Unstructured citation"))
                            .map_err(Into::into)
                    })?;
                }
                write_element_block("IDValue", w, |w| {
                    w.write(XmlEvent::Characters(&id_value)).map_err(Into::into)
                })
            })
        })
    }
}

impl XmlElementBlock<Onix31Thoth> for WorkRelations {
    fn xml_element<W: Write>(&self, w: &mut EventWriter<W>) -> ThothResult<()> {
        if self.relation_type == RelationType::HAS_TRANSLATION
            || self.relation_type == RelationType::IS_TRANSLATION_OF
        {
            write_element_block("RelatedWork", w, |w| {
                let work_relation_code = match &self.relation_type {
                    // 49 Related work is derived from this via translation
                    RelationType::HAS_TRANSLATION => "49",
                    // 29 Derived from via translation
                    RelationType::IS_TRANSLATION_OF => "29",
                    _ => unreachable!(),
                };
                write_element_block("WorkRelationCode", w, |w| {
                    w.write(XmlEvent::Characters(work_relation_code))
                        .map_err(Into::into)
                })?;
                write_element_block("WorkIdentifier", w, |w| {
                    // 06 DOI
                    write_element_block("WorkIDType", w, |w| {
                        w.write(XmlEvent::Characters("06")).map_err(Into::into)
                    })?;
                    write_element_block("IDValue", w, |w| {
                        w.write(XmlEvent::Characters(
                            // Caller must guarantee that DOI is present
                            &self.related_work.doi.as_ref().unwrap().to_string(),
                        ))
                        .map_err(Into::into)
                    })
                })
            })
        } else {
            write_element_block("RelatedProduct", w, |w| {
                let product_relation_code = match &self.relation_type {
                    // 01 Includes
                    RelationType::HAS_PART => "01",
                    // 02 Is part of
                    RelationType::IS_PART_OF => "02",
                    // 03 Replaces
                    RelationType::REPLACES => "03",
                    // 05 Replaced by
                    RelationType::IS_REPLACED_BY => "05",
                    // This implementation is only valid for translation/part/replacement
                    // relationships, not parent/child relationships
                    _ => unreachable!(),
                };
                write_element_block("ProductRelationCode", w, |w| {
                    w.write(XmlEvent::Characters(product_relation_code))
                        .map_err(Into::into)
                })?;
                write_element_block("ProductIdentifier", w, |w| {
                    // 06 DOI
                    write_element_block("ProductIDType", w, |w| {
                        w.write(XmlEvent::Characters("06")).map_err(Into::into)
                    })?;
                    write_element_block("IDValue", w, |w| {
                        w.write(XmlEvent::Characters(
                            // Caller must guarantee that DOI is present
                            &self.related_work.doi.as_ref().unwrap().to_string(),
                        ))
                        .map_err(Into::into)
                    })
                })
            })
        }
    }
}

impl XmlElementBlock<Onix31Thoth> for Measure {
    fn xml_element<W: Write>(&self, w: &mut EventWriter<W>) -> ThothResult<()> {
        write_element_block("Measure", w, |w| {
            write_element_block("MeasureType", w, |w| {
                w.write(XmlEvent::Characters(self.measure_type))
                    .map_err(Into::into)
            })?;
            write_element_block("Measurement", w, |w| {
                w.write(XmlEvent::Characters(&self.measurement.to_string()))
                    .map_err(Into::into)
            })?;
            write_element_block("MeasureUnitCode", w, |w| {
                w.write(XmlEvent::Characters(self.measure_unit_code))
                    .map_err(Into::into)
            })
        })
    }
}

#[cfg(test)]
mod tests {
    // Testing note: Repeated XML nodes cannot be guaranteed to be output in the same order every time
    // We therefore rely on `assert!(contains)` rather than `assert_eq!` in these cases
    // println!s throughout will only be printed if test fails - this assists debugging
    use super::*;
    use std::str::FromStr;
    use thoth_api::model::Doi;
    use thoth_api::model::Isbn;
    use thoth_api::model::Orcid;
    use thoth_api::model::Ror;
    use thoth_client::{
        ContributionType, CurrencyCode, FundingInstitution, LanguageCode, LanguageRelation,
        LocationPlatform, PublicationType, WorkContributionsAffiliations,
        WorkContributionsAffiliationsInstitution, WorkContributionsContributor, WorkImprint,
        WorkImprintPublisher, WorkIssuesSeries, WorkPublications, WorkPublicationsLocations,
        WorkPublicationsPrices, WorkRelationsRelatedWork,
        WorkRelationsRelatedWorkContributionsAffiliations,
        WorkRelationsRelatedWorkContributionsAffiliationsInstitution,
        WorkRelationsRelatedWorkContributionsContributor, WorkRelationsRelatedWorkImprint,
        WorkRelationsRelatedWorkImprintPublisher, WorkRelationsRelatedWorkReferences, WorkStatus,
        WorkSubjects, WorkType,
    };
    use uuid::Uuid;

    fn generate_test_output(expect_ok: bool, input: &impl XmlElementBlock<Onix31Thoth>) -> String {
        // Helper function based on `XmlSpecification::generate`
        let mut buffer = Vec::new();
        let mut writer = xml::writer::EmitterConfig::new()
            .perform_indent(true)
            .create_writer(&mut buffer);
        let wrapped_output = XmlElementBlock::<Onix31Thoth>::xml_element(input, &mut writer)
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
    fn test_onix31_thoth_contributions() {
        let mut test_contribution = WorkContributions {
            contribution_type: ContributionType::AUTHOR,
            first_name: Some("Author".to_string()),
            last_name: "1".to_string(),
            full_name: "Author N. 1".to_string(),
            main_contribution: true,
            biography: Some("Author N. 1 is a made-up author".to_string()),
            contribution_ordinal: 1,
            contributor: WorkContributionsContributor {
                orcid: Some(Orcid::from_str("https://orcid.org/0000-0002-0000-0001").unwrap()),
                website: Some("https://contributor.site".to_string()),
            },
            affiliations: vec![WorkContributionsAffiliations {
                position: Some("Manager".to_string()),
                affiliation_ordinal: 1,
                institution: WorkContributionsAffiliationsInstitution {
                    institution_name: "University of Life".to_string(),
                    institution_doi: None,
                    ror: Some(Ror::from_str("01abcde23").unwrap()),
                    country_code: None,
                },
            }],
        };

        // Test standard output
        let output = generate_test_output(true, &test_contribution);
        println!("{output}");
        assert_eq!(
            output,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<Contributor>
  <SequenceNumber>1</SequenceNumber>
  <ContributorRole>A01</ContributorRole>
  <NameIdentifier>
    <NameIDType>21</NameIDType>
    <IDValue>0000-0002-0000-0001</IDValue>
  </NameIdentifier>
  <PersonName>Author N. 1</PersonName>
  <NamesBeforeKey>Author</NamesBeforeKey>
  <KeyNames>1</KeyNames>
  <ProfessionalAffiliation>
    <ProfessionalPosition>Manager</ProfessionalPosition>
    <AffiliationIdentifier>
      <AffiliationIDType>40</AffiliationIDType>
      <IDValue>01abcde23</IDValue>
    </AffiliationIdentifier>
    <Affiliation>University of Life</Affiliation>
  </ProfessionalAffiliation>
  <BiographicalNote>Author N. 1 is a made-up author</BiographicalNote>
  <Website>
    <WebsiteRole>06</WebsiteRole>
    <WebsiteDescription>Own website</WebsiteDescription>
    <WebsiteLink>https://contributor.site</WebsiteLink>
  </Website>
</Contributor>"#
        );

        // Change all possible values to test that output is updated
        test_contribution.contribution_type = ContributionType::EDITOR;
        test_contribution.contribution_ordinal = 2;
        test_contribution.contributor.orcid = None;
        test_contribution.contributor.website = None;
        test_contribution.first_name = None;
        test_contribution.biography = None;
        test_contribution.affiliations[0].position = None;
        test_contribution.affiliations[0].institution.ror = None;
        let output = generate_test_output(true, &test_contribution);
        println!("{output}");
        assert_eq!(
            output,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<Contributor>
  <SequenceNumber>2</SequenceNumber>
  <ContributorRole>B01</ContributorRole>
  <PersonName>Author N. 1</PersonName>
  <KeyNames>1</KeyNames>
  <ProfessionalAffiliation>
    <Affiliation>University of Life</Affiliation>
  </ProfessionalAffiliation>
</Contributor>"#
        );

        // Test multiple affiliations
        test_contribution
            .affiliations
            .push(WorkContributionsAffiliations {
                position: Some("Janitor".to_string()),
                affiliation_ordinal: 2,
                institution: WorkContributionsAffiliationsInstitution {
                    institution_name: "Institute of Mopping".to_string(),
                    institution_doi: None,
                    ror: Some(Ror::from_str("04k25m262").unwrap()),
                    country_code: None,
                },
            });
        let output = generate_test_output(true, &test_contribution);
        println!("{output}");
        assert!(output.contains(
            r#"
  <ProfessionalAffiliation>
    <Affiliation>University of Life</Affiliation>
  </ProfessionalAffiliation>"#
        ));
        assert!(output.contains(
            r#"
  <ProfessionalAffiliation>
    <ProfessionalPosition>Janitor</ProfessionalPosition>
    <AffiliationIdentifier>
      <AffiliationIDType>40</AffiliationIDType>
      <IDValue>04k25m262</IDValue>
    </AffiliationIdentifier>
    <Affiliation>Institute of Mopping</Affiliation>
  </ProfessionalAffiliation>"#
        ));

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
        test_contribution.contribution_type = ContributionType::RESEARCH_BY;
        let output = generate_test_output(true, &test_contribution);
        assert!(output.contains(r#"  <ContributorRole>A51</ContributorRole>"#));
        test_contribution.contribution_type = ContributionType::CONTRIBUTIONS_BY;
        let output = generate_test_output(true, &test_contribution);
        assert!(output.contains(r#"  <ContributorRole>A32</ContributorRole>"#));
        test_contribution.contribution_type = ContributionType::INDEXER;
        let output = generate_test_output(true, &test_contribution);
        assert!(output.contains(r#"  <ContributorRole>A34</ContributorRole>"#));
    }

    #[test]
    fn test_onix31_thoth_languages() {
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
    fn test_onix31_thoth_issues() {
        let mut test_issue = WorkIssues {
            issue_ordinal: 1,
            series: WorkIssuesSeries {
                series_id: Uuid::parse_str("00000000-0000-0000-BBBB-000000000002").unwrap(),
                series_type: thoth_client::SeriesType::JOURNAL,
                series_name: "Name of series".to_string(),
                issn_print: Some("1234-5678".to_string()),
                issn_digital: Some("8765-4321".to_string()),
                series_url: Some("https://series.url".to_string()),
                series_description: None,
                series_cfp_url: Some("https://series.cfp.url".to_string()),
            },
        };

        // Test standard output
        let output = generate_test_output(true, &test_issue);
        println!("{output}");
        assert!(output.contains(
            r#"
<Collection>
  <CollectionType>10</CollectionType>"#
        ));
        assert!(output.contains(
            r#"
  <CollectionIdentifier>
    <CollectionIDType>01</CollectionIDType>
    <IDTypeName>Series ID</IDTypeName>
    <IDValue>00000000-0000-0000-bbbb-000000000002</IDValue>
  </CollectionIdentifier>"#
        ));
        assert!(output.contains(
            r#"
  <CollectionIdentifier>
    <CollectionIDType>02</CollectionIDType>
    <IDValue>87654321</IDValue>
  </CollectionIdentifier>"#
        ));
        assert!(output.contains(
            r#"
  <CollectionIdentifier>
    <CollectionIDType>01</CollectionIDType>
    <IDTypeName>Series URL</IDTypeName>
    <IDValue>https://series.url</IDValue>
  </CollectionIdentifier>"#
        ));
        assert!(output.contains(
            r#"
  <CollectionIdentifier>
    <CollectionIDType>01</CollectionIDType>
    <IDTypeName>Series Call for Proposals URL</IDTypeName>
    <IDValue>https://series.cfp.url</IDValue>
  </CollectionIdentifier>"#
        ));
        assert!(output.contains(
            r#"
  <CollectionSequence>
    <CollectionSequenceType>03</CollectionSequenceType>
    <CollectionSequenceNumber>1</CollectionSequenceNumber>
  </CollectionSequence>
  <TitleDetail>
    <TitleType>01</TitleType>
    <TitleElement>
      <TitleElementLevel>02</TitleElementLevel>
      <PartNumber>1</PartNumber>
      <TitleText>Name of series</TitleText>
    </TitleElement>
  </TitleDetail>
</Collection>"#
        ));

        // Change all possible values to test that output is updated
        test_issue.issue_ordinal = 2;
        test_issue.series.series_name = "Different series".to_string();
        test_issue.series.issn_digital = Some("1111-2222".to_string());
        test_issue.series.series_url = None;
        test_issue.series.series_cfp_url = None;
        let output = generate_test_output(true, &test_issue);
        println!("{output}");
        assert_eq!(
            output,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<Collection>
  <CollectionType>10</CollectionType>
  <CollectionIdentifier>
    <CollectionIDType>01</CollectionIDType>
    <IDTypeName>Series ID</IDTypeName>
    <IDValue>00000000-0000-0000-bbbb-000000000002</IDValue>
  </CollectionIdentifier>
  <CollectionIdentifier>
    <CollectionIDType>02</CollectionIDType>
    <IDValue>11112222</IDValue>
  </CollectionIdentifier>
  <CollectionSequence>
    <CollectionSequenceType>03</CollectionSequenceType>
    <CollectionSequenceNumber>2</CollectionSequenceNumber>
  </CollectionSequence>
  <TitleDetail>
    <TitleType>01</TitleType>
    <TitleElement>
      <TitleElementLevel>02</TitleElementLevel>
      <PartNumber>2</PartNumber>
      <TitleText>Different series</TitleText>
    </TitleElement>
  </TitleDetail>
</Collection>"#
        );
    }

    #[test]
    fn test_onix31_thoth_fundings() {
        let mut test_funding = WorkFundings {
            program: Some("Name of program".to_string()),
            project_name: Some("Name of project".to_string()),
            project_shortname: Some("Nop".to_string()),
            grant_number: Some("Number of grant".to_string()),
            jurisdiction: Some("Republic of Moldova".to_string()),
            institution: FundingInstitution {
                institution_name: "Name of institution".to_string(),
                institution_doi: Some(
                    Doi::from_str("https://doi.org/10.00001/INSTITUTION.0001").unwrap(),
                ),
                ror: Some(Ror::from_str("https://ror.org/0aaaaaa00").unwrap()),
                country_code: None,
            },
        };

        // Test standard output
        let output = generate_test_output(true, &test_funding);
        println!("{output}");
        assert!(output.contains(r#"<Publisher>"#));
        assert!(output.contains(r#"  <PublishingRole>16</PublishingRole>"#));
        assert!(output.contains(r#"  <PublisherIdentifier>"#));
        assert!(output.contains(r#"    <PublisherIDType>40</PublisherIDType>"#));
        assert!(output.contains(r#"    <IDValue>0aaaaaa00</IDValue>"#));
        assert!(output.contains(r#"    <PublisherIDType>32</PublisherIDType>"#));
        assert!(output.contains(r#"    <IDValue>10.00001/INSTITUTION.0001</IDValue>"#));
        assert!(output.contains(r#"  <PublisherName>Name of institution</PublisherName>"#));
        assert!(output.contains(r#"  <Funding>"#));
        assert!(output.contains(r#"    <FundingIdentifier>"#));
        assert!(output.contains(r#"      <FundingIDType>01</FundingIDType>"#));
        assert!(output.contains(r#"      <IDTypeName>programname</IDTypeName>"#));
        assert!(output.contains(r#"      <IDValue>Name of program</IDValue>"#));
        assert!(output.contains(r#"      <IDTypeName>projectname</IDTypeName>"#));
        assert!(output.contains(r#"      <IDValue>Name of project</IDValue>"#));
        assert!(output.contains(r#"      <IDTypeName>projectshortname</IDTypeName>"#));
        assert!(output.contains(r#"      <IDValue>Nop</IDValue>"#));
        assert!(output.contains(r#"      <IDTypeName>grantnumber</IDTypeName>"#));
        assert!(output.contains(r#"      <IDValue>Number of grant</IDValue>"#));
        assert!(output.contains(r#"      <IDTypeName>jurisdiction</IDTypeName>"#));
        assert!(output.contains(r#"      <IDValue>Republic of Moldova</IDValue>"#));

        // Change all possible values to test that output is updated
        test_funding.institution.institution_name = "Different institution".to_string();
        test_funding.program = None;
        test_funding.institution.institution_doi = None;
        let output = generate_test_output(true, &test_funding);
        assert!(output.contains(r#"<Publisher>"#));
        assert!(output.contains(r#"  <PublishingRole>16</PublishingRole>"#));
        assert!(output.contains(r#"  <PublisherIdentifier>"#));
        assert!(output.contains(r#"    <PublisherIDType>40</PublisherIDType>"#));
        assert!(output.contains(r#"    <IDValue>0aaaaaa00</IDValue>"#));
        // No DOI supplied
        assert!(!output.contains(r#"    <PublisherIDType>32</PublisherIDType>"#));
        assert!(!output.contains(r#"    <IDValue>10.00001/INSTITUTION.0001</IDValue>"#));
        assert!(output.contains(r#"  <PublisherName>Different institution</PublisherName>"#));
        assert!(output.contains(r#"  <Funding>"#));
        assert!(output.contains(r#"    <FundingIdentifier>"#));
        assert!(output.contains(r#"      <FundingIDType>01</FundingIDType>"#));
        // No program supplied
        assert!(!output.contains(r#"      <IDTypeName>programname</IDTypeName>"#));
        assert!(!output.contains(r#"      <IDValue>Name of program</IDValue>"#));
        assert!(output.contains(r#"      <IDTypeName>projectname</IDTypeName>"#));
        assert!(output.contains(r#"      <IDValue>Name of project</IDValue>"#));
        assert!(output.contains(r#"      <IDTypeName>projectshortname</IDTypeName>"#));
        assert!(output.contains(r#"      <IDValue>Nop</IDValue>"#));
        assert!(output.contains(r#"      <IDTypeName>grantnumber</IDTypeName>"#));
        assert!(output.contains(r#"      <IDValue>Number of grant</IDValue>"#));
        assert!(output.contains(r#"      <IDTypeName>jurisdiction</IDTypeName>"#));
        assert!(output.contains(r#"      <IDValue>Republic of Moldova</IDValue>"#));

        test_funding.project_name = None;
        test_funding.institution.ror = None;
        let output = generate_test_output(true, &test_funding);
        assert!(output.contains(r#"<Publisher>"#));
        assert!(output.contains(r#"  <PublishingRole>16</PublishingRole>"#));
        assert!(output.contains(r#"  <PublisherName>Different institution</PublisherName>"#));
        // No DOI or ROR supplied, so PublisherIdentifier block is omitted completely
        assert!(!output.contains(r#"  <PublisherIdentifier>"#));
        assert!(!output.contains(r#"    <PublisherIDType>40</PublisherIDType>"#));
        assert!(!output.contains(r#"    <IDValue>0aaaaaa00</IDValue>"#));
        assert!(!output.contains(r#"    <PublisherIDType>32</PublisherIDType>"#));
        assert!(!output.contains(r#"    <IDValue>10.00001/INSTITUTION.0001</IDValue>"#));
        assert!(output.contains(r#"  <Funding>"#));
        assert!(output.contains(r#"    <FundingIdentifier>"#));
        assert!(output.contains(r#"      <FundingIDType>01</FundingIDType>"#));
        // No program supplied
        assert!(!output.contains(r#"      <IDTypeName>programname</IDTypeName>"#));
        assert!(!output.contains(r#"      <IDValue>Name of program</IDValue>"#));
        // No project supplied
        assert!(!output.contains(r#"      <IDTypeName>projectname</IDTypeName>"#));
        assert!(!output.contains(r#"      <IDValue>Name of project</IDValue>"#));
        assert!(output.contains(r#"      <IDTypeName>projectshortname</IDTypeName>"#));
        assert!(output.contains(r#"      <IDValue>Nop</IDValue>"#));
        assert!(output.contains(r#"      <IDTypeName>grantnumber</IDTypeName>"#));
        assert!(output.contains(r#"      <IDValue>Number of grant</IDValue>"#));
        assert!(output.contains(r#"      <IDTypeName>jurisdiction</IDTypeName>"#));
        assert!(output.contains(r#"      <IDValue>Republic of Moldova</IDValue>"#));

        test_funding.project_shortname = None;
        let output = generate_test_output(true, &test_funding);
        assert!(output.contains(r#"<Publisher>"#));
        assert!(output.contains(r#"  <PublishingRole>16</PublishingRole>"#));
        assert!(!output.contains(r#"  <PublisherIdentifier>"#));
        assert!(!output.contains(r#"    <PublisherIDType>40</PublisherIDType>"#));
        assert!(!output.contains(r#"    <IDValue>0aaaaaa00</IDValue>"#));
        assert!(!output.contains(r#"    <PublisherIDType>32</PublisherIDType>"#));
        assert!(!output.contains(r#"    <IDValue>10.00001/INSTITUTION.0001</IDValue>"#));
        assert!(output.contains(r#"  <PublisherName>Different institution</PublisherName>"#));
        assert!(output.contains(r#"  <Funding>"#));
        assert!(output.contains(r#"    <FundingIdentifier>"#));
        assert!(output.contains(r#"      <FundingIDType>01</FundingIDType>"#));
        // No program supplied
        assert!(!output.contains(r#"      <IDTypeName>programname</IDTypeName>"#));
        assert!(!output.contains(r#"      <IDValue>Name of program</IDValue>"#));
        // No project supplied
        assert!(!output.contains(r#"      <IDTypeName>projectname</IDTypeName>"#));
        assert!(!output.contains(r#"      <IDValue>Name of project</IDValue>"#));
        // No short name supplied
        assert!(!output.contains(r#"      <IDTypeName>projectshortname</IDTypeName>"#));
        assert!(!output.contains(r#"      <IDValue>Nop</IDValue>"#));
        assert!(output.contains(r#"      <IDTypeName>grantnumber</IDTypeName>"#));
        assert!(output.contains(r#"      <IDValue>Number of grant</IDValue>"#));
        assert!(output.contains(r#"      <IDTypeName>jurisdiction</IDTypeName>"#));
        assert!(output.contains(r#"      <IDValue>Republic of Moldova</IDValue>"#));

        test_funding.grant_number = None;
        let output = generate_test_output(true, &test_funding);
        assert!(output.contains(r#"<Publisher>"#));
        assert!(output.contains(r#"  <PublishingRole>16</PublishingRole>"#));
        assert!(!output.contains(r#"  <PublisherIdentifier>"#));
        assert!(!output.contains(r#"    <PublisherIDType>40</PublisherIDType>"#));
        assert!(!output.contains(r#"    <IDValue>0aaaaaa00</IDValue>"#));
        assert!(!output.contains(r#"    <PublisherIDType>32</PublisherIDType>"#));
        assert!(!output.contains(r#"    <IDValue>10.00001/INSTITUTION.0001</IDValue>"#));
        assert!(output.contains(r#"  <PublisherName>Different institution</PublisherName>"#));
        assert!(output.contains(r#"  <Funding>"#));
        assert!(output.contains(r#"    <FundingIdentifier>"#));
        assert!(output.contains(r#"      <FundingIDType>01</FundingIDType>"#));
        // No program supplied
        assert!(!output.contains(r#"      <IDTypeName>programname</IDTypeName>"#));
        assert!(!output.contains(r#"      <IDValue>Name of program</IDValue>"#));
        // No project supplied
        assert!(!output.contains(r#"      <IDTypeName>projectname</IDTypeName>"#));
        assert!(!output.contains(r#"      <IDValue>Name of project</IDValue>"#));
        // No short name supplied
        assert!(!output.contains(r#"      <IDTypeName>projectshortname</IDTypeName>"#));
        assert!(!output.contains(r#"      <IDValue>Nop</IDValue>"#));
        // No grant supplied
        assert!(!output.contains(r#"      <IDTypeName>grantnumber</IDTypeName>"#));
        assert!(!output.contains(r#"      <IDValue>Number of grant</IDValue>"#));
        assert!(output.contains(r#"      <IDTypeName>jurisdiction</IDTypeName>"#));
        assert!(output.contains(r#"      <IDValue>Republic of Moldova</IDValue>"#));

        test_funding.jurisdiction = None;
        let output = generate_test_output(true, &test_funding);
        assert!(output.contains(r#"<Publisher>"#));
        assert!(output.contains(r#"  <PublishingRole>16</PublishingRole>"#));
        assert!(!output.contains(r#"  <PublisherIdentifier>"#));
        assert!(!output.contains(r#"    <PublisherIDType>40</PublisherIDType>"#));
        assert!(!output.contains(r#"    <IDValue>0aaaaaa00</IDValue>"#));
        assert!(!output.contains(r#"    <PublisherIDType>32</PublisherIDType>"#));
        assert!(!output.contains(r#"    <IDValue>10.00001/INSTITUTION.0001</IDValue>"#));
        assert!(output.contains(r#"  <PublisherName>Different institution</PublisherName>"#));
        // No program, project, short name, grant or jurisdiction supplied,
        // so Funding block is omitted completely
        assert!(!output.contains(r#"  <Funding>"#));
        assert!(!output.contains(r#"    <FundingIdentifier>"#));
        assert!(!output.contains(r#"      <FundingIDType>01</FundingIDType>"#));
        assert!(!output.contains(r#"      <IDTypeName>programname</IDTypeName>"#));
        assert!(!output.contains(r#"      <IDValue>Name of program</IDValue>"#));
        assert!(!output.contains(r#"      <IDTypeName>projectname</IDTypeName>"#));
        assert!(!output.contains(r#"      <IDValue>Name of project</IDValue>"#));
        assert!(!output.contains(r#"      <IDTypeName>projectshortname</IDTypeName>"#));
        assert!(!output.contains(r#"      <IDValue>Nop</IDValue>"#));
        assert!(!output.contains(r#"      <IDTypeName>grantnumber</IDTypeName>"#));
        assert!(!output.contains(r#"      <IDValue>Number of grant</IDValue>"#));
        assert!(!output.contains(r#"      <IDTypeName>jurisdiction</IDTypeName>"#));
        assert!(!output.contains(r#"      <IDValue>Republic of Moldova</IDValue>"#));
    }

    #[test]
    fn test_onix31_thoth_references() {
        let mut test_reference = WorkReferences {
            reference_ordinal: 1,
            doi: Some(Doi::from_str("https://doi.org/10.00001/reference").unwrap()),
            unstructured_citation: Some("Author, A. (2022) Article, Journal.".to_string()),
            issn: None,
            isbn: None,
            journal_title: None,
            article_title: None,
            series_title: None,
            volume_title: None,
            edition: None,
            author: None,
            volume: None,
            issue: None,
            first_page: None,
            component_number: None,
            standard_designator: None,
            standards_body_name: None,
            standards_body_acronym: None,
            publication_date: None,
            retrieval_date: None,
        };

        // Test standard output
        let output = generate_test_output(true, &test_reference);
        println!("{output}");
        assert_eq!(
            output,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<RelatedProduct>
  <ProductRelationCode>34</ProductRelationCode>
  <ProductIdentifier>
    <ProductIDType>06</ProductIDType>
    <IDValue>10.00001/reference</IDValue>
  </ProductIdentifier>
</RelatedProduct>"#
        );

        // Remove DOI
        test_reference.doi = None;
        let output = generate_test_output(true, &test_reference);
        println!("{output}");
        assert_eq!(
            output,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<RelatedProduct>
  <ProductRelationCode>34</ProductRelationCode>
  <ProductIdentifier>
    <ProductIDType>01</ProductIDType>
    <IDTypeName>Unstructured citation</IDTypeName>
    <IDValue>Author, A. (2022) Article, Journal.</IDValue>
  </ProductIdentifier>
</RelatedProduct>"#
        );
    }

    #[test]
    fn test_onix31_thoth_relations() {
        let mut test_relation = WorkRelations {
            relation_type: RelationType::HAS_TRANSLATION,
            relation_ordinal: 1,
            related_work: WorkRelationsRelatedWork {
                work_status: WorkStatus::ACTIVE,
                titles: vec![thoth_client::WorkRelationsRelatedWorkTitles {
                    title_id: Uuid::from_str("00000000-0000-0000-CCCC-000000000001").unwrap(),
                    locale_code: thoth_client::LocaleCode::EN,
                    full_title: "N/A".to_string(),
                    title: "N/A".to_string(),
                    subtitle: None,
                    canonical: true,
                }],
                abstracts: vec![
                        thoth_client::WorkRelationsRelatedWorkAbstracts {
                            abstract_id: Uuid::from_str("00000000-0000-0000-AAAA-000000000001").unwrap(),
                            work_id: Uuid::from_str("00000000-0000-0000-AAAA-000000000001").unwrap(),
                            content: "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Vestibulum vel libero eleifend, ultrices purus vitae, suscipit ligula. Aliquam ornare quam et nulla vestibulum, id euismod tellus malesuada. Orci varius natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. Nullam ornare bibendum ex nec dapibus. Proin porta risus elementum odio feugiat tempus. Etiam eu felis ac metus viverra ornare. In consectetur neque sed feugiat ornare. Mauris at purus fringilla orci tincidunt pulvinar sed a massa. Nullam vestibulum posuere augue, sit amet tincidunt nisl pulvinar ac.".to_string(),
                            locale_code: thoth_client::LocaleCode::EN,
                            abstract_type: thoth_client::AbstractType::SHORT,
                            canonical: true,
                        },
                    ],
                edition: None,
                doi: Some(Doi::from_str("https://doi.org/10.00001/RELATION.0001").unwrap()),
                publication_date: None,
                withdrawn_date: None,
                license: None,
                copyright_holder: None,
                // short_abstract: None,
                // long_abstract: None,
                general_note: None,
                place: None,
                first_page: None,
                last_page: None,
                page_count: None,
                page_interval: None,
                landing_page: None,
                imprint: WorkRelationsRelatedWorkImprint {
                    crossmark_doi: None,
                    publisher: WorkRelationsRelatedWorkImprintPublisher {
                        publisher_name: "N/A".to_string(),
                    },
                },
                contributions: vec![],
                languages: vec![],
                publications: vec![],
                references: vec![],
                fundings: vec![],
            },
        };

        // Test RelatedWork type
        let output = generate_test_output(true, &test_relation);
        println!("{output}");
        assert_eq!(
            output,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<RelatedWork>
  <WorkRelationCode>49</WorkRelationCode>
  <WorkIdentifier>
    <WorkIDType>06</WorkIDType>
    <IDValue>10.00001/RELATION.0001</IDValue>
  </WorkIdentifier>
</RelatedWork>"#
        );

        // Test RelatedProduct type, change DOI
        test_relation.relation_type = RelationType::HAS_PART;
        test_relation.related_work.doi =
            Some(Doi::from_str("https://doi.org/10.00002/RELATION.0002").unwrap());
        let output = generate_test_output(true, &test_relation);
        println!("{output}");
        assert_eq!(
            output,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<RelatedProduct>
  <ProductRelationCode>01</ProductRelationCode>
  <ProductIdentifier>
    <ProductIDType>06</ProductIDType>
    <IDValue>10.00002/RELATION.0002</IDValue>
  </ProductIdentifier>
</RelatedProduct>"#
        );

        // Test all other relation codes
        test_relation.relation_type = RelationType::IS_TRANSLATION_OF;
        let output = generate_test_output(true, &test_relation);
        assert!(output.contains(r#"  <WorkRelationCode>29</WorkRelationCode>"#));
        test_relation.relation_type = RelationType::IS_PART_OF;
        let output = generate_test_output(true, &test_relation);
        assert!(output.contains(r#"  <ProductRelationCode>02</ProductRelationCode>"#));
        test_relation.relation_type = RelationType::REPLACES;
        let output = generate_test_output(true, &test_relation);
        assert!(output.contains(r#"  <ProductRelationCode>03</ProductRelationCode>"#));
        test_relation.relation_type = RelationType::IS_REPLACED_BY;
        let output = generate_test_output(true, &test_relation);
        assert!(output.contains(r#"  <ProductRelationCode>05</ProductRelationCode>"#));
    }

    #[test]
    fn test_onix31_thoth_works() {
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
            abstracts: vec![
                thoth_client::WorkAbstracts {
                    abstract_id: Uuid::from_str("00000000-0000-0000-AAAA-000000000001").unwrap(),
                    work_id: Uuid::from_str("00000000-0000-0000-AAAA-000000000001").unwrap(),
                    content: "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Vestibulum vel libero eleifend, ultrices purus vitae, suscipit ligula. Aliquam ornare quam et nulla vestibulum, id euismod tellus malesuada. Orci varius natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. Nullam ornare bibendum ex nec dapibus. Proin porta risus elementum odio feugiat tempus. Etiam eu felis ac metus viverra ornare. In consectetur neque sed feugiat ornare. Mauris at purus fringilla orci tincidunt pulvinar sed a massa. Nullam vestibulum posuere augue, sit amet tincidunt nisl pulvinar ac.".to_string(),
                    locale_code: thoth_client::LocaleCode::EN,
                    abstract_type: thoth_client::AbstractType::SHORT,
                    canonical: true,
                },
            ],
            work_type: WorkType::MONOGRAPH,
            reference: Some("IntRef1".to_string()),
            edition: Some(2),
            doi: Some(Doi::from_str("https://doi.org/10.00001/BOOK.0001").unwrap()),
            publication_date: chrono::NaiveDate::from_ymd_opt(1999, 12, 31),
            withdrawn_date: None,
            license: Some("https://creativecommons.org/licenses/by/4.0/".to_string()),
            copyright_holder: Some("Author 1; Author 2".to_string()),
            // short_abstract: Some("Lorem ipsum dolor sit amet.".to_string()),
            // long_abstract: Some(
            //     "Lorem ipsum dolor sit amet, consectetur adipiscing elit.".to_string(),
            // ),
            general_note: Some("This is a general note".to_string()),
            bibliography_note: Some("This is a bibliography note".to_string()),
            place: Some("León, Spain".to_string()),
            page_count: Some(334),
            page_breakdown: None,
            first_page: None,
            last_page: None,
            page_interval: None,
            image_count: Some(15),
            table_count: Some(20),
            audio_count: Some(25),
            video_count: Some(30),
            landing_page: Some("https://www.book.com".to_string()),
            toc: Some("1. Chapter 1".to_string()),
            lccn: Some("123456789".to_string()),
            oclc: Some("987654321".to_string()),
            cover_url: Some("https://www.book.com/cover".to_string()),
            cover_caption: Some("This is a cover caption".to_string()),
            imprint: WorkImprint {
                imprint_name: "OA Editions Imprint".to_string(),
                imprint_url: Some("https://imprint.oa".to_string()),
                crossmark_doi: None,
                publisher: WorkImprintPublisher {
                    publisher_name: "OA Editions".to_string(),
                    publisher_shortname: None,
                    publisher_url: Some("https://publisher.oa".to_string()),
                },
            },
            issues: vec![],
            contributions: vec![],
            languages: vec![],
            publications: vec![WorkPublications {
                publication_id: Uuid::from_str("00000000-0000-0000-BBBB-000000000001").unwrap(),
                publication_type: PublicationType::PAPERBACK,
                isbn: Some(Isbn::from_str("978-3-16-148410-0").unwrap()),
                width_mm: Some(156.0),
                width_cm: Some(15.6),
                width_in: Some(6.14),
                height_mm: Some(234.0),
                height_cm: Some(23.4),
                height_in: Some(9.21),
                depth_mm: Some(25.0),
                depth_cm: Some(2.5),
                depth_in: Some(1.0),
                weight_g: Some(152.0),
                weight_oz: Some(5.3616),
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
                        unit_price: 8.0,
                    },
                ],
                locations: vec![
                    WorkPublicationsLocations {
                        landing_page: Some("https://www.book.com/pb_landing".to_string()),
                        full_text_url: None,
                        location_platform: LocationPlatform::PUBLISHER_WEBSITE,
                        canonical: true,
                    },
                    WorkPublicationsLocations {
                        landing_page: Some("https://www.jstor.com/pb_landing".to_string()),
                        // Note a paperback can't technically have a Full Text URL - test purposes only
                        full_text_url: Some("https://www.jstor.com/pb_fulltext".to_string()),
                        location_platform: LocationPlatform::JSTOR,
                        canonical: false,
                    },
                ],
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
                    subject_ordinal: 1,
                },
                WorkSubjects {
                    subject_code: "custom2".to_string(),
                    subject_type: SubjectType::CUSTOM,
                    subject_ordinal: 1,
                },
            ],
            fundings: vec![],
            relations: vec![
                WorkRelations {
                    relation_type: RelationType::HAS_CHILD,
                    relation_ordinal: 1,
                    related_work: WorkRelationsRelatedWork {
                        work_status: WorkStatus::ACTIVE,
                        titles: vec![thoth_client::WorkRelationsRelatedWorkTitles {
                            title_id: Uuid::from_str("00000000-0000-0000-CCCC-000000000001")
                                .unwrap(),
                            locale_code: thoth_client::LocaleCode::EN,
                            full_title: "Related work title".to_string(),
                            title: "The first chapter:".to_string(),
                            subtitle: Some("An introduction".to_string()),
                            canonical: true,
                        }],
                        abstracts: vec![
                        thoth_client::WorkRelationsRelatedWorkAbstracts {
                                abstract_id: Uuid::from_str("00000000-0000-0000-AAAA-000000000001").unwrap(),
                                work_id: Uuid::from_str("00000000-0000-0000-AAAA-000000000001").unwrap(),
                                content: "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Vestibulum vel libero eleifend, ultrices purus vitae, suscipit ligula. Aliquam ornare quam et nulla vestibulum, id euismod tellus malesuada. Orci varius natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. Nullam ornare bibendum ex nec dapibus. Proin porta risus elementum odio feugiat tempus. Etiam eu felis ac metus viverra ornare. In consectetur neque sed feugiat ornare. Mauris at purus fringilla orci tincidunt pulvinar sed a massa. Nullam vestibulum posuere augue, sit amet tincidunt nisl pulvinar ac.".to_string(),
                                locale_code: thoth_client::LocaleCode::EN,
                                abstract_type: thoth_client::AbstractType::SHORT,
                                canonical: true,
                            },
                        ],
                        edition: None,
                        doi: Some(Doi::from_str("https://doi.org/10.00001/RELATION.0001").unwrap()),
                        publication_date: None,
                        withdrawn_date: None,
                        license: Some(
                            "https://creativecommons.org/licenses/by-sa/4.0/".to_string(),
                        ),
                        copyright_holder: Some("Chapter Author 1; Chapter Author 2".to_string()),
                        // short_abstract: Some(
                        //     "This is a chapter's very short abstract.".to_string(),
                        // ),
                        // long_abstract: Some(
                        //     "This is a chapter's somewhat longer abstract. It has two sentences."
                        //         .to_string(),
                        // ),
                        general_note: Some("This is a chapter general note.".to_string()),
                        place: None,
                        first_page: Some("10".to_string()),
                        last_page: Some("20".to_string()),
                        page_count: Some(11),
                        page_interval: None,
                        landing_page: None,
                        imprint: WorkRelationsRelatedWorkImprint {
                            crossmark_doi: None,
                            publisher: WorkRelationsRelatedWorkImprintPublisher {
                                publisher_name: "N/A".to_string(),
                            },
                        },
                        contributions: vec![WorkRelationsRelatedWorkContributions {
                            contribution_type: ContributionType::AUTHOR,
                            first_name: Some("Chapter Author".to_string()),
                            last_name: "2".to_string(),
                            full_name: "Chapter Author N. 2".to_string(),
                            biography: Some("Chapter Author N. 2 is a made-up author".to_string()),
                            contribution_ordinal: 1,
                            contributor: WorkRelationsRelatedWorkContributionsContributor {
                                orcid: Some(
                                    Orcid::from_str("https://orcid.org/0000-0003-0000-0002")
                                        .unwrap(),
                                ),
                                website: Some("https://chaptercontributor.site".to_string()),
                            },
                            affiliations: vec![WorkRelationsRelatedWorkContributionsAffiliations {
                                position: Some("Chapter Manager".to_string()),
                                affiliation_ordinal: 1,
                                institution:
                                    WorkRelationsRelatedWorkContributionsAffiliationsInstitution {
                                        institution_name: "University of Chapters".to_string(),
                                        ror: Some(Ror::from_str("08abcde89").unwrap()),
                                    },
                            }],
                        }],
                        publications: vec![],
                        references: vec![WorkRelationsRelatedWorkReferences {
                            reference_ordinal: 1,
                            doi: Some(
                                Doi::from_str("https://doi.org/10.00005/chapter_reference")
                                    .unwrap(),
                            ),
                            unstructured_citation: None,
                            issn: None,
                            isbn: None,
                            journal_title: None,
                            article_title: None,
                            series_title: None,
                            volume_title: None,
                            edition: None,
                            author: None,
                            volume: None,
                            issue: None,
                            first_page: None,
                            component_number: None,
                            standard_designator: None,
                            standards_body_name: None,
                            standards_body_acronym: None,
                            publication_date: None,
                            retrieval_date: None,
                        }],
                        fundings: vec![],
                        languages: vec![WorkRelationsRelatedWorkLanguages {
                            language_code: LanguageCode::BTK,
                            language_relation: LanguageRelation::ORIGINAL,
                            main_language: true,
                        }],
                    },
                },
                WorkRelations {
                    relation_type: RelationType::HAS_PART,
                    relation_ordinal: 2,
                    related_work: WorkRelationsRelatedWork {
                        work_status: WorkStatus::ACTIVE,
                        titles: vec![thoth_client::WorkRelationsRelatedWorkTitles {
                            title_id: Uuid::from_str("00000000-0000-0000-CCCC-000000000001")
                                .unwrap(),
                            locale_code: thoth_client::LocaleCode::EN,
                            full_title: "N/A".to_string(),
                            title: "N/A".to_string(),
                            subtitle: None,
                            canonical: true,
                        }],
                        abstracts: vec![
                            thoth_client::WorkRelationsRelatedWorkAbstracts {
                                abstract_id: Uuid::from_str("00000000-0000-0000-AAAA-000000000001").unwrap(),
                                work_id: Uuid::from_str("00000000-0000-0000-AAAA-000000000001").unwrap(),
                                content: "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Vestibulum vel libero eleifend, ultrices purus vitae, suscipit ligula. Aliquam ornare quam et nulla vestibulum, id euismod tellus malesuada. Orci varius natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. Nullam ornare bibendum ex nec dapibus. Proin porta risus elementum odio feugiat tempus. Etiam eu felis ac metus viverra ornare. In consectetur neque sed feugiat ornare. Mauris at purus fringilla orci tincidunt pulvinar sed a massa. Nullam vestibulum posuere augue, sit amet tincidunt nisl pulvinar ac.".to_string(),
                                locale_code: thoth_client::LocaleCode::EN,
                                abstract_type: thoth_client::AbstractType::SHORT,
                                canonical: true,
                            },
                        ],
                        edition: None,
                        doi: Some(Doi::from_str("https://doi.org/10.00001/RELATION.0002").unwrap()),
                        publication_date: None,
                        withdrawn_date: None,
                        license: None,
                        copyright_holder: None,
                        // short_abstract: None,
                        // long_abstract: None,
                        general_note: None,
                        place: None,
                        first_page: None,
                        last_page: None,
                        page_count: None,
                        page_interval: None,
                        landing_page: None,
                        imprint: WorkRelationsRelatedWorkImprint {
                            crossmark_doi: None,
                            publisher: WorkRelationsRelatedWorkImprintPublisher {
                                publisher_name: "N/A".to_string(),
                            },
                        },
                        contributions: vec![],
                        publications: vec![],
                        references: vec![],
                        fundings: vec![],
                        languages: vec![],
                    },
                },
                WorkRelations {
                    relation_type: RelationType::HAS_TRANSLATION,
                    relation_ordinal: 3,
                    related_work: WorkRelationsRelatedWork {
                        work_status: WorkStatus::ACTIVE,
                        titles: vec![thoth_client::WorkRelationsRelatedWorkTitles {
                            title_id: Uuid::from_str("00000000-0000-0000-CCCC-000000000001")
                                .unwrap(),
                            locale_code: thoth_client::LocaleCode::EN,
                            full_title: "N/A".to_string(),
                            title: "N/A".to_string(),
                            subtitle: None,
                            canonical: true,
                        }],
                        abstracts: vec![
                            thoth_client::WorkRelationsRelatedWorkAbstracts {
                                abstract_id: Uuid::from_str("00000000-0000-0000-AAAA-000000000001").unwrap(),
                                work_id: Uuid::from_str("00000000-0000-0000-AAAA-000000000001").unwrap(),
                                content: "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Vestibulum vel libero eleifend, ultrices purus vitae, suscipit ligula. Aliquam ornare quam et nulla vestibulum, id euismod tellus malesuada. Orci varius natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. Nullam ornare bibendum ex nec dapibus. Proin porta risus elementum odio feugiat tempus. Etiam eu felis ac metus viverra ornare. In consectetur neque sed feugiat ornare. Mauris at purus fringilla orci tincidunt pulvinar sed a massa. Nullam vestibulum posuere augue, sit amet tincidunt nisl pulvinar ac.".to_string(),
                                locale_code: thoth_client::LocaleCode::EN,
                                abstract_type: thoth_client::AbstractType::SHORT,
                                canonical: true,
                            },
                        ],
                        edition: None,
                        doi: Some(Doi::from_str("https://doi.org/10.00001/RELATION.0003").unwrap()),
                        publication_date: None,
                        withdrawn_date: None,
                        license: None,
                        copyright_holder: None,
                        // short_abstract: None,
                        // long_abstract: None,
                        general_note: None,
                        place: None,
                        first_page: None,
                        last_page: None,
                        page_count: None,
                        page_interval: None,
                        landing_page: None,
                        imprint: WorkRelationsRelatedWorkImprint {
                            crossmark_doi: None,
                            publisher: WorkRelationsRelatedWorkImprintPublisher {
                                publisher_name: "N/A".to_string(),
                            },
                        },
                        contributions: vec![],
                        publications: vec![],
                        references: vec![],
                        fundings: vec![],
                        languages: vec![],
                    },
                },
            ],
            references: vec![WorkReferences {
                reference_ordinal: 1,
                doi: Some(Doi::from_str("https://doi.org/10.00001/reference").unwrap()),
                unstructured_citation: None,
                issn: None,
                isbn: None,
                journal_title: None,
                article_title: None,
                series_title: None,
                volume_title: None,
                edition: None,
                author: None,
                volume: None,
                issue: None,
                first_page: None,
                component_number: None,
                standard_designator: None,
                standards_body_name: None,
                standards_body_acronym: None,
                publication_date: None,
                retrieval_date: None,
            }],
        };

        // Test standard output
        let output = generate_test_output(true, &test_work);
        println!("{output}");
        // Non-repeatable blocks should appear in a guaranteed order
        assert!(output.contains(
            r#"
<Product>
  <RecordReference>urn:uuid:00000000-0000-0000-bbbb-000000000001</RecordReference>
  <NotificationType>03</NotificationType>
  <RecordSourceType>01</RecordSourceType>"#
        ));
        assert!(output.contains(
            r#"
  <ProductIdentifier>
    <ProductIDType>01</ProductIDType>
    <IDTypeName>thoth-work-id</IDTypeName>
    <IDValue>urn:uuid:00000000-0000-0000-aaaa-000000000001</IDValue>
  </ProductIdentifier>"#
        ));
        assert!(output.contains(
            r#"
  <ProductIdentifier>
    <ProductIDType>01</ProductIDType>
    <IDTypeName>thoth-publication-id</IDTypeName>
    <IDValue>urn:uuid:00000000-0000-0000-bbbb-000000000001</IDValue>
  </ProductIdentifier>"#
        ));
        assert!(output.contains(
            r#"
  <ProductIdentifier>
    <ProductIDType>15</ProductIDType>
    <IDValue>9783161484100</IDValue>
  </ProductIdentifier>"#
        ));
        assert!(output.contains(
            r#"
  <ProductIdentifier>
    <ProductIDType>06</ProductIDType>
    <IDValue>10.00001/BOOK.0001</IDValue>
  </ProductIdentifier>"#
        ));
        assert!(output.contains(
            r#"
  <ProductIdentifier>
    <ProductIDType>13</ProductIDType>
    <IDValue>123456789</IDValue>
  </ProductIdentifier>"#
        ));
        assert!(output.contains(
            r#"
  <ProductIdentifier>
    <ProductIDType>23</ProductIDType>
    <IDValue>987654321</IDValue>
  </ProductIdentifier>"#
        ));
        assert!(output.contains(
            r#"
  <ProductIdentifier>
    <ProductIDType>01</ProductIDType>
    <IDTypeName>internal-reference</IDTypeName>
    <IDValue>IntRef1</IDValue>
  </ProductIdentifier>"#
        ));
        assert!(output.contains(
            r#"
  <DescriptiveDetail>
    <ProductComposition>00</ProductComposition>
    <ProductForm>BC</ProductForm>
    <PrimaryContentType>10</PrimaryContentType>"#
        ));
        assert!(output.contains(
            r#"
    <Measure>
      <MeasureType>01</MeasureType>
      <Measurement>234</Measurement>
      <MeasureUnitCode>mm</MeasureUnitCode>
    </Measure>"#
        ));
        assert!(output.contains(
            r#"
    <Measure>
      <MeasureType>01</MeasureType>
      <Measurement>23.4</Measurement>
      <MeasureUnitCode>cm</MeasureUnitCode>
    </Measure>"#
        ));
        assert!(output.contains(
            r#"
    <Measure>
      <MeasureType>01</MeasureType>
      <Measurement>9.21</Measurement>
      <MeasureUnitCode>in</MeasureUnitCode>
    </Measure>"#
        ));
        assert!(output.contains(
            r#"
    <Measure>
      <MeasureType>02</MeasureType>
      <Measurement>156</Measurement>
      <MeasureUnitCode>mm</MeasureUnitCode>
    </Measure>"#
        ));
        assert!(output.contains(
            r#"
    <Measure>
      <MeasureType>02</MeasureType>
      <Measurement>15.6</Measurement>
      <MeasureUnitCode>cm</MeasureUnitCode>
    </Measure>"#
        ));
        assert!(output.contains(
            r#"
    <Measure>
      <MeasureType>02</MeasureType>
      <Measurement>6.14</Measurement>
      <MeasureUnitCode>in</MeasureUnitCode>
    </Measure>"#
        ));
        assert!(output.contains(
            r#"
    <Measure>
      <MeasureType>03</MeasureType>
      <Measurement>25</Measurement>
      <MeasureUnitCode>mm</MeasureUnitCode>
    </Measure>"#
        ));
        assert!(output.contains(
            r#"
    <Measure>
      <MeasureType>03</MeasureType>
      <Measurement>2.5</Measurement>
      <MeasureUnitCode>cm</MeasureUnitCode>
    </Measure>"#
        ));
        assert!(output.contains(
            r#"
    <Measure>
      <MeasureType>03</MeasureType>
      <Measurement>1</Measurement>
      <MeasureUnitCode>in</MeasureUnitCode>
    </Measure>"#
        ));
        assert!(output.contains(
            r#"
    <Measure>
      <MeasureType>08</MeasureType>
      <Measurement>152</Measurement>
      <MeasureUnitCode>gr</MeasureUnitCode>
    </Measure>"#
        ));
        assert!(output.contains(
            r#"
    <Measure>
      <MeasureType>08</MeasureType>
      <Measurement>5.3616</Measurement>
      <MeasureUnitCode>oz</MeasureUnitCode>
    </Measure>"#
        ));
        assert!(output.contains(
            r#"
    <EpubLicense>
      <EpubLicenseName>Creative Commons Attribution 4.0 International license (CC BY 4.0).</EpubLicenseName>
      <EpubLicenseExpression>
        <EpubLicenseExpressionType>02</EpubLicenseExpressionType>
        <EpubLicenseExpressionLink>https://creativecommons.org/licenses/by/4.0/</EpubLicenseExpressionLink>
      </EpubLicenseExpression>
    </EpubLicense>
    <TitleDetail>
      <TitleType>01</TitleType>
      <TitleElement>
        <TitleElementLevel>01</TitleElementLevel>
        <TitleText>Book Title</TitleText>
        <Subtitle>Book Subtitle</Subtitle>
      </TitleElement>
    </TitleDetail>
    <Edition>
      <EditionNumber>2</EditionNumber>
    </Edition>
    <Extent>
      <ExtentType>00</ExtentType>
      <ExtentValue>334</ExtentValue>
      <ExtentUnit>03</ExtentUnit>
    </Extent>
    <IllustrationsNote>This is a bibliography note</IllustrationsNote>"#));
        assert!(output.contains(
            r#"
    <AncillaryContent>
      <AncillaryContentType>09</AncillaryContentType>
      <Number>15</Number>
    </AncillaryContent>"#
        ));
        assert!(output.contains(
            r#"
    <AncillaryContent>
      <AncillaryContentType>11</AncillaryContentType>
      <Number>20</Number>
    </AncillaryContent>"#
        ));
        assert!(output.contains(
            r#"
    <AncillaryContent>
      <AncillaryContentType>19</AncillaryContentType>
      <Number>25</Number>
    </AncillaryContent>"#
        ));
        assert!(output.contains(
            r#"
    <AncillaryContent>
      <AncillaryContentType>00</AncillaryContentType>
      <AncillaryContentDescription>Videos</AncillaryContentDescription>
      <Number>30</Number>
    </AncillaryContent>"#
        ));
        assert!(output.contains(
            r#"
    <Subject>
      <MainSubject />
      <SubjectSchemeIdentifier>12</SubjectSchemeIdentifier>
      <SubjectCode>AAB</SubjectCode>
    </Subject>"#
        ));
        assert!(output.contains(
            r#"
    <Subject>
      <SubjectSchemeIdentifier>10</SubjectSchemeIdentifier>
      <SubjectCode>AAA000000</SubjectCode>
    </Subject>"#
        ));
        assert!(output.contains(
            r#"
    <Subject>
      <SubjectSchemeIdentifier>04</SubjectSchemeIdentifier>
      <SubjectCode>JA85</SubjectCode>
    </Subject>"#
        ));
        assert!(output.contains(
            r#"
    <Subject>
      <SubjectSchemeIdentifier>93</SubjectSchemeIdentifier>
      <SubjectCode>JWA</SubjectCode>
    </Subject>"#
        ));
        assert!(output.contains(
            r#"
    <Subject>
      <SubjectSchemeIdentifier>20</SubjectSchemeIdentifier>
      <SubjectHeadingText>keyword1</SubjectHeadingText>
    </Subject>"#
        ));
        assert!(output.contains(
            r#"
    <Subject>
      <MainSubject />
      <SubjectSchemeIdentifier>B2</SubjectSchemeIdentifier>
      <SubjectHeadingText>custom1</SubjectHeadingText>
    </Subject>"#
        ));
        assert!(output.contains(
            r#"
    <Subject>
      <SubjectSchemeIdentifier>B2</SubjectSchemeIdentifier>
      <SubjectHeadingText>custom2</SubjectHeadingText>
    </Subject>"#
        ));
        assert!(output.contains(
            r#"
    <Audience>
      <AudienceCodeType>01</AudienceCodeType>
      <AudienceCodeValue>06</AudienceCodeValue>
    </Audience>
  </DescriptiveDetail>
  <CollateralDetail>"#
        ));
        //     assert!(output.contains(
        //         r#"
        // <TextContent>
        //   <TextType>02</TextType>
        //   <ContentAudience>00</ContentAudience>
        //   <Text>Lorem ipsum dolor sit amet.</Text>
        // </TextContent>"#
        //     ));
        //     assert!(output.contains(
        //         r#"
        // <TextContent>
        //   <TextType>03</TextType>
        //   <ContentAudience>00</ContentAudience>
        //   <Text>Lorem ipsum dolor sit amet, consectetur adipiscing elit.</Text>
        // </TextContent>"#
        //     ));
        //     assert!(output.contains(
        //         r#"
        // <TextContent>
        //   <TextType>30</TextType>
        //   <ContentAudience>00</ContentAudience>
        //   <Text>Lorem ipsum dolor sit amet, consectetur adipiscing elit.</Text>
        // </TextContent>"#
        // ));
        assert!(output.contains(
            r#"
    <TextContent>
      <TextType>04</TextType>
      <ContentAudience>00</ContentAudience>
      <Text>1. Chapter 1</Text>
    </TextContent>"#
        ));
        assert!(output.contains(
            r#"
    <TextContent>
      <TextType>20</TextType>
      <ContentAudience>00</ContentAudience>
      <Text language="eng">Open Access</Text>
    </TextContent>"#
        ));
        assert!(output.contains(
            r#"
    <TextContent>
      <TextType>13</TextType>
      <ContentAudience>00</ContentAudience>
      <Text>This is a general note</Text>
    </TextContent>"#
        ));
        assert!(output.contains(
            r#"
    <SupportingResource>
      <ResourceContentType>01</ResourceContentType>
      <ContentAudience>00</ContentAudience>
      <ResourceMode>03</ResourceMode>
      <ResourceFeature>
        <ResourceFeatureType>02</ResourceFeatureType>
        <FeatureNote>This is a cover caption</FeatureNote>
      </ResourceFeature>
      <ResourceVersion>
        <ResourceForm>02</ResourceForm>
        <ResourceLink>https://www.book.com/cover</ResourceLink>
      </ResourceVersion>
    </SupportingResource>
  </CollateralDetail>"#
        ));
        assert!(output.contains(
            r#"
  <ContentDetail>
    <ContentItem>
      <LevelSequenceNumber>1</LevelSequenceNumber>
      <TextItem>
        <TextItemType>03</TextItemType>
        <TextItemIdentifier>
          <TextItemIDType>06</TextItemIDType>
          <IDValue>10.00001/RELATION.0001</IDValue>
        </TextItemIdentifier>
        <PageRun>
          <FirstPageNumber>10</FirstPageNumber>
          <LastPageNumber>20</LastPageNumber>
        </PageRun>
        <NumberOfPages>11</NumberOfPages>
      </TextItem>"#
        ));
        assert!(output.contains(
            r#"
      <EpubLicense>
        <EpubLicenseName>Creative Commons Attribution-ShareAlike 4.0 International license (CC BY-SA 4.0).</EpubLicenseName>
        <EpubLicenseExpression>
          <EpubLicenseExpressionType>02</EpubLicenseExpressionType>
          <EpubLicenseExpressionLink>https://creativecommons.org/licenses/by-sa/4.0/</EpubLicenseExpressionLink>
        </EpubLicenseExpression>
      </EpubLicense>"#
        ));
        assert!(output.contains(
            r#"
      <ComponentTypeName>Chapter</ComponentTypeName>
      <TitleDetail>
        <TitleType>01</TitleType>
        <TitleElement>
          <TitleElementLevel>01</TitleElementLevel>
          <TitleText>The first chapter:</TitleText>
          <Subtitle>An introduction</Subtitle>
        </TitleElement>
      </TitleDetail>"#
        ));
        assert!(output.contains(
            r#"
      <Contributor>
        <SequenceNumber>1</SequenceNumber>
        <ContributorRole>A01</ContributorRole>
        <NameIdentifier>
          <NameIDType>21</NameIDType>
          <IDValue>0000-0003-0000-0002</IDValue>
        </NameIdentifier>
        <PersonName>Chapter Author N. 2</PersonName>
        <NamesBeforeKey>Chapter Author</NamesBeforeKey>
        <KeyNames>2</KeyNames>
        <ProfessionalAffiliation>
          <ProfessionalPosition>Chapter Manager</ProfessionalPosition>
          <AffiliationIdentifier>
            <AffiliationIDType>40</AffiliationIDType>
            <IDValue>08abcde89</IDValue>
          </AffiliationIdentifier>
          <Affiliation>University of Chapters</Affiliation>
        </ProfessionalAffiliation>
        <BiographicalNote>Chapter Author N. 2 is a made-up author</BiographicalNote>
        <Website>
          <WebsiteRole>06</WebsiteRole>
          <WebsiteDescription>Own website</WebsiteDescription>
          <WebsiteLink>https://chaptercontributor.site</WebsiteLink>
        </Website>
      </Contributor>"#
        ));
        assert!(output.contains(
            r#"
      <Language>
        <LanguageRole>01</LanguageRole>
        <LanguageCode>btk</LanguageCode>
      </Language>"#
        ));
        //     assert!(output.contains(
        //         r#"
        //   <TextContent>
        //     <TextType>02</TextType>
        //     <ContentAudience>00</ContentAudience>
        //     <Text>This is a chapter's very short abstract.</Text>
        //   </TextContent>"#
        //     ));
        //     assert!(output.contains(
        //         r#"
        //   <TextContent>
        //     <TextType>03</TextType>
        //     <ContentAudience>00</ContentAudience>
        //     <Text>This is a chapter's somewhat longer abstract. It has two sentences.</Text>
        //   </TextContent>"#
        //     ));
        //     assert!(output.contains(
        //         r#"
        //   <TextContent>
        //     <TextType>30</TextType>
        //     <ContentAudience>00</ContentAudience>
        //     <Text>This is a chapter's somewhat longer abstract. It has two sentences.</Text>
        //   </TextContent>"#
        //     ));
        assert!(output.contains(
            r#"
      <TextContent>
        <TextType>20</TextType>
        <ContentAudience>00</ContentAudience>
        <Text language="eng">Open Access</Text>
      </TextContent>"#
        ));
        assert!(output.contains(
            r#"
      <TextContent>
        <TextType>13</TextType>
        <ContentAudience>00</ContentAudience>
        <Text>This is a chapter general note.</Text>
      </TextContent>"#
        ));
        assert!(output.contains(
            r#"
      <CopyrightStatement>
        <CopyrightOwner>
          <PersonName>Chapter Author 1; Chapter Author 2</PersonName>
        </CopyrightOwner>
      </CopyrightStatement>"#
        ));
        assert!(output.contains(
            r#"
      <RelatedProduct>
        <ProductRelationCode>34</ProductRelationCode>
        <ProductIdentifier>
          <ProductIDType>06</ProductIDType>
          <IDValue>10.00005/chapter_reference</IDValue>
        </ProductIdentifier>
      </RelatedProduct>
    </ContentItem>
  </ContentDetail>"#
        ));
        assert!(output.contains(
            r#"
  <PublishingDetail>
    <Imprint>
      <ImprintName>OA Editions Imprint</ImprintName>
      <ImprintIdentifier>
        <ImprintIDType>01</ImprintIDType>
        <IDTypeName>URL</IDTypeName>
        <IDValue>https://imprint.oa</IDValue>
      </ImprintIdentifier>
    </Imprint>
    <Publisher>
      <PublishingRole>01</PublishingRole>
      <PublisherName>OA Editions</PublisherName>"#
        ));
        assert!(output.contains(
            r#"
      <Website>
        <WebsiteRole>01</WebsiteRole>
        <WebsiteDescription>Publisher's website: home page</WebsiteDescription>
        <WebsiteLink>https://publisher.oa</WebsiteLink>
      </Website>"#
        ));
        assert!(output.contains(
            r#"
      <Website>
        <WebsiteRole>02</WebsiteRole>
        <WebsiteDescription>Publisher's website: webpage for this title</WebsiteDescription>
        <WebsiteLink>https://www.book.com</WebsiteLink>
      </Website>"#
        ));
        assert!(output.contains(
            r#"
    </Publisher>
    <CityOfPublication>León, Spain</CityOfPublication>
    <PublishingStatus>04</PublishingStatus>
    <PublishingDate>
      <PublishingDateRole>01</PublishingDateRole>
      <Date dateformat="00">19991231</Date>
    </PublishingDate>
    <CopyrightStatement>
      <CopyrightOwner>
        <PersonName>Author 1; Author 2</PersonName>
      </CopyrightOwner>
    </CopyrightStatement>
    <SalesRights>
      <SalesRightsType>02</SalesRightsType>
      <Territory>
        <RegionsIncluded>WORLD</RegionsIncluded>
      </Territory>
    </SalesRights>
  </PublishingDetail>
  <RelatedMaterial>
    <RelatedWork>
      <WorkRelationCode>49</WorkRelationCode>
      <WorkIdentifier>
        <WorkIDType>06</WorkIDType>
        <IDValue>10.00001/RELATION.0003</IDValue>
      </WorkIdentifier>
    </RelatedWork>"#
        ));
        assert!(output.contains(
            r#"
    <RelatedProduct>
      <ProductRelationCode>01</ProductRelationCode>
      <ProductIdentifier>
        <ProductIDType>06</ProductIDType>
        <IDValue>10.00001/RELATION.0002</IDValue>
      </ProductIdentifier>
    </RelatedProduct>"#
        ));
        assert!(output.contains(
            r#"
    <RelatedProduct>
      <ProductRelationCode>34</ProductRelationCode>
      <ProductIdentifier>
        <ProductIDType>06</ProductIDType>
        <IDValue>10.00001/reference</IDValue>
      </ProductIdentifier>
    </RelatedProduct>"#
        ));
        assert!(output.contains(
            r#"
  </RelatedMaterial>
  <ProductSupply>
    <Market>
      <Territory>
        <RegionsIncluded>WORLD</RegionsIncluded>
      </Territory>
    </Market>"#
        ));
        assert!(output.contains(
            r#"
    <SupplyDetail>
      <Supplier>
        <SupplierRole>09</SupplierRole>
        <SupplierName>OA Editions</SupplierName>
        <Website>
          <WebsiteRole>02</WebsiteRole>
          <WebsiteDescription>Publisher's website: webpage for this product</WebsiteDescription>
          <WebsiteLink>https://www.book.com/pb_landing</WebsiteLink>
        </Website>
      </Supplier>
      <ProductAvailability>20</ProductAvailability>
      <SupplyDate>
        <SupplyDateRole>08</SupplyDateRole>
        <Date dateformat="00">19991231</Date>
      </SupplyDate>"#
        ));
        assert!(output.contains(
            r#"
    <SupplyDetail>
      <Supplier>
        <SupplierRole>11</SupplierRole>
        <SupplierName>JSTOR</SupplierName>"#
        ));
        assert!(output.contains(
            r#"
        <Website>
          <WebsiteRole>36</WebsiteRole>
          <WebsiteDescription>JSTOR: webpage for this product</WebsiteDescription>
          <WebsiteLink>https://www.jstor.com/pb_landing</WebsiteLink>
        </Website>"#
        ));
        assert!(output.contains(
            r#"
        <Website>
          <WebsiteRole>29</WebsiteRole>
          <WebsiteDescription>JSTOR: download the title</WebsiteDescription>
          <WebsiteLink>https://www.jstor.com/pb_fulltext</WebsiteLink>
        </Website>"#
        ));
        assert!(output.contains(
            r#"
      <Price>
        <PriceType>02</PriceType>
        <PriceAmount>5.95</PriceAmount>
        <CurrencyCode>EUR</CurrencyCode>
        <Territory>
          <RegionsIncluded>WORLD</RegionsIncluded>
        </Territory>
      </Price>"#
        ));
        assert!(output.contains(
            r#"
      <Price>
        <PriceType>02</PriceType>
        <PriceAmount>4.95</PriceAmount>
        <CurrencyCode>GBP</CurrencyCode>
        <Territory>
          <RegionsIncluded>WORLD</RegionsIncluded>
        </Territory>
      </Price>"#
        ));
        assert!(output.contains(
            r#"
      <Price>
        <PriceType>02</PriceType>
        <PriceAmount>8.00</PriceAmount>
        <CurrencyCode>USD</CurrencyCode>
        <Territory>
          <RegionsIncluded>WORLD</RegionsIncluded>
        </Territory>
      </Price>"#
        ));

        // Add withdrawn_date
        test_work.withdrawn_date = chrono::NaiveDate::from_ymd_opt(2020, 12, 31);
        let output = generate_test_output(true, &test_work);
        assert!(output.contains(
            r#"
    <PublishingDate>
      <PublishingDateRole>13</PublishingDateRole>
      <Date dateformat="00">20201231</Date>
    </PublishingDate>"#
        ));

        // Test ProductForm[Detail] with different publication types
        test_work.publications[0].publication_type = PublicationType::HARDBACK;
        let output = generate_test_output(true, &test_work);
        assert!(!output.contains(r#"    <ProductForm>BC</ProductForm>"#));
        assert!(output.contains(r#"    <ProductForm>BB</ProductForm>"#));
        assert!(!output.contains(r#"    <ProductFormDetail>"#));
        test_work.publications[0].publication_type = PublicationType::PDF;
        let output = generate_test_output(true, &test_work);
        assert!(output.contains(
            r#"
    <ProductForm>EB</ProductForm>
    <ProductFormDetail>E107</ProductFormDetail>"#
        ));
        test_work.publications[0].publication_type = PublicationType::HTML;
        let output = generate_test_output(true, &test_work);
        assert!(output.contains(
            r#"
    <ProductForm>EB</ProductForm>
    <ProductFormDetail>E105</ProductFormDetail>"#
        ));
        test_work.publications[0].publication_type = PublicationType::XML;
        let output = generate_test_output(true, &test_work);
        assert!(output.contains(
            r#"
    <ProductForm>EB</ProductForm>
    <ProductFormDetail>E113</ProductFormDetail>"#
        ));
        test_work.publications[0].publication_type = PublicationType::EPUB;
        let output = generate_test_output(true, &test_work);
        assert!(output.contains(
            r#"
    <ProductForm>EB</ProductForm>
    <ProductFormDetail>E101</ProductFormDetail>"#
        ));
        test_work.publications[0].publication_type = PublicationType::MOBI;
        let output = generate_test_output(true, &test_work);
        assert!(output.contains(
            r#"
    <ProductForm>EB</ProductForm>
    <ProductFormDetail>E127</ProductFormDetail>"#
        ));
        test_work.publications[0].publication_type = PublicationType::AZW3;
        let output = generate_test_output(true, &test_work);
        assert!(output.contains(
            r#"
    <ProductForm>EB</ProductForm>
    <ProductFormDetail>E116</ProductFormDetail>"#
        ));
        test_work.publications[0].publication_type = PublicationType::DOCX;
        let output = generate_test_output(true, &test_work);
        assert!(output.contains(
            r#"
    <ProductForm>EB</ProductForm>
    <ProductFormDetail>E104</ProductFormDetail>"#
        ));
        test_work.publications[0].publication_type = PublicationType::FICTION_BOOK;
        let output = generate_test_output(true, &test_work);
        assert!(output.contains(
            r#"
    <ProductForm>EB</ProductForm>
    <ProductFormDetail>E100</ProductFormDetail>"#
        ));
        test_work.publications[0].publication_type = PublicationType::MP3;
        let output = generate_test_output(true, &test_work);
        assert!(output.contains(
            r#"
    <ProductForm>AN</ProductForm>
    <ProductFormDetail>A103</ProductFormDetail>"#
        ));
        test_work.publications[0].publication_type = PublicationType::WAV;
        let output = generate_test_output(true, &test_work);
        assert!(output.contains(
            r#"
    <ProductForm>AN</ProductForm>
    <ProductFormDetail>A104</ProductFormDetail>"#
        ));
        test_work.publications[0].publication_type = PublicationType::PAPERBACK;

        // Remove/change some values to test (non-)output of optional blocks
        test_work.doi = None;
        test_work.lccn = None;
        test_work.oclc = None;
        test_work.reference = None;
        test_work.license = None;
        test_work.titles[0].subtitle = None;
        test_work.edition = Some(1);
        test_work.page_count = None;
        test_work.bibliography_note = None;
        test_work.image_count = None;
        // test_work.short_abstract = None;
        // test_work.long_abstract = None;
        test_work.toc = None;
        test_work.general_note = None;
        test_work.cover_caption = None;
        test_work.landing_page = None;
        test_work.place = None;
        test_work.publication_date = None;
        test_work.copyright_holder = None;
        test_work.publications[0].isbn = None;
        test_work.publications[0].height_mm = None;
        test_work.publications[0].locations.pop();
        test_work.publications[0].prices.pop();
        test_work.relations[0].related_work.last_page = None;
        test_work.relations[0].related_work.page_count = None;
        test_work.references.clear();
        test_work.imprint.imprint_url = None;
        test_work.imprint.publisher.publisher_url = None;
        test_work.subjects.pop();
        let output = generate_test_output(true, &test_work);
        println!("{output}");
        assert!(!output.contains(
            r#"
  <ProductIdentifier>
    <ProductIDType>15</ProductIDType>
    <IDValue>9783161484100</IDValue>
  </ProductIdentifier>"#
        ));
        assert!(!output.contains(
            r#"
  <ProductIdentifier>
    <ProductIDType>06</ProductIDType>
    <IDValue>10.00001/BOOK.0001</IDValue>
  </ProductIdentifier>"#
        ));
        assert!(!output.contains(
            r#"
  <ProductIdentifier>
    <ProductIDType>13</ProductIDType>
    <IDValue>123456789</IDValue>
  </ProductIdentifier>"#
        ));
        assert!(!output.contains(
            r#"
  <ProductIdentifier>
    <ProductIDType>23</ProductIDType>
    <IDValue>987654321</IDValue>
  </ProductIdentifier>"#
        ));
        assert!(!output.contains(
            r#"
  <ProductIdentifier>
    <ProductIDType>01</ProductIDType>
    <IDTypeName>internal-reference</IDTypeName>
    <IDValue>IntRef1</IDValue>
  </ProductIdentifier>"#
        ));
        assert!(!output.contains(
            r#"
    <Measure>
      <MeasureType>01</MeasureType>
      <Measurement>234</Measurement>
      <MeasureUnitCode>mm</MeasureUnitCode>
    </Measure>"#
        ));
        assert!(!output.contains(
            r#"
    <EpubLicense>
      <EpubLicenseName>Creative Commons Attribution 4.0 International license (CC BY 4.0).</EpubLicenseName>
      <EpubLicenseExpression>
        <EpubLicenseExpressionType>02</EpubLicenseExpressionType>
        <EpubLicenseExpressionLink>https://creativecommons.org/licenses/by/4.0/</EpubLicenseExpressionLink>
      </EpubLicenseExpression>
    </EpubLicense>"#
        ));
        // Title block still present but Subtitle absent
        assert!(output.contains(
            r#"
    <TitleDetail>
      <TitleType>01</TitleType>
      <TitleElement>
        <TitleElementLevel>01</TitleElementLevel>
        <TitleText>Book Title</TitleText>
      </TitleElement>
    </TitleDetail>"#
        ));
        assert!(!output.contains(r#"        <Subtitle>Book Subtitle</Subtitle>"#));
        assert!(!output.contains(r#"    <Edition>"#));
        assert!(!output.contains(r#"    <Extent>"#));
        assert!(!output
            .contains(r#"    <IllustrationsNote>This is a bibliography note</IllustrationsNote>"#));
        assert!(!output.contains(
            r#"
    <AncillaryContent>
      <AncillaryContentType>09</AncillaryContentType>
      <Number>15</Number>
    </AncillaryContent>"#
        ));
        assert!(!output.contains(
            r#"
    <Subject>
      <SubjectSchemeIdentifier>B2</SubjectSchemeIdentifier>
      <SubjectHeadingText>custom2</SubjectHeadingText>
    </Subject>"#
        ));
        assert!(!output.contains(
            r#"
    <TextContent>
      <TextType>02</TextType>
      <ContentAudience>00</ContentAudience>
      <Text>Lorem ipsum dolor sit amet.</Text>
    </TextContent>"#
        ));
        assert!(!output.contains(
            r#"
    <TextContent>
      <TextType>03</TextType>
      <ContentAudience>00</ContentAudience>
      <Text>Lorem ipsum dolor sit amet, consectetur adipiscing elit.</Text>
    </TextContent>"#
        ));
        assert!(!output.contains(
            r#"
    <TextContent>
      <TextType>30</TextType>
      <ContentAudience>00</ContentAudience>
      <Text>Lorem ipsum dolor sit amet, consectetur adipiscing elit.</Text>
    </TextContent>"#
        ));
        assert!(!output.contains(
            r#"
    <TextContent>
      <TextType>04</TextType>
      <ContentAudience>00</ContentAudience>
      <Text>1. Chapter 1</Text>
    </TextContent>"#
        ));
        assert!(!output.contains(
            r#"
    <TextContent>
      <TextType>13</TextType>
      <ContentAudience>00</ContentAudience>
      <Text>This is a general note</Text>
    </TextContent>"#
        ));
        // No licence means we assume the title is non-OA
        assert!(!output.contains(
            r#"
    <TextContent>
      <TextType>20</TextType>
      <ContentAudience>00</ContentAudience>
      <Text language="eng">Open Access</Text>
    </TextContent>"#
        ));
        // SupportingResource block still present but ResourceFeature absent
        assert!(output.contains(
            r#"
    <SupportingResource>
      <ResourceContentType>01</ResourceContentType>
      <ContentAudience>00</ContentAudience>
      <ResourceMode>03</ResourceMode>
      <ResourceVersion>
        <ResourceForm>02</ResourceForm>
        <ResourceLink>https://www.book.com/cover</ResourceLink>
      </ResourceVersion>
    </SupportingResource>"#
        ));
        assert!(!output.contains(
            r#"
      <ResourceFeature>
        <ResourceFeatureType>02</ResourceFeatureType>
        <FeatureNote>This is a cover caption</FeatureNote>
      </ResourceFeature>"#
        ));
        // PageRun block still present but LastPageNumber absent
        assert!(output.contains(
            r#"
        <PageRun>
          <FirstPageNumber>10</FirstPageNumber>
        </PageRun>"#
        ));
        assert!(!output.contains(r#"          <LastPageNumber>20</LastPageNumber>"#));
        assert!(!output.contains(r#"        <NumberOfPages>11</NumberOfPages>"#));
        // Imprint block still present but ImprintIdentifier absent
        assert!(output.contains(
            r#"
    <Imprint>
      <ImprintName>OA Editions Imprint</ImprintName>
    </Imprint>"#
        ));
        assert!(!output.contains(
            r#"
      <ImprintIdentifier>
        <ImprintIDType>01</ImprintIDType>
        <IDTypeName>URL</IDTypeName>
        <IDValue>https://imprint.oa</IDValue>
      </ImprintIdentifier>"#
        ));
        assert!(!output.contains(
            r#"
      <Website>
        <WebsiteRole>01</WebsiteRole>
        <WebsiteDescription>Publisher's website: home page</WebsiteDescription>
        <WebsiteLink>https://publisher.oa</WebsiteLink>
      </Website>"#
        ));
        assert!(!output.contains(
            r#"
      <Website>
        <WebsiteRole>02</WebsiteRole>
        <WebsiteDescription>Publisher's website: webpage for this title</WebsiteDescription>
        <WebsiteLink>https://www.book.com</WebsiteLink>
      </Website>"#
        ));
        assert!(!output.contains(r#"    <CityOfPublication>León, Spain</CityOfPublication>"#));
        assert!(!output.contains(
            r#"
    <PublishingDate>
      <PublishingDateRole>01</PublishingDateRole>
      <Date dateformat="00">19991231</Date>
    </PublishingDate>"#
        ));
        assert!(!output.contains(
            r#"
    <CopyrightStatement>
      <CopyrightOwner>
        <PersonName>Author 1; Author 2</PersonName>
      </CopyrightOwner>
    </CopyrightStatement>"#
        ));
        assert!(!output.contains(
            r#"
    <RelatedProduct>
      <ProductRelationCode>34</ProductRelationCode>
      <ProductIdentifier>
        <ProductIDType>06</ProductIDType>
        <IDValue>10.00001/reference</IDValue>
      </ProductIdentifier>
    </RelatedProduct>"#
        ));
        assert!(!output.contains(
            r#"
    <SupplyDetail>
      <Supplier>
        <SupplierRole>11</SupplierRole>
        <SupplierName>JSTOR</SupplierName>"#
        ));
        assert!(!output.contains(
            r#"
        <Website>
          <WebsiteRole>36</WebsiteRole>
          <WebsiteDescription>JSTOR: webpage for this product</WebsiteDescription>
          <WebsiteLink>https://www.jstor.com/pb_landing</WebsiteLink>
        </Website>"#
        ));
        assert!(!output.contains(
            r#"
        <Website>
          <WebsiteRole>29</WebsiteRole>
          <WebsiteDescription>JSTOR: download the title</WebsiteDescription>
          <WebsiteLink>https://www.jstor.com/pb_fulltext</WebsiteLink>
        </Website>"#
        ));
        assert!(!output.contains(
            r#"
      <Price>
        <PriceType>02</PriceType>
        <PriceAmount>8.00</PriceAmount>
        <CurrencyCode>USD</CurrencyCode>
        <Territory>
          <RegionsIncluded>WORLD</RegionsIncluded>
        </Territory>
      </Price>"#
        ));

        // Test truncation of short abstract
        // test_work.short_abstract = Some("Lorem ipsum dolor sit amet, consectetur adipiscing elit. Vestibulum vel libero eleifend, ultrices purus vitae, suscipit ligula. Aliquam ornare quam et nulla vestibulum, id euismod tellus malesuada. Orci varius natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. Nullam ornare bibendum ex nec dapibus. Proin porta risus elementum odio feugiat tempus. Etiam eu felis ac metus viverra ornare. In consectetur neque sed feugiat ornare. Mauris at purus fringilla orci tincidunt pulvinar sed a massa. Nullam vestibulum posuere augue, sit amet tincidunt nisl pulvinar ac.".to_string());
        // Remove even more values
        test_work.edition = None;
        test_work.table_count = None;
        test_work.audio_count = None;
        test_work.video_count = None;
        test_work.cover_url = None;
        test_work.relations[0].related_work.first_page = None;
        // If first page is missing, last page isn't included even if present
        test_work.relations[0].related_work.last_page = Some("20".to_string());
        test_work.relations.pop();
        test_work.subjects.clear();
        test_work.publications[0].height_cm = None;
        test_work.publications[0].height_in = None;
        test_work.publications[0].width_mm = None;
        test_work.publications[0].width_cm = None;
        test_work.publications[0].width_in = None;
        test_work.publications[0].depth_mm = None;
        test_work.publications[0].depth_cm = None;
        test_work.publications[0].depth_in = None;
        test_work.publications[0].weight_g = None;
        test_work.publications[0].weight_oz = None;
        test_work.publications[0].prices.clear();
        test_work.publications[0].locations.clear();
        let output = generate_test_output(true, &test_work);
        println!("{output}");
        // Still no Edition, same as when value was 1
        assert!(!output.contains(r#"    <Edition>"#));
        // No AncillaryContent or Subject blocks at all - skip from TitleDetail straight to Audience
        assert!(output.contains(
            r#"
    </TitleDetail>
    <Audience>"#
        ));
        assert!(!output.contains(r#"    <AncillaryContent>"#));
        assert!(!output.contains(r#"    <Subject>"#));
        // No cover URL means no SupportingResource block - CollateralDetail only contains short abstract
        assert!(output.contains(
            r#"
  <CollateralDetail>
    <TextContent>
      <TextType>02</TextType>
      <ContentAudience>00</ContentAudience>
      <Text>Lorem ipsum dolor sit amet, consectetur adipiscing elit. Vestibulum vel libero eleifend, ultrices purus vitae, suscipit ligula. Aliquam ornare quam et nulla vestibulum, id euismod tellus malesuada. Orci varius natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. Nullam ornare bibendum ex nec dapibus. Proin porta risus elementu</Text>
    </TextContent>
  </CollateralDetail>"#
        ));
        assert!(!output.contains(r#"    <SupportingResource>"#));
        assert!(!output.contains(r#"    <PageRun>"#));
        assert!(!output.contains(r#"      <FirstPageNumber>10</FirstPageNumber>"#));
        assert!(!output.contains(r#"      <LastPageNumber>20</LastPageNumber>"#));
        // Only one item left in RelatedMaterial
        assert!(output.contains(
            r#"
  <RelatedMaterial>
    <RelatedProduct>
      <ProductRelationCode>01</ProductRelationCode>
      <ProductIdentifier>
        <ProductIDType>06</ProductIDType>
        <IDValue>10.00001/RELATION.0002</IDValue>
      </ProductIdentifier>
    </RelatedProduct>
  </RelatedMaterial>"#
        ));
        assert!(!output.contains(r#"    <RelatedWork>"#));
        // Supplier block for publisher still present but Website absent
        assert!(output.contains(
            r#"
      <Supplier>
        <SupplierRole>09</SupplierRole>
        <SupplierName>OA Editions</SupplierName>
      </Supplier>"#
        ));
        assert!(!output.contains(
            r#"
        <Website>
          <WebsiteRole>02</WebsiteRole>
          <WebsiteDescription>Publisher's website: webpage for this product</WebsiteDescription>
          <WebsiteLink>https://www.book.com/pb_landing</WebsiteLink>
        </Website>"#
        ));
        // UnpricedItemType block instead of any Prices
        assert!(output.contains(r#"      <UnpricedItemType>01</UnpricedItemType>"#));
        assert!(!output.contains(r#"      <Price>"#));

        // Remove chapter DOI: can't output ContentDetail block
        test_work.relations[0].related_work.doi = None;
        // Remove remaining related work DOI: can't output RelatedMaterial block
        test_work.relations[1].related_work.doi = None;
        // Remove short abstract: can't output CollateralDetail block
        test_work.abstracts.clear();
        // test_work.short_abstract = None;
        // Reinstate landing page: supplier block for publisher now contains it
        test_work.landing_page = Some("https://www.book.com".to_string());
        let output = generate_test_output(true, &test_work);
        println!("{output}");
        assert!(!output.contains(r#"  <ContentDetail>"#));
        assert!(!output.contains(r#"  <RelatedMaterial>"#));
        assert!(!output.contains(r#"    <RelatedProduct>"#));
        assert!(!output.contains(r#"  <CollateralDetail>"#));
        assert!(output.contains(
            r#"
      <Supplier>
        <SupplierRole>09</SupplierRole>
        <SupplierName>OA Editions</SupplierName>
        <Website>
          <WebsiteRole>02</WebsiteRole>
          <WebsiteDescription>Publisher's website: webpage for this product</WebsiteDescription>
          <WebsiteLink>https://www.book.com</WebsiteLink>
        </Website>
      </Supplier>"#
        ));
        assert!(!output
            .contains(r#"          <WebsiteLink>https://www.book.com/pb_landing</WebsiteLink>"#));

        // Add second publication, and test that two records are produced
        test_work.publications.push(WorkPublications {
            publication_id: Uuid::from_str("00000000-0000-0000-CCCC-000000000001").unwrap(),
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
        });
        let output = generate_test_output(true, &test_work);
        println!("{output}");
        assert!(output.contains(
            r#"
<Product>
  <RecordReference>urn:uuid:00000000-0000-0000-bbbb-000000000001</RecordReference>"#
        ));
        assert!(output.contains(
            r#"
<Product>
  <RecordReference>urn:uuid:00000000-0000-0000-cccc-000000000001</RecordReference>"#
        ));
        // First record will now have a RelatedMaterial block (again) representing second record
        assert!(output.contains(
            r#"
  </PublishingDetail>
  <RelatedMaterial>
    <RelatedProduct>
      <ProductRelationCode>06</ProductRelationCode>
      <ProductIdentifier>
        <ProductIDType>15</ProductIDType>
        <IDValue>9781402894626</IDValue>
      </ProductIdentifier>
    </RelatedProduct>
  </RelatedMaterial>
  <ProductSupply>"#
        ));
        // No RelatedMaterial block in second record, as no ISBN in first
        // Skip straight from PublishingDetail to ProductSupply
        assert!(output.contains(
            r#"
  </PublishingDetail>
  <ProductSupply>"#
        ));

        // Remove all publications and test that result is error
        test_work.publications.clear();
        let output = generate_test_output(false, &test_work);
        assert_eq!(
            output,
            "Could not generate onix_3.1::thoth: No publications supplied".to_string()
        );
    }
}
