use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
};

use oai_pmh::core::MetadataPrefix;
use quick_xml::{events::Event, Reader, Writer};
use reqwest::Client;
use thoth_api::model::Timestamp;
use thoth_client::{Publisher, QueryParameters, ThothClient, Work};
use thoth_errors::{ThothError, ThothResult};
use uuid::Uuid;

pub(crate) const RECORD_PREFIX: &str = "oai:thoth.pub";
pub(crate) const REPOSITORY_NAME: &str = "Thoth OAI-PMH Repository";
pub(crate) const ADMIN_EMAIL: &str = "support@thoth.pub";
pub(crate) const SAMPLE_ID: &str = "5a08ff03-7d53-42a9-bfb5-7fc81c099c52";
#[cfg(test)]
pub(crate) const PAGE_LIMIT: i64 = 50;

const OAI_DC_SPEC: &str = "dublin_core::thoth";
const OAI_OPENAIRE_SPEC: &str = "openaire::thoth";
const MARCXML_SPEC: &str = "marc21xml::thoth";
const DELEGATED_RECORD_CACHE_LIMIT: usize = 2048;
type DelegatedRecordCacheKey = (Uuid, &'static str);

#[derive(Default)]
struct DelegatedRecordCache {
    entries: HashMap<DelegatedRecordCacheKey, String>,
    insertion_order: VecDeque<DelegatedRecordCacheKey>,
}

impl DelegatedRecordCache {
    fn get(&self, key: &DelegatedRecordCacheKey) -> Option<String> {
        self.entries.get(key).cloned()
    }

    fn insert(&mut self, key: DelegatedRecordCacheKey, value: String) {
        match self.entries.entry(key) {
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                entry.insert(value);
                return;
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                self.insertion_order.push_back(*entry.key());
                entry.insert(value);
            }
        }

        while self.entries.len() > DELEGATED_RECORD_CACHE_LIMIT {
            if let Some(oldest_key) = self.insertion_order.pop_front() {
                self.entries.remove(&oldest_key);
            } else {
                break;
            }
        }
    }
}

#[derive(Clone)]
pub(crate) struct OaiService {
    public_url: String,
    export_url: String,
    thoth_client: Arc<ThothClient>,
    export_client: Client,
    delegated_record_cache: Arc<Mutex<DelegatedRecordCache>>,
}

#[derive(Debug, Clone)]
pub(crate) struct SetRecord {
    pub publisher_id: Uuid,
    pub spec: String,
    pub name: String,
}

