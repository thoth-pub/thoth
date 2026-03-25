use std::sync::Arc;

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use quick_xml::{events::Event, Reader, Writer};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use thoth_api::model::Timestamp;
use thoth_client::{Publisher, QueryParameters, ThothClient, Work};
use thoth_errors::{ThothError, ThothResult};
use uuid::Uuid;

pub(crate) const RECORD_PREFIX: &str = "oai:thoth.pub";
pub(crate) const REPOSITORY_NAME: &str = "Thoth OAI-PMH Repository";
pub(crate) const ADMIN_EMAIL: &str = "support@thoth.pub";
pub(crate) const SAMPLE_ID: &str = "5a08ff03-7d53-42a9-bfb5-7fc81c099c52";
pub(crate) const PAGE_LIMIT: i64 = 50;

#[derive(Clone)]
pub(crate) struct OaiService {
    public_url: String,
    export_url: String,
    thoth_client: Arc<ThothClient>,
    export_client: Client,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum MetadataPrefix {
    OaiDc,
    OaiOpenaire,
    MarcXml,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ResumptionToken {
    pub offset: i64,
    pub metadata_prefix: MetadataPrefix,
    pub set: Option<String>,
    pub identifiers_only: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct SetRecord {
    pub publisher_id: Uuid,
    pub spec: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub(crate) struct RecordPage {
    pub records: Vec<Work>,
    pub cursor: i64,
    pub complete_list_size: i64,
    pub next_token: Option<String>,
}

impl MetadataPrefix {
    pub fn as_str(self) -> &'static str {
        match self {
            MetadataPrefix::OaiDc => "oai_dc",
            MetadataPrefix::OaiOpenaire => "oai_openaire",
            MetadataPrefix::MarcXml => "marcxml",
        }
    }

    pub fn schema(self) -> &'static str {
        match self {
            MetadataPrefix::OaiDc => "http://www.openarchives.org/OAI/2.0/oai_dc.xsd",
            MetadataPrefix::OaiOpenaire => {
                "https://www.openaire.eu/schema/repo-lit/4.0/openaire.xsd"
            }
            MetadataPrefix::MarcXml => {
                "https://www.loc.gov/standards/marcxml/schema/MARC21slim.xsd"
            }
        }
    }

    pub fn namespace(self) -> &'static str {
        match self {
            MetadataPrefix::OaiDc => "http://www.openarchives.org/OAI/2.0/oai_dc/",
            MetadataPrefix::OaiOpenaire => "http://namespace.openaire.eu/schema/oaire/",
            MetadataPrefix::MarcXml => "https://www.loc.gov/standards/marcxml/",
        }
    }
}

impl TryFrom<&str> for MetadataPrefix {
    type Error = ThothError;

    fn try_from(value: &str) -> ThothResult<Self> {
        match value {
            "oai_dc" => Ok(Self::OaiDc),
            "oai_openaire" => Ok(Self::OaiOpenaire),
            "marcxml" => Ok(Self::MarcXml),
            other => Err(ThothError::InvalidMetadataSpecification(other.to_string())),
        }
    }
}

impl OaiService {
    pub(crate) fn new(public_url: String, gql_endpoint: String, export_url: String) -> Self {
        Self {
            public_url,
            export_url,
            thoth_client: Arc::new(ThothClient::new(gql_endpoint)),
            export_client: Client::new(),
        }
    }

    pub(crate) fn repository_url(&self) -> String {
        format!("{}/oai", self.public_url.trim_end_matches('/'))
    }

    pub(crate) async fn earliest(&self) -> ThothResult<Timestamp> {
        self.thoth_client.get_oai_earliest_works_updated().await
    }

    pub(crate) async fn latest(&self) -> ThothResult<Timestamp> {
        self.thoth_client.get_oai_latest_works_updated().await
    }

