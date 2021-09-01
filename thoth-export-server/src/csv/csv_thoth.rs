use csv::Writer;
use serde::Serialize;
use std::io::Write;
use thoth_client::{
    SubjectType, Work, WorkContributions, WorkFundings, WorkIssues, WorkLanguages,
    WorkPublications, WorkPublicationsPrices, WorkSubjects,
};
use thoth_errors::ThothResult;

use super::{CsvCell, CsvRow, CsvSpecification};

pub(crate) struct CsvThoth;

#[derive(Debug, Serialize)]
struct CsvThothRow {
    publisher: String,
    imprint: String,
    work_type: String,
    work_status: String,
    title: String,
    subtitle: Option<String>,
    edition: i64,
    doi: Option<String>,
    publication_date: Option<String>,
    publication_place: Option<String>,
    license: Option<String>,
    copyright_holder: String,
    landing_page: Option<String>,
    #[serde(rename = "width (mm)")]
    width_mm: Option<f64>,
    #[serde(rename = "width (cm)")]
    width_cm: Option<f64>,
    #[serde(rename = "width (in)")]
    width_in: Option<f64>,
    #[serde(rename = "height (mm)")]
    height_mm: Option<f64>,
    #[serde(rename = "height (cm)")]
    height_cm: Option<f64>,
    #[serde(rename = "height (in)")]
    height_in: Option<f64>,
    page_count: Option<i64>,
    page_breakdown: Option<String>,
    image_count: Option<i64>,
    table_count: Option<i64>,
    audio_count: Option<i64>,
    video_count: Option<i64>,
    lccn: Option<String>,
    oclc: Option<String>,
    short_abstract: Option<String>,
    long_abstract: Option<String>,
    general_note: Option<String>,
    toc: Option<String>,
    cover_url: Option<String>,
    cover_caption: Option<String>,
    #[serde(
        rename = "contributions [(type, first_name, last_name, full_name, institution, orcid)]"
    )]
    contributions: String,
    #[serde(rename = "publications [(type, isbn, url, [(ISO_4217_currency, price)])]")]
    publications: String,
    #[serde(rename = "series [(type, name, issn_print, issn_digital, url, issue)]")]
    series: String,
    #[serde(rename = "languages [(relation, ISO_639-3/B_language, is_main)]")]
    languages: String,
    #[serde(rename = "BIC [code]")]
    bic: String,
    #[serde(rename = "THEMA [code]")]
    thema: String,
    #[serde(rename = "BISAC [code]")]
    bisac: String,
    #[serde(rename = "LCC [code]")]
    lcc: String,
    #[serde(rename = "custom_categories [category]")]
    custom: String,
    #[serde(rename = "keywords [keyword]")]
    keywords: String,
    #[serde(rename = "funding [(funder, funder_doi, program, project, grant, jurisdiction)]")]
    funding: String,
}

impl CsvSpecification for CsvThoth {
    fn handle_event<W: Write>(w: &mut Writer<W>, works: &[Work]) -> ThothResult<()> {
        for work in works.iter() {
            CsvRow::<CsvThoth>::csv_row(work, w)?;
        }
        Ok(())
    }
}

impl CsvRow<CsvThoth> for Work {
    fn csv_row<W: Write>(&self, w: &mut Writer<W>) -> ThothResult<()> {
        w.serialize(CsvThothRow::from(self.clone()))
            .map_err(|e| e.into())
    }
}