impl OaiService {
    pub(crate) fn new(public_url: String, gql_endpoint: String, export_url: String) -> Self {
        Self {
            public_url,
            export_url,
            thoth_client: Arc::new(ThothClient::new(gql_endpoint)),
            export_client: Client::new(),
            delegated_record_cache: Arc::new(Mutex::new(DelegatedRecordCache::default())),
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
        self.thoth_client
            .get_work(identifier, Self::query_parameters())
            .await
    }

    pub(crate) async fn list_source_count(
        &self,
        metadata_prefix: MetadataPrefix,
        set_spec: Option<&str>,
    ) -> ThothResult<i64> {
        let publishers = self.publishers_for_set(set_spec).await?;
        if metadata_prefix == MetadataPrefix::MarcXml {
            self.thoth_client.get_oai_book_count(publishers).await
        } else {
            self.thoth_client.get_oai_work_count(publishers).await
        }
    }

    pub(crate) async fn list_source_batch(
        &self,
        metadata_prefix: MetadataPrefix,
        set_spec: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> ThothResult<Vec<Work>> {
        let publishers = self.publishers_for_set(set_spec).await?;
        if metadata_prefix == MetadataPrefix::MarcXml {
            self.thoth_client
                .get_oai_books(publishers, limit, offset, Self::query_parameters())
                .await
        } else {
            self.thoth_client
                .get_oai_works(publishers, limit, offset, Self::query_parameters())
                .await
        }
    }

    pub(crate) async fn get_marcxml_record(&self, work_id: Uuid) -> ThothResult<String> {
        self.get_delegated_record(
            work_id,
            MetadataPrefix::MarcXml,
            MARCXML_SPEC,
            b"record",
            "MARCXML",
        )
            .await
    }

    pub(crate) async fn get_oai_dc_record(&self, work_id: Uuid) -> ThothResult<String> {
        self.get_delegated_record(
            work_id,
            MetadataPrefix::OaiDc,
            OAI_DC_SPEC,
            b"dc",
            "Dublin Core",
        )
            .await
    }

    pub(crate) async fn get_oai_openaire_record(&self, work_id: Uuid) -> ThothResult<String> {
        self.get_delegated_record(
            work_id,
            MetadataPrefix::OaiOpenaire,
            OAI_OPENAIRE_SPEC,
            b"resource",
            "OpenAIRE",
        )
            .await
    }

    pub(crate) async fn has_metadata_dissemination(
        &self,
        work_id: Uuid,
        metadata_prefix: MetadataPrefix,
    ) -> ThothResult<bool> {
        let dissemination = match metadata_prefix {
            MetadataPrefix::OaiDc => self.get_oai_dc_record(work_id).await,
            MetadataPrefix::OaiOpenaire => self.get_oai_openaire_record(work_id).await,
            MetadataPrefix::MarcXml => self.get_marcxml_record(work_id).await,
        };

        match dissemination {
            Ok(_) => Ok(true),
            Err(error) if Self::is_transient_export_error(&error) => Err(error),
            Err(_) => Ok(false),
        }
    }

    async fn get_delegated_record(
        &self,
        work_id: Uuid,
        metadata_prefix: MetadataPrefix,
        specification: &str,
        element_local_name: &[u8],
        format_name: &str,
    ) -> ThothResult<String> {
        let cache_key = (work_id, metadata_prefix.as_str());
        if let Some(record) = self.get_cached_delegated_record(&cache_key) {
            return Ok(record);
        }

        let response = self
            .export_client
            .get(format!(
                "{}/specifications/{}/work/{}",
                self.export_url.trim_end_matches('/'),
                specification,
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

        let record = Self::extract_xml_element(&body, element_local_name, format_name)?;
        self.cache_delegated_record(cache_key, record.clone());
        Ok(record)
    }

    fn get_cached_delegated_record(&self, key: &DelegatedRecordCacheKey) -> Option<String> {
        self.delegated_record_cache
            .lock()
            .ok()
            .and_then(|cache| cache.get(key))
    }

    fn cache_delegated_record(&self, key: DelegatedRecordCacheKey, value: String) {
        if let Ok(mut cache) = self.delegated_record_cache.lock() {
            cache.insert(key, value);
        }
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

    async fn publishers_for_set(&self, set_spec: Option<&str>) -> ThothResult<Option<Vec<Uuid>>> {
        let set = self.find_set(set_spec).await?;
        Ok(set.map(|set_record| vec![set_record.publisher_id]))
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
        SetRecord {
            publisher_id: publisher.publisher_id,
            spec: Self::set_spec(&publisher.publisher_name),
            name: publisher.publisher_name,
        }
    }

    fn extract_xml_element(
        body: &str,
        element_local_name: &[u8],
        format_name: &str,
    ) -> ThothResult<String> {
        let mut reader = Reader::from_str(body);
        reader.config_mut().trim_text(false);
        let mut writer = Writer::new(Vec::new());
        let mut capture_depth = 0usize;
        let mut capturing = false;

        loop {
            match reader.read_event() {
                Ok(Event::Start(event)) => {
                    let is_record = event.local_name().as_ref() == element_local_name;
                    if capturing {
                        capture_depth += 1;
                        writer
                            .write_event(Event::Start(event.to_owned()))
                            .map_err(|error| {
                                ThothError::InternalError(format!(
                                    "Could not write {format_name}: {error}"
                                ))
                            })?;
                    } else if is_record {
                        capturing = true;
                        capture_depth = 1;
                        writer
                            .write_event(Event::Start(event.to_owned()))
                            .map_err(|error| {
                                ThothError::InternalError(format!(
                                    "Could not write {format_name}: {error}"
                                ))
                            })?;
                    }
                }
                Ok(Event::Empty(event)) => {
                    let is_record = event.local_name().as_ref() == element_local_name;
                    if capturing || is_record {
                        writer
                            .write_event(Event::Empty(event.to_owned()))
                            .map_err(|error| {
                                ThothError::InternalError(format!(
                                    "Could not write {format_name}: {error}"
                                ))
                            })?;
                        if is_record && !capturing {
                            return String::from_utf8(writer.into_inner()).map_err(|_| {
                                ThothError::InternalError(format!(
                                    "Could not parse {format_name} XML"
                                ))
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
                                    "Could not write {format_name}: {error}"
                                ))
                            })?;
                        capture_depth -= 1;
                        if capture_depth == 0 {
                            return String::from_utf8(writer.into_inner()).map_err(|_| {
                                ThothError::InternalError(format!(
                                    "Could not parse {format_name} XML"
                                ))
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
                                    "Could not write {format_name}: {error}"
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
                                    "Could not write {format_name}: {error}"
                                ))
                            })?;
                    }
                }
                Ok(Event::GeneralRef(event)) => {
                    if capturing {
                        writer
                            .write_event(Event::GeneralRef(event.to_owned()))
                            .map_err(|error| {
                                ThothError::InternalError(format!(
                                    "Could not write {format_name}: {error}"
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
                                    "Could not write {format_name}: {error}"
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
                                    "Could not write {format_name}: {error}"
                                ))
                            })?;
                    }
                }
                Ok(Event::Decl(_)) | Ok(Event::DocType(_)) => {}
                Ok(Event::Eof) => {
                    return Err(ThothError::InternalError(format!(
                        "No {format_name} element found"
                    )));
                }
                Err(error) => {
                    return Err(ThothError::InternalError(format!(
                        "Could not parse {format_name} XML: {error}"
                    )));
                }
            }
        }
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
    fn extract_xml_element_returns_record_element() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<collection xmlns="http://www.loc.gov/MARC21/slim">
  <record>
    <leader>00000nam a2200000 i 4500</leader>
    <controlfield tag="001">123</controlfield>
  </record>
</collection>"#;
        let record = OaiService::extract_xml_element(xml, b"record", "MARCXML").unwrap();

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
}
