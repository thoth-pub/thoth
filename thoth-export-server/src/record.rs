use actix_web::{http::StatusCode, HttpRequest, Responder};
use csv::QuoteStyle;
use paperclip::actix::web::HttpResponse;
use paperclip::actix::OperationModifier;
use paperclip::v2::models::{DefaultOperationRaw, Either, Response};
use paperclip::v2::schema::Apiv2Schema;
use std::fmt::Display;
use std::str::FromStr;
use std::sync::Arc;
use thoth_api::model::Timestamp;
use thoth_api::redis::{del, get, set, RedisPool};
use thoth_client::Work;
use thoth_errors::{ThothError, ThothResult};

use crate::bibtex::{BibtexSpecification, BibtexThoth};
use crate::csv::{CsvSpecification, CsvThoth, KbartOclc};
use crate::json::{JsonSpecification, JsonThoth};
use crate::marc21::{Marc21MarkupThoth, Marc21RecordThoth, Marc21Specification};
use crate::specification_query::SpecificationQuery;
use crate::xml::{
    DoiDepositCrossref, Marc21XmlThoth, Onix21EbscoHost, Onix21ProquestEbrary, Onix31Thoth,
    Onix3GoogleBooks, Onix3Jstor, Onix3Oapen, Onix3Overdrive, Onix3ProjectMuse, Onix3Thoth,
    XmlSpecification,
};

pub const DELIMITER_COMMA: u8 = b',';
pub const DELIMITER_TAB: u8 = b'\t';
pub const XML_DECLARATION: &str = "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n";
pub const DOCTYPE_ONIX21_REF: &str = "<!DOCTYPE ONIXMessage SYSTEM \"http://www.editeur.org/onix/2.1/reference/onix-international.dtd\">\n";

#[derive(Copy, Clone)]
pub(crate) enum MetadataSpecification {
    Onix31Thoth(Onix31Thoth),
    Onix3Thoth(Onix3Thoth),
    Onix3ProjectMuse(Onix3ProjectMuse),
    Onix3Oapen(Onix3Oapen),
    Onix3Jstor(Onix3Jstor),
    Onix3GoogleBooks(Onix3GoogleBooks),
    Onix3Overdrive(Onix3Overdrive),
    Onix21EbscoHost(Onix21EbscoHost),
    Onix21ProquestEbrary(Onix21ProquestEbrary),
    CsvThoth(CsvThoth),
    JsonThoth(JsonThoth),
    KbartOclc(KbartOclc),
    BibtexThoth(BibtexThoth),
    DoiDepositCrossref(DoiDepositCrossref),
    Marc21RecordThoth(Marc21RecordThoth),
    Marc21MarkupThoth(Marc21MarkupThoth),
    Marc21XmlThoth(Marc21XmlThoth),
}

pub(crate) struct MetadataRecord {
    id: String,
    specification: MetadataSpecification,
    record: ThothResult<String>,
    last_updated: Timestamp,
}

impl MetadataRecord {
    const XML_MIME_TYPE: &'static str = "text/xml; charset=utf-8";
    const CSV_MIME_TYPE: &'static str = "text/csv; charset=utf-8";
    const TXT_MIME_TYPE: &'static str = "text/plain; charset=utf-8";
    const BIB_MIME_TYPE: &'static str = "application/x-bibtex; charset=utf-8";
    const JSON_MIME_TYPE: &'static str = "application/json; charset=utf-8";
    const MARC_MIME_TYPE: &'static str = "application/marc; charset=utf-8";
    const XML_EXTENSION: &'static str = ".xml";
    const CSV_EXTENSION: &'static str = ".csv";
    const TXT_EXTENSION: &'static str = ".txt";
    const BIB_EXTENSION: &'static str = ".bib";
    const JSON_EXTENSION: &'static str = ".json";
    const MARC_RECORD_EXTENSION: &'static str = ".mrc";
    const MARC_MARKUP_EXTENSION: &'static str = ".mrk";

    pub(crate) fn new(
        id: String,
        specification: MetadataSpecification,
        last_updated: Timestamp,
    ) -> Self {
        MetadataRecord {
            id,
            specification,
            record: Err(ThothError::MetadataRecordNotGenerated),
            last_updated,
        }
    }

