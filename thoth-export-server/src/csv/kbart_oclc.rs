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
    publication_type: String,
    date_monograph_published_print: Option<i64>,
    date_monograph_published_online: i64,
    monograph_volume: Option<i64>,
    monograph_edition: Option<i64>,
    first_editor: Option<String>,
    parent_publication_title_id: Option<String>,
    preceding_publication_title_id: Option<String>,
    access_type: String,
}

impl CsvSpecification for KbartOclc {
    fn handle_event<W: Write>(w: &mut Writer<W>, works: &[Work]) -> ThothResult<()> {
        match works.len() {
            0 => Err(ThothError::IncompleteMetadataRecord(
                "onix_3.0::project_muse".to_string(),
                "Not enough data".to_string(),
            )),
            1 => CsvRow::<KbartOclc>::csv_row(works.first().unwrap(), w),
            _ => {
                for work in works.iter() {
                    CsvRow::<KbartOclc>::csv_row(work, w).ok();
                }
                Ok(())
            }
        }
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
        if work.landing_page.is_none() {
            Err(ThothError::IncompleteMetadataRecord(
                "kbart::oclc".to_string(),
                "Missing Landing Page".to_string(),
            ))
        // Don't output works with no publication date (mandatory in KBART)
        } else if work.publication_date.is_none() {
            Err(ThothError::IncompleteMetadataRecord(
                "kbart::oclc".to_string(),
                "Missing Publication Date".to_string(),
            ))
        } else {
            let mut print_identifier = None;
            let mut online_identifier = None;
            let mut print_edition_exists = false;
            for publication in work.publications {
                if publication.publication_type == PublicationType::PDF
                    && publication.isbn.is_some()
                {
                    online_identifier = publication.isbn.clone();
                }
                if publication.publication_type == PublicationType::PAPERBACK {
                    print_edition_exists = true;
                    if publication.isbn.is_some() {
                        print_identifier = publication.isbn.clone();
                    }
                }
                if publication.publication_type == PublicationType::HARDBACK {
                    print_edition_exists = true;
                }
            }
            let mut first_author = None;
            let mut first_editor = None;
            let mut contributions = work.contributions;
            // The first author/editor will usually be the contributor with contribution_ordinal 1,
            // but this is not guaranteed, so we select the highest-ranked contributor of the
            // appropriate contribution type who is listed as a "main" contributor.
            contributions.sort_by(|a, b| a.contribution_ordinal.cmp(&b.contribution_ordinal));
            for contribution in contributions {
                if contribution.main_contribution {
                    if work.work_type == WorkType::EDITED_BOOK {
                        if contribution.contribution_type == ContributionType::EDITOR {
                            first_editor = Some(contribution.last_name);
                            break;
                        }
                    } else if contribution.contribution_type == ContributionType::AUTHOR {
                        first_author = Some(contribution.last_name);
                        break;
                    }
                }
            }
            let date_monograph_published_online = work
                .publication_date
                .map(|date| chrono::Datelike::year(&date).into())
                .unwrap();
            let date_monograph_published_print = match print_edition_exists {
                true => Some(date_monograph_published_online),
                false => None,
            };
            Ok(KbartOclcRow {
                publication_title: match work.subtitle {
                    Some(subtitle) => format!("{}: {}", work.title, subtitle),
                    None => work.full_title,
                },
                print_identifier,
                online_identifier,
                date_first_issue_online: None,
                num_first_vol_online: None,
                num_first_issue_online: None,
                date_last_issue_online: None,
                num_last_vol_online: None,
                num_last_issue_online: None,
                title_url: work.landing_page.unwrap(),
                first_author,
                title_id: work.doi,
                embargo_info: None,
                coverage_depth: "fulltext".to_string(),
                notes: None,
                publisher_name: Some(work.imprint.publisher.publisher_name),
                publication_type: match work.work_type {
                    WorkType::BOOK_SET => "Serial".to_string(),
                    _ => "Monograph".to_string(),
                },
                date_monograph_published_print,
                date_monograph_published_online,
                // Note that it is possible for a work to belong to more than one series.
                // Only one series can be listed in KBART, so we select the first one found (if any).
                monograph_volume: work.issues.first().map(|i| i.issue_ordinal),
                monograph_edition: Some(work.edition),
                first_editor,
                // This should match the series' `title_id` if also provided in the KBART.
                parent_publication_title_id: work
                    .issues
                    .first()
                    .map(|i| i.series.issn_digital.to_string()),
                preceding_publication_title_id: None,
                access_type: "F".to_string(),
            })
        }
    }
}
