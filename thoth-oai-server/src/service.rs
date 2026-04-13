use std::sync::Arc;

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chrono::{DateTime, NaiveDate, Utc};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum DatestampGranularity {
    Day,
    Second,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ResumptionToken {
    pub offset: i64,
    pub metadata_prefix: MetadataPrefix,
    pub set: Option<String>,
    pub identifiers_only: bool,
    pub from: Option<String>,
    pub until: Option<String>,
    pub granularity: Option<DatestampGranularity>,
    pub scan_offset: Option<i64>,
    pub returned_count: Option<i64>,
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
    pub complete_list_size: Option<i64>,
    pub next_token: Option<String>,
    pub terminal_resumption_token: bool,
}

#[derive(Debug, Clone)]
struct DatestampBounds {
    from: Option<Timestamp>,
    until: Option<Timestamp>,
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
        self.public_url.trim_end_matches('/').to_string()
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
        _metadata_prefix: MetadataPrefix,
    ) -> ThothResult<Work> {
        let work = self
            .thoth_client
            .get_work(identifier, Self::query_parameters())
            .await?;
        Ok(work)
    }

    pub(crate) async fn list_records(
        &self,
        token: &ResumptionToken,
        resumed: bool,
    ) -> ThothResult<RecordPage> {
        let set_record = self.find_set(token.set.as_deref()).await?;
        let publishers = set_record
            .as_ref()
            .map(|set_record| vec![set_record.publisher_id]);
        let bounds = Self::build_datestamp_bounds(token)?;
        let date_filter_active = bounds.is_some();
        let filtering_active = bounds.is_some() || token.metadata_prefix == MetadataPrefix::MarcXml;
        let scan_offset = token.scan_offset.unwrap_or(token.offset);
        let returned_count = token.returned_count.unwrap_or(token.offset);

        let total = if token.metadata_prefix == MetadataPrefix::MarcXml {
            self.thoth_client
                .get_oai_book_count(publishers.clone())
                .await?
        } else {
            self.thoth_client
                .get_oai_work_count(publishers.clone())
                .await?
        };

        let mut records = Vec::new();
        let mut raw_offset = scan_offset;
        while raw_offset < total && records.len() < PAGE_LIMIT as usize {
            let batch = self
                .fetch_record_batch(token.metadata_prefix, publishers.clone(), raw_offset)
                .await?;

            if batch.is_empty() {
                break;
            }

            let batch_len = batch.len() as i64;
            let mut consumed = 0i64;
            for work in batch {
                consumed += 1;
                if !self
                    .matches_record(&work, token.metadata_prefix, bounds.as_ref())
                    .await?
                {
                    continue;
                }
                records.push(work);
                if records.len() == PAGE_LIMIT as usize {
                    break;
                }
            }
            raw_offset += consumed;
            if consumed < batch_len {
                break;
            }
        }

        let has_next_page = if raw_offset < total && !records.is_empty() {
            if filtering_active {
                self.has_more_matching_records(
                    token.metadata_prefix,
                    publishers.clone(),
                    raw_offset,
                    total,
                    bounds.as_ref(),
                )
                .await?
            } else {
                true
            }
        } else {
            false
        };

        let next_token = has_next_page.then(|| {
            Self::encode_resumption_token(ResumptionToken {
                offset: raw_offset,
                metadata_prefix: token.metadata_prefix,
                set: token.set.clone(),
                identifiers_only: token.identifiers_only,
                from: token.from.clone(),
                until: token.until.clone(),
                granularity: token.granularity,
                scan_offset: Some(raw_offset),
                returned_count: Some(returned_count + records.len() as i64),
            })
        });

        let terminal_resumption_token = resumed && next_token.is_none();
        Ok(RecordPage {
            records,
            cursor: returned_count,
            complete_list_size: (!date_filter_active
                && token.metadata_prefix != MetadataPrefix::MarcXml)
                .then_some(total),
            next_token,
            terminal_resumption_token,
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
            .strip_prefix(&format!("{RECORD_PREFIX}:"))
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

    pub(crate) fn query_parameters() -> QueryParameters {
        QueryParameters::new()
            .with_all_abstracts()
            .with_all_titles()
            .with_issues()
            .with_languages()
            .with_publications()
            .with_subjects()
            .with_fundings()
            .with_relations()
            .with_references()
    }

    pub(crate) async fn has_marcxml_dissemination(&self, work_id: Uuid) -> ThothResult<bool> {
        match self.get_marcxml_record(work_id).await {
            Ok(_) => Ok(true),
            Err(error) if Self::is_transient_export_error(&error) => Err(error),
            Err(_) => Ok(false),
        }
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

    async fn fetch_record_batch(
        &self,
        metadata_prefix: MetadataPrefix,
        publishers: Option<Vec<Uuid>>,
        raw_offset: i64,
    ) -> ThothResult<Vec<Work>> {
        if metadata_prefix == MetadataPrefix::MarcXml {
            self.thoth_client
                .get_oai_books(publishers, PAGE_LIMIT, raw_offset, Self::query_parameters())
                .await
        } else {
            self.thoth_client
                .get_oai_works(publishers, PAGE_LIMIT, raw_offset, Self::query_parameters())
                .await
        }
    }

    async fn matches_record(
        &self,
        work: &Work,
        metadata_prefix: MetadataPrefix,
        bounds: Option<&DatestampBounds>,
    ) -> ThothResult<bool> {
        if !Self::matches_datestamp_filter(work.updated_at_with_relations, bounds)? {
            return Ok(false);
        }
        if metadata_prefix == MetadataPrefix::MarcXml {
            return self.has_marcxml_dissemination(work.work_id).await;
        }
        Ok(true)
    }

    async fn has_more_matching_records(
        &self,
        metadata_prefix: MetadataPrefix,
        publishers: Option<Vec<Uuid>>,
        mut raw_offset: i64,
        total: i64,
        bounds: Option<&DatestampBounds>,
    ) -> ThothResult<bool> {
        while raw_offset < total {
            let batch = self
                .fetch_record_batch(metadata_prefix, publishers.clone(), raw_offset)
                .await?;

            if batch.is_empty() {
                break;
            }

            raw_offset += batch.len() as i64;
            for work in batch {
                if self.matches_record(&work, metadata_prefix, bounds).await? {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    fn build_datestamp_bounds(token: &ResumptionToken) -> ThothResult<Option<DatestampBounds>> {
        if token.from.is_none() && token.until.is_none() {
            return Ok(None);
        }

        let granularity = token.granularity.ok_or_else(|| {
            ThothError::RequestError("badResumptionToken: missing date granularity".to_string())
        })?;

        let from = token
            .from
            .as_deref()
            .map(|value| Self::parse_datestamp_boundary(value, granularity, true))
            .transpose()?;
        let until = token
            .until
            .as_deref()
            .map(|value| Self::parse_datestamp_boundary(value, granularity, false))
            .transpose()?;

        Ok(Some(DatestampBounds { from, until }))
    }

    fn parse_datestamp_boundary(
        value: &str,
        granularity: DatestampGranularity,
        is_from: bool,
    ) -> ThothResult<Timestamp> {
        match granularity {
            DatestampGranularity::Day => {
                let date = NaiveDate::parse_from_str(value, "%Y-%m-%d").map_err(|_| {
                    ThothError::RequestError(
                        "badResumptionToken: invalid day datestamp".to_string(),
                    )
                })?;
                let value = if is_from {
                    format!("{date}T00:00:00Z")
                } else {
                    format!("{date}T23:59:59Z")
                };
                Timestamp::parse_from_rfc3339(&value)
            }
            DatestampGranularity::Second => {
                let datetime =
                    DateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%SZ").map_err(|_| {
                        ThothError::RequestError(
                            "badResumptionToken: invalid second datestamp".to_string(),
                        )
                    })?;
                let canonical = datetime
                    .with_timezone(&Utc)
                    .format("%Y-%m-%dT%H:%M:%SZ")
                    .to_string();
                Timestamp::parse_from_rfc3339(&canonical)
            }
        }
    }

    fn matches_datestamp_filter(
        datestamp: Timestamp,
        bounds: Option<&DatestampBounds>,
    ) -> ThothResult<bool> {
        let Some(bounds) = bounds else {
            return Ok(true);
        };
        if let Some(from) = bounds.from {
            if datestamp < from {
                return Ok(false);
            }
        }
        if let Some(until) = bounds.until {
            if datestamp > until {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn is_transient_export_error(error: &ThothError) -> bool {
        let ThothError::RequestError(message) = error else {
            return false;
        };
        let message = message.to_ascii_lowercase();
        let has_transient_status = [429, 500, 502, 503, 504]
            .iter()
            .any(|status| message.contains(&format!("export {status}")));
        let has_network_failure = [
            "timed out",
            "timeout",
            "connection refused",
            "connection reset",
            "error sending request",
            "temporary failure",
            "dns error",
            "failed to lookup address",
        ]
        .iter()
        .any(|needle| message.contains(needle));
        has_transient_status || has_network_failure
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
            from: Some("2024-01-01".to_string()),
            until: Some("2024-12-31".to_string()),
            granularity: Some(DatestampGranularity::Day),
            scan_offset: Some(200),
            returned_count: Some(75),
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

    #[test]
    fn parse_oai_identifier_requires_exact_prefix() {
        assert!(OaiService::parse_oai_identifier(
            "oai:another.repo:5a08ff03-7d53-42a9-bfb5-7fc81c099c52"
        )
        .is_err());
    }

    #[test]
    fn decode_legacy_resumption_token_without_new_fields() {
        let legacy = URL_SAFE_NO_PAD.encode(
            serde_json::to_vec(&serde_json::json!({
                "offset": 100,
                "metadata_prefix": "OaiDc",
                "set": null,
                "identifiers_only": false
            }))
            .unwrap(),
        );
        let token = OaiService::decode_resumption_token(&legacy).unwrap();
        assert_eq!(token.offset, 100);
        assert_eq!(token.scan_offset, None);
        assert_eq!(token.returned_count, None);
        assert_eq!(token.from, None);
        assert_eq!(token.until, None);
        assert_eq!(token.granularity, None);
    }
}
