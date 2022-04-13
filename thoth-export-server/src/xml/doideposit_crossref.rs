use chrono::Utc;
use std::collections::HashMap;
use std::io::Write;
use thoth_client::{
    ContributionType, PublicationType, RelationType, Work, WorkRelationsRelatedWork,
    WorkRelationsRelatedWorkContributions, WorkRelationsRelatedWorkPublications, WorkType,
};
use xml::writer::{EventWriter, XmlEvent};

use super::{write_element_block, XmlSpecification};
use crate::xml::{write_full_element_block, XmlElementBlock};
use thoth_errors::{ThothError, ThothResult};

pub struct DoiDepositCrossref {}

// Output format based on schema documentation at https://data.crossref.org/reports/help/schema_doc/5.3.1/index.html
// (retrieved via https://www.crossref.org/documentation/schema-library/xsd-schema-quick-reference/).
// Output validity tested using tool at https://www.crossref.org/02publishers/parser.html
// (retrieved via https://www.crossref.org/documentation/member-setup/direct-deposit-xml/testing-your-xml/).
impl XmlSpecification for DoiDepositCrossref {
    fn handle_event<W: Write>(w: &mut EventWriter<W>, works: &[Work]) -> ThothResult<()> {
        match works.len() {
            0 => Err(ThothError::IncompleteMetadataRecord(
                "doideposit::crossref".to_string(),
                "Not enough data".to_string(),
            )),
            1 => {
                let work = works.first().unwrap();
                let timestamp = Utc::now().format("%Y%m%d%H%M").to_string();
                let work_id = format!("{}_{}", work.work_id, timestamp);
                let mut attr_map: HashMap<&str, &str> = HashMap::new();

                attr_map.insert("version", "4.3.5");
                attr_map.insert("xmlns", "http://www.crossref.org/schema/4.3.5");
                attr_map.insert("xmlns:xsi", "http://www.w3.org/2001/XMLSchema-instance");
                attr_map.insert("xsi:schemaLocation", "http://www.crossref.org/schema/4.3.5 http://www.crossref.org/schemas/crossref4.3.5.xsd");
                attr_map.insert("xmlns:ai", "http://www.crossref.org/AccessIndicators.xsd");

                write_full_element_block("doi_batch", None, Some(attr_map), w, |w| {
                    write_element_block("head", w, |w| {
                        write_element_block("doi_batch_id", w, |w| {
                            w.write(XmlEvent::Characters(&work_id))
                                .map_err(|e| e.into())
                        })?;
                        write_element_block("timestamp", w, |w| {
                            w.write(XmlEvent::Characters(&timestamp))
                                .map_err(|e| e.into())
                        })?;
                        write_element_block("depositor", w, |w| {
                            write_element_block("depositor_name", w, |w| {
                                w.write(XmlEvent::Characters("Thoth")).map_err(|e| e.into())
                            })?;
                            write_element_block("email_address", w, |w| {
                                w.write(XmlEvent::Characters("info@thoth.pub"))
                                    .map_err(|e| e.into())
                            })
                        })?;
                        write_element_block("registrant", w, |w| {
                            w.write(XmlEvent::Characters("Thoth")).map_err(|e| e.into())
                        })
                    })?;
                    XmlElementBlock::<DoiDepositCrossref>::xml_element(work, w)
                })
            }
            // handler::by_publisher() prevents generation of output for multiple records
            _ => unreachable!(),
        }
    }
}

