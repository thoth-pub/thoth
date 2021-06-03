use csv::Writer;
use serde::Serialize;
use std::io::Write;
use thoth_api::errors::ThothResult;
use thoth_client::{Work, WorkContributions, WorkIssues, WorkPublications, WorkPublicationsPrices};

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
    #[serde(rename = "publications [(type, isbn, url, [(currency, price)])]")]
    publications: String,
    #[serde(rename = "series [(type, name, issn_print, issn_digital, url, issue)]")]
    series: String,
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
