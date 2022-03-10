use actix_web::{http::StatusCode, HttpRequest, Responder};
use csv::QuoteStyle;
use paperclip::actix::web::HttpResponse;
use paperclip::actix::OperationModifier;
use paperclip::util::{ready, Ready};
use paperclip::v2::models::{DefaultOperationRaw, Either, Response};
use paperclip::v2::schema::Apiv2Schema;
use std::str::FromStr;
use thoth_client::Work;
use thoth_errors::{ThothError, ThothResult};

use crate::bibtex::{BibtexSpecification, BibtexThoth};
use crate::csv::{CsvSpecification, CsvThoth, KbartOclc};
use crate::xml::{Onix21EbscoHost, Onix3Jstor, Onix3Oapen, Onix3ProjectMuse, XmlSpecification};

pub(crate) trait AsRecord {}
impl AsRecord for Vec<Work> {}

pub const DELIMITER_COMMA: u8 = b',';
pub const DELIMITER_TAB: u8 = b'\t';
pub const XML_DECLARATION: &str = "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n";
pub const DOCTYPE_ONIX21_REF: &str = "<!DOCTYPE ONIXMessage SYSTEM \"http://www.editeur.org/onix/2.1/reference/onix-international.dtd\">\n";

pub(crate) enum MetadataSpecification {
    Onix3ProjectMuse(Onix3ProjectMuse),
    Onix3Oapen(Onix3Oapen),
    Onix3Jstor(Onix3Jstor),
    Onix21EbscoHost(Onix21EbscoHost),
    CsvThoth(CsvThoth),
    KbartOclc(KbartOclc),
    BibtexThoth(BibtexThoth),
}

pub(crate) struct MetadataRecord<T: AsRecord> {
    id: String,
    data: T,
    specification: MetadataSpecification,
}

impl<T> MetadataRecord<T>
where
    T: AsRecord + IntoIterator,
{
    const XML_MIME_TYPE: &'static str = "text/xml; charset=utf-8";
    const CSV_MIME_TYPE: &'static str = "text/csv; charset=utf-8";
    const TXT_MIME_TYPE: &'static str = "text/plain; charset=utf-8";
    const BIB_MIME_TYPE: &'static str = "application/x-bibtex; charset=utf-8";
    const XML_EXTENSION: &'static str = ".xml";
    const CSV_EXTENSION: &'static str = ".csv";
    const TXT_EXTENSION: &'static str = ".txt";
    const BIB_EXTENSION: &'static str = ".bib";

    pub(crate) fn new(id: String, specification: MetadataSpecification, data: T) -> Self {
        MetadataRecord {
            id,
            data,
            specification,
        }
    }

    fn content_type(&self) -> &'static str {
        match &self.specification {
            MetadataSpecification::Onix3ProjectMuse(_) => Self::XML_MIME_TYPE,
            MetadataSpecification::Onix3Oapen(_) => Self::XML_MIME_TYPE,
            MetadataSpecification::Onix3Jstor(_) => Self::XML_MIME_TYPE,
            MetadataSpecification::Onix21EbscoHost(_) => Self::XML_MIME_TYPE,
            MetadataSpecification::CsvThoth(_) => Self::CSV_MIME_TYPE,
            MetadataSpecification::KbartOclc(_) => Self::TXT_MIME_TYPE,
            MetadataSpecification::BibtexThoth(_) => Self::BIB_MIME_TYPE,
        }
    }

    fn file_name(&self) -> String {
        match &self.specification {
            MetadataSpecification::Onix3ProjectMuse(_) => self.xml_file_name(),
            MetadataSpecification::Onix3Oapen(_) => self.xml_file_name(),
            MetadataSpecification::Onix3Jstor(_) => self.xml_file_name(),
            MetadataSpecification::Onix21EbscoHost(_) => self.xml_file_name(),
            MetadataSpecification::CsvThoth(_) => self.csv_file_name(),
            MetadataSpecification::KbartOclc(_) => self.txt_file_name(),
            MetadataSpecification::BibtexThoth(_) => self.bib_file_name(),
        }
    }

    fn xml_file_name(&self) -> String {
        self.format_file_name(Self::XML_EXTENSION)
    }

    fn csv_file_name(&self) -> String {
        self.format_file_name(Self::CSV_EXTENSION)
    }

    fn txt_file_name(&self) -> String {
        self.format_file_name(Self::TXT_EXTENSION)
    }

    fn bib_file_name(&self) -> String {
        self.format_file_name(Self::BIB_EXTENSION)
    }

    fn format_file_name(&self, extension: &'static str) -> String {
        format!(
            "{}__{}{}",
            self.specification.to_string().replace("::", "__"),
            self.id,
            extension
        )
    }

    fn content_disposition(&self) -> String {
        format!("attachment; filename=\"{}\"", self.file_name())
    }
}

impl MetadataRecord<Vec<Work>> {
    fn generate(&self) -> ThothResult<String> {
        match &self.specification {
            MetadataSpecification::Onix3ProjectMuse(onix3_project_muse) => {
                onix3_project_muse.generate(&self.data, None)
            }
            MetadataSpecification::Onix3Oapen(onix3_oapen) => {
                onix3_oapen.generate(&self.data, None)
            }
            MetadataSpecification::Onix3Jstor(onix3_jstor) => {
                onix3_jstor.generate(&self.data, None)
            }
            MetadataSpecification::Onix21EbscoHost(onix21_ebsco_host) => {
                onix21_ebsco_host.generate(&self.data, Some(DOCTYPE_ONIX21_REF))
            }
            MetadataSpecification::CsvThoth(csv_thoth) => {
                csv_thoth.generate(&self.data, QuoteStyle::Always, DELIMITER_COMMA)
            }
            MetadataSpecification::KbartOclc(kbart_oclc) => {
                kbart_oclc.generate(&self.data, QuoteStyle::Necessary, DELIMITER_TAB)
            }
            MetadataSpecification::BibtexThoth(bibtex_thoth) => bibtex_thoth.generate(&self.data),
        }
    }
}

