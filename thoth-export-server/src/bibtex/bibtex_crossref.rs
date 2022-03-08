use serde::Serialize;
use std::convert::TryFrom;
use std::io::Write;
use thoth_client::{ContributionType, PublicationType, Work, WorkType};
use thoth_errors::{ThothError, ThothResult};

use super::{BibtexEntry, BibtexSpecification, BibtexWriter};

pub(crate) struct BibtexCrossref;

#[derive(Debug, Serialize)]
struct BibtexCrossrefEntry {
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
    pages: Option<String>,
    doi: Option<String>,
    isbn: Option<String>,
    issn: Option<String>,
    url: Option<String>,
    copyright: Option<String>,
    long_abstract: Option<String>,
}

impl BibtexSpecification for BibtexCrossref {
    fn handle_event<W: Write>(w: &mut BibtexWriter<W>, works: &[Work]) -> ThothResult<()> {
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
    fn bibtex_entry<W: Write>(&self, w: &mut BibtexWriter<W>) -> ThothResult<()> {
        write!(w, "{:?}", BibtexCrossrefEntry::try_from(self.clone())?);
        Ok(())
    }
}

impl TryFrom<Work> for BibtexCrossrefEntry {
    type Error = ThothError;

    fn try_from(work: Work) -> ThothResult<Self> {
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
        // BibTeX book records must contain either author or editor
        if author_list.is_empty() && editor_list.is_empty() {
            Err(ThothError::IncompleteMetadataRecord(
                "bibtex::crossref".to_string(),
                "Missing Author/Editor Details".to_string(),
            ))
        // Publication year is mandatory for books in BibTeX
        } else if work.publication_date.is_none() {
            Err(ThothError::IncompleteMetadataRecord(
                "bibtex::crossref".to_string(),
                "Missing Publication Date".to_string(),
            ))
        } else {
            let mut isbn = None;
            for publication in work.publications {
                if publication.publication_type == PublicationType::PDF
                    && publication.isbn.is_some()
                {
                    isbn = publication.isbn.as_ref().map(|i| i.to_string());
                }
            }
            let author = match author_list.is_empty() {
                true => None,
                false => Some(author_list.join(" and ")),
            };
            let editor = match editor_list.is_empty() {
                true => None,
                false => Some(editor_list.join(" and ")),
            };
            let mut title = work.full_title;
            let mut shorttitle = None;
            if let Some(subtitle) = work.subtitle {
                title = format!("{}: {}", work.title, subtitle);
                shorttitle = Some(work.title);
            }
            Ok(BibtexCrossrefEntry {
                title,
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
                pages: work.page_interval,
                doi: work.doi.map(|d| d.to_string()),
                isbn,
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
