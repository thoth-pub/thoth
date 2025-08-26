use std::convert::TryFrom;
use std::fmt;
use std::io::Write;
use thoth_client::{
    AbstractType, ContributionType, PublicationType, RelationType, Work, WorkContributions,
    WorkType,
};
use thoth_errors::{ThothError, ThothResult};

use super::{BibtexEntry, BibtexSpecification};

#[derive(Copy, Clone)]
pub(crate) struct BibtexThoth;

const BIBTEX_ERROR: &str = "bibtex::thoth";

#[derive(Debug)]
struct BibtexThothEntry {
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

impl BibtexSpecification for BibtexThoth {
    fn handle_event(w: &mut Vec<u8>, works: &[Work]) -> ThothResult<()> {
        match works {
            [] => Err(ThothError::IncompleteMetadataRecord(
                BIBTEX_ERROR.to_string(),
                "Not enough data".to_string(),
            )),
            [work] => BibtexEntry::<BibtexThoth>::bibtex_entry(work, w),
            _ => {
                for work in works.iter() {
                    // Do not include Chapters in full publisher metadata record
                    // (assumes that a publisher will always have more than one work)
                    if work.work_type != WorkType::BOOK_CHAPTER {
                        BibtexEntry::<BibtexThoth>::bibtex_entry(work, w).ok();
                    }
                }
                Ok(())
            }
        }
    }
}

impl BibtexEntry<BibtexThoth> for Work {
    fn bibtex_entry(&self, w: &mut Vec<u8>) -> ThothResult<()> {
        w.write_all(
            BibtexThothEntry::try_from(self.clone())?
                .to_string()
                .as_bytes(),
        )?;
        Ok(())
    }
}

impl fmt::Display for BibtexThothEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Cite key must be unique and alphanumeric ("-_:" also permitted)
        // Most records will have an ISBN, but fall back on publication date if not found
        let citekey = self
            .isbn
            .clone()
            .unwrap_or_else(|| format!("{}-{}-{}", self.year, self.month, self.day));
        write!(f, "@{}{{{}", self.entry_type, citekey)?;

        write_field!(f, self, title);
        write_optional_field!(f, self, shorttitle);
        write_optional_field!(f, self, author);
        write_optional_field!(f, self, editor);
        write_field!(f, self, year, i64);
        write_field!(f, self, month, i64);
        write_field!(f, self, day, i64);
        write_field!(f, self, publisher);
        write_optional_field!(f, self, address);
        write_optional_field!(f, self, series);
        write_optional_field!(f, self, volume, i64);
        write_optional_field!(f, self, booktitle);
        write_optional_field!(f, self, chapter, i64);
        write_optional_field!(f, self, pages);
        write_optional_field!(f, self, doi);
        write_optional_field!(f, self, isbn);
        write_optional_field!(f, self, issn);
        write_optional_field!(f, self, url);
        write_optional_field!(f, self, copyright);
        write_optional_field!(f, self, long_abstract, "abstract");

        writeln!(f, "\n}}")
    }
}

impl TryFrom<Work> for BibtexThothEntry {
    type Error = ThothError;

    fn try_from(work: Work) -> ThothResult<Self> {
        // Publication year is mandatory for books/chapters in BibTeX
        if work.publication_date.is_none() {
            return Err(ThothError::IncompleteMetadataRecord(
                BIBTEX_ERROR.to_string(),
                "Missing Publication Date".to_string(),
            ));
        }

        let mut contributions = work.contributions;
        // WorkQuery should already have retrieved these sorted by ordinal, but sort again for safety
        contributions.sort_by(|a, b| a.contribution_ordinal.cmp(&b.contribution_ordinal));
        let (author, editor) = extract_authors_and_editors(contributions)?;

        let shorttitle = work.titles[0]
            .subtitle
            .as_ref()
            .map(|_| work.titles[0].title.clone());
        let (entry_type, booktitle, chapter, pages) = match work.work_type {
            WorkType::BOOK_CHAPTER => {
                let (booktitle, chapter, pages) = work
                    .relations
                    .iter()
                    .filter_map(|r| {
                        if r.relation_type == RelationType::IS_CHILD_OF {
                            Some((
                                r.related_work.titles[0].full_title.clone(),
                                r.relation_ordinal,
                                // BibTeX page ranges require a double dash between the page numbers
                                work.page_interval.as_ref().map(|p| p.replace('–', "--")),
                            ))
                        } else {
                            None
                        }
                    })
                    .next()
                    .unwrap_or_default();
                ("inbook".to_string(), Some(booktitle), Some(chapter), pages)
            }
            // None of the standard BibTeX entry types are suitable for Book Sets
            WorkType::BOOK_SET => ("misc".to_string(), None, None, None),
            _ => ("book".to_string(), None, None, None),
        };

        Ok(BibtexThothEntry {
            entry_type,
            title: work.titles[0].full_title.clone(),
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
                .and_then(|i| i.series.issn_digital.as_ref().map(|s| s.to_string())),
            url: work.landing_page,
            copyright: work.license,
            long_abstract: work
                .abstracts
                .iter()
                .find(|a| a.abstract_type == AbstractType::LONG)
                .map(|a| a.content.clone()),
        })
    }
}