impl Responder for MetadataRecord<Vec<Work>>
where
    actix_web::dev::Body: From<String>,
{
    type Error = ThothError;
    type Future = Ready<ThothResult<HttpResponse>>;

    fn respond_to(self, _: &HttpRequest) -> Self::Future {
        match self.generate() {
            Ok(record) => ready(Ok(HttpResponse::build(StatusCode::OK)
                .content_type(self.content_type())
                .header("Content-Disposition", self.content_disposition())
                .body(record))),
            Err(e) => ready(Err(e)),
        }
    }
}

impl<T: AsRecord> Apiv2Schema for MetadataRecord<T> {}

impl<T> OperationModifier for MetadataRecord<T>
where
    T: AsRecord,
{
    fn update_response(op: &mut DefaultOperationRaw) {
        let status: StatusCode = StatusCode::OK;
        op.responses.insert(
            status.as_str().into(),
            Either::Right(Response {
                description: status.canonical_reason().map(ToString::to_string),
                schema: None,
                ..Default::default()
            }),
        );
    }
}

impl FromStr for MetadataSpecification {
    type Err = ThothError;

    fn from_str(input: &str) -> ThothResult<Self> {
        match input {
            "onix_3.0::project_muse" => {
                Ok(MetadataSpecification::Onix3ProjectMuse(Onix3ProjectMuse {}))
            }
            "onix_3.0::oapen" => Ok(MetadataSpecification::Onix3Oapen(Onix3Oapen {})),
            "onix_3.0::jstor" => Ok(MetadataSpecification::Onix3Jstor(Onix3Jstor {})),
            "onix_2.1::ebsco_host" => {
                Ok(MetadataSpecification::Onix21EbscoHost(Onix21EbscoHost {}))
            }
            "csv::thoth" => Ok(MetadataSpecification::CsvThoth(CsvThoth {})),
            "kbart::oclc" => Ok(MetadataSpecification::KbartOclc(KbartOclc {})),
            "bibtex::thoth" => Ok(MetadataSpecification::BibtexThoth(BibtexThoth {})),
            _ => Err(ThothError::InvalidMetadataSpecification(input.to_string())),
        }
    }
}

impl ToString for MetadataSpecification {
    fn to_string(&self) -> String {
        match self {
            MetadataSpecification::Onix3ProjectMuse(_) => "onix_3.0::project_muse".to_string(),
            MetadataSpecification::Onix3Oapen(_) => "onix_3.0::oapen".to_string(),
            MetadataSpecification::Onix3Jstor(_) => "onix_3.0::jstor".to_string(),
            MetadataSpecification::Onix21EbscoHost(_) => "onix_2.1::ebsco_host".to_string(),
            MetadataSpecification::CsvThoth(_) => "csv::thoth".to_string(),
            MetadataSpecification::KbartOclc(_) => "kbart::oclc".to_string(),
            MetadataSpecification::BibtexThoth(_) => "bibtex::thoth".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_specifications_are_supported_metadata_specification() {
        for s in crate::data::ALL_SPECIFICATIONS.iter() {
            assert!(MetadataSpecification::from_str(s.id).is_ok())
        }
    }

    #[test]
    fn test_unsupported_specification_error() {
        assert!(MetadataSpecification::from_str("some_random_format").is_err())
    }

    #[test]
    fn test_record_file_name() {
        let to_test = MetadataRecord::new(
            "some_id".to_string(),
            MetadataSpecification::CsvThoth(CsvThoth {}),
            vec![],
        );
        assert_eq!(to_test.file_name(), "csv__thoth__some_id.csv".to_string());
        let to_test = MetadataRecord::new(
            "some_id".to_string(),
            MetadataSpecification::Onix3ProjectMuse(Onix3ProjectMuse {}),
            vec![],
        );
        assert_eq!(
            to_test.file_name(),
            "onix_3.0__project_muse__some_id.xml".to_string()
        );
        let to_test = MetadataRecord::new(
            "some_id".to_string(),
            MetadataSpecification::Onix3Oapen(Onix3Oapen {}),
            vec![],
        );
        assert_eq!(
            to_test.file_name(),
            "onix_3.0__oapen__some_id.xml".to_string()
        );
        let to_test = MetadataRecord::new(
            "some_id".to_string(),
            MetadataSpecification::Onix3Jstor(Onix3Jstor {}),
            vec![],
        );
        assert_eq!(
            to_test.file_name(),
            "onix_3.0__jstor__some_id.xml".to_string()
        );
        let to_test = MetadataRecord::new(
            "some_id".to_string(),
            MetadataSpecification::Onix21EbscoHost(Onix21EbscoHost {}),
            vec![],
        );
        assert_eq!(
            to_test.file_name(),
            "onix_2.1__ebsco_host__some_id.xml".to_string()
        );
        let to_test = MetadataRecord::new(
            "some_id".to_string(),
            MetadataSpecification::KbartOclc(KbartOclc {}),
            vec![],
        );
        assert_eq!(to_test.file_name(), "kbart__oclc__some_id.txt".to_string());
        let to_test = MetadataRecord::new(
            "some_id".to_string(),
            MetadataSpecification::BibtexThoth(BibtexThoth {}),
            vec![],
        );
        assert_eq!(
            to_test.file_name(),
            "bibtex__thoth__some_id.bib".to_string()
        );
    }
}
