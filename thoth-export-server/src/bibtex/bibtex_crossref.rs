use std::convert::TryFrom;
use std::fmt;
use std::io::Write;
use thoth_client::{ContributionType, PublicationType, RelationType, Work, WorkType};
use thoth_errors::{ThothError, ThothResult};

use super::{BibtexEntry, BibtexSpecification};

pub(crate) struct BibtexCrossref;

#[derive(Debug)]
struct BibtexCrossrefEntry {
    entry_type: String,
    title: String,
    shorttitle: Option<String>,
    author: Option<String>,
    editor: Option<String>,
    year: i64,
    month: i64,
    day: i64,
    publisher: String,
    address: Option<String>,
    series: Option<String>,
    volume: Option<i64>,
    booktitle: Option<String>,
    chapter: Option<i64>,
    pages: Option<String>,
    doi: Option<String>,
    isbn: Option<String>,
    issn: Option<String>,
    url: Option<String>,
    copyright: Option<String>,
    // BibTeX field name is "abstract" but this is a reserved Rust keyword
    long_abstract: Option<String>,
}

impl BibtexSpecification for BibtexCrossref {
    fn handle_event(w: &mut Vec<u8>, works: &[Work]) -> ThothResult<()> {
        match works.len() {
            0 => Err(ThothError::IncompleteMetadataRecord(
                "kbart::oclc".to_string(),
                "Not enough data".to_string(),
            )),
            1 => BibtexEntry::<BibtexCrossref>::bibtex_entry(works.first().unwrap(), w),
            _ => {
                for work in works.iter() {
                    // Do not include Chapters in full publisher metadata record
                    // (assumes that a publisher will always have more than one work)
                    if work.work_type != WorkType::BOOK_CHAPTER {
                        BibtexEntry::<BibtexCrossref>::bibtex_entry(work, w).ok();
                    }
                }
                Ok(())
            }
        }
    }
}

impl BibtexEntry<BibtexCrossref> for Work {
    fn bibtex_entry(&self, w: &mut Vec<u8>) -> ThothResult<()> {
        w.write_all(
            BibtexCrossrefEntry::try_from(self.clone())?
                .to_string()
                .as_bytes(),
        )?;
        Ok(())
    }
}

impl fmt::Display for BibtexCrossrefEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Cite key must be unique and alphanumeric ("-_:" also permitted)
        // Most records will have an ISBN, but fall back on publication date if not found
        let mut citekey = self.isbn.clone().unwrap_or_default();
        if citekey.is_empty() {
            citekey = format!("{}-{}-{}", self.year, self.month, self.day);
        }
        writeln!(f, "@{}{{{},", self.entry_type, citekey)?;
        writeln!(f, "\ttitle\t\t= {{{}}},", self.title)?;
        if let Some(shorttitle) = &self.shorttitle {
            writeln!(f, "\tshorttitle\t= {{{shorttitle}}},")?;
        }
        if let Some(author) = &self.author {
            writeln!(f, "\tauthor\t\t= {{{author}}},")?;
        }
        if let Some(editor) = &self.editor {
            writeln!(f, "\teditor\t\t= {{{editor}}},")?;
        }
        writeln!(f, "\tyear\t\t= {},", self.year)?;
        writeln!(f, "\tmonth\t\t= {},", self.month)?;
        writeln!(f, "\tday\t\t\t= {},", self.day)?;
        writeln!(f, "\tpublisher\t= {{{}}},", self.publisher)?;
        if let Some(address) = &self.address {
            writeln!(f, "\taddress\t\t= {{{address}}},")?;
        }
        if let Some(series) = &self.series {
            writeln!(f, "\tseries\t\t= {{{series}}},")?;
        }
        if let Some(volume) = &self.volume {
            writeln!(f, "\tvolume\t\t= {volume},")?;
        }
        if let Some(booktitle) = &self.booktitle {
            writeln!(f, "\tbooktitle\t= {{{booktitle}}},")?;
        }
        if let Some(chapter) = &self.chapter {
            writeln!(f, "\tchapter\t\t= {chapter},")?;
        }
        if let Some(pages) = &self.pages {
            writeln!(f, "\tpages\t\t= {{{pages}}},")?;
        }
        if let Some(doi) = &self.doi {
            writeln!(f, "\tdoi\t\t\t= {{{doi}}},")?;
        }
        if let Some(isbn) = &self.isbn {
            writeln!(f, "\tisbn\t\t= {{{isbn}}},")?;
        }
        if let Some(issn) = &self.issn {
            writeln!(f, "\tissn\t\t= {{{issn}}},")?;
        }
        if let Some(url) = &self.url {
            writeln!(f, "\turl\t\t\t= {{{url}}},")?;
        }
        if let Some(copyright) = &self.copyright {
            writeln!(f, "\tcopyright\t= {{{copyright}}},")?;
        }
        if let Some(long_abstract) = &self.long_abstract {
            writeln!(f, "\tabstract\t= {{{long_abstract}}}")?;
        }
        writeln!(f, "}}")
    }
}