impl XmlElementBlock<DoiDepositCrossref> for Work {
    fn xml_element<W: Write>(&self, w: &mut EventWriter<W>) -> ThothResult<()> {
        let work_type = match &self.work_type {
            WorkType::MONOGRAPH => "monograph",
            WorkType::EDITED_BOOK => "edited_book",
            WorkType::TEXTBOOK => "reference",
            WorkType::JOURNAL_ISSUE | WorkType::BOOK_SET | WorkType::BOOK_CHAPTER => "other",
            WorkType::Other(_) => unreachable!(),
        };
        write_element_block("body", w, |w| {
            write_full_element_block(
                "book",
                None,
                Some(HashMap::from([("book_type", work_type)])),
                w,
                |w| {
                    // Only one series can be listed, so we select the first one found (if any).
                    if let Some((series, ordinal)) =
                        self.issues.first().map(|i| (&i.series, i.issue_ordinal))
                    {
                        write_full_element_block(
                            "book_series_metadata",
                            None,
                            Some(HashMap::from([("language", "en")])),
                            w,
                            |w| {
                                write_element_block("series_metadata", w, |w| {
                                    write_element_block("titles", w, |w| {
                                        write_element_block("title", w, |w| {
                                            w.write(XmlEvent::Characters(&series.series_name))
                                                .map_err(|e| e.into())
                                        })
                                    })?;
                                    write_full_element_block(
                                        "issn",
                                        None,
                                        Some(HashMap::from([("media_type", "print")])),
                                        w,
                                        |w| {
                                            w.write(XmlEvent::Characters(&series.issn_print))
                                                .map_err(|e| e.into())
                                        },
                                    )?;
                                    write_full_element_block(
                                        "issn",
                                        None,
                                        Some(HashMap::from([("media_type", "electronic")])),
                                        w,
                                        |w| {
                                            w.write(XmlEvent::Characters(&series.issn_digital))
                                                .map_err(|e| e.into())
                                        },
                                    )
                                })?;
                                work_metadata(
                                    w,
                                    &WorkRelationsRelatedWork::from(self.clone()),
                                    None,
                                    Some(ordinal),
                                )
                            },
                        )?;
                    } else {
                        write_full_element_block(
                            "book_metadata",
                            None,
                            Some(HashMap::from([("language", "en")])),
                            w,
                            |w| {
                                work_metadata(
                                    w,
                                    &WorkRelationsRelatedWork::from(self.clone()),
                                    None,
                                    None,
                                )
                            },
                        )?;
                    }
                    // As an alternative to `book_metadata` and `book_series_metadata` above,
                    // `book_set_metadata` can be used for works which are part of a set.
                    // Omitted at present but could be considered as a future enhancement.
                    let mut chapters = self.relations.clone();
                    chapters.sort_by(|a, b| a.relation_ordinal.cmp(&b.relation_ordinal));
                    for (chapter, ordinal) in chapters
                        .iter()
                        .filter(|r| r.relation_type == RelationType::HAS_CHILD)
                        .map(|r| (&r.related_work, r.relation_ordinal))
                    {
                        write_full_element_block(
                            "content_item",
                            None,
                            Some(HashMap::from([("component_type", "chapter")])),
                            w,
                            |w| work_metadata(w, chapter, Some(ordinal), None),
                        )?;
                    }
                    Ok(())
                },
            )
        })
    }
}

