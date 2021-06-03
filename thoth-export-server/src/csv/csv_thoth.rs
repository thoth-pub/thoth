use csv::Writer;
use serde::Serialize;
use std::io::Write;
use thoth_api::errors::ThothResult;
use thoth_client::{
    SubjectType, Work, WorkContributions, WorkFundings, WorkIssues, WorkLanguages,
    WorkPublications, WorkPublicationsPrices, WorkSubjects,
};

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
    doi: Option<String>,
    publication_date: Option<String>,
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
    landing_page: Option<String>,
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
            doi: work.doi,
            publication_date: work.publication_date.map(|d| d.to_string()),
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
                    .filter(|s| s.subject_type.eq(&SubjectType::BIC))
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
            landing_page: work.landing_page,
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
            self.isbn.clone().unwrap_or_else(|| "".to_string()),
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
                .clone()
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
                .clone()
                .unwrap_or_else(|| "".to_string()),
            self.program.clone().unwrap_or_else(|| "".to_string()),
            self.project_name.clone().unwrap_or_else(|| "".to_string()),
            self.grant_number.clone().unwrap_or_else(|| "".to_string()),
            self.jurisdiction.clone().unwrap_or_else(|| "".to_string()),
        )
    }
}