impl From<Work> for CsvThothRow {
    fn from(work: Work) -> Self {
        let mut subjects = work.subjects;
        subjects.sort_by(|a, b| a.subject_ordinal.cmp(&b.subject_ordinal));
        CsvThothRow {
            publisher: work.imprint.publisher.publisher_name,
            imprint: work.imprint.imprint_name,
            work_type: format!("{:?}", work.work_type),
            work_status: format!("{:?}", work.work_status),
            title: work.title,
            subtitle: work.subtitle,
            edition: work.edition,
            doi: work.doi.map(|d| d.to_string()),
            publication_date: work.publication_date.map(|d| d.to_string()),
            publication_place: work.place,
            license: work.license,
            copyright_holder: work.copyright_holder,
            landing_page: work.landing_page,
            width_mm: work.width_mm,
            width_cm: work.width_cm,
            width_in: work.width_in,
            height_mm: work.height_mm,
            height_cm: work.height_cm,
            height_in: work.height_in,
            page_count: work.page_count,
            page_breakdown: work.page_breakdown,
            image_count: work.image_count,
            table_count: work.table_count,
            audio_count: work.audio_count,
            video_count: work.video_count,
            lccn: work.lccn,
            oclc: work.oclc,
            short_abstract: work.short_abstract,
            long_abstract: work.long_abstract,
            general_note: work.general_note,
            toc: work.toc,
            cover_url: work.cover_url,
            cover_caption: work.cover_caption,
            contributions: CsvCell::<CsvThoth>::csv_cell(
                &work
                    .contributions
                    .iter()
                    .map(|c| CsvCell::<CsvThoth>::csv_cell(c))
                    .collect::<Vec<String>>(),
            ),
            publications: CsvCell::<CsvThoth>::csv_cell(
                &work
                    .publications
                    .iter()
                    .map(|p| CsvCell::<CsvThoth>::csv_cell(p))
                    .collect::<Vec<String>>(),
            ),
            series: CsvCell::<CsvThoth>::csv_cell(
                &work
                    .issues
                    .iter()
                    .map(|i| CsvCell::<CsvThoth>::csv_cell(i))
                    .collect::<Vec<String>>(),
            ),
            languages: CsvCell::<CsvThoth>::csv_cell(
                &work
                    .languages
                    .iter()
                    .map(|l| CsvCell::<CsvThoth>::csv_cell(l))
                    .collect::<Vec<String>>(),
            ),
            bic: CsvCell::<CsvThoth>::csv_cell(
                &subjects
                    .iter()
                    .filter(|s| s.subject_type.eq(&SubjectType::BIC))
                    .map(|s| CsvCell::<CsvThoth>::csv_cell(s))
                    .collect::<Vec<String>>(),
            ),
            thema: CsvCell::<CsvThoth>::csv_cell(
                &subjects
                    .iter()
                    .filter(|s| s.subject_type.eq(&SubjectType::THEMA))
                    .map(|s| CsvCell::<CsvThoth>::csv_cell(s))
                    .collect::<Vec<String>>(),
            ),
            bisac: CsvCell::<CsvThoth>::csv_cell(
                &subjects
                    .iter()
                    .filter(|s| s.subject_type.eq(&SubjectType::BISAC))
                    .map(|s| CsvCell::<CsvThoth>::csv_cell(s))
                    .collect::<Vec<String>>(),
            ),
            lcc: CsvCell::<CsvThoth>::csv_cell(
                &subjects
                    .iter()
                    .filter(|s| s.subject_type.eq(&SubjectType::LCC))
                    .map(|s| CsvCell::<CsvThoth>::csv_cell(s))
                    .collect::<Vec<String>>(),
            ),
            custom: CsvCell::<CsvThoth>::csv_cell(
                &subjects
                    .iter()
                    .filter(|s| s.subject_type.eq(&SubjectType::CUSTOM))
                    .map(|s| CsvCell::<CsvThoth>::csv_cell(s))
                    .collect::<Vec<String>>(),
            ),
            keywords: CsvCell::<CsvThoth>::csv_cell(
                &subjects
                    .iter()
                    .filter(|s| s.subject_type.eq(&SubjectType::KEYWORD))
                    .map(|s| CsvCell::<CsvThoth>::csv_cell(s))
                    .collect::<Vec<String>>(),
            ),
            funding: CsvCell::<CsvThoth>::csv_cell(
                &work
                    .fundings
                    .iter()
                    .map(|f| CsvCell::<CsvThoth>::csv_cell(f))
                    .collect::<Vec<String>>(),
            ),
        }
    }
}

impl CsvCell<CsvThoth> for Vec<String> {
    fn csv_cell(&self) -> String {
        if self.is_empty() {
            "".to_string()
        } else {
            format!("[{}]", self.join(","))
        }
    }
}