fn work_metadata<W: Write>(
    w: &mut EventWriter<W>,
    work: &WorkRelationsRelatedWork,
    chapter_number: Option<i64>,
    volume_number: Option<i64>,
) -> ThothResult<()> {
    let is_chapter = chapter_number.is_some();
    // Only Author, Editor and Translator are supported by this format. Omit any other contributors.
    let contributions: Vec<WorkRelationsRelatedWorkContributions> = work
        .contributions
        .clone()
        .into_iter()
        .filter(|c| {
            c.contribution_type == ContributionType::AUTHOR
                || c.contribution_type == ContributionType::EDITOR
                || c.contribution_type == ContributionType::TRANSLATOR
        })
        .collect();
    if !contributions.is_empty() {
        write_element_block("contributors", w, |w| {
            for contribution in &contributions {
                XmlElementBlock::<DoiDepositCrossref>::xml_element(contribution, w).ok();
            }
            Ok(())
        })?;
    }
    write_element_block("titles", w, |w| {
        write_element_block("title", w, |w| {
            w.write(XmlEvent::Characters(&work.title))
                .map_err(|e| e.into())
        })?;
        if let Some(subtitle) = &work.subtitle {
            write_element_block("subtitle", w, |w| {
                w.write(XmlEvent::Characters(subtitle))
                    .map_err(|e| e.into())
            })?;
        }
        Ok(())
    })?;
    if let Some(chapter) = chapter_number {
        // If the work is a chapter of another work, caller should have passed in its chapter number
        write_element_block("component_number", w, |w| {
            w.write(XmlEvent::Characters(&chapter.to_string()))
                .map_err(|e| e.into())
        })?;
    } else if let Some(volume) = volume_number {
        // If the work is part of a series, caller should have passed in its issue number
        write_element_block("volume", w, |w| {
            w.write(XmlEvent::Characters(&volume.to_string()))
                .map_err(|e| e.into())
        })?;
    }
    // Abstract can also optionally be provided here, but only in JATS format.
    // Omitted at present but could be considered as a future enhancement.
    if let Some(edition) = work.edition {
        if is_chapter {
            // `edition_number` is not supported for chapters,
            // but edition should always be None for Thoth chapters.
            return Err(ThothError::IncompleteMetadataRecord(
                "doideposit::crossref".to_string(),
                "Chapters cannot have Edition numbers".to_string(),
            ));
        }
        write_element_block("edition_number", w, |w| {
            w.write(XmlEvent::Characters(&edition.to_string()))
                .map_err(|e| e.into())
        })?;
    }
    if let Some(date) = work.publication_date {
        write_element_block("publication_date", w, |w| {
            write_element_block("month", w, |w| {
                w.write(XmlEvent::Characters(&date.format("%m").to_string()))
                    .map_err(|e| e.into())
            })?;
            write_element_block("day", w, |w| {
                w.write(XmlEvent::Characters(&date.format("%d").to_string()))
                    .map_err(|e| e.into())
            })?;
            write_element_block("year", w, |w| {
                w.write(XmlEvent::Characters(&date.format("%Y").to_string()))
                    .map_err(|e| e.into())
            })
        })?;
    } else if !is_chapter {
        // `publication_date` element is mandatory for `book_metadata`
        return Err(ThothError::IncompleteMetadataRecord(
            "doideposit::crossref".to_string(),
            "Missing Publication Date".to_string(),
        ));
    }
    if is_chapter {
        if let Some(first_page) = &work.first_page {
            write_element_block("pages", w, |w| {
                write_element_block("first_page", w, |w| {
                    w.write(XmlEvent::Characters(first_page))
                        .map_err(|e| e.into())
                })?;
                if let Some(last_page) = &work.last_page {
                    write_element_block("last_page", w, |w| {
                        w.write(XmlEvent::Characters(last_page))
                            .map_err(|e| e.into())
                    })?;
                }
                Ok(())
            })?;
        }
    } else {
        let publications: Vec<WorkRelationsRelatedWorkPublications> = work
            .publications
            .clone()
            .into_iter()
            .filter(|p| p.isbn.is_some())
            .collect();
        if !publications.is_empty() {
            for publication in &publications {
                XmlElementBlock::<DoiDepositCrossref>::xml_element(publication, w).ok();
            }
        } else {
            // `book_metadata` must have either at least one `isbn` element or a `noisbn`
            // element with a `reason` attribute - assume missing ISBNs are erroneous
            return Err(ThothError::IncompleteMetadataRecord(
                "doideposit::crossref".to_string(),
                "No ISBNs provided".to_string(),
            ));
        }
        write_element_block("publisher", w, |w| {
            write_element_block("publisher_name", w, |w| {
                w.write(XmlEvent::Characters(&work.imprint.publisher.publisher_name))
                    .map_err(|e| e.into())
            })?;
            if let Some(place) = &work.place {
                write_element_block("publisher_place", w, |w| {
                    w.write(XmlEvent::Characters(place)).map_err(|e| e.into())
                })?;
            }
            Ok(())
        })?;
    }
    write_full_element_block(
        "ai:program",
        None,
        Some(HashMap::from([("name", "AccessIndicators")])),
        w,
        |w| {
            write_element_block("ai:free_to_read", w, |_w| Ok(()))?;
            if let Some(license) = &work.license {
                write_element_block("ai:license_ref", w, |w| {
                    w.write(XmlEvent::Characters(license)).map_err(|e| e.into())
                })?;
            }
            Ok(())
        },
    )?;
    if let Some(doi) = &work.doi {
        if let Some(landing_page) = &work.landing_page {
            write_element_block("doi_data", w, |w| {
                write_element_block("doi", w, |w| {
                    w.write(XmlEvent::Characters(&doi.to_string()))
                        .map_err(|e| e.into())
                })?;
                write_element_block("resource", w, |w| {
                    w.write(XmlEvent::Characters(landing_page))
                        .map_err(|e| e.into())
                })?;
                if let Some(pdf_url) = work
                    .publications
                    .iter()
                    .find(|p| {
                        p.publication_type.eq(&PublicationType::PDF) && !p.locations.is_empty()
                    })
                    .and_then(|p| p.locations.iter().find(|l| l.canonical))
                    .and_then(|l| l.full_text_url.as_ref())
                {
                    // Used for CrossRef Similarity Check. URL must point directly to full-text PDF.
                    // Alternatively, a direct link to full-text HTML can be used (not implemented here).
                    write_full_element_block(
                        "collection",
                        None,
                        Some(HashMap::from([("property", "crawler-based")])),
                        w,
                        |w| {
                            for crawler in [
                                "iParadigms",
                                "google",
                                "msn",
                                "altavista",
                                "yahoo",
                                "scirus",
                            ] {
                                write_full_element_block(
                                    "item",
                                    None,
                                    Some(HashMap::from([("crawler", crawler)])),
                                    w,
                                    |w| {
                                        write_full_element_block(
                                            "resource",
                                            None,
                                            Some(HashMap::from([("mime_type", "application/pdf")])),
                                            w,
                                            |w| {
                                                w.write(XmlEvent::Characters(pdf_url))
                                                    .map_err(|e| e.into())
                                            },
                                        )
                                    },
                                )?;
                            }
                            Ok(())
                        },
                    )?;
                    // Used for CrossRef Text and Data Mining. URL must point directly to full-text PDF.
                    // Alternatively, a direct link to full-text XML can be used (not implemented here).
                    write_full_element_block(
                        "collection",
                        None,
                        Some(HashMap::from([("property", "text-mining")])),
                        w,
                        |w| {
                            write_element_block("item", w, |w| {
                                write_full_element_block(
                                    "resource",
                                    None,
                                    Some(HashMap::from([("mime_type", "application/pdf")])),
                                    w,
                                    |w| {
                                        w.write(XmlEvent::Characters(pdf_url)).map_err(|e| e.into())
                                    },
                                )
                            })
                        },
                    )?;
                }
                Ok(())
            })?;
        } else if is_chapter {
            // `doi_data` element is mandatory for `content_item`, and must contain
            // both `doi` element and `resource` (landing page) element
            return Err(ThothError::IncompleteMetadataRecord(
                "doideposit::crossref".to_string(),
                "Missing chapter Landing Page".to_string(),
            ));
        }
    } else if is_chapter {
        // `doi_data` element is mandatory for `content_item`, and must contain
        // both `doi` element and `resource` (landing page) element
        return Err(ThothError::IncompleteMetadataRecord(
            "doideposit::crossref".to_string(),
            "Missing chapter DOI".to_string(),
        ));
    }
    Ok(())
}