    pub(crate) async fn list_sets(&self) -> ThothResult<Vec<SetRecord>> {
        let publishers = self.thoth_client.get_publishers().await?;
        Ok(publishers.into_iter().map(Self::to_set_record).collect())
    }

    pub(crate) async fn get_record(
        &self,
        identifier: Uuid,
        metadata_prefix: MetadataPrefix,
    ) -> ThothResult<Work> {
        let work = self
            .thoth_client
            .get_work(identifier, Self::query_parameters())
            .await?;
        if metadata_prefix == MetadataPrefix::MarcXml && !Self::is_marcxml_record_candidate(&work) {
            return Err(ThothError::IncompleteMetadataRecord(
                metadata_prefix.as_str().to_string(),
                "Record cannot be disseminated as MARCXML".to_string(),
            ));
        }
        Ok(work)
    }

    pub(crate) async fn list_records(
        &self,
        metadata_prefix: MetadataPrefix,
        set: Option<String>,
        offset: i64,
        identifiers_only: bool,
    ) -> ThothResult<RecordPage> {
        let set_record = self.find_set(set.as_deref()).await?;
        let publishers = set_record
            .as_ref()
            .map(|set_record| vec![set_record.publisher_id]);
        let cursor = offset;

        if metadata_prefix == MetadataPrefix::MarcXml {
            let total = self
                .thoth_client
                .get_oai_book_count(publishers.clone())
                .await?;
            let mut records = Vec::new();
            let mut raw_offset = offset;

            while raw_offset < total && records.len() < PAGE_LIMIT as usize {
                let batch = self
                    .thoth_client
                    .get_oai_books(
                        publishers.clone(),
                        PAGE_LIMIT,
                        raw_offset,
                        Self::query_parameters(),
                    )
                    .await?;
                if batch.is_empty() {
                    break;
                }
                raw_offset += batch.len() as i64;
                for work in batch {
                    if Self::is_marcxml_record_candidate(&work) {
                        records.push(work);
                        if records.len() == PAGE_LIMIT as usize {
                            break;
                        }
                    }
                }
            }

            let next_token = (raw_offset < total && !records.is_empty()).then(|| {
                Self::encode_resumption_token(ResumptionToken {
                    offset: raw_offset,
                    metadata_prefix,
                    set,
                    identifiers_only,
                })
            });

            return Ok(RecordPage {
                records,
                cursor,
                complete_list_size: total,
                next_token,
            });
        }

        let total = self
            .thoth_client
            .get_oai_work_count(publishers.clone())
            .await?;
        let records = self
            .thoth_client
            .get_oai_works(publishers, PAGE_LIMIT, offset, Self::query_parameters())
            .await?;
        let next_offset = offset + records.len() as i64;
        let next_token = (next_offset < total && !records.is_empty()).then(|| {
            Self::encode_resumption_token(ResumptionToken {
                offset: next_offset,
                metadata_prefix,
                set,
                identifiers_only,
            })
        });

        Ok(RecordPage {
            records,
            cursor,
            complete_list_size: total,
            next_token,
        })
    }

    pub(crate) async fn get_marcxml_record(&self, work_id: Uuid) -> ThothResult<String> {
        let response = self
            .export_client
            .get(format!(
                "{}/specifications/marc21xml::thoth/work/{}",
                self.export_url.trim_end_matches('/'),
                work_id
            ))
            .send()
            .await
            .map_err(|error| ThothError::RequestError(error.to_string()))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|error| ThothError::RequestError(error.to_string()))?;
        if !status.is_success() {
            return Err(ThothError::RequestError(format!(
                "Export {}: {}",
                status.as_u16(),
                body
            )));
        }

        Self::extract_marc_record(&body)
    }

    pub(crate) fn oai_identifier(work_id: Uuid) -> String {
        format!("{RECORD_PREFIX}:{work_id}")
    }

