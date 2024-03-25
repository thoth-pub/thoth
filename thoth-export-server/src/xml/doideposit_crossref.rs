use chrono::Utc;
use std::io::Write;
use thoth_api::model::IdentifierWithDomain;
use thoth_client::{
    ContributionType, PublicationType, RelationType, Work, WorkIssuesSeries, WorkRelations,
    WorkRelationsRelatedWork, WorkRelationsRelatedWorkContributions,
    WorkRelationsRelatedWorkContributionsAffiliationsInstitution, WorkRelationsRelatedWorkFundings,
    WorkRelationsRelatedWorkPublications, WorkRelationsRelatedWorkReferences, WorkType,
};
use xml::writer::{EventWriter, XmlEvent};

use super::{write_element_block, XmlSpecification};
use crate::xml::{write_full_element_block, XmlElementBlock};
use thoth_errors::{ThothError, ThothResult};

#[derive(Copy, Clone)]
pub struct DoiDepositCrossref {}

const DEPOSIT_ERROR: &str = "doideposit::crossref";
const CROSSREF_NS: &[(&str, &str)] = &[
    ("version", "5.3.1"),
    ("xmlns", "http://www.crossref.org/schema/5.3.1"),
    ("xmlns:xsi", "http://www.w3.org/2001/XMLSchema-instance"),
    (
        "xsi:schemaLocation",
        "http://www.crossref.org/schema/5.3.1 http://www.crossref.org/schemas/crossref5.3.1.xsd",
    ),
    ("xmlns:ai", "http://www.crossref.org/AccessIndicators.xsd"),
    ("xmlns:jats", "http://www.ncbi.nlm.nih.gov/JATS1"),
    ("xmlns:fr", "http://www.crossref.org/fundref.xsd"),
];