impl CsvCell<CsvThoth> for WorkPublications {
    fn csv_cell(&self) -> String {
        format!(
            "(\"{:?}\", \"{}\", \"{}\", {})",
            self.publication_type,
            self.isbn
                .as_ref()
                .map(|i| i.to_string())
                .unwrap_or_else(|| "".to_string()),
            self.publication_url
                .clone()
                .unwrap_or_else(|| "".to_string()),
            CsvCell::<CsvThoth>::csv_cell(
                &self
                    .prices
                    .iter()
                    .map(|p| CsvCell::<CsvThoth>::csv_cell(p))
                    .collect::<Vec<String>>(),
            )
        )
    }
}

impl CsvCell<CsvThoth> for WorkContributions {
    fn csv_cell(&self) -> String {
        format!(
            "(\"{:?}\", \"{}\", \"{}\", \"{}\", \"{}\", \"{}\")",
            self.contribution_type,
            self.first_name.clone().unwrap_or_else(|| "".to_string()),
            self.last_name,
            self.full_name,
            self.institution.clone().unwrap_or_else(|| "".to_string()),
            self.contributor
                .orcid
                .as_ref()
                .map(|o| o.to_string())
                .unwrap_or_else(|| "".to_string())
        )
    }
}

impl CsvCell<CsvThoth> for WorkPublicationsPrices {
    fn csv_cell(&self) -> String {
        format!("(\"{:?}\", \"{}\")", self.currency_code, self.unit_price,)
    }
}

impl CsvCell<CsvThoth> for WorkIssues {
    fn csv_cell(&self) -> String {
        format!(
            "(\"{:?}\", \"{}\", \"{}\", \"{}\", \"{}\", \"{}\")",
            self.series.series_type,
            self.series.series_name,
            self.series.issn_print,
            self.series.issn_digital,
            self.series
                .series_url
                .clone()
                .unwrap_or_else(|| "".to_string()),
            self.issue_ordinal,
        )
    }
}

impl CsvCell<CsvThoth> for WorkLanguages {
    fn csv_cell(&self) -> String {
        format!(
            "(\"{:?}\", \"{:?}\", \"{}\")",
            self.language_relation, self.language_code, self.main_language,
        )
    }
}

impl CsvCell<CsvThoth> for WorkSubjects {
    fn csv_cell(&self) -> String {
        format!("{:?}", self.subject_code)
    }
}

