use csv::Writer;
use serde::Serialize;
use std::io::Write;
use thoth_api::errors::ThothResult;
use thoth_client::{ContributionType, Work, WorkContributions};

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
    #[serde(rename = "authors [(first, last, full, institution, orcid)]")]
    authors: String,
    #[serde(rename = "editors [(first, last, full, institution, orcid)]")]
    editors: String,
    #[serde(rename = "translators [(first, last, full, institution, orcid)]")]
    translators: String,
    #[serde(rename = "photographers [(first, last, full, institution, orcid)]")]
    photographers: String,
    #[serde(rename = "illustrators [(first, last, full, institution, orcid)]")]
    illustrators: String,
    #[serde(rename = "music_editors [(first, last, full, institution, orcid)]")]
    music_editors: String,
    #[serde(rename = "foreword_by [(first, last, full, institution, orcid)]")]
    foreword_by: String,
    #[serde(rename = "introduction_by [(first, last, full, institution, orcid)]")]
    introduction_by: String,
    #[serde(rename = "afterword_by [(first, last, full, institution, orcid)]")]
    afterword_by: String,
    #[serde(rename = "preface_by [(first, last, full, institution, orcid)]")]
    preface_by: String,
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

impl CsvCell<CsvThoth> for Vec<String> {
    fn csv_cell(&self) -> String {
        if self.is_empty() {
            "".to_string()
        } else {
            format!("[{}]", self.join(","))
        }
    }
}

impl CsvCell<CsvThoth> for WorkContributions {
    fn csv_cell(&self) -> String {
        format!(
            "(\"{}\", \"{}\", \"{}\", \"{}\", \"{}\")",
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
            authors: flatten_contributions(&work.contributions, ContributionType::AUTHOR),
            editors: flatten_contributions(&work.contributions, ContributionType::EDITOR),
            translators: flatten_contributions(&work.contributions, ContributionType::TRANSLATOR),
            photographers: flatten_contributions(
                &work.contributions,
                ContributionType::PHOTOGRAPHER,
            ),
            illustrators: flatten_contributions(&work.contributions, ContributionType::ILUSTRATOR),
            music_editors: flatten_contributions(
                &work.contributions,
                ContributionType::MUSIC_EDITOR,
            ),
            foreword_by: flatten_contributions(&work.contributions, ContributionType::FOREWORD_BY),
            introduction_by: flatten_contributions(
                &work.contributions,
                ContributionType::INTRODUCTION_BY,
            ),
            afterword_by: flatten_contributions(
                &work.contributions,
                ContributionType::AFTERWORD_BY,
            ),
            preface_by: flatten_contributions(&work.contributions, ContributionType::PREFACE_BY),
        }
    }
}

fn flatten_contributions(
    contributions: &[WorkContributions],
    contribution_type: ContributionType,
) -> String {
    CsvCell::<CsvThoth>::csv_cell(
        &contributions
            .iter()
            .filter(|c| c.contribution_type.eq(&contribution_type))
            .map(|c| CsvCell::<CsvThoth>::csv_cell(c))
            .collect::<Vec<String>>(),
    )
}