impl XmlElementBlock<DoiDepositCrossref> for WorkRelationsRelatedWorkPublications {
    fn xml_element<W: Write>(&self, w: &mut EventWriter<W>) -> ThothResult<()> {
        if let Some(isbn) = self.isbn.as_ref().map(|i| i.to_string()) {
            let isbn_type = match self.publication_type.eq(&PublicationType::PAPERBACK)
                || self.publication_type.eq(&PublicationType::HARDBACK)
            {
                true => "print".to_string(),
                false => "electronic".to_string(),
            };
            write_full_element_block(
                "isbn",
                None,
                Some(HashMap::from([("media_type", isbn_type.as_str())])),
                w,
                |w| w.write(XmlEvent::Characters(&isbn)).map_err(|e| e.into()),
            )?;
        } else {
            // Publications with no ISBN are not output.
            unreachable!()
        }
        Ok(())
    }
}

impl XmlElementBlock<DoiDepositCrossref> for WorkRelationsRelatedWorkContributions {
    fn xml_element<W: Write>(&self, w: &mut EventWriter<W>) -> ThothResult<()> {
        let role = match &self.contribution_type {
            ContributionType::AUTHOR => "author",
            ContributionType::EDITOR => "editor",
            ContributionType::TRANSLATOR => "translator",
            // Only the above roles are supported by this format.
            ContributionType::PHOTOGRAPHER
            | ContributionType::ILUSTRATOR
            | ContributionType::MUSIC_EDITOR
            | ContributionType::FOREWORD_BY
            | ContributionType::INTRODUCTION_BY
            | ContributionType::AFTERWORD_BY
            | ContributionType::PREFACE_BY
            | ContributionType::Other(_) => unreachable!(),
        };
        let ordinal = match &self.contribution_ordinal {
            1 => "first",
            _ => "additional",
        };
        write_full_element_block(
            "person_name",
            None,
            Some(HashMap::from([
                ("sequence", ordinal),
                ("contributor_role", role),
            ])),
            w,
            |w| {
                if let Some(first_name) = &self.first_name {
                    write_element_block("given_name", w, |w| {
                        w.write(XmlEvent::Characters(first_name))
                            .map_err(|e| e.into())
                    })?;
                }
                write_element_block("surname", w, |w| {
                    w.write(XmlEvent::Characters(&self.last_name))
                        .map_err(|e| e.into())
                })?;
                if let Some(orcid) = &self.contributor.orcid {
                    write_element_block("ORCID", w, |w| {
                        // Leading `https://orcid.org` is required, and omitted by orcid.to_string()
                        w.write(XmlEvent::Characters(&format!(
                            "https://orcid.org/{}",
                            orcid
                        )))
                        .map_err(|e| e.into())
                    })?;
                }
                Ok(())
                // Affiliation information can also optionally be provided here.
                // Omitted at present but could be considered as a future enhancement.
            },
        )
    }
}