    pub(crate) fn parse_oai_identifier(identifier: &str) -> ThothResult<Uuid> {
        identifier
            .rsplit_once(':')
            .map(|(_, value)| value)
            .ok_or(ThothError::InvalidUuid)
            .and_then(|value| Uuid::parse_str(value).map_err(|_| ThothError::InvalidUuid))
    }

    pub(crate) fn encode_resumption_token(token: ResumptionToken) -> String {
        URL_SAFE_NO_PAD.encode(serde_json::to_vec(&token).expect("resumption token to serialize"))
    }

    pub(crate) fn decode_resumption_token(value: &str) -> ThothResult<ResumptionToken> {
        let bytes = URL_SAFE_NO_PAD
            .decode(value)
            .map_err(|_| ThothError::RequestError("badResumptionToken".to_string()))?;
        serde_json::from_slice(&bytes)
            .map_err(|_| ThothError::RequestError("badResumptionToken".to_string()))
    }

    pub(crate) fn timestamp_xml(timestamp: Timestamp) -> String {
        timestamp.to_rfc3339().replace("+00:00", "Z")
    }

    pub(crate) fn set_spec(publisher_name: &str) -> String {
        publisher_name
            .chars()
            .filter(|ch| ch.is_alphanumeric() || ch.is_whitespace() || *ch == '_')
            .collect::<String>()
            .to_lowercase()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join("-")
    }

    pub(crate) fn is_marcxml_record_candidate(work: &Work) -> bool {
        !work.contributions.is_empty()
            && !work.languages.is_empty()
            && work
                .publications
                .iter()
                .any(|publication| publication.isbn.is_some())
    }

    pub(crate) fn query_parameters() -> QueryParameters {
        QueryParameters::new()
            .with_canonical_abstracts_only()
            .with_canonical_title_only()
            .with_issues()
            .with_languages()
            .with_publications()
            .with_subjects()
            .with_fundings()
            .with_relations()
    }

    async fn find_set(&self, set_spec: Option<&str>) -> ThothResult<Option<SetRecord>> {
        let Some(set_spec) = set_spec else {
            return Ok(None);
        };

        let sets = self.list_sets().await?;
        sets.into_iter()
            .find(|set_record| set_record.spec == set_spec)
            .map(Some)
            .ok_or(ThothError::EntityNotFound)
    }

    fn to_set_record(publisher: Publisher) -> SetRecord {
        let spec = Self::set_spec(&publisher.publisher_name);
        SetRecord {
            publisher_id: publisher.publisher_id,
            spec,
            name: publisher.publisher_name,
        }
    }