    fn content_type(&self) -> &'static str {
        match &self.specification {
            MetadataSpecification::Onix31Thoth(_) => Self::XML_MIME_TYPE,
            MetadataSpecification::Onix3Thoth(_) => Self::XML_MIME_TYPE,
            MetadataSpecification::Onix3ProjectMuse(_) => Self::XML_MIME_TYPE,
            MetadataSpecification::Onix3Oapen(_) => Self::XML_MIME_TYPE,
            MetadataSpecification::Onix3Jstor(_) => Self::XML_MIME_TYPE,
            MetadataSpecification::Onix3GoogleBooks(_) => Self::XML_MIME_TYPE,
            MetadataSpecification::Onix3Overdrive(_) => Self::XML_MIME_TYPE,
            MetadataSpecification::Onix21EbscoHost(_) => Self::XML_MIME_TYPE,
            MetadataSpecification::Onix21ProquestEbrary(_) => Self::XML_MIME_TYPE,
            MetadataSpecification::CsvThoth(_) => Self::CSV_MIME_TYPE,
            MetadataSpecification::JsonThoth(_) => Self::JSON_MIME_TYPE,
            MetadataSpecification::KbartOclc(_) => Self::TXT_MIME_TYPE,
            MetadataSpecification::BibtexThoth(_) => Self::BIB_MIME_TYPE,
            MetadataSpecification::DoiDepositCrossref(_) => Self::XML_MIME_TYPE,
            MetadataSpecification::Marc21RecordThoth(_) => Self::MARC_MIME_TYPE,
            MetadataSpecification::Marc21MarkupThoth(_) => Self::TXT_MIME_TYPE,
            MetadataSpecification::Marc21XmlThoth(_) => Self::XML_MIME_TYPE,
        }
    }

    fn file_name(&self) -> String {
        match &self.specification {
            MetadataSpecification::Onix31Thoth(_) => self.xml_file_name(),
            MetadataSpecification::Onix3Thoth(_) => self.xml_file_name(),
            MetadataSpecification::Onix3ProjectMuse(_) => self.xml_file_name(),
            MetadataSpecification::Onix3Oapen(_) => self.xml_file_name(),
            MetadataSpecification::Onix3Jstor(_) => self.xml_file_name(),
            MetadataSpecification::Onix3GoogleBooks(_) => self.xml_file_name(),
            MetadataSpecification::Onix3Overdrive(_) => self.xml_file_name(),
            MetadataSpecification::Onix21EbscoHost(_) => self.xml_file_name(),
            MetadataSpecification::Onix21ProquestEbrary(_) => self.xml_file_name(),
            MetadataSpecification::CsvThoth(_) => self.csv_file_name(),
            MetadataSpecification::JsonThoth(_) => self.json_file_name(),
            MetadataSpecification::KbartOclc(_) => self.txt_file_name(),
            MetadataSpecification::BibtexThoth(_) => self.bib_file_name(),
            MetadataSpecification::DoiDepositCrossref(_) => self.xml_file_name(),
            MetadataSpecification::Marc21RecordThoth(_) => self.marc_record_file_name(),
            MetadataSpecification::Marc21MarkupThoth(_) => self.marc_markup_file_name(),
            MetadataSpecification::Marc21XmlThoth(_) => self.xml_file_name(),
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

    fn json_file_name(&self) -> String {
        self.format_file_name(Self::JSON_EXTENSION)
    }

    fn marc_record_file_name(&self) -> String {
        self.format_file_name(Self::MARC_RECORD_EXTENSION)
    }

    fn marc_markup_file_name(&self) -> String {
        self.format_file_name(Self::MARC_MARKUP_EXTENSION)
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

    fn cache_key(&self) -> String {
        format!("{}:{}", self.specification, self.id)
    }

    fn cache_timestamp_key(&self) -> String {
        format!("{}:timestamp", self.cache_key())
    }

    fn cache_error_key(&self) -> String {
        format!("{}:error", self.cache_key())
    }

    pub(crate) async fn load_or_generate(
        &mut self,
        specification_query: SpecificationQuery,
        redis_pool: Arc<RedisPool>,
    ) -> ThothResult<()> {
        let cache_key = self.cache_key();
        let cache_timestamp_key = self.cache_timestamp_key();
        let cache_error_key = self.cache_error_key();

        // Check for cached record or error
        if let Ok(cached_timestamp_value) = get(&redis_pool, &cache_timestamp_key).await {
            let cached_timestamp = Timestamp::parse_from_rfc3339(&cached_timestamp_value)?;
            if cached_timestamp >= self.last_updated {
                if let Ok(cached_record) = get(&redis_pool, &cache_key).await {
                    self.record = Ok(cached_record);
                    return Ok(());
                }
                if let Ok(cached_error) = get(&redis_pool, &cache_error_key).await {
                    self.record = Err(ThothError::from_json(&cached_error)?);
                    return Ok(());
                }
            }
        }

        let data = specification_query.run().await?;
        self.record = self.generate(data);
        self.update_cache(&redis_pool).await?;
        Ok(())
    }

    /// Cache the record, update the timestamp, and delete previous errors or records
    async fn update_cache(&self, redis_pool: &RedisPool) -> ThothResult<()> {
        set(
            redis_pool,
            &self.cache_timestamp_key(),
            &self.last_updated.to_rfc3339(),
        )
        .await?;
        match &self.record {
            Ok(record) => {
                set(redis_pool, &self.cache_key(), record).await?;
                del(redis_pool, &self.cache_error_key()).await?;
            }
            Err(error) => {
                set(redis_pool, &self.cache_error_key(), &error.to_json()?).await?;
                del(redis_pool, &self.cache_key()).await?;
            }
        }
        Ok(())
    }

    fn generate(&self, data: Vec<Work>) -> ThothResult<String> {
        match &self.specification {
            MetadataSpecification::Onix31Thoth(onix31_thoth) => onix31_thoth.generate(&data, None),
            MetadataSpecification::Onix3Thoth(onix3_thoth) => onix3_thoth.generate(&data, None),
            MetadataSpecification::Onix3ProjectMuse(onix3_project_muse) => {
                onix3_project_muse.generate(&data, None)
            }
            MetadataSpecification::Onix3Oapen(onix3_oapen) => onix3_oapen.generate(&data, None),
            MetadataSpecification::Onix3Jstor(onix3_jstor) => onix3_jstor.generate(&data, None),
            MetadataSpecification::Onix3GoogleBooks(onix3_google_books) => {
                onix3_google_books.generate(&data, None)
            }
            MetadataSpecification::Onix3Overdrive(onix3_overdrive) => {
                onix3_overdrive.generate(&data, None)
            }
            MetadataSpecification::Onix21EbscoHost(onix21_ebsco_host) => {
                onix21_ebsco_host.generate(&data, Some(DOCTYPE_ONIX21_REF))
            }
            MetadataSpecification::Onix21ProquestEbrary(onix21_proquest_ebrary) => {
                onix21_proquest_ebrary.generate(&data, Some(DOCTYPE_ONIX21_REF))
            }
            MetadataSpecification::CsvThoth(csv_thoth) => {
                csv_thoth.generate(&data, QuoteStyle::Always, DELIMITER_COMMA)
            }
            MetadataSpecification::JsonThoth(json_thoth) => json_thoth.generate(&data),
            MetadataSpecification::KbartOclc(kbart_oclc) => {
                kbart_oclc.generate(&data, QuoteStyle::Necessary, DELIMITER_TAB)
            }
            MetadataSpecification::BibtexThoth(bibtex_thoth) => bibtex_thoth.generate(&data),
            MetadataSpecification::DoiDepositCrossref(doideposit_crossref) => {
                doideposit_crossref.generate(&data, None)
            }
            MetadataSpecification::Marc21RecordThoth(marc21record_thoth) => {
                marc21record_thoth.generate(&data)
            }
            MetadataSpecification::Marc21MarkupThoth(marc21markup_thoth) => {
                marc21markup_thoth.generate(&data)
            }
            MetadataSpecification::Marc21XmlThoth(marc21xml_thoth) => {
                marc21xml_thoth.generate(&data)
            }
        }
    }
}

impl Responder for MetadataRecord {
    type Body = actix_web::body::BoxBody;

    fn respond_to(self, _: &HttpRequest) -> HttpResponse {
        match self.record {
            Ok(ref record) => HttpResponse::build(StatusCode::OK)
                .content_type(self.content_type())
                .append_header(("Content-Disposition", self.content_disposition()))
                .body(record.to_owned()),
            Err(e) => HttpResponse::from_error(e),
        }
    }
}

impl Apiv2Schema for MetadataRecord {}

impl OperationModifier for MetadataRecord {
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
            "onix_3.1::thoth" => Ok(MetadataSpecification::Onix31Thoth(Onix31Thoth {})),
            "onix_3.0::thoth" => Ok(MetadataSpecification::Onix3Thoth(Onix3Thoth {})),
            "onix_3.0::project_muse" => {
                Ok(MetadataSpecification::Onix3ProjectMuse(Onix3ProjectMuse {}))
            }
            "onix_3.0::oapen" => Ok(MetadataSpecification::Onix3Oapen(Onix3Oapen {})),
            "onix_3.0::jstor" => Ok(MetadataSpecification::Onix3Jstor(Onix3Jstor {})),
            "onix_3.0::google_books" => {
                Ok(MetadataSpecification::Onix3GoogleBooks(Onix3GoogleBooks {}))
            }
            "onix_3.0::overdrive" => Ok(MetadataSpecification::Onix3Overdrive(Onix3Overdrive {})),
            "onix_2.1::ebsco_host" => {
                Ok(MetadataSpecification::Onix21EbscoHost(Onix21EbscoHost {}))
            }
            "onix_2.1::proquest_ebrary" => Ok(MetadataSpecification::Onix21ProquestEbrary(
                Onix21ProquestEbrary {},
            )),
            "csv::thoth" => Ok(MetadataSpecification::CsvThoth(CsvThoth {})),
            "json::thoth" => Ok(MetadataSpecification::JsonThoth(JsonThoth {})),
            "kbart::oclc" => Ok(MetadataSpecification::KbartOclc(KbartOclc {})),
            "bibtex::thoth" => Ok(MetadataSpecification::BibtexThoth(BibtexThoth {})),
            "doideposit::crossref" => Ok(MetadataSpecification::DoiDepositCrossref(
                DoiDepositCrossref {},
            )),
            "marc21record::thoth" => Ok(MetadataSpecification::Marc21RecordThoth(
                Marc21RecordThoth {},
            )),
            "marc21markup::thoth" => Ok(MetadataSpecification::Marc21MarkupThoth(
                Marc21MarkupThoth {},
            )),
            "marc21xml::thoth" => Ok(MetadataSpecification::Marc21XmlThoth(Marc21XmlThoth {})),
            _ => Err(ThothError::InvalidMetadataSpecification(input.to_string())),
        }
    }
}