/// Returns a list of authors and a list of editors concatenated by " and "
///
/// BibTeX book/chapter records must contain either author or editor
fn extract_authors_and_editors(
    contributions: Vec<WorkContributions>,
) -> ThothResult<(Option<String>, Option<String>)> {
    let (authors, editors): (Vec<String>, Vec<String>) = contributions.into_iter().fold(
        (Vec::new(), Vec::new()),
        |(mut authors, mut editors), contribution| {
            if contribution.main_contribution {
                match contribution.contribution_type {
                    ContributionType::AUTHOR => authors.push(contribution.full_name),
                    ContributionType::EDITOR => editors.push(contribution.full_name),
                    _ => (),
                }
            }
            (authors, editors)
        },
    );

    if authors.is_empty() && editors.is_empty() {
        return Err(ThothError::IncompleteMetadataRecord(
            BIBTEX_ERROR.to_string(),
            "Missing Author/Editor Details".to_string(),
        ));
    }

    let format = |v: Vec<String>| {
        if !v.is_empty() {
            Some(v.join(" and "))
        } else {
            None
        }
    };
    Ok((format(authors), format(editors)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use thoth_api::model::Doi;
    use thoth_api::model::Isbn;
    use thoth_api::model::Orcid;
    use thoth_client::{
        ContributionType, PublicationType, SeriesType, WorkContributions,
        WorkContributionsContributor, WorkImprint, WorkImprintPublisher, WorkIssues,
        WorkIssuesSeries, WorkPublications, WorkRelations, WorkRelationsRelatedWork,
        WorkRelationsRelatedWorkImprint, WorkRelationsRelatedWorkImprintPublisher, WorkStatus,
        WorkType,
    };
    use uuid::Uuid;

    const TEST_RESULT: &str = "@book{978-1-56619-909-4,
\ttitle = {Work Title: Work Subtitle},
\tshorttitle = {Work Title},
\tauthor = {Author 1 and Author 2 and Author 3},
\teditor = {Editor 1 and Editor 2},
\tyear = 1999,
\tmonth = 12,
\tday = 31,
\tpublisher = {OA Editions},
\taddress = {León, Spain},
\tseries = {Name of series},
\tvolume = 5,
\tdoi = {10.00001/BOOK.0001},
\tisbn = {978-1-56619-909-4},
\tissn = {8765-4321},
\turl = {https://www.book.com},
\tcopyright = {http://creativecommons.org/licenses/by/4.0/},
\tabstract = {<p>Lorem ipsum dolor sit amet, consectetur adipiscing elit. Vestibulum vel libero eleifend, ultrices purus vitae, suscipit ligula. Aliquam ornare quam et nulla vestibulum, id euismod tellus malesuada. Orci varius natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. Nullam ornare bibendum ex nec dapibus. Proin porta risus elementum odio feugiat tempus. Etiam eu felis ac metus viverra ornare. In consectetur neque sed feugiat ornare. Mauris at purus fringilla orci tincidunt pulvinar sed a massa. Nullam vestibulum posuere augue, sit amet tincidunt nisl pulvinar ac.</p>}
}
";

    fn test_work() -> Work {
        Work {
            abstracts: vec![
                thoth_client::WorkAbstracts {
                    abstract_id: Uuid::from_str("00000000-0000-0000-AAAA-000000000001").unwrap(),
                    work_id: Uuid::from_str("00000000-0000-0000-AAAA-000000000001").unwrap(),
                    content: "<p>Lorem ipsum dolor sit amet, consectetur adipiscing elit. Vestibulum vel libero eleifend, ultrices purus vitae, suscipit ligula. Aliquam ornare quam et nulla vestibulum, id euismod tellus malesuada. Orci varius natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. Nullam ornare bibendum ex nec dapibus. Proin porta risus elementum odio feugiat tempus. Etiam eu felis ac metus viverra ornare. In consectetur neque sed feugiat ornare. Mauris at purus fringilla orci tincidunt pulvinar sed a massa. Nullam vestibulum posuere augue, sit amet tincidunt nisl pulvinar ac.</p>".to_string(),
                    locale_code: thoth_client::LocaleCode::EN,
                    abstract_type: thoth_client::AbstractType::LONG,
                    canonical: true,
                },
            ],
            work_id: Uuid::from_str("00000000-0000-0000-AAAA-000000000001").unwrap(),
            work_status: WorkStatus::ACTIVE,
            titles: vec![thoth_client::WorkTitles {
                title_id: Uuid::from_str("00000000-0000-0000-CCCC-000000000001").unwrap(),
                locale_code: thoth_client::LocaleCode::EN,
                full_title: "Work Title: Work Subtitle".to_string(),
                title: "Work Title".to_string(),
                subtitle: Some("Work Subtitle".to_string()),
                canonical: true,
            }],
            work_type: WorkType::MONOGRAPH,
            reference: None,
            edition: Some(1),
            doi: Some(Doi::from_str("https://doi.org/10.00001/BOOK.0001").unwrap()),
            publication_date: chrono::NaiveDate::from_ymd_opt(1999, 12, 31),
            withdrawn_date: None,
            license: Some("http://creativecommons.org/licenses/by/4.0/".to_string()),
            copyright_holder: Some("Author 1; Author 2".to_string()),
            general_note: Some("This is a general note".to_string()),
            bibliography_note: None,
            place: Some("León, Spain".to_string()),
            page_count: Some(334),
            page_breakdown: Some("x+334".to_string()),
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
                imprint_url: None,
                crossmark_doi: None,
                publisher: WorkImprintPublisher {
                    publisher_name: "OA Editions".to_string(),
                    publisher_shortname: Some("OAE".to_string()),
                    publisher_url: None,
                },
            },
            issues: vec![WorkIssues {
                issue_ordinal: 5,
                series: WorkIssuesSeries {
                    series_id: Uuid::parse_str("00000000-0000-0000-BBBB-000000000002").unwrap(),
                    series_type: SeriesType::JOURNAL,
                    series_name: "Name of series".to_string(),
                    issn_print: Some("1234-5678".to_string()),
                    issn_digital: Some("8765-4321".to_string()),
                    series_url: Some("https://www.series.com".to_string()),
                    series_description: Some("Description of series".to_string()),
                    series_cfp_url: Some("https://www.series.com/cfp".to_string()),
                },
            }],
            contributions: vec![
                WorkContributions {
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
                },
                WorkContributions {
                    contribution_type: ContributionType::AUTHOR,
                    first_name: Some("Author".to_string()),
                    last_name: "2".to_string(),
                    full_name: "Author 2".to_string(),
                    main_contribution: true,
                    biography: None,
                    contribution_ordinal: 2,
                    contributor: WorkContributionsContributor {
                        orcid: None,
                        website: None,
                    },
                    affiliations: vec![],
                },
                WorkContributions {
                    contribution_type: ContributionType::AUTHOR,
                    first_name: Some("Author".to_string()),
                    last_name: "3".to_string(),
                    full_name: "Author 3".to_string(),
                    main_contribution: true,
                    biography: None,
                    contribution_ordinal: 3,
                    contributor: WorkContributionsContributor {
                        orcid: None,
                        website: None,
                    },
                    affiliations: vec![],
                },
                WorkContributions {
                    contribution_type: ContributionType::EDITOR,
                    first_name: Some("Editor".to_string()),
                    last_name: "1".to_string(),
                    full_name: "Editor 1".to_string(),
                    main_contribution: true,
                    biography: None,
                    contribution_ordinal: 4,
                    contributor: WorkContributionsContributor {
                        orcid: None,
                        website: None,
                    },
                    affiliations: vec![],
                },
                WorkContributions {
                    contribution_type: ContributionType::EDITOR,
                    first_name: Some("Editor".to_string()),
                    last_name: "2".to_string(),
                    full_name: "Editor 2".to_string(),
                    main_contribution: true,
                    biography: None,
                    contribution_ordinal: 5,
                    contributor: WorkContributionsContributor {
                        orcid: None,
                        website: None,
                    },
                    affiliations: vec![],
                },
                WorkContributions {
                    contribution_type: ContributionType::EDITOR,
                    first_name: Some("Editor".to_string()),
                    last_name: "3".to_string(),
                    full_name: "Editor 3".to_string(),
                    main_contribution: false,
                    biography: None,
                    contribution_ordinal: 6,
                    contributor: WorkContributionsContributor {
                        orcid: None,
                        website: None,
                    },
                    affiliations: vec![],
                },
                WorkContributions {
                    contribution_type: ContributionType::TRANSLATOR,
                    first_name: Some("Translator".to_string()),
                    last_name: "1".to_string(),
                    full_name: "Translator 1".to_string(),
                    main_contribution: true,
                    biography: None,
                    contribution_ordinal: 7,
                    contributor: WorkContributionsContributor {
                        orcid: None,
                        website: None,
                    },
                    affiliations: vec![],
                },
            ],
            languages: vec![],
            publications: vec![
                WorkPublications {
                    publication_id: Uuid::from_str("00000000-0000-0000-BBBB-000000000002").unwrap(),
                    publication_type: PublicationType::PAPERBACK,
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
                    publication_id: Uuid::from_str("00000000-0000-0000-DDDD-000000000004").unwrap(),
                    publication_type: PublicationType::PDF,
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
            ],
            subjects: vec![],
            fundings: vec![],
            relations: vec![WorkRelations {
                relation_type: RelationType::IS_CHILD_OF,
                relation_ordinal: 7,
                related_work: WorkRelationsRelatedWork {
                    abstracts: vec![
                        thoth_client::WorkRelationsRelatedWorkAbstracts {
                            abstract_id: Uuid::from_str("00000000-0000-0000-AAAA-000000000001").unwrap(),
                            work_id: Uuid::from_str("00000000-0000-0000-AAAA-000000000001").unwrap(),
                            content: "<p>Lorem ipsum dolor sit amet, consectetur adipiscing elit. Vestibulum vel libero eleifend, ultrices purus vitae, suscipit ligula. Aliquam ornare quam et nulla vestibulum, id euismod tellus malesuada. Orci varius natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. Nullam ornare bibendum ex nec dapibus. Proin porta risus elementum odio feugiat tempus. Etiam eu felis ac metus viverra ornare. In consectetur neque sed feugiat ornare. Mauris at purus fringilla orci tincidunt pulvinar sed a massa. Nullam vestibulum posuere augue, sit amet tincidunt nisl pulvinar ac.</p>".to_string(),
                            locale_code: thoth_client::LocaleCode::EN,
                            abstract_type: thoth_client::AbstractType::LONG,
                            canonical: true,
                        },
                    ],
                    work_status: WorkStatus::ACTIVE,
                    titles: vec![thoth_client::WorkRelationsRelatedWorkTitles {
                        title_id: Uuid::from_str("00000000-0000-0000-CCCC-000000000001").unwrap(),
                        locale_code: thoth_client::LocaleCode::EN,
                        full_title: "Related work title".to_string(),
                        title: "N/A".to_string(),
                        subtitle: None,
                        canonical: true,
                    }],
                    edition: None,
                    doi: None,
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
                relation_ordinal: 4,
                related_work: WorkRelationsRelatedWork {
                    work_status: WorkStatus::ACTIVE,
                    titles: vec![thoth_client::WorkRelationsRelatedWorkTitles {
                        title_id: Uuid::from_str("00000000-0000-0000-CCCC-000000000001").unwrap(),
                        locale_code: thoth_client::LocaleCode::EN,
                        full_title: "Irrelevant related work".to_string(),
                        title: "N/A".to_string(),
                        subtitle: None,
                        canonical: true,
                    }],
                    abstracts: vec![
                        thoth_client::WorkRelationsRelatedWorkAbstracts {
                            abstract_id: Uuid::from_str("00000000-0000-0000-AAAA-000000000001").unwrap(),
                            work_id: Uuid::from_str("00000000-0000-0000-AAAA-000000000001").unwrap(),
                            content: "<p>Lorem ipsum dolor sit amet, consectetur adipiscing elit. Vestibulum vel libero eleifend, ultrices purus vitae, suscipit ligula. Aliquam ornare quam et nulla vestibulum, id euismod tellus malesuada. Orci varius natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. Nullam ornare bibendum ex nec dapibus. Proin porta risus elementum odio feugiat tempus. Etiam eu felis ac metus viverra ornare. In consectetur neque sed feugiat ornare. Mauris at purus fringilla orci tincidunt pulvinar sed a massa. Nullam vestibulum posuere augue, sit amet tincidunt nisl pulvinar ac.</p>".to_string(),
                            locale_code: thoth_client::LocaleCode::EN,
                            abstract_type: thoth_client::AbstractType::LONG,
                            canonical: true,
                        },
                    ],
                    edition: None,
                    doi: None,
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
            }],
            references: vec![]
        }
    }

    #[test]
    fn test_generate_record() {
        let to_test = BibtexThoth.generate(&[test_work().clone()]);
        assert_eq!(to_test, Ok(TEST_RESULT.to_string()));
    }

    #[test]
    fn test_generate_record_book_set() {
        let mut test_work = test_work();
        // Change work type to Book Set: entry type becomes "misc"
        test_work.work_type = WorkType::BOOK_SET;
        let to_test = BibtexThoth.generate(&[test_work.clone()]);
        assert_eq!(
            to_test,
            Ok(TEST_RESULT.to_string().replace("@book", "@misc"))
        );
    }

    #[test]
    fn test_generate_record_edited_book() {
        let mut test_work = test_work();
        // Change work type to Edited Book: should have no effect
        test_work.work_type = WorkType::EDITED_BOOK;
        let to_test = BibtexThoth.generate(&[test_work.clone()]);
        assert_eq!(to_test, Ok(TEST_RESULT.to_string()));
    }

    #[test]
    fn test_bibtex_thoth_textbook() {
        let mut test_work = test_work();
        // Change work type to textbook: should have no effect
        test_work.work_type = WorkType::TEXTBOOK;
        let to_test = BibtexThoth.generate(&[test_work.clone()]);
        assert_eq!(to_test, Ok(TEST_RESULT.to_string()));
    }

    #[test]
    fn test_publication_date_as_cite_key() {
        let mut test_work = test_work();
        // Remove PDF ISBN field: isbn is removed, cite key becomes publication date
        test_work.publications[1].isbn = None;
        let to_test = BibtexThoth.generate(&[test_work.clone()]);
        assert_eq!(
            to_test,
            Ok(TEST_RESULT
                .to_string()
                .replace("@book{978-1-56619-909-4,", "@book{1999-12-31,")
                .replace("\tisbn = {978-1-56619-909-4},\n", ""))
        );
    }

    #[test]
    fn test_work_without_subtitle() {
        let mut test_work = test_work();
        // Remove subtitle field: shorttitle is removed (as it would duplicate title)
        test_work.titles[0].subtitle = None;
        // We need to manually update the   full title to remove the subtitle
        // in this test framework, but within the Thoth database this is automatic
        test_work.titles[0].full_title = "Work Title".to_string();
        let to_test = BibtexThoth.generate(&[test_work.clone()]);
        assert_eq!(
            to_test,
            Ok(TEST_RESULT.to_string().replace(
                "\ttitle = {Work Title: Work Subtitle},\n\tshorttitle = {Work Title},",
                "\ttitle = {Work Title},"
            ))
        );
    }

    #[test]
    fn test_bibtex_thoth_chapter() {
        let mut test_work = test_work();
        test_work.titles[0].subtitle = None;
        test_work.titles[0].full_title = "Work Title".to_string();
        test_work.publications[1].isbn = None;
        test_work.place = None;
        test_work.doi = None;
        test_work.landing_page = None;
        test_work.license = None;
        test_work.issues.clear();
        // Change work type to Chapter and add chapter-specific details (page range):
        // entry type becomes "inbook", booktitle/chapter/pages fields will be added
        test_work.work_type = WorkType::BOOK_CHAPTER;
        // We need to manually set the page range in this test framework, but within
        // the Thoth database this is automatically derived from first + last page
        test_work.page_interval = Some("10–20".to_string());
        let to_test = BibtexThoth.generate(&[test_work.clone()]);
        let test_result = "@inbook{1999-12-31,
\ttitle = {Work Title},
\tauthor = {Author 1 and Author 2 and Author 3},
\teditor = {Editor 1 and Editor 2},
\tyear = 1999,
\tmonth = 12,
\tday = 31,
\tpublisher = {OA Editions},
\tbooktitle = {Related work title},
\tchapter = 7,
\tpages = {10--20},
\tabstract = {<p>Lorem ipsum dolor sit amet, consectetur adipiscing elit. Vestibulum vel libero eleifend, ultrices purus vitae, suscipit ligula. Aliquam ornare quam et nulla vestibulum, id euismod tellus malesuada. Orci varius natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. Nullam ornare bibendum ex nec dapibus. Proin porta risus elementum odio feugiat tempus. Etiam eu felis ac metus viverra ornare. In consectetur neque sed feugiat ornare. Mauris at purus fringilla orci tincidunt pulvinar sed a massa. Nullam vestibulum posuere augue, sit amet tincidunt nisl pulvinar ac.</p>}
}
"
        .to_string();
        assert_eq!(to_test, Ok(test_result));
    }

    #[test]
    fn test_missing_publication_date_error() {
        let mut test_work = test_work();
        test_work.publication_date = None;
        let to_test = BibtexThoth.generate(&[test_work.clone()]);
        assert_eq!(
            to_test.unwrap_err(),
            ThothError::IncompleteMetadataRecord(
                BIBTEX_ERROR.to_string(),
                "Missing Publication Date".to_string(),
            )
        );
    }

    #[test]
    fn test_missing_author_details_error() {
        let mut test_work = test_work();
        test_work.contributions.clear();
        let to_test = BibtexThoth.generate(&[test_work.clone()]);
        assert_eq!(
            to_test.unwrap_err(),
            ThothError::IncompleteMetadataRecord(
                BIBTEX_ERROR.to_string(),
                "Missing Author/Editor Details".to_string()
            )
        );
    }

    #[test]
    fn test_extract_authors_and_editors_both_present() {
        let test_work = test_work();
        let to_test = extract_authors_and_editors(test_work.contributions);
        assert_eq!(
            to_test.unwrap(),
            (
                Some("Author 1 and Author 2 and Author 3".to_string()),
                Some("Editor 1 and Editor 2".to_string())
            )
        );
    }

    #[test]
    fn test_extract_authors_and_editors_only_authors() {
        let test_work = test_work();
        let to_test = extract_authors_and_editors(vec![
            test_work.contributions[0].clone(),
            test_work.contributions[1].clone(),
        ]);
        assert_eq!(
            to_test.unwrap(),
            (Some("Author 1 and Author 2".to_string()), None)
        );
    }

    #[test]
    fn test_extract_authors_and_editors_only_editors() {
        let test_work = test_work();
        let to_test = extract_authors_and_editors(vec![
            test_work.contributions[3].clone(),
            test_work.contributions[4].clone(),
        ]);
        assert_eq!(
            to_test.unwrap(),
            (None, Some("Editor 1 and Editor 2".to_string()))
        );
    }

    #[test]
    fn test_extract_authors_and_editors_missing_details() {
        let test_work = test_work();
        let to_test = extract_authors_and_editors(vec![test_work.contributions[5].clone()]);
        assert_eq!(
            to_test.unwrap_err(),
            ThothError::IncompleteMetadataRecord(
                BIBTEX_ERROR.to_string(),
                "Missing Author/Editor Details".to_string()
            )
        );
    }
}