    fn extract_marc_record(body: &str) -> ThothResult<String> {
        let mut reader = Reader::from_str(body);
        reader.config_mut().trim_text(false);
        let mut writer = Writer::new(Vec::new());
        let mut capture_depth = 0usize;
        let mut capturing = false;

        loop {
            match reader.read_event() {
                Ok(Event::Start(event)) => {
                    let is_record = event.local_name().as_ref() == b"record";
                    if capturing {
                        capture_depth += 1;
                        writer
                            .write_event(Event::Start(event.to_owned()))
                            .map_err(|error| {
                                ThothError::InternalError(format!(
                                    "Could not write MARCXML: {error}"
                                ))
                            })?;
                    } else if is_record {
                        capturing = true;
                        capture_depth = 1;
                        writer
                            .write_event(Event::Start(event.to_owned()))
                            .map_err(|error| {
                                ThothError::InternalError(format!(
                                    "Could not write MARCXML: {error}"
                                ))
                            })?;
                    }
                }
                Ok(Event::Empty(event)) => {
                    let is_record = event.local_name().as_ref() == b"record";
                    if capturing || is_record {
                        writer
                            .write_event(Event::Empty(event.to_owned()))
                            .map_err(|error| {
                                ThothError::InternalError(format!(
                                    "Could not write MARCXML: {error}"
                                ))
                            })?;
                        if is_record && !capturing {
                            return String::from_utf8(writer.into_inner()).map_err(|_| {
                                ThothError::InternalError("Could not parse MARCXML".to_string())
                            });
                        }
                    }
                }
                Ok(Event::End(event)) => {
                    if capturing {
                        writer
                            .write_event(Event::End(event.to_owned()))
                            .map_err(|error| {
                                ThothError::InternalError(format!(
                                    "Could not write MARCXML: {error}"
                                ))
                            })?;
                        capture_depth -= 1;
                        if capture_depth == 0 {
                            return String::from_utf8(writer.into_inner()).map_err(|_| {
                                ThothError::InternalError("Could not parse MARCXML".to_string())
                            });
                        }
                    }
                }
                Ok(Event::Text(event)) => {
                    if capturing {
                        writer
                            .write_event(Event::Text(event.to_owned()))
                            .map_err(|error| {
                                ThothError::InternalError(format!(
                                    "Could not write MARCXML: {error}"
                                ))
                            })?;
                    }
                }
                Ok(Event::CData(event)) => {
                    if capturing {
                        writer
                            .write_event(Event::CData(event.to_owned()))
                            .map_err(|error| {
                                ThothError::InternalError(format!(
                                    "Could not write MARCXML: {error}"
                                ))
                            })?;
                    }
                }
                Ok(Event::Comment(event)) => {
                    if capturing {
                        writer
                            .write_event(Event::Comment(event.to_owned()))
                            .map_err(|error| {
                                ThothError::InternalError(format!(
                                    "Could not write MARCXML: {error}"
                                ))
                            })?;
                    }
                }
                Ok(Event::PI(event)) => {
                    if capturing {
                        writer
                            .write_event(Event::PI(event.to_owned()))
                            .map_err(|error| {
                                ThothError::InternalError(format!(
                                    "Could not write MARCXML: {error}"
                                ))
                            })?;
                    }
                }
                Ok(Event::Decl(_)) | Ok(Event::DocType(_)) => {}
                Ok(Event::Eof) => {
                    return Err(ThothError::InternalError(
                        "No marc:record element found".to_string(),
                    ));
                }
                Err(error) => {
                    return Err(ThothError::InternalError(format!(
                        "Could not parse MARCXML: {error}"
                    )));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_spec_normalizes_publisher_name() {
        assert_eq!(
            OaiService::set_spec("Punctum Books, Inc."),
            "punctum-books-inc"
        );
        assert_eq!(
            OaiService::set_spec("Open Access_ Press"),
            "open-access_-press"
        );
    }

    #[test]
    fn oai_identifier_round_trip() {
        let work_id = Uuid::parse_str("5a08ff03-7d53-42a9-bfb5-7fc81c099c52").unwrap();
        let identifier = OaiService::oai_identifier(work_id);

        assert_eq!(
            OaiService::parse_oai_identifier(&identifier).unwrap(),
            work_id
        );
    }

    #[test]
    fn resumption_token_round_trip() {
        let token = ResumptionToken {
            offset: 150,
            metadata_prefix: MetadataPrefix::MarcXml,
            set: Some("open-book-publishers".to_string()),
            identifiers_only: true,
        };

        let encoded = OaiService::encode_resumption_token(token.clone());

        assert_eq!(
            OaiService::decode_resumption_token(&encoded).unwrap(),
            token
        );
    }

    #[test]
    fn extract_marc_record_returns_record_element() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<collection xmlns="http://www.loc.gov/MARC21/slim">
  <record>
    <leader>00000nam a2200000 i 4500</leader>
    <controlfield tag="001">123</controlfield>
  </record>
</collection>"#;
        let record = OaiService::extract_marc_record(xml).unwrap();

        assert!(record.starts_with("<record"));
        assert!(record.contains("<leader>00000nam a2200000 i 4500</leader>"));
        assert!(record.contains("<controlfield tag=\"001\">123</controlfield>"));
        assert!(!record.contains("<collection"));
    }
}
