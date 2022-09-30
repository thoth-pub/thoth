use std::convert::TryFrom;
use std::fmt;
use std::io::Write;
use thoth_client::{ContributionType, PublicationType, RelationType, Work, WorkType};
use thoth_errors::{ThothError, ThothResult};

use super::{BibtexEntry, BibtexSpecification};

#[derive(Copy, Clone)]
pub(crate) struct BibtexThoth;

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
        match works.len() {
            0 => Err(ThothError::IncompleteMetadataRecord(
                "bibtex::thoth".to_string(),
                "Not enough data".to_string(),
            )),
            1 => BibtexEntry::<BibtexThoth>::bibtex_entry(works.first().unwrap(), w),
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
        let mut citekey = self.isbn.clone().unwrap_or_default();
        if citekey.is_empty() {
            citekey = format!("{}-{}-{}", self.year, self.month, self.day);
        }
        writeln!(f, "@{}{{{},", self.entry_type, citekey)?;
        write!(f, "\ttitle\t\t= {{{}}}", self.title)?;
        if let Some(shorttitle) = &self.shorttitle {
            write!(f, ",\n\tshorttitle\t= {{{}}}", shorttitle)?;
        }
        if let Some(author) = &self.author {
            write!(f, ",\n\tauthor\t\t= {{{}}}", author)?;
        }
        if let Some(editor) = &self.editor {
            write!(f, ",\n\teditor\t\t= {{{}}}", editor)?;
        }
        write!(f, ",\n\tyear\t\t= {}", self.year)?;
        write!(f, ",\n\tmonth\t\t= {}", self.month)?;
        write!(f, ",\n\tday\t\t\t= {}", self.day)?;
        write!(f, ",\n\tpublisher\t= {{{}}}", self.publisher)?;
        if let Some(address) = &self.address {
            write!(f, ",\n\taddress\t\t= {{{}}}", address)?;
        }
        if let Some(series) = &self.series {
            write!(f, ",\n\tseries\t\t= {{{}}}", series)?;
        }
        if let Some(volume) = &self.volume {
            write!(f, ",\n\tvolume\t\t= {}", volume)?;
        }
        if let Some(booktitle) = &self.booktitle {
            write!(f, ",\n\tbooktitle\t= {{{}}}", booktitle)?;
        }
        if let Some(chapter) = &self.chapter {
            write!(f, ",\n\tchapter\t\t= {}", chapter)?;
        }
        if let Some(pages) = &self.pages {
            write!(f, ",\n\tpages\t\t= {{{}}}", pages)?;
        }
        if let Some(doi) = &self.doi {
            write!(f, ",\n\tdoi\t\t\t= {{{}}}", doi)?;
        }
        if let Some(isbn) = &self.isbn {
            write!(f, ",\n\tisbn\t\t= {{{}}}", isbn)?;
        }
        if let Some(issn) = &self.issn {
            write!(f, ",\n\tissn\t\t= {{{}}}", issn)?;
        }
        if let Some(url) = &self.url {
            write!(f, ",\n\turl\t\t\t= {{{}}}", url)?;
        }
        if let Some(copyright) = &self.copyright {
            write!(f, ",\n\tcopyright\t= {{{}}}", copyright)?;
        }
        if let Some(long_abstract) = &self.long_abstract {
            write!(f, ",\n\tabstract\t= {{{}}}", long_abstract)?;
        }
        writeln!(f, "\n}}")
    }
}

impl TryFrom<Work> for BibtexThothEntry {
    type Error = ThothError;