// Output format based on schema documentation at https://data.crossref.org/reports/help/schema_doc/5.3.1/index.html
// (retrieved via https://www.crossref.org/documentation/schema-library/xsd-schema-quick-reference/).
// Output validity tested using tool at https://www.crossref.org/02publishers/parser.html
// (retrieved via https://www.crossref.org/documentation/member-setup/direct-deposit-xml/testing-your-xml/).
impl XmlSpecification for DoiDepositCrossref {
    fn handle_event<W: Write>(w: &mut EventWriter<W>, works: &[Work]) -> ThothResult<()> {
        match works {
            [] => Err(ThothError::IncompleteMetadataRecord(
                DEPOSIT_ERROR.to_string(),
                "Not enough data".to_string(),
            )),
            [work] => {
                let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
                let work_id = format!("{}_{}", work.work_id, timestamp);

                write_full_element_block("doi_batch", Some(CROSSREF_NS.to_vec()), w, |w| {
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
                                w.write(XmlEvent::Characters("distribution@thoth.pub"))
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
            write_full_element_block("book", Some(vec![("book_type", work_type)]), w, |w| {
                // Only one series can be listed, so we select the first one found (if any).
                if let Some((series, ordinal)) =
                    self.issues.first().map(|i| (&i.series, i.issue_ordinal))
                {
                    write_full_element_block(
                        "book_series_metadata",
                        Some(vec![("language", "en")]),
                        w,
                        |w| {
                            XmlElementBlock::<DoiDepositCrossref>::xml_element(series, w)?;
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
                        Some(vec![("language", "en")]),
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
                // WorkQuery should already have retrieved these sorted by ordinal, but sort again for safety
                chapters.sort_by(|a, b| a.relation_ordinal.cmp(&b.relation_ordinal));
                for chapter in chapters
                    .iter()
                    .filter(|r| r.relation_type == RelationType::HAS_CHILD)
                {
                    XmlElementBlock::<DoiDepositCrossref>::xml_element(chapter, w)?;
                }
                Ok(())
            })
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
                XmlElementBlock::<DoiDepositCrossref>::xml_element(contribution, w)?;
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

    // Crossref supports multiple abstracts when tagged with the "abstract-type" attribute,
    // which can be set to any value. In our case we use "long" or "short".
    // Abstracts must be output in JATS, we simply convert them into JATS by extracting its
    // paragraphs and tagging them with <jats:p>
    if let Some(long_abstract) = &work.long_abstract {
        write_full_element_block(
            "jats:abstract",
            Some(vec![("abstract-type", "long")]),
            w,
            |w| {
                for paragraph in long_abstract.lines() {
                    if !paragraph.is_empty() {
                        write_element_block("jats:p", w, |w| {
                            w.write(XmlEvent::Characters(paragraph))
                                .map_err(|e| e.into())
                        })?;
                    }
                }
                Ok(())
            },
        )?;
    }
    if let Some(short_abstract) = &work.short_abstract {
        write_full_element_block(
            "jats:abstract",
            Some(vec![("abstract-type", "short")]),
            w,
            |w| {
                for paragraph in short_abstract.lines() {
                    if !paragraph.is_empty() {
                        write_element_block("jats:p", w, |w| {
                            w.write(XmlEvent::Characters(paragraph))
                                .map_err(|e| e.into())
                        })?;
                    }
                }
                Ok(())
            },
        )?;
    }

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
    if let Some(edition) = work.edition {
        if is_chapter {
            // `edition_number` is not supported for chapters,
            // but edition should always be None for Thoth chapters.
            return Err(ThothError::IncompleteMetadataRecord(
                DEPOSIT_ERROR.to_string(),
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
        // `publication_date` element is mandatory for `book_metadata` and `book_series_metadata`
        return Err(ThothError::IncompleteMetadataRecord(
            DEPOSIT_ERROR.to_string(),
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
        let mut publications: Vec<WorkRelationsRelatedWorkPublications> = work
            .publications
            .clone()
            .into_iter()
            .filter(|p| p.isbn.is_some())
            .collect();
        if !publications.is_empty() {
            // Workaround for CrossRef's limit of 6 on the number of ISBNs permissible within a deposit file.
            // We raised this with CrossRef and they believe they should be able to increase the limit.
            // Remove this workaround once this is done (see https://github.com/thoth-pub/thoth/issues/379).
            // In the meantime, this is only encountered with OBP works, which have 7 ISBNs as standard.
            // The least important of these is the HTML ISBN, so omit it.
            if publications.len() > 6 {
                if let Some(html_index) = publications
                    .iter()
                    .position(|p| p.publication_type == PublicationType::HTML)
                {
                    publications.swap_remove(html_index);
                }
            }
            // If there are still more than 6 ISBNs, assume they were added in decreasing order of importance.
            while publications.len() > 6 {
                publications.pop();
            }
            for publication in &publications {
                XmlElementBlock::<DoiDepositCrossref>::xml_element(publication, w)?;
            }
        } else {
            // `book_metadata` must have either at least one `isbn` element or a `noisbn`
            // element with a `reason` attribute - assume missing ISBNs are erroneous
            return Err(ThothError::IncompleteMetadataRecord(
                DEPOSIT_ERROR.to_string(),
                "This work does not have any ISBNs".to_string(),
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
    if !work.fundings.is_empty() {
        write_full_element_block("fr:program", Some(vec![("name", "fundref")]), w, |w| {
            for funding in &work.fundings {
                XmlElementBlock::<DoiDepositCrossref>::xml_element(funding, w)?;
            }
            Ok(())
        })?;
    }
    write_full_element_block(
        "ai:program",
        Some(vec![("name", "AccessIndicators")]),
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
                        Some(vec![("property", "crawler-based")]),
                        w,
                        |w| {
                            for crawler in ["iParadigms", "google", "msn", "yahoo", "scirus"] {
                                write_full_element_block(
                                    "item",
                                    Some(vec![("crawler", crawler)]),
                                    w,
                                    |w| {
                                        write_full_element_block(
                                            "resource",
                                            Some(vec![("mime_type", "application/pdf")]),
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
                        Some(vec![("property", "text-mining")]),
                        w,
                        |w| {
                            write_element_block("item", w, |w| {
                                write_full_element_block(
                                    "resource",
                                    Some(vec![("mime_type", "application/pdf")]),
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
                DEPOSIT_ERROR.to_string(),
                "Missing chapter Landing Page".to_string(),
            ));
        }
    } else if is_chapter {
        // `doi_data` element is mandatory for `content_item`, and must contain
        // both `doi` element and `resource` (landing page) element
        return Err(ThothError::IncompleteMetadataRecord(
            DEPOSIT_ERROR.to_string(),
            "Missing chapter DOI".to_string(),
        ));
    }

    if !work.references.is_empty() {
        write_element_block("citation_list", w, |w| {
            for reference in &work.references {
                XmlElementBlock::<DoiDepositCrossref>::xml_element(reference, w)?;
            }
            Ok(())
        })?;
    }

    Ok(())
}

impl XmlElementBlock<DoiDepositCrossref> for WorkIssuesSeries {
  fn xml_element<W: Write>(&self, w: &mut EventWriter<W>) -> ThothResult<()> {
    if self.issn_digital.is_some() || self.issn_print.is_some() {
      write_element_block("series_metadata", w, |w| {
        write_element_block("titles", w, |w| {
          write_element_block("title", w, |w| {
            w.write(XmlEvent::Characters(&self.series_name))
              .map_err(|e| e.into())
          })
        })?;
        if let Some(issn_print) = &self.issn_print {
          write_full_element_block("issn", Some(vec![("media_type", "print")]), w, |w| {
            w.write(XmlEvent::Characters(issn_print))
              .map_err(|e| e.into())
          })?;
        }
        if let Some(issn_digital) = &self.issn_digital {
          write_full_element_block("issn", Some(vec![("media_type", "electronic")]), w, |w| {
            w.write(XmlEvent::Characters(issn_digital))
              .map_err(|e| e.into())
          })?;
        }
        Ok(())
      })
    }
    else {
      Ok(())
    }
  }
}

impl XmlElementBlock<DoiDepositCrossref> for WorkRelations {
    fn xml_element<W: Write>(&self, w: &mut EventWriter<W>) -> ThothResult<()> {
        if !self.relation_type.eq(&RelationType::HAS_CHILD) {
            // Caller should only pass in child works (chapters), not any other relations.
            unreachable!()
        }
        write_full_element_block(
            "content_item",
            Some(vec![("component_type", "chapter")]),
            w,
            |w| work_metadata(w, &self.related_work, Some(self.relation_ordinal), None),
        )
    }
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
                Some(vec![("media_type", isbn_type.as_str())]),
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
            | ContributionType::ILLUSTRATOR
            | ContributionType::MUSIC_EDITOR
            | ContributionType::FOREWORD_BY
            | ContributionType::INTRODUCTION_BY
            | ContributionType::AFTERWORD_BY
            | ContributionType::PREFACE_BY
            | ContributionType::SOFTWARE_BY
            | ContributionType::RESEARCH_BY
            | ContributionType::CONTRIBUTIONS_BY
            | ContributionType::INDEXER
            | ContributionType::Other(_) => unreachable!(),
        };
        let ordinal = match &self.contribution_ordinal {
            1 => "first",
            _ => "additional",
        };
        write_full_element_block(
            "person_name",
            Some(vec![("sequence", ordinal), ("contributor_role", role)]),
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
                if !self.affiliations.is_empty() {
                    write_element_block("affiliations", w, |w| {
                        for affiliation in &self.affiliations {
                            XmlElementBlock::<DoiDepositCrossref>::xml_element(
                                &affiliation.institution,
                                w,
                            )?;
                        }
                        Ok(())
                    })?;
                }
                if let Some(orcid) = &self.contributor.orcid {
                    write_element_block("ORCID", w, |w| {
                        // Leading `https://orcid.org` is required, and omitted by orcid.to_string()
                        w.write(XmlEvent::Characters(&orcid.with_domain()))
                            .map_err(|e| e.into())
                    })?;
                }
                Ok(())
            },
        )
    }
}

impl XmlElementBlock<DoiDepositCrossref>
    for WorkRelationsRelatedWorkContributionsAffiliationsInstitution
{
    fn xml_element<W: Write>(&self, w: &mut EventWriter<W>) -> ThothResult<()> {
        write_element_block("institution", w, |w| {
            write_element_block("institution_name", w, |w| {
                w.write(XmlEvent::Characters(&self.institution_name))
                    .map_err(|e| e.into())
            })?;
            if let Some(ror) = &self.ror {
                write_full_element_block("institution_id", Some(vec![("type", "ror")]), w, |w| {
                    w.write(XmlEvent::Characters(&ror.with_domain()))
                        .map_err(|e| e.into())
                })?;
            }
            Ok(())
        })
    }
}

impl XmlElementBlock<DoiDepositCrossref> for WorkRelationsRelatedWorkFundings {
    fn xml_element<W: Write>(&self, w: &mut EventWriter<W>) -> ThothResult<()> {
        write_full_element_block("fr:assertion", Some(vec![("name", "fundgroup")]), w, |w| {
            write_full_element_block(
                "fr:assertion",
                Some(vec![("name", "funder_name")]),
                w,
                |w| {
                    w.write(XmlEvent::Characters(&self.institution.institution_name))?;
                    if let Some(doi) = &self.institution.institution_doi {
                        write_full_element_block(
                            "fr:assertion",
                            Some(vec![("name", "funder_identifier")]),
                            w,
                            |w| {
                                w.write(XmlEvent::Characters(&doi.with_domain()))
                                    .map_err(|e| e.into())
                            },
                        )?;
                    }
                    Ok(())
                },
            )?;
            if let Some(grant_number) = &self.grant_number {
                write_full_element_block(
                    "fr:assertion",
                    Some(vec![("name", "award_number")]),
                    w,
                    |w| {
                        w.write(XmlEvent::Characters(grant_number))
                            .map_err(|e| e.into())
                    },
                )?;
            }
            Ok(())
        })
    }
}

impl XmlElementBlock<DoiDepositCrossref> for WorkRelationsRelatedWorkReferences {
    fn xml_element<W: Write>(&self, w: &mut EventWriter<W>) -> ThothResult<()> {
        let key = format!("ref{}", &self.reference_ordinal);
        write_full_element_block("citation", Some(vec![("key", key.as_ref())]), w, |w| {
            if let Some(doi) = &self.doi {
                write_element_block("doi", w, |w| {
                    w.write(XmlEvent::Characters(&doi.to_string()))
                        .map_err(|e| e.into())
                })?;
            }
            if let Some(unstructured_citation) = &self.unstructured_citation {
                write_element_block("unstructured_citation", w, |w| {
                    w.write(XmlEvent::Characters(unstructured_citation))
                        .map_err(|e| e.into())
                })?;
            }
            if let Some(issn) = &self.issn {
                write_element_block("issn", w, |w| {
                    w.write(XmlEvent::Characters(issn)).map_err(|e| e.into())
                })?;
            }
            if let Some(isbn) = &self.isbn {
                write_element_block("isbn", w, |w| {
                    w.write(XmlEvent::Characters(&isbn.to_string()))
                        .map_err(|e| e.into())
                })?;
            }
            if let Some(journal_title) = &self.journal_title {
                write_element_block("journal_title", w, |w| {
                    w.write(XmlEvent::Characters(journal_title))
                        .map_err(|e| e.into())
                })?;
            }
            if let Some(article_title) = &self.article_title {
                write_element_block("article_title", w, |w| {
                    w.write(XmlEvent::Characters(article_title))
                        .map_err(|e| e.into())
                })?;
            }
            if let Some(series_title) = &self.series_title {
                write_element_block("series_title", w, |w| {
                    w.write(XmlEvent::Characters(series_title))
                        .map_err(|e| e.into())
                })?;
            }
            if let Some(volume_title) = &self.volume_title {
                write_element_block("volume_title", w, |w| {
                    w.write(XmlEvent::Characters(volume_title))
                        .map_err(|e| e.into())
                })?;
            }
            if let Some(edition) = &self.edition {
                write_element_block("edition_number", w, |w| {
                    w.write(XmlEvent::Characters(&edition.to_string()))
                        .map_err(|e| e.into())
                })?;
            }
            if let Some(author) = &self.author {
                write_element_block("author", w, |w| {
                    w.write(XmlEvent::Characters(author)).map_err(|e| e.into())
                })?;
            }
            if let Some(volume) = &self.volume {
                write_element_block("volume", w, |w| {
                    w.write(XmlEvent::Characters(volume)).map_err(|e| e.into())
                })?;
            }
            if let Some(issue) = &self.issue {
                write_element_block("issue", w, |w| {
                    w.write(XmlEvent::Characters(issue)).map_err(|e| e.into())
                })?;
            }
            if let Some(first_page) = &self.first_page {
                write_element_block("first_page", w, |w| {
                    w.write(XmlEvent::Characters(first_page))
                        .map_err(|e| e.into())
                })?;
            }
            if let Some(component_number) = &self.component_number {
                write_element_block("component_number", w, |w| {
                    w.write(XmlEvent::Characters(component_number))
                        .map_err(|e| e.into())
                })?;
            }
            // a citation for a standard must contain all three fields
            if self.standard_designator.is_some()
                && self.standards_body_name.is_some()
                && self.standards_body_acronym.is_some()
            {
                write_element_block("std_designator", w, |w| {
                    w.write(XmlEvent::Characters(
                        self.standard_designator.as_ref().unwrap(),
                    ))
                    .map_err(|e| e.into())
                })?;
                write_element_block("standards_body", w, |w| {
                    write_element_block("standards_body_name", w, |w| {
                        w.write(XmlEvent::Characters(
                            self.standards_body_name.as_ref().unwrap(),
                        ))
                        .map_err(|e| e.into())
                    })?;
                    write_element_block("standards_body_acronym", w, |w| {
                        w.write(XmlEvent::Characters(
                            self.standards_body_acronym.as_ref().unwrap(),
                        ))
                        .map_err(|e| e.into())
                    })
                })?;
            }
            if let Some(date) = &self.publication_date {
                write_element_block("cYear", w, |w| {
                    w.write(XmlEvent::Characters(&date.format("%Y").to_string()))
                        .map_err(|e| e.into())
                })?;
            }
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    // Testing note: XML nodes cannot be guaranteed to be output in the same order every time
    // We therefore rely on `assert!(contains)` rather than `assert_eq!`
    use super::*;
    use std::str::FromStr;
    use thoth_api::model::{Doi, Isbn, Orcid, Ror};
    use thoth_client::{
        ContributionType, FundingInstitution, LocationPlatform, PublicationType, SeriesType,
        WorkContributions, WorkContributionsAffiliations, WorkContributionsAffiliationsInstitution,
        WorkContributionsContributor, WorkFundings, WorkImprint, WorkImprintPublisher, WorkIssues,
        WorkIssuesSeries, WorkPublications, WorkPublicationsLocations, WorkReferences,
        WorkRelations, WorkRelationsRelatedWorkContributions,
        WorkRelationsRelatedWorkContributionsAffiliations,
        WorkRelationsRelatedWorkContributionsAffiliationsInstitution,
        WorkRelationsRelatedWorkContributionsContributor, WorkRelationsRelatedWorkImprint,
        WorkRelationsRelatedWorkImprintPublisher, WorkRelationsRelatedWorkPublicationsLocations,
        WorkStatus, WorkType,
    };
    use uuid::Uuid;

    fn generate_test_output(
        expect_ok: bool,
        input: &impl XmlElementBlock<DoiDepositCrossref>,
    ) -> String {
        // Helper function based on `XmlSpecification::generate`
        let mut buffer = Vec::new();
        let mut writer = xml::writer::EmitterConfig::new()
            .perform_indent(true)
            .create_writer(&mut buffer);
        let wrapped_output = XmlElementBlock::<DoiDepositCrossref>::xml_element(input, &mut writer)
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
    fn test_doideposit_crossref_relatedworks() {
        let mut test_relations = WorkRelations {
            relation_type: RelationType::HAS_CHILD,
            relation_ordinal: 1,
            related_work: WorkRelationsRelatedWork {
                full_title: "Chapter: One".to_string(),
                title: "Chapter".to_string(),
                subtitle: Some("One".to_string()),
                edition: None,
                doi: Some(Doi::from_str("https://doi.org/10.00001/CHAPTER.0001").unwrap()),
                publication_date: chrono::NaiveDate::from_ymd_opt(2000, 2, 28),
                license: Some("https://creativecommons.org/licenses/by-nd/4.0/".to_string()),
                short_abstract: Some("A shorter abstract".to_string()),
                long_abstract: Some("First paragraph.\n\nSecond paragraph.".to_string()),
                place: Some("Other Place".to_string()),
                first_page: Some("10".to_string()),
                last_page: Some("20".to_string()),
                page_count: Some(11),
                page_interval: Some("10–20".to_string()),
                landing_page: Some("https://www.book.com/chapter_one".to_string()),
                imprint: WorkRelationsRelatedWorkImprint {
                    publisher: WorkRelationsRelatedWorkImprintPublisher {
                        publisher_name: "Chapter One Publisher".to_string(),
                    },
                },
                contributions: vec![WorkRelationsRelatedWorkContributions {
                    contribution_type: ContributionType::AUTHOR,
                    first_name: Some("Chapter One".to_string()),
                    last_name: "Author".to_string(),
                    full_name: "Chapter One Author".to_string(),
                    contribution_ordinal: 1,
                    contributor: WorkRelationsRelatedWorkContributionsContributor {
                        orcid: Some(
                            Orcid::from_str("https://orcid.org/0000-0002-0000-0011").unwrap(),
                        ),
                    },
                    affiliations: vec![WorkRelationsRelatedWorkContributionsAffiliations {
                        position: None,
                        affiliation_ordinal: 1,
                        institution: WorkRelationsRelatedWorkContributionsAffiliationsInstitution {
                            institution_name: "Thoth University".to_string(),
                            ror: Some(Ror::from_str("https://ror.org/0abcdef12").unwrap()),
                        },
                    }],
                }],
                publications: vec![WorkRelationsRelatedWorkPublications {
                    publication_type: PublicationType::PDF,
                    isbn: Some(Isbn::from_str("978-1-56619-909-4").unwrap()),
                    locations: vec![WorkRelationsRelatedWorkPublicationsLocations {
                        full_text_url: Some("https://www.book.com/chapterone_fulltext".to_string()),
                        canonical: true,
                    }],
                }],
                references: vec![],
                fundings: vec![],
            },
        };

        let output = generate_test_output(true, &test_relations);
        assert!(output.contains(r#"<content_item component_type="chapter">"#));
        assert!(output.contains(r#"  <contributors>"#));
        assert!(
            output.contains(r#"    <person_name contributor_role="author" sequence="first">"#)
                || output
                    .contains(r#"    <person_name sequence="first" contributor_role="author">"#)
        );
        assert!(output.contains(r#"      <given_name>Chapter One</given_name>"#));
        assert!(output.contains(r#"      <surname>Author</surname>"#));
        assert!(output.contains(r#"      <ORCID>https://orcid.org/0000-0002-0000-0011</ORCID>"#));
        assert!(output.contains(r#"      <affiliations>"#));
        assert!(output.contains(r#"        <institution>"#));
        assert!(
            output.contains(r#"          <institution_name>Thoth University</institution_name>"#)
        );
        assert!(output.contains(
            r#"          <institution_id type="ror">https://ror.org/0abcdef12</institution_id>"#
        ));
        assert!(output.contains(r#"  <titles>"#));
        assert!(output.contains(r#"    <title>Chapter</title>"#));
        assert!(output.contains(r#"    <subtitle>One</subtitle>"#));
        assert!(output.contains(r#"  </titles>"#));
        assert!(output.contains(r#"  <component_number>1</component_number>"#));
        assert!(output.contains(r#"  <jats:abstract abstract-type="long">"#));
        assert!(output.contains(r#"    <jats:p>First paragraph.</jats:p>"#));
        assert!(output.contains(r#"    <jats:p>Second paragraph.</jats:p>"#));
        assert!(output.contains(r#"  <jats:abstract abstract-type="short">"#));
        assert!(output.contains(r#"    <jats:p>A shorter abstract</jats:p>"#));
        assert!(!output.contains(r#"    <jats:p></jats:p>"#));
        assert!(output.contains(r#"  <publication_date>"#));
        assert!(output.contains(r#"    <month>02</month>"#));
        assert!(output.contains(r#"    <day>28</day>"#));
        assert!(output.contains(r#"    <year>2000</year>"#));
        // ISBNs are not output for chapters
        assert!(!output.contains(r#"  <isbn media_type="print">978-1-4028-9462-6</isbn>"#));
        // Publisher data is not output for chapters
        assert!(!output.contains(r#"  <publisher>"#));
        assert!(!output.contains(r#"    <publisher_name>OA Editions</publisher_name>"#));
        assert!(!output.contains(r#"    <publisher_place>León, Spain</publisher_place>"#));
        assert!(output.contains(r#"  <pages>"#));
        assert!(output.contains(r#"    <first_page>10</first_page>"#));
        assert!(output.contains(r#"    <last_page>20</last_page>"#));
        assert!(output.contains(r#"  <ai:program name="AccessIndicators">"#));
        assert!(output.contains(r#"    <ai:free_to_read />"#));
        assert!(output.contains(r#"    <ai:license_ref>https://creativecommons.org/licenses/by-nd/4.0/</ai:license_ref>"#));
        assert!(output.contains(r#"  <doi_data>"#));
        assert!(output.contains(r#"    <doi>10.00001/CHAPTER.0001</doi>"#));
        assert!(output.contains(r#"    <resource>https://www.book.com/chapter_one</resource>"#));
        assert!(output.contains(r#"    <collection property="crawler-based">"#));
        assert!(output.contains(r#"      <item crawler="iParadigms">"#));
        assert!(output.contains(r#"        <resource mime_type="application/pdf">https://www.book.com/chapterone_fulltext</resource>"#));
        assert!(output.contains(r#"      <item crawler="google">"#));
        assert!(output.contains(r#"      <item crawler="msn">"#));
        assert!(output.contains(r#"      <item crawler="yahoo">"#));
        assert!(output.contains(r#"      <item crawler="scirus">"#));
        assert!(output.contains(r#"    <collection property="text-mining">"#));

        // Remove/change some values to test variations/non-output of optional blocks
        test_relations.related_work.subtitle = None;
        test_relations.related_work.last_page = None;
        test_relations.related_work.publication_date = None;
        test_relations.related_work.license = None;
        test_relations.related_work.contributions.clear();
        test_relations.related_work.publications.clear();
        let output = generate_test_output(true, &test_relations);
        // Sole contributor removed
        assert!(!output.contains(r#"  <contributors>"#));
        assert!(
            !output.contains(r#"    <person_name contributor_role="author" sequence="first">"#)
                && !output
                    .contains(r#"    <person_name sequence="first" contributor_role="author">"#)
        );
        assert!(!output.contains(r#"      <given_name>Chapter One</given_name>"#));
        assert!(!output.contains(r#"      <surname>Author</surname>"#));
        assert!(!output.contains(r#"      <ORCID>https://orcid.org/0000-0002-0000-0011</ORCID>"#));
        // No subtitle supplied
        assert!(!output.contains(r#"    <subtitle>One</subtitle>"#));
        // No last page supplied
        assert!(output.contains(r#"  <pages>"#));
        assert!(output.contains(r#"    <first_page>10</first_page>"#));
        assert!(!output.contains(r#"    <last_page>20</last_page>"#));
        // No publication date supplied
        assert!(!output.contains(r#"  <publication_date>"#));
        assert!(!output.contains(r#"    <month>02</month>"#));
        assert!(!output.contains(r#"    <day>28</day>"#));
        assert!(!output.contains(r#"    <year>2000</year>"#));
        // No licence supplied
        assert!(output.contains(r#"  <ai:program name="AccessIndicators">"#));
        assert!(output.contains(r#"    <ai:free_to_read />"#));
        assert!(!output.contains(
            r#"    <ai:license_ref>https://creativecommons.org/licenses/by/4.0/</ai:license_ref>"#
        ));
        // No PDF URL supplied: all `collection` elements omitted
        assert!(output.contains(r#"  <doi_data>"#));
        assert!(output.contains(r#"    <doi>10.00001/CHAPTER.0001</doi>"#));
        assert!(output.contains(r#"    <resource>https://www.book.com/chapter_one</resource>"#));
        assert!(!output.contains(r#"    <collection property="crawler-based">"#));
        assert!(!output.contains(r#"      <item crawler="iParadigms">"#));
        assert!(!output.contains(r#"        <resource mime_type="application/pdf">https://www.book.com/chapterone_fulltext</resource>"#));
        assert!(!output.contains(r#"      <item crawler="google">"#));
        assert!(!output.contains(r#"      <item crawler="msn">"#));
        assert!(!output.contains(r#"      <item crawler="yahoo">"#));
        assert!(!output.contains(r#"      <item crawler="scirus">"#));
        assert!(!output.contains(r#"    <collection property="text-mining">"#));

        test_relations.related_work.first_page = None;
        let output = generate_test_output(true, &test_relations);
        // No first page supplied: `pages` element omitted entirely
        assert!(!output.contains(r#"  <pages>"#));
        assert!(!output.contains(r#"    <first_page>10</first_page>"#));

        // Editions are not valid chapter metadata
        test_relations.related_work.edition = Some(1);
        let output = generate_test_output(false, &test_relations);
        assert_eq!(
            output,
            "Could not generate doideposit::crossref: Chapters cannot have Edition numbers"
                .to_string()
        );

        // Remove landing page. Result: error (cannot generate mandatory `doi_data` element)
        test_relations.related_work.edition = None;
        test_relations.related_work.landing_page = None;
        let output = generate_test_output(false, &test_relations);
        assert_eq!(
            output,
            "Could not generate doideposit::crossref: Missing chapter Landing Page".to_string()
        );

        // Restore landing page but remove DOI. Result: error, as above
        test_relations.related_work.edition = None;
        test_relations.related_work.landing_page =
            Some("https://www.book.com/chapter_one".to_string());
        test_relations.related_work.doi = None;
        let output = generate_test_output(false, &test_relations);
        assert_eq!(
            output,
            "Could not generate doideposit::crossref: Missing chapter DOI".to_string()
        );
    }

    #[test]
    fn test_doideposit_crossref_works() {
        let mut test_work = Work {
            work_id: Uuid::from_str("00000000-0000-0000-AAAA-000000000001").unwrap(),
            work_status: WorkStatus::ACTIVE,
            full_title: "Book Title: Book Subtitle".to_string(),
            title: "Book Title".to_string(),
            subtitle: Some("Book Subtitle".to_string()),
            work_type: WorkType::MONOGRAPH,
            reference: None,
            edition: Some(100),
            doi: Some(Doi::from_str("https://doi.org/10.00001/BOOK.0001").unwrap()),
            publication_date: chrono::NaiveDate::from_ymd_opt(1999, 12, 31),
            license: Some("https://creativecommons.org/licenses/by/4.0/".to_string()),
            copyright_holder: Some("Author 1; Author 2".to_string()),
            short_abstract: None,
            long_abstract: Some("Lorem ipsum dolor sit amet".to_string()),
            general_note: None,
            bibliography_note: None,
            place: Some("León, Spain".to_string()),
            page_count: None,
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
            cover_url: None,
            cover_caption: None,
            imprint: WorkImprint {
                imprint_name: "OA Editions Imprint".to_string(),
                imprint_url: None,
                publisher: WorkImprintPublisher {
                    publisher_name: "OA Editions".to_string(),
                    publisher_shortname: Some("OAE".to_string()),
                    publisher_url: None,
                },
            },
            issues: vec![
                WorkIssues {
                    issue_ordinal: 11,
                    series: WorkIssuesSeries {
                        series_id: Uuid::parse_str("00000000-0000-0000-BBBB-000000000002").unwrap(),
                        series_type: SeriesType::BOOK_SERIES,
                        series_name: "Name of series".to_string(),
                        issn_print: Some("1234-5678".to_string()),
                        issn_digital: Some("8765-4321".to_string()),
                        series_url: None,
                        series_description: None,
                        series_cfp_url: None,
                    },
                },
                WorkIssues {
                    issue_ordinal: 22,
                    series: WorkIssuesSeries {
                        series_id: Uuid::parse_str("00000000-0000-0000-BBBB-000000000002").unwrap(),
                        series_type: SeriesType::BOOK_SERIES,
                        series_name: "Irrelevant series".to_string(),
                        issn_print: Some("1111-2222".to_string()),
                        issn_digital: Some("3333-4444".to_string()),
                        series_url: None,
                        series_description: None,
                        series_cfp_url: None,
                    },
                },
            ],
            contributions: vec![
                WorkContributions {
                    contribution_type: ContributionType::PHOTOGRAPHER,
                    first_name: Some("Omitted".to_string()),
                    last_name: "Contributor".to_string(),
                    full_name: "Omitted Contributor".to_string(),
                    main_contribution: true,
                    biography: None,
                    contribution_ordinal: 4,
                    contributor: WorkContributionsContributor {
                        orcid: Some(
                            Orcid::from_str("https://orcid.org/0000-0002-0000-0004").unwrap(),
                        ),
                        website: None,
                    },
                    affiliations: vec![],
                },
                WorkContributions {
                    contribution_type: ContributionType::AUTHOR,
                    first_name: Some("Sole".to_string()),
                    last_name: "Author".to_string(),
                    full_name: "Sole Author".to_string(),
                    main_contribution: true,
                    biography: None,
                    contribution_ordinal: 1,
                    contributor: WorkContributionsContributor {
                        orcid: Some(
                            Orcid::from_str("https://orcid.org/0000-0002-0000-0001").unwrap(),
                        ),
                        website: None,
                    },
                    affiliations: vec![WorkContributionsAffiliations {
                        position: None,
                        affiliation_ordinal: 1,
                        institution: WorkContributionsAffiliationsInstitution {
                            institution_name: "Thoth University".to_string(),
                            institution_doi: None,
                            ror: Some(Ror::from_str("https://ror.org/0abcdef12").unwrap()),
                            country_code: None,
                        },
                    }],
                },
                WorkContributions {
                    contribution_type: ContributionType::EDITOR,
                    first_name: Some("Only".to_string()),
                    last_name: "Editor".to_string(),
                    full_name: "Only Editor".to_string(),
                    main_contribution: true,
                    biography: None,
                    contribution_ordinal: 2,
                    contributor: WorkContributionsContributor {
                        orcid: Some(
                            Orcid::from_str("https://orcid.org/0000-0002-0000-0002").unwrap(),
                        ),
                        website: None,
                    },
                    affiliations: vec![],
                },
                WorkContributions {
                    contribution_type: ContributionType::TRANSLATOR,
                    first_name: None,
                    last_name: "Translator".to_string(),
                    full_name: "Translator".to_string(),
                    main_contribution: true,
                    biography: None,
                    contribution_ordinal: 3,
                    contributor: WorkContributionsContributor {
                        orcid: None,
                        website: None,
                    },
                    affiliations: vec![WorkContributionsAffiliations {
                        position: None,
                        affiliation_ordinal: 1,
                        institution: WorkContributionsAffiliationsInstitution {
                            institution_name: "COPIM".to_string(),
                            institution_doi: None,
                            ror: None,
                            country_code: None,
                        },
                    }],
                },
            ],
            languages: vec![],
            publications: vec![
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
                WorkPublications {
                    publication_id: Uuid::from_str("00000000-0000-0000-FFFF-000000000006").unwrap(),
                    publication_type: PublicationType::XML,
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
                        landing_page: Some("https://www.book.com/xml_landing".to_string()),
                        full_text_url: Some("https://www.book.com/xml_fulltext".to_string()),
                        location_platform: LocationPlatform::OTHER,
                        canonical: true,
                    }],
                },
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
            ],
            subjects: vec![],
            fundings: vec![
                WorkFundings {
                    program: None,
                    project_name: None,
                    project_shortname: None,
                    grant_number: Some("12345".to_string()),
                    jurisdiction: None,
                    institution: FundingInstitution {
                        institution_name: "Funding Body".to_string(),
                        institution_doi: None,
                        ror: None,
                        country_code: None,
                    },
                },
                WorkFundings {
                    program: None,
                    project_name: None,
                    project_shortname: None,
                    grant_number: None,
                    jurisdiction: None,
                    institution: FundingInstitution {
                        institution_name: "Some Funder".to_string(),
                        institution_doi: Some(
                            Doi::from_str("https://doi.org/10.00001/funder").unwrap(),
                        ),
                        ror: None,
                        country_code: None,
                    },
                },
            ],
            relations: vec![WorkRelations {
                relation_type: RelationType::HAS_PART,
                relation_ordinal: 1,
                related_work: WorkRelationsRelatedWork {
                    full_title: "Part: One".to_string(),
                    title: "Part".to_string(),
                    subtitle: Some("One".to_string()),
                    edition: None,
                    doi: Some(Doi::from_str("https://doi.org/10.00001/PART.0001").unwrap()),
                    publication_date: chrono::NaiveDate::from_ymd_opt(2000, 2, 28),
                    license: Some("https://creativecommons.org/licenses/by-nd/4.0/".to_string()),
                    short_abstract: None,
                    long_abstract: None,
                    place: Some("Other Place".to_string()),
                    first_page: Some("10".to_string()),
                    last_page: Some("20".to_string()),
                    page_count: Some(11),
                    page_interval: Some("10–20".to_string()),
                    landing_page: Some("https://www.book.com/part_one".to_string()),
                    imprint: WorkRelationsRelatedWorkImprint {
                        publisher: WorkRelationsRelatedWorkImprintPublisher {
                            publisher_name: "Part One Publisher".to_string(),
                        },
                    },
                    contributions: vec![],
                    publications: vec![],
                    references: vec![],
                    fundings: vec![],
                },
            }],
            references: vec![WorkReferences {
                reference_ordinal: 1,
                doi: Some(Doi::from_str("https://doi.org/10.00001/reference").unwrap()),
                unstructured_citation: Some("Author, A. (2022) Article, Journal.".to_string()),
                issn: Some("1111-2222".to_string()),
                isbn: None,
                journal_title: Some("Journal".to_string()),
                article_title: Some("Article".to_string()),
                series_title: None,
                volume_title: None,
                edition: None,
                author: Some("Author, A".to_string()),
                volume: None,
                issue: None,
                first_page: Some("3".to_string()),
                component_number: None,
                standard_designator: None,
                standards_body_name: None,
                standards_body_acronym: None,
                publication_date: chrono::NaiveDate::from_ymd_opt(2022, 1, 1),
                retrieval_date: None,
            }],
        };

        // Test standard output
        let output = generate_test_output(true, &test_work);
        assert!(output.contains(r#"  <book book_type="monograph">"#));
        assert!(output.contains(r#"    <book_series_metadata language="en">"#));
        assert!(output.contains(r#"      <series_metadata>"#));
        assert!(output.contains(r#"        <titles>"#));
        assert!(output.contains(r#"          <title>Name of series</title>"#));
        assert!(output.contains(r#"        <issn media_type="print">1234-5678</issn>"#));
        assert!(output.contains(r#"        <issn media_type="electronic">8765-4321</issn>"#));
        assert!(!output.contains(r#"          <title>Irrelevant series</title>"#));
        assert!(!output.contains(r#"        <issn media_type="print">1111-2222</issn>"#));
        assert!(!output.contains(r#"        <issn media_type="electronic">3333-4444</issn>"#));
        assert!(output.contains(r#"      <contributors>"#));
        assert!(
            output.contains(r#"        <person_name contributor_role="author" sequence="first">"#)
                || output.contains(
                    r#"        <person_name sequence="first" contributor_role="author">"#
                )
        );
        assert!(output.contains(r#"          <given_name>Sole</given_name>"#));
        assert!(output.contains(r#"          <surname>Author</surname>"#));
        assert!(
            output.contains(r#"          <ORCID>https://orcid.org/0000-0002-0000-0001</ORCID>"#)
        );
        assert!(output.contains(r#"      <affiliations>"#));
        assert!(output.contains(r#"        <institution>"#));
        assert!(
            output.contains(r#"          <institution_name>Thoth University</institution_name>"#)
        );
        assert!(output.contains(
            r#"          <institution_id type="ror">https://ror.org/0abcdef12</institution_id>"#
        ));
        assert!(
            output.contains(
                r#"        <person_name contributor_role="editor" sequence="additional">"#
            ) || output.contains(
                r#"        <person_name sequence="additional" contributor_role="editor">"#
            )
        );
        assert!(output.contains(r#"          <given_name>Only</given_name>"#));
        assert!(output.contains(r#"          <surname>Editor</surname>"#));
        assert!(
            output.contains(r#"          <ORCID>https://orcid.org/0000-0002-0000-0002</ORCID>"#)
        );
        assert!(
            output.contains(
                r#"        <person_name contributor_role="translator" sequence="additional">"#
            ) || output.contains(
                r#"        <person_name sequence="additional" contributor_role="translator">"#
            )
        );
        assert!(output.contains(r#"          <surname>Translator</surname>"#));
        // Contributors other than authors, editors and translators are not output
        assert!(!output.contains(r#"          <given_name>Omitted</given_name>"#));
        assert!(!output.contains(r#"          <surname>Contributor</surname>"#));
        assert!(
            !output.contains(r#"          <ORCID>https://orcid.org/0000-0002-0000-0004</ORCID>"#)
        );
        assert!(output.contains(r#"      <affiliations>"#));
        assert!(output.contains(r#"        <institution>"#));
        assert!(output.contains(r#"          <institution_name>COPIM</institution_name>"#));
        assert!(output.contains(r#"      <titles>"#));
        assert!(output.contains(r#"        <title>Book Title</title>"#));
        assert!(output.contains(r#"        <subtitle>Book Subtitle</subtitle>"#));
        assert!(output.contains(r#"      <jats:abstract abstract-type="long">"#));
        assert!(output.contains(r#"        <jats:p>Lorem ipsum dolor sit amet</jats:p>"#));
        assert!(!output.contains(r#"      <jats:abstract abstract-type="short">"#));
        assert!(output.contains(r#"      <volume>11</volume>"#));
        assert!(output.contains(r#"      <edition_number>100</edition_number>"#));
        assert!(output.contains(r#"      <publication_date>"#));
        assert!(output.contains(r#"        <month>12</month>"#));
        assert!(output.contains(r#"        <day>31</day>"#));
        assert!(output.contains(r#"        <year>1999</year>"#));
        assert!(output.contains(r#"      <isbn media_type="electronic">978-3-16-148410-0</isbn>"#));
        assert!(output.contains(r#"      <isbn media_type="electronic">978-92-95055-02-5</isbn>"#));
        assert!(output.contains(r#"      <isbn media_type="print">978-1-4028-9462-6</isbn>"#));
        assert!(output.contains(r#"      <publisher>"#));
        assert!(output.contains(r#"        <publisher_name>OA Editions</publisher_name>"#));
        assert!(output.contains(r#"        <publisher_place>León, Spain</publisher_place>"#));
        assert!(output.contains(r#"      <fr:program name="fundref">"#));
        assert!(output.contains(r#"        <fr:assertion name="fundgroup">"#));
        assert!(output
            .contains(r#"          <fr:assertion name="funder_name">Funding Body</fr:assertion>"#));
        assert!(
            output.contains(r#"          <fr:assertion name="award_number">12345</fr:assertion>"#)
        );
        assert!(output.contains(r#"          <fr:assertion name="funder_name">Some Funder<fr:assertion name="funder_identifier">https://doi.org/10.00001/funder</fr:assertion>"#));
        assert!(output.contains(r#"      <ai:program name="AccessIndicators">"#));
        assert!(output.contains(r#"        <ai:free_to_read />"#));
        assert!(output.contains(
            r#"      <ai:license_ref>https://creativecommons.org/licenses/by/4.0/</ai:license_ref>"#
        ));
        assert!(output.contains(r#"      <citation_list>"#));
        assert!(output.contains(r#"        <citation key="ref1">"#));
        assert!(output.contains(r#"          <doi>10.00001/reference</doi>"#));
        assert!(output.contains(r#"          <unstructured_citation>Author, A. (2022) Article, Journal.</unstructured_citation>"#));
        assert!(output.contains(r#"          <issn>1111-2222</issn>"#));
        assert!(output.contains(r#"          <journal_title>Journal</journal_title>"#));
        assert!(output.contains(r#"          <article_title>Article</article_title>"#));
        assert!(output.contains(r#"          <author>Author, A</author>"#));
        assert!(output.contains(r#"          <first_page>3</first_page>"#));
        assert!(output.contains(r#"          <cYear>2022</cYear>"#));
        assert!(output.contains(r#"      <doi_data>"#));
        assert!(output.contains(r#"        <doi>10.00001/BOOK.0001</doi>"#));
        assert!(output.contains(r#"        <resource>https://www.book.com</resource>"#));
        assert!(output.contains(r#"        <collection property="crawler-based">"#));
        assert!(output.contains(r#"          <item crawler="iParadigms">"#));
        assert!(output.contains(r#"            <resource mime_type="application/pdf">https://www.book.com/pdf_fulltext</resource>"#));
        assert!(output.contains(r#"          <item crawler="google">"#));
        assert!(output.contains(r#"          <item crawler="msn">"#));
        assert!(output.contains(r#"          <item crawler="yahoo">"#));
        assert!(output.contains(r#"          <item crawler="scirus">"#));
        assert!(output.contains(r#"        <collection property="text-mining">"#));
        // Only omitted relation types supplied: no `content_item` elements emitted
        assert!(!output.contains(r#"    <content_item component_type="chapter">"#));
        assert!(!output.contains(r#"        <title>Part</title>"#));
        assert!(!output.contains(r#"        <subtitle>One</subtitle>"#));
        assert!(!output.contains(r#"      <component_number>1</component_number>"#));
        assert!(!output.contains(r#"        <month>02</month>"#));
        assert!(!output.contains(r#"        <day>28</day>"#));
        assert!(!output.contains(r#"        <year>2000</year>"#));
        assert!(!output.contains(r#"      <pages>"#));
        assert!(!output.contains(r#"        <first_page>10</first_page>"#));
        assert!(!output.contains(r#"        <last_page>20</last_page>"#));
        assert!(!output.contains(r#"        <doi>10.00001/PART.0001</doi>"#));
        assert!(!output.contains(r#"        <resource>https://www.book.com/part_one</resource>"#));

        // Remove/change some values to test variations/non-output of optional blocks
        test_work.work_type = WorkType::EDITED_BOOK;
        test_work.issues.clear();
        test_work.subtitle = None;
        test_work.place = None;
        test_work.license = None;
        // Remove last (translator) contributor
        test_work.contributions.pop();
        test_work.publications[0].locations.clear();
        // Remove last (hardback) publication
        test_work.publications.pop();

        let output = generate_test_output(true, &test_work);
        // Work type changed
        assert!(!output.contains(r#"  <book book_type="monograph">"#));
        assert!(output.contains(r#"  <book book_type="edited_book">"#));
        // No series supplied
        assert!(!output.contains(r#"    <book_series_metadata language="en">"#));
        assert!(output.contains(r#"    <book_metadata language="en">"#));
        assert!(!output.contains(r#"      <series_metadata>"#));
        assert!(!output.contains(r#"        <titles>"#));
        assert!(!output.contains(r#"          <title>Name of series</title>"#));
        assert!(!output.contains(r#"        <issn media_type="print">1234-5678</issn>"#));
        assert!(!output.contains(r#"        <issn media_type="electronic">8765-4321</issn>"#));
        assert!(!output.contains(r#"      <volume>11</volume>"#));
        // Contributor removed
        assert!(
            !output.contains(
                r#"        <person_name contributor_role="translator" sequence="additional">"#
            ) && !output.contains(
                r#"        <person_name sequence="additional" contributor_role="translator">"#
            )
        );
        assert!(!output.contains(r#"          <surname>Translator</surname>"#));
        // No subtitle supplied
        assert!(!output.contains(r#"        <subtitle>Book Subtitle</subtitle>"#));
        // Hardback publication removed
        assert!(!output.contains(r#"      <isbn media_type="print">978-1-4028-9462-6</isbn>"#));
        // No place supplied
        assert!(!output.contains(r#"        <publisher_place>León, Spain</publisher_place>"#));
        // No licence supplied
        assert!(output.contains(r#"      <ai:program name="AccessIndicators">"#));
        assert!(output.contains(r#"        <ai:free_to_read />"#));
        assert!(!output.contains(
            r#"      <ai:license_ref>https://creativecommons.org/licenses/by/4.0/</ai:license_ref>"#
        ));
        // No PDF URL supplied: all `collection` elements omitted
        // (although XML URL is still present)
        assert!(output.contains(r#"      <doi_data>"#));
        assert!(output.contains(r#"        <doi>10.00001/BOOK.0001</doi>"#));
        assert!(output.contains(r#"        <resource>https://www.book.com</resource>"#));
        assert!(!output.contains(r#"        <collection property="crawler-based">"#));
        assert!(!output.contains(r#"          <item crawler="iParadigms">"#));
        assert!(!output.contains(r#"            <resource mime_type="application/pdf">https://www.book.com/pdf_fulltext</resource>"#));
        assert!(!output.contains(r#"          <item crawler="google">"#));
        assert!(!output.contains(r#"          <item crawler="msn">"#));
        assert!(!output.contains(r#"          <item crawler="yahoo">"#));
        assert!(!output.contains(r#"          <item crawler="scirus">"#));
        assert!(!output.contains(r#"        <collection property="text-mining">"#));

        // Change work type, remove landing page, remove XML ISBN,
        // remove all but the omitted contributor
        test_work.work_type = WorkType::TEXTBOOK;
        test_work.landing_page = None;
        test_work.contributions.drain(1..);
        test_work.publications[1].isbn = None;
        let output = generate_test_output(true, &test_work);
        // Work type changed
        assert!(!output.contains(r#"  <book book_type="edited_book">"#));
        assert!(output.contains(r#"  <book book_type="reference">"#));
        // Only omitted contributors supplied: `contributors` element omitted
        assert!(!output.contains(r#"      <contributors>"#));
        assert!(
            !output.contains(r#"        <person_name contributor_role="author" sequence="first">"#)
                && !output.contains(
                    r#"        <person_name sequence="first" contributor_role="author">"#
                )
        );
        assert!(!output.contains(r#"          <given_name>Sole</given_name>"#));
        assert!(!output.contains(r#"          <surname>Author</surname>"#));
        assert!(
            !output.contains(r#"          <ORCID>https://orcid.org/0000-0002-0000-0001</ORCID>"#)
        );
        assert!(
            !output.contains(
                r#"        <person_name contributor_role="editor" sequence="additional">"#
            ) && !output.contains(
                r#"        <person_name sequence="additional" contributor_role="editor">"#
            )
        );
        assert!(!output.contains(r#"          <given_name>Only</given_name>"#));
        assert!(!output.contains(r#"          <surname>Editor</surname>"#));
        assert!(
            !output.contains(r#"          <ORCID>https://orcid.org/0000-0002-0000-0002</ORCID>"#)
        );
        // XML ISBN removed
        assert!(!output.contains(r#"      <isbn media_type="electronic">978-92-95055-02-5</isbn>"#));
        // No landing page: entire `doi_data` element omitted
        assert!(!output.contains(r#"      <doi_data>"#));
        assert!(!output.contains(r#"        <doi>10.00001/BOOK.0001</doi>"#));
        assert!(!output.contains(r#"        <resource>https://www.book.com</resource>"#));

        // Change work type again, replace landing page but remove DOI
        test_work.work_type = WorkType::JOURNAL_ISSUE;
        test_work.landing_page = Some("https://www.book.com".to_string());
        test_work.doi = None;
        let output = generate_test_output(true, &test_work);
        // Work type changed
        assert!(!output.contains(r#"  <book book_type="reference">"#));
        assert!(output.contains(r#"  <book book_type="other">"#));
        // No DOI: entire `doi_data` element omitted (even though landing page restored)
        assert!(!output.contains(r#"      <doi_data>"#));
        assert!(!output.contains(r#"        <doi>10.00001/BOOK.0001</doi>"#));
        assert!(!output.contains(r#"        <resource>https://www.book.com</resource>"#));

        // Remove publication date. Result: error
        test_work.publication_date = None;
        let output = generate_test_output(false, &test_work);
        assert_eq!(
            output,
            "Could not generate doideposit::crossref: Missing Publication Date".to_string()
        );

        // Restore publication date and remove all publication ISBNs. Result: error
        test_work.publication_date = chrono::NaiveDate::from_ymd_opt(1999, 12, 31);
        test_work.publications[0].isbn = None;
        let output = generate_test_output(false, &test_work);
        assert_eq!(
            output,
            "Could not generate doideposit::crossref: This work does not have any ISBNs"
                .to_string()
        );
    }

    #[test]
    // Test that no more than 6 ISBNs are ever output.
    // Remove/change this test once the CrossRef 6-ISBN limit is removed/increased -
    // at this point, we need to remove the workaround and ensure that all ISBNs are included.
    fn test_doideposit_crossref_isbns_workaround() {
        let mut test_work = Work {
            work_id: Uuid::from_str("00000000-0000-0000-AAAA-000000000001").unwrap(),
            work_status: WorkStatus::ACTIVE,
            full_title: "Book Title: Book Subtitle".to_string(),
            title: "Book Title".to_string(),
            subtitle: Some("Book Subtitle".to_string()),
            work_type: WorkType::MONOGRAPH,
            reference: None,
            edition: Some(100),
            doi: Some(Doi::from_str("https://doi.org/10.00001/BOOK.0001").unwrap()),
            publication_date: chrono::NaiveDate::from_ymd_opt(1999, 12, 31),
            license: Some("https://creativecommons.org/licenses/by/4.0/".to_string()),
            copyright_holder: Some("Author 1; Author 2".to_string()),
            short_abstract: None,
            long_abstract: Some("Lorem ipsum dolor sit amet".to_string()),
            general_note: None,
            bibliography_note: None,
            place: Some("León, Spain".to_string()),
            page_count: None,
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
            cover_url: None,
            cover_caption: None,
            imprint: WorkImprint {
                imprint_name: "OA Editions Imprint".to_string(),
                imprint_url: None,
                publisher: WorkImprintPublisher {
                    publisher_name: "OA Editions".to_string(),
                    publisher_shortname: Some("OAE".to_string()),
                    publisher_url: None,
                },
            },
            issues: vec![],
            contributions: vec![],
            languages: vec![],
            publications: vec![
                WorkPublications {
                    publication_id: Uuid::from_str("00000000-0000-0000-AAAA-000000000001").unwrap(),
                    publication_type: PublicationType::HTML,
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
                    locations: vec![],
                },
                WorkPublications {
                    publication_id: Uuid::from_str("00000000-0000-0000-BBBB-000000000002").unwrap(),
                    publication_type: PublicationType::PAPERBACK,
                    isbn: Some(Isbn::from_str("978-1-78839-908-1").unwrap()),
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
                    prices: vec![],
                    locations: vec![],
                },
                WorkPublications {
                    publication_id: Uuid::from_str("00000000-0000-0000-CCCC-000000000003").unwrap(),
                    publication_type: PublicationType::HARDBACK,
                    isbn: Some(Isbn::from_str("978-1-7343145-0-2").unwrap()),
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
                    isbn: Some(Isbn::from_str("978-0-07-063546-3").unwrap()),
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
                    publication_id: Uuid::from_str("00000000-0000-0000-EEEE-000000000005").unwrap(),
                    publication_type: PublicationType::EPUB,
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
                    locations: vec![],
                },
                WorkPublications {
                    publication_id: Uuid::from_str("00000000-0000-0000-FFFF-000000000006").unwrap(),
                    publication_type: PublicationType::XML,
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
                    locations: vec![],
                },
                WorkPublications {
                    publication_id: Uuid::from_str("00000000-0000-0000-AAAB-000000000007").unwrap(),
                    publication_type: PublicationType::DOCX,
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
            ],
            subjects: vec![],
            fundings: vec![],
            relations: vec![],
            references: vec![],
        };

        // 7 ISBNs are present and one is HTML - confirm that it is omitted
        let output = generate_test_output(true, &test_work);
        assert!(output.contains(r#"      <isbn media_type="print">978-1-78839-908-1</isbn>"#));
        assert!(output.contains(r#"      <isbn media_type="print">978-1-7343145-0-2</isbn>"#));
        assert!(output.contains(r#"      <isbn media_type="electronic">978-0-07-063546-3</isbn>"#));
        assert!(output.contains(r#"      <isbn media_type="electronic">978-1-56619-909-4</isbn>"#));
        assert!(output.contains(r#"      <isbn media_type="electronic">978-92-95055-02-5</isbn>"#));
        assert!(output.contains(r#"      <isbn media_type="electronic">978-1-4028-9462-6</isbn>"#));
        assert!(!output.contains(r#"      <isbn media_type="electronic">978-3-16-148410-0</isbn>"#));

        // Change the HTML publication to a different format
        test_work.publications[0].publication_type = PublicationType::MOBI;
        // 7 ISBNs are present and none are HTML - confirm that the last one is omitted
        let output = generate_test_output(true, &test_work);
        assert!(output.contains(r#"      <isbn media_type="electronic">978-3-16-148410-0</isbn>"#));
        assert!(output.contains(r#"      <isbn media_type="print">978-1-78839-908-1</isbn>"#));
        assert!(output.contains(r#"      <isbn media_type="print">978-1-7343145-0-2</isbn>"#));
        assert!(output.contains(r#"      <isbn media_type="electronic">978-0-07-063546-3</isbn>"#));
        assert!(output.contains(r#"      <isbn media_type="electronic">978-1-56619-909-4</isbn>"#));
        assert!(output.contains(r#"      <isbn media_type="electronic">978-92-95055-02-5</isbn>"#));
        assert!(!output.contains(r#"      <isbn media_type="electronic">978-1-4028-9462-6</isbn>"#));
    }
}