impl Display for MetadataSpecification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            MetadataSpecification::Onix31Thoth(_) => "onix_3.1::thoth",
            MetadataSpecification::Onix3Thoth(_) => "onix_3.0::thoth",
            MetadataSpecification::Onix3ProjectMuse(_) => "onix_3.0::project_muse",
            MetadataSpecification::Onix3Oapen(_) => "onix_3.0::oapen",
            MetadataSpecification::Onix3Jstor(_) => "onix_3.0::jstor",
            MetadataSpecification::Onix3GoogleBooks(_) => "onix_3.0::google_books",
            MetadataSpecification::Onix3Overdrive(_) => "onix_3.0::overdrive",
            MetadataSpecification::Onix21EbscoHost(_) => "onix_2.1::ebsco_host",
            MetadataSpecification::Onix21ProquestEbrary(_) => "onix_2.1::proquest_ebrary",
            MetadataSpecification::CsvThoth(_) => "csv::thoth",
            MetadataSpecification::JsonThoth(_) => "json::thoth",
            MetadataSpecification::KbartOclc(_) => "kbart::oclc",
            MetadataSpecification::BibtexThoth(_) => "bibtex::thoth",
            MetadataSpecification::DoiDepositCrossref(_) => "doideposit::crossref",
            MetadataSpecification::Marc21RecordThoth(_) => "marc21record::thoth",
            MetadataSpecification::Marc21MarkupThoth(_) => "marc21markup::thoth",
            MetadataSpecification::Marc21XmlThoth(_) => "marc21xml::thoth",
        };
        write!(f, "{}", str)
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
        let timestamp = Timestamp::default();
        let to_test = MetadataRecord::new(
            "some_id".to_string(),
            MetadataSpecification::CsvThoth(CsvThoth {}),
            timestamp,
        );
        assert_eq!(to_test.file_name(), "csv__thoth__some_id.csv".to_string());
        let to_test = MetadataRecord::new(
            "some_id".to_string(),
            MetadataSpecification::JsonThoth(JsonThoth {}),
            timestamp,
        );
        assert_eq!(to_test.file_name(), "json__thoth__some_id.json".to_string());
        let to_test = MetadataRecord::new(
            "some_id".to_string(),
            MetadataSpecification::Onix3Thoth(Onix3Thoth {}),
            timestamp,
        );
        assert_eq!(
            to_test.file_name(),
            "onix_3.0__thoth__some_id.xml".to_string()
        );
        let to_test = MetadataRecord::new(
            "some_id".to_string(),
            MetadataSpecification::Onix31Thoth(Onix31Thoth {}),
            timestamp,
        );
        assert_eq!(
            to_test.file_name(),
            "onix_3.1__thoth__some_id.xml".to_string()
        );
        let to_test = MetadataRecord::new(
            "some_id".to_string(),
            MetadataSpecification::Onix3ProjectMuse(Onix3ProjectMuse {}),
            timestamp,
        );
        assert_eq!(
            to_test.file_name(),
            "onix_3.0__project_muse__some_id.xml".to_string()
        );
        let to_test = MetadataRecord::new(
            "some_id".to_string(),
            MetadataSpecification::Onix3Oapen(Onix3Oapen {}),
            timestamp,
        );
        assert_eq!(
            to_test.file_name(),
            "onix_3.0__oapen__some_id.xml".to_string()
        );
        let to_test = MetadataRecord::new(
            "some_id".to_string(),
            MetadataSpecification::Onix3Jstor(Onix3Jstor {}),
            timestamp,
        );
        assert_eq!(
            to_test.file_name(),
            "onix_3.0__jstor__some_id.xml".to_string()
        );
        let to_test = MetadataRecord::new(
            "some_id".to_string(),
            MetadataSpecification::Onix3GoogleBooks(Onix3GoogleBooks {}),
            timestamp,
        );
        assert_eq!(
            to_test.file_name(),
            "onix_3.0__google_books__some_id.xml".to_string()
        );
        let to_test = MetadataRecord::new(
            "some_id".to_string(),
            MetadataSpecification::Onix3Overdrive(Onix3Overdrive {}),
            timestamp,
        );
        assert_eq!(
            to_test.file_name(),
            "onix_3.0__overdrive__some_id.xml".to_string()
        );
        let to_test = MetadataRecord::new(
            "some_id".to_string(),
            MetadataSpecification::Onix21EbscoHost(Onix21EbscoHost {}),
            timestamp,
        );
        assert_eq!(
            to_test.file_name(),
            "onix_2.1__ebsco_host__some_id.xml".to_string()
        );
        let to_test = MetadataRecord::new(
            "some_id".to_string(),
            MetadataSpecification::Onix21ProquestEbrary(Onix21ProquestEbrary {}),
            timestamp,
        );
        assert_eq!(
            to_test.file_name(),
            "onix_2.1__proquest_ebrary__some_id.xml".to_string()
        );
        let to_test = MetadataRecord::new(
            "some_id".to_string(),
            MetadataSpecification::KbartOclc(KbartOclc {}),
            timestamp,
        );
        assert_eq!(to_test.file_name(), "kbart__oclc__some_id.txt".to_string());
        let to_test = MetadataRecord::new(
            "some_id".to_string(),
            MetadataSpecification::BibtexThoth(BibtexThoth {}),
            timestamp,
        );
        assert_eq!(
            to_test.file_name(),
            "bibtex__thoth__some_id.bib".to_string()
        );
        let to_test = MetadataRecord::new(
            "some_id".to_string(),
            MetadataSpecification::DoiDepositCrossref(DoiDepositCrossref {}),
            timestamp,
        );
        assert_eq!(
            to_test.file_name(),
            "doideposit__crossref__some_id.xml".to_string()
        );
        let to_test = MetadataRecord::new(
            "some_id".to_string(),
            MetadataSpecification::Marc21RecordThoth(Marc21RecordThoth {}),
            timestamp,
        );
        assert_eq!(
            to_test.file_name(),
            "marc21record__thoth__some_id.mrc".to_string()
        );
        let to_test = MetadataRecord::new(
            "some_id".to_string(),
            MetadataSpecification::Marc21MarkupThoth(Marc21MarkupThoth {}),
            timestamp,
        );
        assert_eq!(
            to_test.file_name(),
            "marc21markup__thoth__some_id.mrk".to_string()
        );
        let to_test = MetadataRecord::new(
            "some_id".to_string(),
            MetadataSpecification::Marc21XmlThoth(Marc21XmlThoth {}),
            timestamp,
        );
        assert_eq!(
            to_test.file_name(),
            "marc21xml__thoth__some_id.xml".to_string()
        );
    }
}