impl TryFrom<Work> for BibtexCrossrefEntry {
    type Error = ThothError;

    fn try_from(work: Work) -> ThothResult<Self> {
        // Publication year is mandatory for books/chapters in BibTeX
        if work.publication_date.is_none() {
            return Err(ThothError::IncompleteMetadataRecord(
                "bibtex::crossref".to_string(),
                "Missing Publication Date".to_string(),
            ));
        }
        let mut author_list = vec![];
        let mut editor_list = vec![];
        let mut contributions = work.contributions;
        contributions.sort_by(|a, b| a.contribution_ordinal.cmp(&b.contribution_ordinal));
        for contribution in contributions {
            if contribution.main_contribution {
                if work.work_type == WorkType::EDITED_BOOK {
                    if contribution.contribution_type == ContributionType::EDITOR {
                        editor_list.push(contribution.full_name);
                    }
                } else if contribution.contribution_type == ContributionType::AUTHOR {
                    author_list.push(contribution.full_name);
                }
            }
        }
        // BibTeX book/chapter records must contain either author or editor
        if author_list.is_empty() && editor_list.is_empty() {
            Err(ThothError::IncompleteMetadataRecord(
                "bibtex::crossref".to_string(),
                "Missing Author/Editor Details".to_string(),
            ))
        } else {
            let author = match author_list.is_empty() {
                true => None,
                false => Some(author_list.join(" and ")),
            };
            let editor = match editor_list.is_empty() {
                true => None,
                false => Some(editor_list.join(" and ")),
            };
            let mut shorttitle = None;
            if work.subtitle.is_some() {
                shorttitle = Some(work.title);
            }
            let mut booktitle = None;
            let mut chapter = None;
            let mut pages = None;
            let mut entry_type = "book".to_string();
            if work.work_type == WorkType::BOOK_CHAPTER {
                entry_type = "inbook".to_string();
                if let Some(parent_relation) = work
                    .relations
                    .iter()
                    .find(|r| r.relation_type == RelationType::IS_CHILD_OF)
                {
                    booktitle = Some(parent_relation.related_work.full_title.clone());
                    chapter = Some(parent_relation.relation_ordinal);
                }
                // BibTeX page ranges require a double dash between the page numbers
                pages = work.page_interval.map(|p| p.replace('-', "--"));
            } else if work.work_type == WorkType::BOOK_SET {
                // None of the standard BibTeX entry types are suitable for Book Sets
                entry_type = "misc".to_string();
            }
            Ok(BibtexCrossrefEntry {
                entry_type,
                title: work.full_title,
                shorttitle,
                author,
                editor,
                year: work
                    .publication_date
                    .map(|date| chrono::Datelike::year(&date).into())
                    .unwrap(),
                month: work
                    .publication_date
                    .map(|date| chrono::Datelike::month(&date).into())
                    .unwrap(),
                day: work
                    .publication_date
                    .map(|date| chrono::Datelike::day(&date).into())
                    .unwrap(),
                publisher: work.imprint.publisher.publisher_name,
                address: work.place,
                series: work
                    .issues
                    .first()
                    .map(|i| i.series.series_name.to_string()),
                volume: work.issues.first().map(|i| i.issue_ordinal),
                booktitle,
                chapter,
                pages,
                doi: work.doi.map(|d| d.to_string()),
                // Take digital ISBN/ISSN as canonical
                isbn: work
                    .publications
                    .iter()
                    .find(|p| p.publication_type.eq(&PublicationType::PDF))
                    .and_then(|p| p.isbn.as_ref().map(|i| i.to_string())),
                issn: work
                    .issues
                    .first()
                    .map(|i| i.series.issn_digital.to_string()),
                url: work.landing_page,
                copyright: work.license,
                long_abstract: work.long_abstract,
            })
        }
    }
}