impl CsvCell<CsvThoth> for WorkFundings {
    fn csv_cell(&self) -> String {
        format!(
            "(\"{}\", \"{}\", \"{}\", \"{}\", \"{}\", \"{}\")",
            self.funder.funder_name,
            self.funder
                .funder_doi
                .as_ref()
                .map(|d| d.to_string())
                .unwrap_or_else(|| "".to_string()),
            self.program.clone().unwrap_or_else(|| "".to_string()),
            self.project_name.clone().unwrap_or_else(|| "".to_string()),
            self.grant_number.clone().unwrap_or_else(|| "".to_string()),
            self.jurisdiction.clone().unwrap_or_else(|| "".to_string()),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lazy_static::lazy_static;
    use std::str::FromStr;
    use thoth_api::model::Doi;
    use thoth_api::model::Isbn;
    use thoth_api::model::Orcid;
    use thoth_client::{
        ContributionType, CurrencyCode, LanguageCode, LanguageRelation, PublicationType,
        WorkContributionsContributor, WorkImprint, WorkImprintPublisher, WorkStatus, WorkType,
    };
    use uuid::Uuid;

    lazy_static! {
        static ref TEST_WORK: Work = Work {
            work_id: Uuid::from_str("00000000-0000-0000-AAAA-000000000001").unwrap(),
            work_status: WorkStatus::ACTIVE,
            full_title: "Book Title: Book Subtitle".to_string(),
            title: "Book Title".to_string(),
            subtitle: Some("Book Subtitle".to_string()),
            work_type: WorkType::MONOGRAPH,
            edition: 1,
            doi: Some(Doi::from_str("https://doi.org/10.00001/BOOK.0001").unwrap()),
            publication_date: None,
            license: Some("http://creativecommons.org/licenses/by/4.0/".to_string()),
            copyright_holder: "Author 1; Author 2".to_string(),
            short_abstract: Some("Lorem ipsum dolor sit amet, consectetur adipiscing elit. Vestibulum vel libero eleifend, ultrices purus vitae, suscipit ligula. Aliquam ornare quam et nulla vestibulum, id euismod tellus malesuada. Orci varius natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus.".to_string()),
            long_abstract: Some("Lorem ipsum dolor sit amet, consectetur adipiscing elit. Vestibulum vel libero eleifend, ultrices purus vitae, suscipit ligula. Aliquam ornare quam et nulla vestibulum, id euismod tellus malesuada. Orci varius natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. Nullam ornare bibendum ex nec dapibus. Proin porta risus elementum odio feugiat tempus. Etiam eu felis ac metus viverra ornare. In consectetur neque sed feugiat ornare. Mauris at purus fringilla orci tincidunt pulvinar sed a massa. Nullam vestibulum posuere augue, sit amet tincidunt nisl pulvinar ac.".to_string()),
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
            issues: vec![],
            contributions: vec![
                WorkContributions {
                    contribution_type: ContributionType::AUTHOR,
                    first_name: Some("Author".to_string()),
                    last_name: "1".to_string(),
                    full_name: "Author 1".to_string(),
                    main_contribution: true,
                    biography: None,
                    institution: None,
                    contribution_ordinal: 1,
                    contributor: WorkContributionsContributor {
                        orcid: Some(Orcid::from_str("https://orcid.org/0000-0002-0000-0001").unwrap()),
                    },
                },
                WorkContributions {
                    contribution_type: ContributionType::AUTHOR,
                    first_name: Some("Author".to_string()),
                    last_name: "2".to_string(),
                    full_name: "Author 2".to_string(),
                    main_contribution: true,
                    biography: None,
                    institution: None,
                    contribution_ordinal: 2,
                    contributor: WorkContributionsContributor {
                        orcid: None,
                    },
                },
            ],
            languages: vec![
                WorkLanguages {
                    language_code: LanguageCode::SPA,
                    language_relation: LanguageRelation::ORIGINAL,
                    main_language: true,
                },
            ],
            publications: vec![
                WorkPublications {
                    publication_id: Uuid::from_str("00000000-0000-0000-BBBB-000000000002").unwrap(),
                    publication_type: PublicationType::PAPERBACK,
                    publication_url: Some("https://www.book.com/paperback".to_string()),
                    isbn: Some(Isbn::from_str("978-3-16-148410-0").unwrap()),
                    prices: vec![
                        WorkPublicationsPrices {
                            currency_code: CurrencyCode::EUR,
                            unit_price: 25.95,
                        },
                        WorkPublicationsPrices {
                            currency_code: CurrencyCode::GBP,
                            unit_price: 22.95,
                        },
                        WorkPublicationsPrices {
                            currency_code: CurrencyCode::USD,
                            unit_price: 31.95,
                        },
                    ],
                },
                WorkPublications {
                    publication_id: Uuid::from_str("00000000-0000-0000-CCCC-000000000003").unwrap(),
                    publication_type: PublicationType::HARDBACK,
                    publication_url: Some("https://www.book.com/hardback".to_string()),
                    isbn: Some(Isbn::from_str("978-1-4028-9462-6").unwrap()),
                    prices: vec![
                        WorkPublicationsPrices {
                            currency_code: CurrencyCode::EUR,
                            unit_price: 36.95,
                        },
                        WorkPublicationsPrices {
                            currency_code: CurrencyCode::GBP,
                            unit_price: 32.95,
                        },
                        WorkPublicationsPrices {
                            currency_code: CurrencyCode::USD,
                            unit_price: 40.95,
                        },
                    ],
                },
                WorkPublications {
                    publication_id: Uuid::from_str("00000000-0000-0000-DDDD-000000000004").unwrap(),
                    publication_type: PublicationType::PDF,
                    publication_url: Some("https://www.book.com/pdf".to_string()),
                    isbn: Some(Isbn::from_str("978-1-56619-909-4").unwrap()),
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
                    isbn: Some(Isbn::from_str("978-92-95055-02-5").unwrap()),
                    prices: vec![],
                },
            ],
            subjects: vec![
                WorkSubjects {
                    subject_code: "AAB".to_string(),
                    subject_type: SubjectType::BIC,
                    subject_ordinal: 2,
                },
                WorkSubjects {
                    subject_code: "AAA".to_string(),
                    subject_type: SubjectType::BIC,
                    subject_ordinal: 1,
                },
                WorkSubjects {
                    subject_code: "AAA000001".to_string(),
                    subject_type: SubjectType::BISAC,
                    subject_ordinal: 2,
                },
                WorkSubjects {
                    subject_code: "AAA000000".to_string(),
                    subject_type: SubjectType::BISAC,
                    subject_ordinal: 1,
                },
                WorkSubjects {
                    subject_code: "Category1".to_string(),
                    subject_type: SubjectType::CUSTOM,
                    subject_ordinal: 1,
                },
                WorkSubjects {
                    subject_code: "keyword2".to_string(),
                    subject_type: SubjectType::KEYWORD,
                    subject_ordinal: 2,
                },
                WorkSubjects {
                    subject_code: "keyword1".to_string(),
                    subject_type: SubjectType::KEYWORD,
                    subject_ordinal: 1,
                },
            ],
            fundings: vec![],
        };
    }

    const TEST_RESULT: &str = r#""publisher","imprint","work_type","work_status","title","subtitle","edition","doi","publication_date","publication_place","license","copyright_holder","landing_page","width (mm)","width (cm)","width (in)","height (mm)","height (cm)","height (in)","page_count","page_breakdown","image_count","table_count","audio_count","video_count","lccn","oclc","short_abstract","long_abstract","general_note","toc","cover_url","cover_caption","contributions [(type, first_name, last_name, full_name, institution, orcid)]","publications [(type, isbn, url, [(ISO_4217_currency, price)])]","series [(type, name, issn_print, issn_digital, url, issue)]","languages [(relation, ISO_639-3/B_language, is_main)]","BIC [code]","THEMA [code]","BISAC [code]","LCC [code]","custom_categories [category]","keywords [keyword]","funding [(funder, funder_doi, program, project, grant, jurisdiction)]"
"OA Editions","OA Editions Imprint","MONOGRAPH","ACTIVE","Book Title","Book Subtitle","1","10.00001/BOOK.0001","","León, Spain","http://creativecommons.org/licenses/by/4.0/","Author 1; Author 2","https://www.book.com","156.0","15.6","6.14","234.0","23.4","9.21","334","x+334","15","","","","","","Lorem ipsum dolor sit amet, consectetur adipiscing elit. Vestibulum vel libero eleifend, ultrices purus vitae, suscipit ligula. Aliquam ornare quam et nulla vestibulum, id euismod tellus malesuada. Orci varius natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus.","Lorem ipsum dolor sit amet, consectetur adipiscing elit. Vestibulum vel libero eleifend, ultrices purus vitae, suscipit ligula. Aliquam ornare quam et nulla vestibulum, id euismod tellus malesuada. Orci varius natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. Nullam ornare bibendum ex nec dapibus. Proin porta risus elementum odio feugiat tempus. Etiam eu felis ac metus viverra ornare. In consectetur neque sed feugiat ornare. Mauris at purus fringilla orci tincidunt pulvinar sed a massa. Nullam vestibulum posuere augue, sit amet tincidunt nisl pulvinar ac.","","","https://www.book.com/cover","","[(""AUTHOR"", ""Author"", ""1"", ""Author 1"", """", ""0000-0002-0000-0001""),(""AUTHOR"", ""Author"", ""2"", ""Author 2"", """", """")]","[(""PAPERBACK"", ""978-3-16-148410-0"", ""https://www.book.com/paperback"", [(""EUR"", ""25.95""),(""GBP"", ""22.95""),(""USD"", ""31.95"")]),(""HARDBACK"", ""978-1-4028-9462-6"", ""https://www.book.com/hardback"", [(""EUR"", ""36.95""),(""GBP"", ""32.95""),(""USD"", ""40.95"")]),(""PDF"", ""978-1-56619-909-4"", ""https://www.book.com/pdf"", ),(""HTML"", """", ""https://www.book.com/html"", ),(""XML"", ""978-92-95055-02-5"", ""https://www.book.com/xml"", )]","","[(""ORIGINAL"", ""SPA"", ""true"")]","[""AAA"",""AAB""]","","[""AAA000000"",""AAA000001""]","","[""Category1""]","[""keyword1"",""keyword2""]",""
"#;

    #[test]
    fn test_csv_thoth() {
        let to_test = CsvThoth.generate(&[TEST_WORK.clone()]);

        assert_eq!(to_test, Ok(TEST_RESULT.to_string()))
    }
}
