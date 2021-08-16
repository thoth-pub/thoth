use csv::Writer;
use serde::Serialize;
use std::convert::TryFrom;
use std::io::Write;
use thoth_client::{ContributionType, PublicationType, Work, WorkType};
use thoth_errors::{ThothError, ThothResult};

use super::{CsvRow, CsvSpecification};

pub(crate) struct KbartOclc;

#[derive(Debug, Serialize)]
struct KbartOclcRow {
    publication_title: String,
    print_identifier: Option<String>,
    online_identifier: Option<String>,
    date_first_issue_online: Option<i64>,
    num_first_vol_online: Option<i64>,
    num_first_issue_online: Option<i64>,
    date_last_issue_online: Option<i64>,
    num_last_vol_online: Option<i64>,
    num_last_issue_online: Option<i64>,
    title_url: String,
    first_author: Option<String>,
    title_id: Option<String>,
    embargo_info: Option<String>,
    coverage_depth: String,
    notes: Option<String>,
    publisher_name: Option<String>,
    publication_type: Option<String>,
    date_monograph_published_print: Option<i64>,
    date_monograph_published_online: Option<i64>,
    monograph_volume: Option<i64>,
    monograph_edition: Option<i64>,
    first_editor: Option<String>,
    parent_publication_title_id: Option<String>,
    preceding_publication_title_id: Option<String>,
    access_type: Option<String>,
}

impl CsvSpecification for KbartOclc {
    fn handle_event<W: Write>(w: &mut Writer<W>, works: &[Work]) -> ThothResult<()> {
        for work in works.iter() {
            CsvRow::<KbartOclc>::csv_row(work, w)?;
        }
        Ok(())
    }
}

impl CsvRow<KbartOclc> for Work {
    fn csv_row<W: Write>(&self, w: &mut Writer<W>) -> ThothResult<()> {
        w.serialize(KbartOclcRow::try_from(self.clone())?)
            .map_err(|e| e.into())
    }
}

impl TryFrom<Work> for KbartOclcRow {
    type Error = ThothError;

    fn try_from(work: Work) -> ThothResult<Self> {
        // title_url is mandatory in KBART but optional in Thoth
        if let Some(title_url) = work.landing_page {
            let publication_title = match work.subtitle {
                Some(subtitle) => format!("{}: {}", work.title, subtitle),
                None => work.full_title,
            };
            let mut print_identifier = None;
            let mut online_identifier = None;
            for publication in work.publications {
                if publication.publication_type == PublicationType::PAPERBACK {
                    print_identifier = publication.isbn.clone();
                }
                if publication.publication_type == PublicationType::PDF {
                    online_identifier = publication.isbn.clone();
                }
            }
            let mut main_authors = vec![];
            let mut main_editors = vec![];
            let mut contributions = work.contributions;
            contributions.sort_by(|a, b| a.contribution_ordinal.cmp(&b.contribution_ordinal));
            for contribution in contributions {
                if contribution.main_contribution {
                    if work.work_type == WorkType::EDITED_BOOK {
                        if contribution.contribution_type == ContributionType::EDITOR {
                            main_editors.push(contribution.last_name);
                        }
                    } else if contribution.contribution_type == ContributionType::AUTHOR {
                        main_authors.push(contribution.last_name);
                    }
                }
            }
            let first_author = match main_authors.is_empty() {
                true => None,
                false => Some(main_authors.join("; ")),
            };
            let first_editor = match main_editors.is_empty() {
                true => None,
                false => Some(main_editors.join("; ")),
            };
            let publication_type = match work.work_type {
                WorkType::BOOK_SET => Some("serial".to_string()),
                _ => Some("monograph".to_string()),
            };
            let publication_year = work
                .publication_date
                .map(|date| chrono::Datelike::year(&date).into());
            let mut monograph_volume = None;
            let mut parent_publication_title_id = None;
            if !work.issues.is_empty() {
                monograph_volume = Some(work.issues[0].issue_ordinal);
                parent_publication_title_id = Some(work.issues[0].series.series_name.clone());
            }
            Ok(KbartOclcRow {
                publication_title,
                print_identifier,
                online_identifier,
                date_first_issue_online: None,
                num_first_vol_online: None,
                num_first_issue_online: None,
                date_last_issue_online: None,
                num_last_vol_online: None,
                num_last_issue_online: None,
                title_url,
                first_author,
                title_id: work.doi,
                embargo_info: None,
                coverage_depth: "fulltext".to_string(),
                notes: None,
                publisher_name: Some(work.imprint.publisher.publisher_name),
                publication_type,
                date_monograph_published_print: publication_year,
                date_monograph_published_online: publication_year,
                monograph_volume,
                monograph_edition: Some(work.edition),
                first_editor,
                parent_publication_title_id,
                preceding_publication_title_id: None,
                access_type: Some("F".to_string()),
            })
        } else {
            Err(ThothError::IncompleteMetadataRecord(
                "kbart::oclc".to_string(),
                "Missing Title URL".to_string(),
            ))
        }
    }
}