    fn try_from(work: Work) -> ThothResult<Self> {
        // Publication year is mandatory for books/chapters in BibTeX
        if work.publication_date.is_none() {
            return Err(ThothError::IncompleteMetadataRecord(
                "bibtex::thoth".to_string(),
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
                "bibtex::thoth".to_string(),
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
            Ok(BibtexThothEntry {
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
\ttitle\t\t= {Work Title: Work Subtitle},
\tshorttitle\t= {Work Title},
\tauthor\t\t= {Author 1 and Author 2 and Author 3},
\tyear\t\t= 1999,
\tmonth\t\t= 12,
\tday\t\t\t= 31,
\tpublisher\t= {OA Editions},
\taddress\t\t= {León, Spain},
\tseries\t\t= {Name of series},
\tvolume\t\t= 5,
\tdoi\t\t\t= {10.00001/BOOK.0001},
\tisbn\t\t= {978-1-56619-909-4},
\tissn\t\t= {8765-4321},
\turl\t\t\t= {https://www.book.com},
\tcopyright\t= {http://creativecommons.org/licenses/by/4.0/},
\tabstract\t= {Lorem ipsum dolor sit amet, consectetur adipiscing elit. Vestibulum vel libero eleifend, ultrices purus vitae, suscipit ligula. Aliquam ornare quam et nulla vestibulum, id euismod tellus malesuada. Orci varius natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. Nullam ornare bibendum ex nec dapibus. Proin porta risus elementum odio feugiat tempus. Etiam eu felis ac metus viverra ornare. In consectetur neque sed feugiat ornare. Mauris at purus fringilla orci tincidunt pulvinar sed a massa. Nullam vestibulum posuere augue, sit amet tincidunt nisl pulvinar ac.}
}
";

    #[test]
    fn test_bibtex_thoth() {
        let mut test_work: Work = Work {
            work_id: Uuid::from_str("00000000-0000-0000-AAAA-000000000001").unwrap(),
            work_status: WorkStatus::ACTIVE,
            full_title: "Work Title: Work Subtitle".to_string(),
            title: "Work Title".to_string(),
            subtitle: Some("Work Subtitle".to_string()),
            work_type: WorkType::MONOGRAPH,
            edition: Some(1),
            doi: Some(Doi::from_str("https://doi.org/10.00001/BOOK.0001").unwrap()),
            publication_date: Some(chrono::NaiveDate::from_ymd(1999, 12, 31)),
            license: Some("http://creativecommons.org/licenses/by/4.0/".to_string()),
            copyright_holder: Some("Author 1; Author 2".to_string()),
            short_abstract: Some("Lorem ipsum dolor sit amet, consectetur adipiscing elit. Vestibulum vel libero eleifend, ultrices purus vitae, suscipit ligula. Aliquam ornare quam et nulla vestibulum, id euismod tellus malesuada. Orci varius natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus.".to_string()),
            long_abstract: Some("Lorem ipsum dolor sit amet, consectetur adipiscing elit. Vestibulum vel libero eleifend, ultrices purus vitae, suscipit ligula. Aliquam ornare quam et nulla vestibulum, id euismod tellus malesuada. Orci varius natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. Nullam ornare bibendum ex nec dapibus. Proin porta risus elementum odio feugiat tempus. Etiam eu felis ac metus viverra ornare. In consectetur neque sed feugiat ornare. Mauris at purus fringilla orci tincidunt pulvinar sed a massa. Nullam vestibulum posuere augue, sit amet tincidunt nisl pulvinar ac.".to_string()),
            general_note: Some("This is a general note".to_string()),
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
                publisher: WorkImprintPublisher {
                    publisher_name: "OA Editions".to_string(),
                    publisher_url: None,
                },
            },
            issues: vec![WorkIssues {
                issue_ordinal: 5,
                series: WorkIssuesSeries {
                    series_type: SeriesType::JOURNAL,
                    series_name: "Name of series".to_string(),
                    issn_print: "1234-5678".to_string(),
                    issn_digital: "8765-4321".to_string(),
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
                    },
                    affiliations: vec![],
                },
                WorkContributions {
                    contribution_type: ContributionType::EDITOR,
                    first_name: Some("Editor".to_string()),
                    last_name: "1".to_string(),
                    full_name: "Editor 2".to_string(),
                    main_contribution: false,
                    biography: None,
                    contribution_ordinal: 5,
                    contributor: WorkContributionsContributor {
                        orcid: None,
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
                    full_title: "Related work title".to_string(),
                    title: "N/A".to_string(),
                    subtitle: None,
                    edition: None,
                    doi: None,
                    publication_date: None,
                    license: None,
                    place: None,
                    first_page: None,
                    last_page: None,
                    landing_page: None,
                    imprint: WorkRelationsRelatedWorkImprint {
                        publisher: WorkRelationsRelatedWorkImprintPublisher {
                            publisher_name: "N/A".to_string(),
                        },
                    },
                    contributions: vec![],
                    publications: vec![],
                },
            },
            WorkRelations {
                relation_type: RelationType::HAS_TRANSLATION,
                relation_ordinal: 4,
                related_work: WorkRelationsRelatedWork {
                    full_title: "Irrelevant related work".to_string(),
                    title: "N/A".to_string(),
                    subtitle: None,
                    edition: None,
                    doi: None,
                    publication_date: None,
                    license: None,
                    place: None,
                    first_page: None,
                    last_page: None,
                    landing_page: None,
                    imprint: WorkRelationsRelatedWorkImprint {
                        publisher: WorkRelationsRelatedWorkImprintPublisher {
                            publisher_name: "N/A".to_string(),
                        },
                    },
                    contributions: vec![],
                    publications: vec![],
                },
            }]
        };

        let to_test = BibtexThoth.generate(&[test_work.clone()]);
        assert_eq!(to_test, Ok(TEST_RESULT.to_string()));

        // Change work type to Book Set: entry type becomes "misc"
        test_work.work_type = WorkType::BOOK_SET;
        let to_test = BibtexThoth.generate(&[test_work.clone()]);
        assert_eq!(
            to_test,
            Ok(TEST_RESULT.to_string().replace("@book", "@misc"))
        );

        // Change work type to Edited Book: author field replaced by editor field
        test_work.work_type = WorkType::EDITED_BOOK;
        let to_test = BibtexThoth.generate(&[test_work.clone()]);
        assert_eq!(
            to_test,
            Ok(TEST_RESULT.to_string().replace(
                "\tauthor\t\t= {Author 1 and Author 2 and Author 3},",
                "\teditor\t\t= {Editor 1},"
            ))
        );

        test_work.work_type = WorkType::MONOGRAPH;
        // Remove PDF ISBN field: isbn is removed, cite key becomes publication date
        test_work.publications[1].isbn = None;
        // Remove subtitle field: shorttitle is removed (as it would duplicate title)
        test_work.subtitle = None;
        // We need to manually update the full title to remove the subtitle
        // in this test framework, but within the Thoth database this is automatic
        test_work.full_title = "Work Title".to_string();
        let to_test = BibtexThoth.generate(&[test_work.clone()]);
        assert_eq!(
            to_test,
            Ok(TEST_RESULT
                .to_string()
                .replace("@book{978-1-56619-909-4,", "@book{1999-12-31,")
                .replace("\tisbn\t\t= {978-1-56619-909-4},\n", "")
                .replace(
                    "\ttitle\t\t= {Work Title: Work Subtitle},\n\tshorttitle\t= {Work Title},",
                    "\ttitle\t\t= {Work Title},"
                ))
        );

        // Remove all other optional fields: corresponding fields will be removed
        test_work.place = None;
        test_work.doi = None;
        test_work.landing_page = None;
        test_work.license = None;
        test_work.long_abstract = None;
        test_work.issues.clear();
        // Change work type to Chapter and add chapter-specific details (page range):
        // entry type becomes "inbook", booktitle/chapter/pages fields will be added
        test_work.work_type = WorkType::BOOK_CHAPTER;
        // We need to manually set the page range in this test framework, but within
        // the Thoth database this is automatically derived from first + last page
        test_work.page_interval = Some("10-20".to_string());
        let to_test = BibtexThoth.generate(&[test_work.clone()]);
        let test_result = "@inbook{1999-12-31,
\ttitle\t\t= {Work Title},
\tauthor\t\t= {Author 1 and Author 2 and Author 3},
\tyear\t\t= 1999,
\tmonth\t\t= 12,
\tday\t\t\t= 31,
\tpublisher\t= {OA Editions},
\tbooktitle\t= {Related work title},
\tchapter\t\t= 7,
\tpages\t\t= {10--20}
}
"
        .to_string();
        assert_eq!(to_test, Ok(test_result));

        // Remove publication date: BibTeX fails to generate
        test_work.publication_date = None;
        let to_test = BibtexThoth.generate(&[test_work.clone()]);
        assert_eq!(
            to_test,
            Err(ThothError::IncompleteMetadataRecord(
                "bibtex::thoth".to_string(),
                "Missing Publication Date".to_string(),
            ))
        );

        // Reinstate publication date but remove author/editor details: ditto
        test_work.publication_date = Some(chrono::NaiveDate::from_ymd(1999, 12, 31));
        test_work.contributions.clear();
        let to_test = BibtexThoth.generate(&[test_work.clone()]);
        assert_eq!(
            to_test,
            Err(ThothError::IncompleteMetadataRecord(
                "bibtex::thoth".to_string(),
                "Missing Author/Editor Details".to_string(),
            ))
        );
    }
}
