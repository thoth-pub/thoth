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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::DELIMITER_TAB;
    use csv::QuoteStyle;
    use lazy_static::lazy_static;
    use std::str::FromStr;
    use thoth_client::{
        ContributionType, PublicationType, WorkContributions, WorkContributionsContributor,
        WorkImprint, WorkImprintPublisher, WorkIssues, WorkIssuesSeries, WorkPublications,
        WorkStatus, WorkType,
    };
    use uuid::Uuid;

    lazy_static! {
        static ref TEST_WORK: Work = Work {
            work_id: Uuid::from_str("00000000-0000-0000-AAAA-000000000001").unwrap(),
            work_status: WorkStatus::ACTIVE,
            full_title: "Book Title: Book Subtitle".to_string(),
            title: "Book Title".to_string(),
            subtitle: Some("Separate Subtitle".to_string()),
            work_type: WorkType::MONOGRAPH,
            edition: 1,
            doi: Some("https://doi.org/10.00001/BOOK.0001".to_string()),
            publication_date: Some(chrono::NaiveDate::from_ymd(1999, 12, 31)),
            license: Some("http://creativecommons.org/licenses/by/4.0/".to_string()),
            copyright_holder: "Author 1; Author 2".to_string(),
            short_abstract: Some("Lorem ipsum dolor sit amet".to_string()),
            long_abstract: Some(
                "Lorem ipsum dolor sit amet, consectetur adipiscing elit".to_string()
            ),
            general_note: None,
            place: Some("León, Spain".to_string()),
            width: Some(156.0),
            height: Some(234.0),
            page_count: Some(334),
            page_breakdown: Some("x+334".to_string()),
            image_count: Some(15),
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
                publisher: WorkImprintPublisher {
                    publisher_name: "OA Editions".to_string(),
                },
            },
            issues: vec![
                WorkIssues {
                    issue_ordinal: 20,
                    series: WorkIssuesSeries {
                        series_type: thoth_client::SeriesType::BOOK_SERIES,
                        series_name: "Name of series".to_string(),
                        issn_print: "1234-5678".to_string(),
                        issn_digital: "8765-4321".to_string(),
                        series_url: None,
                    },
                },
                WorkIssues {
                    issue_ordinal: 50,
                    series: WorkIssuesSeries {
                        series_type: thoth_client::SeriesType::BOOK_SERIES,
                        series_name: "Name of second series".to_string(),
                        issn_print: "1111-2222".to_string(),
                        issn_digital: "3333-4444".to_string(),
                        series_url: None,
                    },
                }
            ],
            contributions: vec![
                WorkContributions {
                    contribution_type: ContributionType::AUTHOR,
                    first_name: Some("Author".to_string()),
                    last_name: "First".to_string(),
                    full_name: "Author First".to_string(),
                    main_contribution: true,
                    biography: None,
                    institution: None,
                    contribution_ordinal: 1,
                    contributor: WorkContributionsContributor {
                        orcid: Some("https://orcid.org/0000-0000-0000-0001".to_string()),
                    },
                },
                WorkContributions {
                    contribution_type: ContributionType::AUTHOR,
                    first_name: Some("Author".to_string()),
                    last_name: "Second".to_string(),
                    full_name: "Author Second".to_string(),
                    main_contribution: true,
                    biography: None,
                    institution: None,
                    contribution_ordinal: 2,
                    contributor: WorkContributionsContributor { orcid: None },
                },
            ],
            languages: vec![],
            publications: vec![
                WorkPublications {
                    publication_id: Uuid::from_str("00000000-0000-0000-BBBB-000000000002").unwrap(),
                    publication_type: PublicationType::PAPERBACK,
                    publication_url: Some("https://www.book.com/paperback".to_string()),
                    isbn: Some("978-1-00000-000-0".to_string()),
                    prices: vec![],
                },
                WorkPublications {
                    publication_id: Uuid::from_str("00000000-0000-0000-CCCC-000000000003").unwrap(),
                    publication_type: PublicationType::HARDBACK,
                    publication_url: Some("https://www.book.com/hardback".to_string()),
                    isbn: Some("978-1-00000-000-1".to_string()),
                    prices: vec![],
                },
                WorkPublications {
                    publication_id: Uuid::from_str("00000000-0000-0000-DDDD-000000000004").unwrap(),
                    publication_type: PublicationType::PDF,
                    publication_url: Some("https://www.book.com/pdf".to_string()),
                    isbn: Some("978-1-00000-000-2".to_string()),
                    prices: vec![],
                },
                WorkPublications {
                    publication_id: Uuid::from_str("00000000-0000-0000-EEEE-000000000005").unwrap(),
                    publication_type: PublicationType::HTML,
                    publication_url: Some("https://www.book.com/html".to_string()),
                    isbn: None,
                    prices: vec![],
                },
                WorkPublications {
                    publication_id: Uuid::from_str("00000000-0000-0000-FFFF-000000000006").unwrap(),
                    publication_type: PublicationType::XML,
                    publication_url: Some("https://www.book.com/xml".to_string()),
                    isbn: Some("978-1-00000-000-3".to_string()),
                    prices: vec![],
                },
            ],
            subjects: vec![],
            fundings: vec![],
        };
    }

    const TEST_RESULT: &str = "publication_title\tprint_identifier\tonline_identifier\tdate_first_issue_online\tnum_first_vol_online\tnum_first_issue_online\tdate_last_issue_online\tnum_last_vol_online\tnum_last_issue_online\ttitle_url\tfirst_author\ttitle_id\tembargo_info\tcoverage_depth\tnotes\tpublisher_name\tpublication_type\tdate_monograph_published_print\tdate_monograph_published_online\tmonograph_volume\tmonograph_edition\tfirst_editor\tparent_publication_title_id\tpreceding_publication_title_id\taccess_type\nBook Title: Separate Subtitle\t978-1-00000-000-0\t978-1-00000-000-2\t\t\t\t\t\t\thttps://www.book.com\tFirst\thttps://doi.org/10.00001/BOOK.0001\t\tfulltext\t\tOA Editions\tMonograph\t1999\t1999\t20\t1\t\t8765-4321\t\tF\n";

    #[test]
    fn test_kbart_oclc() {
        let to_test =
            KbartOclc.generate(&[TEST_WORK.clone()], QuoteStyle::Necessary, DELIMITER_TAB);

        assert_eq!(to_test, Ok(TEST_RESULT.to_string()))
    }
}
