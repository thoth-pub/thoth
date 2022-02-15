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
                "kbart::oclc".to_string(),
                "Not enough data".to_string(),
            )),
            1 => CsvRow::<KbartOclc>::csv_row(works.first().unwrap(), w),
            _ => {
                for work in works.iter() {
                    // Do not include Chapters in full publisher metadata record
                    // (assumes that a publisher will always have more than one work)
                    if !(work.work_type == WorkType::BOOK_CHAPTER) {
                        CsvRow::<KbartOclc>::csv_row(work, w).ok();
                    }
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
                    online_identifier = publication.isbn.as_ref().map(|i| i.to_string());
                }
                if publication.publication_type == PublicationType::PAPERBACK {
                    print_edition_exists = true;
                    if publication.isbn.is_some() {
                        print_identifier = publication.isbn.as_ref().map(|i| i.to_string());
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
                title_id: work.doi.map(|d| d.to_string()),
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
                monograph_edition: work.edition,
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
    use std::fmt;
    use std::str::FromStr;
    use thoth_api::model::Doi;
    use thoth_api::model::Isbn;
    use thoth_api::model::Orcid;
    use thoth_client::{
        ContributionType, PublicationType, WorkContributions, WorkContributionsContributor,
        WorkImprint, WorkImprintPublisher, WorkIssues, WorkIssuesSeries, WorkPublications,
        WorkStatus, WorkType,
    };
    use uuid::Uuid;

    struct TestResult {
        headers: String,
        title: String,
        print_identifier: String,
        online_identifier: String,
        title_url: String,
        first_author: String,
        title_id: String,
        publisher_name: String,
        publication_type: String,
        date_monograph_published_print: String,
        date_monograph_published_online: String,
        monograph_volume: String,
        monograph_edition: String,
        first_editor: String,
        parent_publication_title_id: String,
    }

    impl fmt::Display for TestResult {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            writeln!(f,
                "{}{}\t{}\t{}\t\t\t\t\t\t\t{}\t{}\t{}\t\tfulltext\t\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t\tF",
                self.headers,
                self.title,
                self.print_identifier,
                self.online_identifier,
                self.title_url,
                self.first_author,
                self.title_id,
                self.publisher_name,
                self.publication_type,
                self.date_monograph_published_print,
                self.date_monograph_published_online,
                self.monograph_volume,
                self.monograph_edition,
                self.first_editor,
                self.parent_publication_title_id,
            )
        }
    }

    #[test]
    fn test_kbart_oclc() {
        let mut test_work: Work = Work {
            work_id: Uuid::from_str("00000000-0000-0000-AAAA-000000000001").unwrap(),
            work_status: WorkStatus::ACTIVE,
            full_title: "Book Title: Book Subtitle".to_string(),
            title: "Book Title".to_string(),
            subtitle: Some("Separate Subtitle".to_string()),
            work_type: WorkType::MONOGRAPH,
            edition: Some(1),
            doi: Some(Doi::from_str("https://doi.org/10.00001/BOOK.0001").unwrap()),
            publication_date: Some(chrono::NaiveDate::from_ymd(1999, 12, 31)),
            license: Some("http://creativecommons.org/licenses/by/4.0/".to_string()),
            copyright_holder: "Author 1; Author 2".to_string(),
            short_abstract: Some("Lorem ipsum dolor sit amet".to_string()),
            long_abstract: Some(
                "Lorem ipsum dolor sit amet, consectetur adipiscing elit".to_string(),
            ),
            general_note: None,
            place: Some("León, Spain".to_string()),
            width_mm: Some(156.0),
            width_cm: Some(15.6),
            width_in: Some(6.14),
            height_mm: Some(234.0),
            height_cm: Some(23.4),
            height_in: Some(9.21),
            page_count: Some(334),
            page_breakdown: Some("x+334".to_string()),
            first_page: None,
            last_page: None,
            page_interval: None,
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
                    publisher_url: None,
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
                        series_description: None,
                        series_cfp_url: None,
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
                        series_description: None,
                        series_cfp_url: None,
                    },
                },
            ],
            contributions: vec![
                WorkContributions {
                    contribution_type: ContributionType::AUTHOR,
                    first_name: Some("Author".to_string()),
                    last_name: "First".to_string(),
                    full_name: "Author First".to_string(),
                    main_contribution: true,
                    biography: None,
                    contribution_ordinal: 1,
                    contributor: WorkContributionsContributor {
                        orcid: Some(
                            Orcid::from_str("https://orcid.org/0000-0002-0000-0001").unwrap(),
                        ),
                    },
                    affiliations: vec![],
                },
                WorkContributions {
                    contribution_type: ContributionType::AUTHOR,
                    first_name: Some("Author".to_string()),
                    last_name: "Second".to_string(),
                    full_name: "Author Second".to_string(),
                    main_contribution: true,
                    biography: None,
                    contribution_ordinal: 2,
                    contributor: WorkContributionsContributor { orcid: None },
                    affiliations: vec![],
                },
                WorkContributions {
                    contribution_type: ContributionType::EDITOR,
                    first_name: Some("Editor".to_string()),
                    last_name: "FirstEd".to_string(),
                    full_name: "Editor FirstEd".to_string(),
                    main_contribution: true,
                    biography: None,
                    contribution_ordinal: 3,
                    contributor: WorkContributionsContributor { orcid: None },
                    affiliations: vec![],
                },
            ],
            languages: vec![],
            publications: vec![
                WorkPublications {
                    publication_id: Uuid::from_str("00000000-0000-0000-DDDD-000000000004").unwrap(),
                    publication_type: PublicationType::PDF,
                    isbn: Some(Isbn::from_str("978-1-56619-909-4").unwrap()),
                    weight_g: None,
                    weight_oz: None,
                    prices: vec![],
                    locations: vec![],
                },
                WorkPublications {
                    publication_id: Uuid::from_str("00000000-0000-0000-CCCC-000000000003").unwrap(),
                    publication_type: PublicationType::HARDBACK,
                    isbn: Some(Isbn::from_str("978-1-4028-9462-6").unwrap()),
                    weight_g: None,
                    weight_oz: None,
                    prices: vec![],
                    locations: vec![],
                },
                WorkPublications {
                    publication_id: Uuid::from_str("00000000-0000-0000-BBBB-000000000002").unwrap(),
                    publication_type: PublicationType::PAPERBACK,
                    isbn: Some(Isbn::from_str("978-3-16-148410-0").unwrap()),
                    weight_g: None,
                    weight_oz: None,
                    prices: vec![],
                    locations: vec![],
                },
                WorkPublications {
                    publication_id: Uuid::from_str("00000000-0000-0000-EEEE-000000000005").unwrap(),
                    publication_type: PublicationType::HTML,
                    isbn: None,
                    weight_g: None,
                    weight_oz: None,
                    prices: vec![],
                    locations: vec![],
                },
                WorkPublications {
                    publication_id: Uuid::from_str("00000000-0000-0000-FFFF-000000000006").unwrap(),
                    publication_type: PublicationType::XML,
                    isbn: Some(Isbn::from_str("978-92-95055-02-5").unwrap()),
                    weight_g: None,
                    weight_oz: None,
                    prices: vec![],
                    locations: vec![],
                },
            ],
            subjects: vec![],
            fundings: vec![],
            relations: vec![],
        };
        let mut test_result = TestResult {
            headers: "publication_title\tprint_identifier\tonline_identifier\tdate_first_issue_online\tnum_first_vol_online\tnum_first_issue_online\tdate_last_issue_online\tnum_last_vol_online\tnum_last_issue_online\ttitle_url\tfirst_author\ttitle_id\tembargo_info\tcoverage_depth\tnotes\tpublisher_name\tpublication_type\tdate_monograph_published_print\tdate_monograph_published_online\tmonograph_volume\tmonograph_edition\tfirst_editor\tparent_publication_title_id\tpreceding_publication_title_id\taccess_type\n".to_string(),
            title: "Book Title: Separate Subtitle".to_string(),
            print_identifier: "978-3-16-148410-0".to_string(),
            online_identifier: "978-1-56619-909-4".to_string(),
            title_url: "https://www.book.com".to_string(),
            first_author: "First".to_string(),
            title_id: "10.00001/BOOK.0001".to_string(),
            publisher_name: "OA Editions".to_string(),
            publication_type: "Monograph".to_string(),
            date_monograph_published_print: "1999".to_string(),
            date_monograph_published_online: "1999".to_string(),
            monograph_volume: "20".to_string(),
            monograph_edition: "1".to_string(),
            first_editor: "".to_string(),
            parent_publication_title_id: "8765-4321".to_string(),
        };
        let to_test =
            KbartOclc.generate(&[test_work.clone()], QuoteStyle::Necessary, DELIMITER_TAB);
        assert_eq!(to_test, Ok(test_result.to_string()));

        // Remove subtitle: full title is used instead of title + subtitle
        test_work.subtitle = None;
        test_result.title = "Book Title: Book Subtitle".to_string();
        // Remove DOI: no title_id
        test_work.doi = None;
        test_result.title_id = "".to_string();
        // Remove paperback publication: date_monograph_published_print (for hardback)
        // still appears, but no print_identifier (paperback ISBN) is present
        test_work.publications.remove(2);
        test_result.print_identifier = "".to_string();
        // Make first-numbered author a non-main contributor:
        // second-numbered author appears as first_author instead
        test_work.contributions[0].main_contribution = false;
        test_result.first_author = "Second".to_string();
        // Make work a book set: publication_type becomes Serial
        test_work.work_type = WorkType::BOOK_SET;
        test_result.publication_type = "Serial".to_string();
        let to_test =
            KbartOclc.generate(&[test_work.clone()], QuoteStyle::Necessary, DELIMITER_TAB);
        assert_eq!(to_test, Ok(test_result.to_string()));

        // Remove hardback publication: no date_monograph_published_print
        test_work.publications.remove(1);
        test_result.date_monograph_published_print = "".to_string();
        // Remove PDF publication's ISBN: no online_identifier
        test_work.publications[0].isbn = None;
        test_result.online_identifier = "".to_string();
        // Make work an edited book: first_author becomes empty,
        // first_editor lists first-numbered editor,
        // publication_type reverts to Monograph
        test_work.work_type = WorkType::EDITED_BOOK;
        test_result.first_author = "".to_string();
        test_result.first_editor = "FirstEd".to_string();
        test_result.publication_type = "Monograph".to_string();
        let to_test =
            KbartOclc.generate(&[test_work.clone()], QuoteStyle::Necessary, DELIMITER_TAB);
        assert_eq!(to_test, Ok(test_result.to_string()));

        // Remove landing page: KBART fails to generate
        test_work.landing_page = None;
        let to_test =
            KbartOclc.generate(&[test_work.clone()], QuoteStyle::Necessary, DELIMITER_TAB);
        assert_eq!(
            to_test,
            Err(ThothError::IncompleteMetadataRecord(
                "kbart::oclc".to_string(),
                "Missing Landing Page".to_string(),
            ))
        );

        // Reinstate landing page but remove publication date: ditto
        test_work.landing_page = Some("https://www.book.com".to_string());
        test_work.publication_date = None;
        let to_test =
            KbartOclc.generate(&[test_work.clone()], QuoteStyle::Necessary, DELIMITER_TAB);
        assert_eq!(
            to_test,
            Err(ThothError::IncompleteMetadataRecord(
                "kbart::oclc".to_string(),
                "Missing Publication Date".to_string(),
            ))
        );
    }
}
