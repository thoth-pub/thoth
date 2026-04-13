#![recursion_limit = "512"]

mod metadata;
mod service;

use std::{
    collections::HashMap,
    io::{self, Write},
    time::Duration,
};

use actix_cors::Cors;
use actix_web::{
    http::header, middleware::Logger, web, App, HttpRequest, HttpResponse, HttpServer,
};
use chrono::{DateTime, NaiveDate, Utc};
use flate2::{write::GzEncoder, Compression};
use quick_xml::escape::escape;
use service::{
    DatestampGranularity, MetadataPrefix, OaiService, RecordPage, ResumptionToken, ADMIN_EMAIL,
    RECORD_PREFIX, REPOSITORY_NAME, SAMPLE_ID,
};
use thoth_errors::ThothError;
use uuid::Uuid;

const LOG_FORMAT: &str = r#"%{r}a %a "%r" %s %b "%{Referer}i" "%{User-Agent}i" %T"#;
const XSL_STYLESHEET: &str = include_str!("../assets/oai2.xsl");
const METADATA_RIGHTS_STATEMENT: &str = "Metadata is licensed under the terms of Creative Commons CC0 1.0 Universal: https://creativecommons.org/publicdomain/zero/1.0/.";
const METADATA_RIGHTS_URI: &str = "https://creativecommons.org/publicdomain/zero/1.0/";
#[cfg(test)]
const DEFAULT_RETRY_AFTER_SECONDS: u64 = 30;

#[derive(Clone)]
struct AppState {
    service: OaiService,
    retry_after_seconds: u64,
}

#[derive(Debug)]
struct ProtocolError {
    code: &'static str,
    message: String,
}

enum HandlerError {
    Protocol(ProtocolError),
    Internal(ThothError),
}

type HandlerResult<T> = Result<T, HandlerError>;

impl From<ProtocolError> for HandlerError {
    fn from(value: ProtocolError) -> Self {
        Self::Protocol(value)
    }
}

#[derive(Debug, Default, Clone)]
struct ParsedParams {
    values: HashMap<String, String>,
    has_repeated: bool,
}

#[derive(Debug, Clone)]
struct ParsedListRequest {
    token: ResumptionToken,
    resumed: bool,
}

async fn index() -> HttpResponse {
    HttpResponse::Found()
        .append_header(("Location", "/oai"))
        .finish()
}

async fn stylesheet() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/xsl; charset=utf-8")
        .body(XSL_STYLESHEET)
}

async fn oai_get(request: HttpRequest, state: web::Data<AppState>) -> HttpResponse {
    let params = parse_form_encoded(request.query_string());
    oai_with_params(request, params, state).await
}

async fn oai_post(
    request: HttpRequest,
    body: web::Bytes,
    state: web::Data<AppState>,
) -> HttpResponse {
    let mut params = parse_form_encoded(request.query_string());
    match std::str::from_utf8(&body) {
        Ok(body) => params.merge(parse_form_encoded(body)),
        Err(_) => {
            return xml_response(
                &request,
                error_document(
                    &state.service,
                    &params.values,
                    "badArgument",
                    "Invalid UTF-8 request body",
                ),
            )
        }
    }
    oai_with_params(request, params, state).await
}

async fn oai_with_params(
    request: HttpRequest,
    params: ParsedParams,
    state: web::Data<AppState>,
) -> HttpResponse {
    if params.has_repeated {
        return xml_response(
            &request,
            error_document(
                &state.service,
                &params.values,
                "badArgument",
                "The request includes repeated arguments",
            ),
        );
    }
    match handle_oai_request(&request, &params.values, &state.service).await {
        Ok(body) => xml_response(
            &request,
            success_document(&state.service, &params.values, &body),
        ),
        Err(HandlerError::Protocol(error)) => xml_response(
            &request,
            error_document(&state.service, &params.values, error.code, &error.message),
        ),
        Err(HandlerError::Internal(error)) => {
            log::error!("OAI request failed: {error}");
            if is_transient_upstream_error(&error) {
                transient_service_unavailable(state.retry_after_seconds)
            } else {
                HttpResponse::InternalServerError()
                    .content_type("text/plain; charset=utf-8")
                    .body("Internal Server Error")
            }
        }
    }
}

fn is_transient_upstream_error(error: &ThothError) -> bool {
    let message = match error {
        ThothError::RequestError(message) | ThothError::GraphqlError(message) => {
            message.to_ascii_lowercase()
        }
        _ => return false,
    };

    let has_transient_status = [500, 502, 503, 504, 429].iter().any(|status| {
        message.contains(&format!("graphql {status}"))
            || message.contains(&format!("export {status}"))
    });
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

fn transient_service_unavailable(retry_after_seconds: u64) -> HttpResponse {
    HttpResponse::ServiceUnavailable()
        .insert_header((header::RETRY_AFTER, retry_after_seconds.to_string()))
        .content_type("text/plain; charset=utf-8")
        .body("Service Unavailable")
}

impl ParsedParams {
    fn merge(&mut self, other: ParsedParams) {
        self.has_repeated = self.has_repeated || other.has_repeated;
        for (key, value) in other.values {
            if self.values.insert(key, value).is_some() {
                self.has_repeated = true;
            }
        }
    }
}

fn parse_form_encoded(input: &str) -> ParsedParams {
    let mut parsed = ParsedParams::default();
    for (key, value) in url::form_urlencoded::parse(input.as_bytes()) {
        if parsed
            .values
            .insert(key.into_owned(), value.into_owned())
            .is_some()
        {
            parsed.has_repeated = true;
        }
    }
    parsed
}

async fn not_found() -> HttpResponse {
    HttpResponse::NotFound()
        .content_type("text/html; charset=utf-8")
        .body(
            r##"<!DOCTYPE html>
<html>
<head>
  <title>404 - Page Not Found</title>
  <link rel="shortcut icon" href="https://cdn.thoth.pub/favicons/thoth-head-20260331/transparent/favicon.ico" />
  <link rel="apple-touch-icon" sizes="57x57" href="https://cdn.thoth.pub/favicons/thoth-head-20260331/transparent/apple-icon-57x57.png">
  <link rel="apple-touch-icon" sizes="60x60" href="https://cdn.thoth.pub/favicons/thoth-head-20260331/transparent/apple-icon-60x60.png">
  <link rel="apple-touch-icon" sizes="72x72" href="https://cdn.thoth.pub/favicons/thoth-head-20260331/transparent/apple-icon-72x72.png">
  <link rel="apple-touch-icon" sizes="76x76" href="https://cdn.thoth.pub/favicons/thoth-head-20260331/transparent/apple-icon-76x76.png">
  <link rel="apple-touch-icon" sizes="114x114" href="https://cdn.thoth.pub/favicons/thoth-head-20260331/transparent/apple-icon-114x114.png">
  <link rel="apple-touch-icon" sizes="120x120" href="https://cdn.thoth.pub/favicons/thoth-head-20260331/transparent/apple-icon-120x120.png">
  <link rel="apple-touch-icon" sizes="144x144" href="https://cdn.thoth.pub/favicons/thoth-head-20260331/transparent/apple-icon-144x144.png">
  <link rel="apple-touch-icon" sizes="152x152" href="https://cdn.thoth.pub/favicons/thoth-head-20260331/transparent/apple-icon-152x152.png">
  <link rel="apple-touch-icon" sizes="180x180" href="https://cdn.thoth.pub/favicons/thoth-head-20260331/transparent/apple-icon-180x180.png">
  <link rel="icon" type="image/png" sizes="192x192" href="https://cdn.thoth.pub/favicons/thoth-head-20260331/transparent/android-icon-192x192.png">
  <link rel="icon" type="image/png" sizes="32x32" href="https://cdn.thoth.pub/favicons/thoth-head-20260331/transparent/favicon-32x32.png">
  <link rel="icon" type="image/png" sizes="96x96" href="https://cdn.thoth.pub/favicons/thoth-head-20260331/transparent/favicon-96x96.png">
  <link rel="icon" type="image/png" sizes="16x16" href="https://cdn.thoth.pub/favicons/thoth-head-20260331/transparent/favicon-16x16.png">
  <link rel="manifest" href="https://cdn.thoth.pub/favicons/thoth-head-20260331/transparent/manifest.json">
  <meta name="msapplication-TileColor" content="#FFDD57">
  <meta name="msapplication-TileImage" content="https://cdn.thoth.pub/favicons/thoth-head-20260331/transparent/ms-icon-144x144.png">
  <meta name="theme-color" content="#FFDD57">
  <style>
    body { font-family: Arial, sans-serif; margin: 40px; text-align: center; }
    h1 { color: #666; }
  </style>
</head>
<body>
  <h1>404 - Page Not Found</h1>
  <p>The requested page was not found.</p>
  <p><a href="/oai">OAI-PMH Interface</a></p>
</body>
</html>"##,
        )
}

async fn handle_oai_request(
    _request: &HttpRequest,
    params: &HashMap<String, String>,
    service: &OaiService,
) -> HandlerResult<String> {
    let verb = params
        .get("verb")
        .map(String::as_str)
        .ok_or_else(|| bad_verb("Missing verb parameter"))?;

    match verb {
        "Identify" => {
            require_only(params, &["verb"])?;
            let earliest = service.earliest().await.map_err(HandlerError::Internal)?;
            let latest = service.latest().await.map_err(HandlerError::Internal)?;
            Ok(render_identify(service, earliest, latest))
        }
        "ListMetadataFormats" => {
            require_only(params, &["verb", "identifier"])?;
            let mut prefixes = vec![
                MetadataPrefix::OaiDc,
                MetadataPrefix::OaiOpenaire,
                MetadataPrefix::MarcXml,
            ];
            if let Some(identifier) = params.get("identifier") {
                let work_id = parse_identifier_for_lookup(identifier)?;
                service
                    .get_record(work_id, MetadataPrefix::OaiDc)
                    .await
                    .map_err(map_get_record_error(MetadataPrefix::OaiDc))?;
                prefixes = vec![MetadataPrefix::OaiDc, MetadataPrefix::OaiOpenaire];
                if service
                    .has_marcxml_dissemination(work_id)
                    .await
                    .map_err(HandlerError::Internal)?
                {
                    prefixes.push(MetadataPrefix::MarcXml);
                }
                if prefixes.is_empty() {
                    return Err(ProtocolError {
                        code: "noMetadataFormats",
                        message: "No metadata formats are available for this identifier"
                            .to_string(),
                    }
                    .into());
                }
            }
            Ok(render_list_metadata_formats(&prefixes))
        }
        "ListSets" => {
            if params.contains_key("resumptionToken") {
                return Err(ProtocolError {
                    code: "badResumptionToken",
                    message: "This repository does not support set resumption tokens".to_string(),
                }
                .into());
            }
            require_only(params, &["verb"])?;
            let sets = service.list_sets().await.map_err(HandlerError::Internal)?;
            Ok(render_list_sets(&sets))
        }
        "GetRecord" => {
            require_only(params, &["verb", "identifier", "metadataPrefix"])?;
            let identifier = params
                .get("identifier")
                .ok_or_else(|| bad_argument("Missing identifier parameter"))?;
            let metadata_prefix = params
                .get("metadataPrefix")
                .ok_or_else(|| bad_argument("Missing metadataPrefix parameter"))?;
            let work_id = parse_identifier_for_lookup(identifier)?;
            let metadata_prefix = parse_metadata_prefix(metadata_prefix)?;
            let work = service
                .get_record(work_id, metadata_prefix)
                .await
                .map_err(map_get_record_error(metadata_prefix))?;
            Ok(render_get_record(service, &work, metadata_prefix).await?)
        }
        "ListIdentifiers" => {
            validate_list_verb(params)?;
            let parsed = parse_list_token(params, true)?;
            let page = service
                .list_records(&parsed.token, parsed.resumed)
                .await
                .map_err(map_list_error)?;
            if page.records.is_empty() && !parsed.resumed {
                return Err(HandlerError::Protocol(no_records_match()));
            }
            Ok(render_list_identifiers(&page))
        }
        "ListRecords" => {
            validate_list_verb(params)?;
            let parsed = parse_list_token(params, false)?;
            let page = service
                .list_records(&parsed.token, parsed.resumed)
                .await
                .map_err(map_list_error)?;
            if page.records.is_empty() && !parsed.resumed {
                return Err(HandlerError::Protocol(no_records_match()));
            }
            Ok(render_list_records(service, &page, parsed.token.metadata_prefix).await?)
        }
        other => Err(HandlerError::Protocol(bad_verb(&format!(
            "Unknown verb {other}"
        )))),
    }
}

fn render_identify(
    service: &OaiService,
    earliest: thoth_api::model::Timestamp,
    latest: thoth_api::model::Timestamp,
) -> String {
    format!(
        "<Identify>\
<repositoryName>{}</repositoryName>\
<baseURL>{}</baseURL>\
<protocolVersion>2.0</protocolVersion>\
<adminEmail>{}</adminEmail>\
<earliestDatestamp>{}</earliestDatestamp>\
<deletedRecord>no</deletedRecord>\
<granularity>YYYY-MM-DDThh:mm:ssZ</granularity>\
<compression>gzip</compression>\
<description>\
<oai-identifier xmlns=\"http://www.openarchives.org/OAI/2.0/oai-identifier\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xsi:schemaLocation=\"http://www.openarchives.org/OAI/2.0/oai-identifier http://www.openarchives.org/OAI/2.0/oai-identifier.xsd\">\
<scheme>oai</scheme>\
<repositoryIdentifier>thoth.pub</repositoryIdentifier>\
<delimiter>:</delimiter>\
<sampleIdentifier>{}:{}</sampleIdentifier>\
</oai-identifier>\
</description>\
<description>\
<thoth:repository xmlns:thoth=\"https://thoth.pub/oai/\">\
<thoth:latestDatestamp>{}</thoth:latestDatestamp>\
<thoth:rightsStatement>{}</thoth:rightsStatement>\
<thoth:rightsUri>{}</thoth:rightsUri>\
</thoth:repository>\
</description>\
</Identify>",
        xml_escape(REPOSITORY_NAME),
        xml_escape(&service.repository_url()),
        xml_escape(ADMIN_EMAIL),
        xml_escape(&OaiService::timestamp_xml(earliest)),
        RECORD_PREFIX,
        SAMPLE_ID,
        xml_escape(&OaiService::timestamp_xml(latest)),
        xml_escape(METADATA_RIGHTS_STATEMENT),
        xml_escape(METADATA_RIGHTS_URI),
    )
}

fn render_list_metadata_formats(prefixes: &[MetadataPrefix]) -> String {
    let mut xml = String::from("<ListMetadataFormats>");
    for prefix in prefixes.iter().copied() {
        xml.push_str("<metadataFormat>");
        push_text_element(&mut xml, "metadataPrefix", prefix.as_str());
        push_text_element(&mut xml, "schema", prefix.schema());
        push_text_element(&mut xml, "metadataNamespace", prefix.namespace());
        xml.push_str("</metadataFormat>");
    }
    xml.push_str("</ListMetadataFormats>");
    xml
}

fn render_list_sets(sets: &[service::SetRecord]) -> String {
    let mut xml = String::from("<ListSets>");
    for set in sets {
        xml.push_str("<set>");
        push_text_element(&mut xml, "setSpec", &set.spec);
        push_text_element(&mut xml, "setName", &set.name);
        xml.push_str("</set>");
    }
    xml.push_str("</ListSets>");
    xml
}

async fn render_get_record(
    service: &OaiService,
    work: &thoth_client::Work,
    metadata_prefix: MetadataPrefix,
) -> HandlerResult<String> {
    let mut xml = String::from("<GetRecord>");
    xml.push_str(&render_record_xml(service, work, metadata_prefix).await?);
    xml.push_str("</GetRecord>");
    Ok(xml)
}

fn render_list_identifiers(page: &RecordPage) -> String {
    let mut xml = String::from("<ListIdentifiers>");
    for work in &page.records {
        xml.push_str(&render_header_xml(work));
    }
    if page.terminal_resumption_token {
        xml.push_str("<resumptionToken/>");
    } else if let Some(token) = &page.next_token {
        xml.push_str(&render_resumption_token(
            token,
            page.cursor,
            page.complete_list_size,
        ));
    }
    xml.push_str("</ListIdentifiers>");
    xml
}

async fn render_list_records(
    service: &OaiService,
    page: &RecordPage,
    metadata_prefix: MetadataPrefix,
) -> HandlerResult<String> {
    let mut xml = String::from("<ListRecords>");
    for work in &page.records {
        xml.push_str(&render_record_xml(service, work, metadata_prefix).await?);
    }
    if page.terminal_resumption_token {
        xml.push_str("<resumptionToken/>");
    } else if let Some(token) = &page.next_token {
        xml.push_str(&render_resumption_token(
            token,
            page.cursor,
            page.complete_list_size,
        ));
    }
    xml.push_str("</ListRecords>");
    Ok(xml)
}

async fn render_record_xml(
    service: &OaiService,
    work: &thoth_client::Work,
    metadata_prefix: MetadataPrefix,
) -> HandlerResult<String> {
    let metadata = match metadata_prefix {
        MetadataPrefix::OaiDc => metadata::map_oai_dc(work).map_err(HandlerError::Internal)?,
        MetadataPrefix::OaiOpenaire => {
            metadata::map_oai_openaire(work).map_err(HandlerError::Internal)?
        }
        MetadataPrefix::MarcXml => service
            .get_marcxml_record(work.work_id)
            .await
            .map_err(map_marcxml_error(metadata_prefix))?,
    };

    Ok(format!(
        "<record>{}<metadata>{}</metadata></record>",
        render_header_xml(work),
        metadata
    ))
}

fn render_header_xml(work: &thoth_client::Work) -> String {
    let set_spec = OaiService::set_spec(&work.imprint.publisher.publisher_name);
    format!(
        "<header>\
<identifier>{}</identifier>\
<datestamp>{}</datestamp>\
<setSpec>{}</setSpec>\
</header>",
        xml_escape(&OaiService::oai_identifier(work.work_id)),
        xml_escape(&OaiService::timestamp_xml(work.updated_at_with_relations)),
        xml_escape(&set_spec),
    )
}

fn render_resumption_token(token: &str, cursor: i64, complete_list_size: Option<i64>) -> String {
    if let Some(complete_list_size) = complete_list_size {
        format!(
            "<resumptionToken cursor=\"{}\" completeListSize=\"{}\">{}</resumptionToken>",
            cursor,
            complete_list_size,
            xml_escape(token)
        )
    } else {
        format!(
            "<resumptionToken cursor=\"{}\">{}</resumptionToken>",
            cursor,
            xml_escape(token)
        )
    }
}

fn validate_list_verb(params: &HashMap<String, String>) -> HandlerResult<()> {
    require_only(
        params,
        &[
            "verb",
            "metadataPrefix",
            "set",
            "resumptionToken",
            "from",
            "until",
        ],
    )
}

fn parse_list_token(
    params: &HashMap<String, String>,
    identifiers_only: bool,
) -> HandlerResult<ParsedListRequest> {
    if let Some(value) = params.get("resumptionToken") {
        if params.len() != 2 {
            return Err(
                bad_argument("resumptionToken cannot be combined with other arguments").into(),
            );
        }
        let mut token = OaiService::decode_resumption_token(value).map_err(|_| ProtocolError {
            code: "badResumptionToken",
            message: "Invalid resumptionToken".to_string(),
        })?;
        if token.identifiers_only != identifiers_only {
            return Err(ProtocolError {
                code: "badResumptionToken",
                message: "resumptionToken does not match the request verb".to_string(),
            }
            .into());
        }
        let (from, until, granularity) = parse_datestamp_filter(
            token.from.as_deref(),
            token.until.as_deref(),
            token.granularity,
            true,
        )?;
        token.from = from;
        token.until = until;
        token.granularity = granularity;
        if token.scan_offset.is_none() {
            token.scan_offset = Some(token.offset);
        }
        if token.returned_count.is_none() {
            token.returned_count = Some(token.offset);
        }
        return Ok(ParsedListRequest {
            token,
            resumed: true,
        });
    }

    let metadata_prefix = params
        .get("metadataPrefix")
        .ok_or_else(|| bad_argument("Missing metadataPrefix parameter"))?;
    let (from, until, granularity) = parse_datestamp_filter(
        params.get("from").map(String::as_str),
        params.get("until").map(String::as_str),
        None,
        false,
    )?;
    Ok(ParsedListRequest {
        token: ResumptionToken {
            offset: 0,
            metadata_prefix: parse_metadata_prefix(metadata_prefix)?,
            set: params.get("set").cloned(),
            identifiers_only,
            from,
            until,
            granularity,
            scan_offset: Some(0),
            returned_count: Some(0),
        },
        resumed: false,
    })
}

fn parse_datestamp_filter(
    from: Option<&str>,
    until: Option<&str>,
    expected_granularity: Option<DatestampGranularity>,
    is_resumption_token: bool,
) -> HandlerResult<(Option<String>, Option<String>, Option<DatestampGranularity>)> {
    let parse_error = |message: &str| {
        if is_resumption_token {
            ProtocolError {
                code: "badResumptionToken",
                message: message.to_string(),
            }
        } else {
            bad_argument(message)
        }
    };

    let from_value = from
        .map(|value| parse_datestamp_value(value, expected_granularity))
        .transpose()
        .map_err(|message| parse_error(&message))?;
    let until_value = until
        .map(|value| parse_datestamp_value(value, expected_granularity))
        .transpose()
        .map_err(|message| parse_error(&message))?;

    let mut granularity = expected_granularity;
    if let Some((value_granularity, _)) = from_value {
        granularity = Some(value_granularity);
    }
    if let Some((value_granularity, _)) = until_value {
        if let Some(existing) = granularity {
            if existing != value_granularity {
                return Err(
                    parse_error("from and until must use the same datestamp granularity").into(),
                );
            }
        } else {
            granularity = Some(value_granularity);
        }
    }

    let canonical_from = from_value.map(|(_, value)| value);
    let canonical_until = until_value.map(|(_, value)| value);

    if let (Some(from_value), Some(until_value), Some(granularity)) = (
        canonical_from.as_deref(),
        canonical_until.as_deref(),
        granularity,
    ) {
        let ordered = match granularity {
            DatestampGranularity::Day => from_value <= until_value,
            DatestampGranularity::Second => {
                let from = DateTime::parse_from_str(from_value, "%Y-%m-%dT%H:%M:%SZ")
                    .map_err(|_| parse_error("Invalid from datestamp"))?;
                let until = DateTime::parse_from_str(until_value, "%Y-%m-%dT%H:%M:%SZ")
                    .map_err(|_| parse_error("Invalid until datestamp"))?;
                from <= until
            }
        };
        if !ordered {
            return Err(parse_error("from datestamp must be less than or equal to until").into());
        }
    }

    Ok((canonical_from, canonical_until, granularity))
}

fn parse_datestamp_value(
    value: &str,
    expected_granularity: Option<DatestampGranularity>,
) -> Result<(DatestampGranularity, String), String> {
    match expected_granularity {
        Some(DatestampGranularity::Day) => NaiveDate::parse_from_str(value, "%Y-%m-%d")
            .map(|date| {
                (
                    DatestampGranularity::Day,
                    date.format("%Y-%m-%d").to_string(),
                )
            })
            .map_err(|_| "Invalid day datestamp".to_string()),
        Some(DatestampGranularity::Second) => {
            let datetime = DateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%SZ")
                .map_err(|_| "Invalid second datestamp".to_string())?;
            Ok((
                DatestampGranularity::Second,
                datetime
                    .with_timezone(&Utc)
                    .format("%Y-%m-%dT%H:%M:%SZ")
                    .to_string(),
            ))
        }
        None => {
            if let Ok(date) = NaiveDate::parse_from_str(value, "%Y-%m-%d") {
                return Ok((
                    DatestampGranularity::Day,
                    date.format("%Y-%m-%d").to_string(),
                ));
            }
            let datetime = DateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%SZ")
                .map_err(|_| "Invalid datestamp".to_string())?;
            Ok((
                DatestampGranularity::Second,
                datetime
                    .with_timezone(&Utc)
                    .format("%Y-%m-%dT%H:%M:%SZ")
                    .to_string(),
            ))
        }
    }
}

fn parse_metadata_prefix(value: &str) -> HandlerResult<MetadataPrefix> {
    MetadataPrefix::try_from(value).map_err(|_| {
        ProtocolError {
            code: "cannotDisseminateFormat",
            message: format!("Unsupported metadataPrefix {value}"),
        }
        .into()
    })
}

fn parse_identifier_for_lookup(value: &str) -> HandlerResult<Uuid> {
    OaiService::parse_oai_identifier(value).map_err(|_| {
        ProtocolError {
            code: "idDoesNotExist",
            message: "The requested identifier does not exist".to_string(),
        }
        .into()
    })
}

fn map_get_record_error(
    metadata_prefix: MetadataPrefix,
) -> impl Fn(ThothError) -> HandlerError + Copy {
    move |error| match error {
        ThothError::EntityNotFound => HandlerError::Protocol(ProtocolError {
            code: "idDoesNotExist",
            message: "The requested identifier does not exist".to_string(),
        }),
        ThothError::IncompleteMetadataRecord(_, _)
        | ThothError::InvalidMetadataSpecification(_) => HandlerError::Protocol(ProtocolError {
            code: "cannotDisseminateFormat",
            message: format!(
                "Record cannot be disseminated as {}",
                metadata_prefix.as_str()
            ),
        }),
        other => HandlerError::Internal(other),
    }
}

fn map_marcxml_error(
    metadata_prefix: MetadataPrefix,
) -> impl Fn(ThothError) -> HandlerError + Copy {
    move |error| match error {
        ThothError::RequestError(_) if is_transient_upstream_error(&error) => {
            HandlerError::Internal(error)
        }
        _ => HandlerError::Protocol(ProtocolError {
            code: "cannotDisseminateFormat",
            message: format!(
                "Record cannot be disseminated as {}",
                metadata_prefix.as_str()
            ),
        }),
    }
}

fn map_list_error(error: ThothError) -> HandlerError {
    match error {
        ThothError::EntityNotFound => no_records_match().into(),
        ThothError::RequestError(message) if message.starts_with("badResumptionToken") => {
            HandlerError::Protocol(ProtocolError {
                code: "badResumptionToken",
                message: "Invalid resumptionToken".to_string(),
            })
        }
        other => HandlerError::Internal(other),
    }
}

fn require_only(params: &HashMap<String, String>, allowed: &[&str]) -> HandlerResult<()> {
    if params.keys().all(|key| allowed.contains(&key.as_str())) {
        Ok(())
    } else {
        Err(bad_argument("The request included unsupported arguments").into())
    }
}

fn bad_argument(message: &str) -> ProtocolError {
    ProtocolError {
        code: "badArgument",
        message: message.to_string(),
    }
}

fn bad_verb(message: &str) -> ProtocolError {
    ProtocolError {
        code: "badVerb",
        message: message.to_string(),
    }
}

fn no_records_match() -> ProtocolError {
    ProtocolError {
        code: "noRecordsMatch",
        message: "The request matched no records".to_string(),
    }
}

fn success_document(service: &OaiService, params: &HashMap<String, String>, body: &str) -> String {
    format!(
        "{}{}<OAI-PMH xmlns=\"http://www.openarchives.org/OAI/2.0/\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xsi:schemaLocation=\"http://www.openarchives.org/OAI/2.0/ http://www.openarchives.org/OAI/2.0/OAI-PMH.xsd\"><responseDate>{}</responseDate>{}{}</OAI-PMH>",
        xml_declaration(),
        stylesheet_pi(),
        response_date(),
        request_element(service, params, true),
        body
    )
}

fn error_document(
    service: &OaiService,
    params: &HashMap<String, String>,
    code: &str,
    message: &str,
) -> String {
    let include_attributes = !matches!(code, "badVerb" | "badArgument");
    format!(
        "{}{}<OAI-PMH xmlns=\"http://www.openarchives.org/OAI/2.0/\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xsi:schemaLocation=\"http://www.openarchives.org/OAI/2.0/ http://www.openarchives.org/OAI/2.0/OAI-PMH.xsd\"><responseDate>{}</responseDate>{}<error code=\"{}\">{}</error></OAI-PMH>",
        xml_declaration(),
        stylesheet_pi(),
        response_date(),
        request_element(service, params, include_attributes),
        xml_escape(code),
        xml_escape(message)
    )
}

fn request_element(
    service: &OaiService,
    params: &HashMap<String, String>,
    include_attributes: bool,
) -> String {
    let mut attrs = params.iter().collect::<Vec<_>>();
    attrs.sort_by(|(left, _), (right, _)| left.cmp(right));
    let mut element = String::from("<request");
    if include_attributes {
        for (key, value) in attrs {
            element.push(' ');
            element.push_str(key);
            element.push_str("=\"");
            element.push_str(&xml_escape(value));
            element.push('"');
        }
    }
    element.push('>');
    element.push_str(&xml_escape(&service.repository_url()));
    element.push_str("</request>");
    element
}

fn response_date() -> String {
    Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

fn xml_declaration() -> &'static str {
    "<?xml version=\"1.0\" encoding=\"UTF-8\"?>"
}

fn stylesheet_pi() -> &'static str {
    "\n<?xml-stylesheet type=\"text/xsl\" href=\"/oai2.xsl\"?>\n"
}

fn xml_escape(value: &str) -> String {
    escape(value).into_owned()
}

fn push_text_element(xml: &mut String, name: &str, text: &str) {
    xml.push('<');
    xml.push_str(name);
    xml.push('>');
    xml.push_str(&xml_escape(text));
    xml.push_str("</");
    xml.push_str(name);
    xml.push('>');
}

fn xml_response(request: &HttpRequest, body: String) -> HttpResponse {
    if request_accepts_gzip(request) {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        if encoder.write_all(body.as_bytes()).is_err() {
            return HttpResponse::InternalServerError()
                .content_type("text/plain; charset=utf-8")
                .body("Internal Server Error");
        }
        match encoder.finish() {
            Ok(compressed_body) => {
                return HttpResponse::Ok()
                    .insert_header((header::CONTENT_ENCODING, "gzip"))
                    .content_type("text/xml; charset=utf-8")
                    .body(compressed_body);
            }
            Err(_) => {
                return HttpResponse::InternalServerError()
                    .content_type("text/plain; charset=utf-8")
                    .body("Internal Server Error");
            }
        }
    }

    HttpResponse::Ok()
        .content_type("text/xml; charset=utf-8")
        .body(body)
}

fn request_accepts_gzip(request: &HttpRequest) -> bool {
    let Some(value) = request.headers().get(header::ACCEPT_ENCODING) else {
        return false;
    };
    let Ok(value) = value.to_str() else {
        return false;
    };

    value.split(',').any(|item| {
        let mut parts = item.trim().split(';');
        let coding = parts
            .next()
            .map(str::trim)
            .unwrap_or_default()
            .to_ascii_lowercase();
        if coding != "gzip" && coding != "*" {
            return false;
        }
        let quality = parts
            .map(str::trim)
            .find_map(|part| part.strip_prefix("q="))
            .and_then(|value| value.parse::<f32>().ok())
            .unwrap_or(1.0);
        quality > 0.0
    })
}

#[actix_web::main]
#[allow(clippy::too_many_arguments)]
pub async fn start_server(
    host: String,
    port: String,
    threads: usize,
    keep_alive: u64,
    public_url: String,
    gql_endpoint: String,
    export_url: String,
    retry_after_seconds: u64,
) -> io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let state = AppState {
        service: OaiService::new(public_url, gql_endpoint, export_url),
        retry_after_seconds,
    };

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::new(LOG_FORMAT))
            .wrap(Cors::default().allowed_methods(vec!["GET", "POST", "OPTIONS"]))
            .app_data(web::Data::new(state.clone()))
            .service(web::resource("/").route(web::get().to(index)))
            .service(
                web::resource("/oai")
                    .route(web::get().to(oai_get))
                    .route(web::post().to(oai_post)),
            )
            .service(web::resource("/oai2.xsl").route(web::get().to(stylesheet)))
            .default_service(web::route().to(not_found))
    })
    .workers(threads)
    .keep_alive(Duration::from_secs(keep_alive))
    .bind(format!("{host}:{port}"))?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::PAGE_LIMIT;
    use actix_web::{dev::ServerHandle, http::header, test, App, HttpResponse, HttpServer};
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
    use chrono::{Duration, NaiveDate};
    use flate2::read::GzDecoder;
    use serde_json::{json, Value};
    use std::{collections::HashSet, io::Read, net::TcpListener};

    const PUBLISHER_ID: &str = "00000000-0000-0000-1111-000000000001";
    const PUBLISHER_NAME: &str = "Open Access Press";

    #[derive(Clone)]
    struct MockGraphqlState {
        works: Vec<Value>,
        publishers: Vec<Value>,
        latest: String,
        earliest: String,
    }

    #[derive(Clone, Default)]
    struct MockExportState {
        failing_work_ids: HashSet<Uuid>,
        non_disseminatable_work_ids: HashSet<Uuid>,
        malformed_work_ids: HashSet<Uuid>,
    }

    struct RunningMockServer {
        base_url: String,
        handle: ServerHandle,
    }

    impl RunningMockServer {
        async fn stop(self) {
            self.handle.stop(true).await;
        }
    }

    async fn spawn_graphql_server(state: MockGraphqlState) -> RunningMockServer {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind graphql mock server");
        let address = listener.local_addr().expect("graphql local address");
        let state = web::Data::new(state);

        let server = HttpServer::new(move || {
            App::new()
                .app_data(state.clone())
                .route("/graphql", web::post().to(graphql_mock_handler))
        })
        .listen(listener)
        .expect("listen graphql mock server")
        .run();
        let handle = server.handle();
        actix_web::rt::spawn(server);

        RunningMockServer {
            base_url: format!("http://{address}"),
            handle,
        }
    }

    async fn spawn_graphql_error_server(status: actix_web::http::StatusCode) -> RunningMockServer {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind graphql error server");
        let address = listener.local_addr().expect("graphql error local address");

        let server = HttpServer::new(move || {
            App::new().route(
                "/graphql",
                web::post().to(move || async move {
                    HttpResponse::build(status)
                        .content_type("application/json; charset=utf-8")
                        .body(r#"{"errors":[{"message":"upstream failure"}]}"#)
                }),
            )
        })
        .listen(listener)
        .expect("listen graphql error server")
        .run();
        let handle = server.handle();
        actix_web::rt::spawn(server);

        RunningMockServer {
            base_url: format!("http://{address}"),
            handle,
        }
    }

    async fn spawn_export_server(state: MockExportState) -> RunningMockServer {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind export mock server");
        let address = listener.local_addr().expect("export local address");
        let state = web::Data::new(state);

        let server = HttpServer::new(move || {
            App::new().app_data(state.clone()).route(
                "/specifications/marc21xml::thoth/work/{work_id}",
                web::get().to(export_mock_handler),
            )
        })
        .listen(listener)
        .expect("listen export mock server")
        .run();
        let handle = server.handle();
        actix_web::rt::spawn(server);

        RunningMockServer {
            base_url: format!("http://{address}"),
            handle,
        }
    }

    async fn graphql_mock_handler(
        state: web::Data<MockGraphqlState>,
        payload: web::Json<Value>,
    ) -> HttpResponse {
        let payload = payload.into_inner();
        let variables = payload
            .get("variables")
            .cloned()
            .unwrap_or_else(|| json!({}));
        let operation_name = request_operation_name(&payload);

        let response = match operation_name.as_deref() {
            Some("OaiLatestWorksUpdatedQuery") => {
                json!({ "data": { "works": [{ "updatedAtWithRelations": state.latest.clone() }] } })
            }
            Some("OaiEarliestWorksUpdatedQuery") => {
                json!({ "data": { "works": [{ "updatedAtWithRelations": state.earliest.clone() }] } })
            }
            Some("PublishersQuery") => {
                json!({ "data": { "publishers": state.publishers.clone() } })
            }
            Some("WorkQuery") => {
                let work_id = variables
                    .get("workId")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned);
                match work_id.and_then(|work_id| find_work_by_id(&state, &work_id)) {
                    Some(work) => json!({ "data": { "work": work } }),
                    None => json!({ "errors": [{ "message": "work not found" }] }),
                }
            }
            Some("OaiWorkCountQuery") => {
                let works = filter_works_by_publishers(&state, &variables);
                json!({ "data": { "workCount": works.len() as i64 } })
            }
            Some("OaiBookCountQuery") => {
                let works = filter_works_by_publishers(&state, &variables);
                json!({ "data": { "bookCount": works.len() as i64 } })
            }
            Some("OaiWorksQuery") => {
                let works =
                    paginate_works(filter_works_by_publishers(&state, &variables), &variables);
                json!({ "data": { "works": works } })
            }
            Some("OaiBooksQuery") => {
                let works =
                    paginate_works(filter_works_by_publishers(&state, &variables), &variables);
                json!({ "data": { "books": works } })
            }
            _ => json!({ "errors": [{ "message": "unsupported operation" }] }),
        };

        HttpResponse::Ok().json(response)
    }

    async fn export_mock_handler(
        state: web::Data<MockExportState>,
        work_id: web::Path<Uuid>,
    ) -> HttpResponse {
        let work_id = work_id.into_inner();
        if state.failing_work_ids.contains(&work_id) {
            return HttpResponse::InternalServerError()
                .content_type("text/plain; charset=utf-8")
                .body("export failed");
        }
        if state.non_disseminatable_work_ids.contains(&work_id) {
            return HttpResponse::NotFound()
                .content_type("text/plain; charset=utf-8")
                .body("record not available");
        }
        if state.malformed_work_ids.contains(&work_id) {
            return HttpResponse::Ok()
                .content_type("application/xml; charset=utf-8")
                .body("<collection xmlns=\"http://www.loc.gov/MARC21/slim\"></collection>");
        }
        HttpResponse::Ok()
            .content_type("application/xml; charset=utf-8")
            .body(format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<collection xmlns="http://www.loc.gov/MARC21/slim">
  <record>
    <leader>00000nam a2200000 i 4500</leader>
    <controlfield tag="001">{work_id}</controlfield>
  </record>
</collection>"#
            ))
    }

    fn request_operation_name(payload: &Value) -> Option<String> {
        payload
            .get("operationName")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
            .or_else(|| {
                let query = payload.get("query").and_then(Value::as_str)?;
                [
                    "OaiLatestWorksUpdatedQuery",
                    "OaiEarliestWorksUpdatedQuery",
                    "PublishersQuery",
                    "WorkQuery",
                    "OaiWorkCountQuery",
                    "OaiBookCountQuery",
                    "OaiWorksQuery",
                    "OaiBooksQuery",
                ]
                .iter()
                .find(|name| query.contains(**name))
                .map(|name| (*name).to_string())
            })
    }

    fn find_work_by_id(state: &MockGraphqlState, work_id: &str) -> Option<Value> {
        state
            .works
            .iter()
            .find(|work| work.get("workId").and_then(Value::as_str) == Some(work_id))
            .cloned()
    }

    fn filter_works_by_publishers(state: &MockGraphqlState, variables: &Value) -> Vec<Value> {
        let Some(publishers) = variables.get("publishers") else {
            return state.works.clone();
        };
        if publishers.is_null() {
            return state.works.clone();
        }
        let Some(ids) = publishers.as_array() else {
            return state.works.clone();
        };
        if ids.is_empty() {
            return Vec::new();
        }
        let allowed_names = ids
            .iter()
            .filter_map(Value::as_str)
            .filter_map(|publisher_id| {
                state
                    .publishers
                    .iter()
                    .find(|publisher| {
                        publisher.get("publisherId").and_then(Value::as_str) == Some(publisher_id)
                    })
                    .and_then(|publisher| publisher.get("publisherName").and_then(Value::as_str))
                    .map(ToOwned::to_owned)
            })
            .collect::<HashSet<_>>();
        state
            .works
            .iter()
            .filter(|work| {
                work.get("imprint")
                    .and_then(|imprint| imprint.get("publisher"))
                    .and_then(|publisher| publisher.get("publisherName"))
                    .and_then(Value::as_str)
                    .is_some_and(|publisher_name| allowed_names.contains(publisher_name))
            })
            .cloned()
            .collect()
    }

    fn paginate_works(works: Vec<Value>, variables: &Value) -> Vec<Value> {
        let offset = variables.get("offset").and_then(Value::as_i64).unwrap_or(0);
        let limit = variables
            .get("limit")
            .and_then(Value::as_i64)
            .unwrap_or(PAGE_LIMIT);
        works
            .into_iter()
            .skip(offset.max(0) as usize)
            .take(limit.max(0) as usize)
            .collect()
    }

    fn mock_graphql_state(mut works: Vec<Value>) -> MockGraphqlState {
        works.sort_by(|left, right| {
            let left = left
                .get("updatedAtWithRelations")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let right = right
                .get("updatedAtWithRelations")
                .and_then(Value::as_str)
                .unwrap_or_default();
            right.cmp(left)
        });
        let latest = works
            .first()
            .and_then(|work| work.get("updatedAtWithRelations"))
            .and_then(Value::as_str)
            .unwrap_or("2024-12-31T00:00:00Z")
            .to_string();
        let earliest = works
            .last()
            .and_then(|work| work.get("updatedAtWithRelations"))
            .and_then(Value::as_str)
            .unwrap_or("2024-01-01T00:00:00Z")
            .to_string();

        MockGraphqlState {
            works,
            publishers: vec![json!({
                "publisherId": PUBLISHER_ID,
                "publisherName": PUBLISHER_NAME,
            })],
            latest,
            earliest,
        }
    }

    fn make_work(
        work_id: Uuid,
        updated_at_with_relations: &str,
        publisher_name: &str,
        marc_eligible: bool,
        include_xml_publication: bool,
    ) -> Value {
        let contributions = if marc_eligible {
            vec![json!({
                "contributionType": "AUTHOR",
                "firstName": "Ada",
                "lastName": "Lovelace",
                "fullName": "Ada Lovelace",
                "mainContribution": true,
                "biographies": [],
                "contributionOrdinal": 1,
                "contributor": {
                    "orcid": "https://orcid.org/0000-0002-0000-0001",
                    "website": null
                },
                "affiliations": []
            })]
        } else {
            Vec::new()
        };
        let languages = if marc_eligible {
            vec![json!({
                "languageCode": "ENG",
                "languageRelation": "ORIGINAL"
            })]
        } else {
            Vec::new()
        };
        let mut publications = vec![json!({
            "publicationId": Uuid::from_u128(work_id.as_u128() + 10),
            "publicationType": "PDF",
            "isbn": if marc_eligible { Value::String("978-1-4028-9462-6".to_string()) } else { Value::Null },
            "weightG": null,
            "weightOz": null,
            "widthMm": null,
            "widthCm": null,
            "widthIn": null,
            "heightMm": null,
            "heightCm": null,
            "heightIn": null,
            "depthMm": null,
            "depthCm": null,
            "depthIn": null,
            "accessibilityStandard": null,
            "accessibilityAdditionalStandard": null,
            "accessibilityException": null,
            "accessibilityReportUrl": null,
            "prices": [],
            "locations": [
                {
                    "landingPage": "https://example.org/book",
                    "fullTextUrl": "https://example.org/book.pdf",
                    "locationPlatform": "OTHER",
                    "canonical": true
                }
            ]
        })];
        if include_xml_publication {
            publications.push(json!({
                "publicationId": Uuid::from_u128(work_id.as_u128() + 11),
                "publicationType": "XML",
                "isbn": "978-92-95055-02-5",
                "weightG": null,
                "weightOz": null,
                "widthMm": null,
                "widthCm": null,
                "widthIn": null,
                "heightMm": null,
                "heightCm": null,
                "heightIn": null,
                "depthMm": null,
                "depthCm": null,
                "depthIn": null,
                "accessibilityStandard": null,
                "accessibilityAdditionalStandard": null,
                "accessibilityException": null,
                "accessibilityReportUrl": null,
                "prices": [],
                "locations": []
            }));
        }

        let mut work = serde_json::Map::new();
        work.insert("workId".to_string(), json!(work_id));
        work.insert(
            "updatedAtWithRelations".to_string(),
            json!(updated_at_with_relations),
        );
        work.insert("workStatus".to_string(), json!("ACTIVE"));
        work.insert("workType".to_string(), json!("MONOGRAPH"));
        work.insert("reference".to_string(), Value::Null);
        work.insert("edition".to_string(), json!(1));
        work.insert(
            "doi".to_string(),
            json!(format!("https://doi.org/10.00001/{work_id}")),
        );
        work.insert("publicationDate".to_string(), json!("2024-01-01"));
        work.insert("withdrawnDate".to_string(), Value::Null);
        work.insert(
            "license".to_string(),
            json!("http://creativecommons.org/licenses/by/4.0/"),
        );
        work.insert("copyrightHolder".to_string(), json!("Author"));
        work.insert("generalNote".to_string(), Value::Null);
        work.insert("bibliographyNote".to_string(), Value::Null);
        work.insert("place".to_string(), json!("London"));
        work.insert("pageCount".to_string(), json!(100));
        work.insert("pageBreakdown".to_string(), Value::Null);
        work.insert("firstPage".to_string(), Value::Null);
        work.insert("lastPage".to_string(), Value::Null);
        work.insert("pageInterval".to_string(), Value::Null);
        work.insert("imageCount".to_string(), Value::Null);
        work.insert("tableCount".to_string(), Value::Null);
        work.insert("audioCount".to_string(), Value::Null);
        work.insert("videoCount".to_string(), Value::Null);
        work.insert("landingPage".to_string(), json!("https://example.org/book"));
        work.insert("toc".to_string(), Value::Null);
        work.insert("lccn".to_string(), Value::Null);
        work.insert("oclc".to_string(), Value::Null);
        work.insert("coverUrl".to_string(), Value::Null);
        work.insert("coverCaption".to_string(), Value::Null);
        work.insert(
            "titles".to_string(),
            json!([{
                "titleId": Uuid::from_u128(work_id.as_u128() + 1),
                "localeCode": "EN",
                "fullTitle": "Sample Title",
                "title": "Sample Title",
                "subtitle": null,
                "canonical": true
            }]),
        );
        work.insert("abstracts".to_string(), json!([]));
        work.insert(
            "imprint".to_string(),
            json!({
                "imprintName": "Imprint",
                "imprintUrl": null,
                "crossmarkDoi": null,
                "defaultCurrency": "EUR",
                "defaultPlace": "London",
                "defaultLocale": "EN",
                "publisher": {
                    "publisherName": publisher_name,
                    "publisherShortname": "OAP",
                    "publisherUrl": null,
                    "accessibilityStatement": null,
                    "contacts": []
                }
            }),
        );
        work.insert("issues".to_string(), json!([]));
        work.insert("contributions".to_string(), json!(contributions));
        work.insert("languages".to_string(), json!(languages));
        work.insert("publications".to_string(), json!(publications));
        work.insert("subjects".to_string(), json!([]));
        work.insert("fundings".to_string(), json!([]));
        work.insert("relations".to_string(), json!([]));
        work.insert("references".to_string(), json!([]));
        Value::Object(work)
    }

    fn make_descending_work_series(count: usize) -> Vec<Value> {
        let base_date = NaiveDate::from_ymd_opt(2024, 12, 31).expect("valid base date");
        (0..count)
            .map(|index| {
                let updated_at = (base_date - Duration::days(index as i64))
                    .format("%Y-%m-%dT12:00:00Z")
                    .to_string();
                let work_id =
                    Uuid::from_u128(0x1000_0000_0000_0000_0000_0000_0000_0000 + index as u128);
                make_work(work_id, &updated_at, PUBLISHER_NAME, true, true)
            })
            .collect()
    }

    fn normalize_response_date(xml: &str) -> String {
        let open = "<responseDate>";
        let close = "</responseDate>";
        let Some(start) = xml.find(open) else {
            return xml.to_string();
        };
        let value_start = start + open.len();
        let Some(value_end_rel) = xml[value_start..].find(close) else {
            return xml.to_string();
        };
        let value_end = value_start + value_end_rel;
        let mut normalized = String::new();
        normalized.push_str(&xml[..value_start]);
        normalized.push_str("RESPONSE_DATE");
        normalized.push_str(&xml[value_end..]);
        normalized
    }

    fn request_opening_tag(xml: &str) -> String {
        let start = xml.find("<request").expect("request element exists");
        let end = xml[start..]
            .find('>')
            .map(|offset| start + offset)
            .expect("request closing bracket");
        xml[start..=end].to_string()
    }

    fn extract_resumption_token(xml: &str) -> Option<String> {
        let token_start = xml.find("<resumptionToken")?;
        let content_start = token_start + xml[token_start..].find('>')? + 1;
        let content_end = content_start + xml[content_start..].find("</resumptionToken>")?;
        let value = xml[content_start..content_end].trim();
        if value.is_empty() {
            None
        } else {
            Some(value.to_string())
        }
    }

    fn count_occurrences(haystack: &str, needle: &str) -> usize {
        haystack.matches(needle).count()
    }

    #[actix_web::test]
    async fn get_and_post_are_equivalent_for_all_oai_verbs() {
        let work_id = Uuid::from_u128(1);
        let works = vec![make_work(
            work_id,
            "2024-12-30T12:00:00Z",
            PUBLISHER_NAME,
            true,
            true,
        )];

        let graphql_server = spawn_graphql_server(mock_graphql_state(works)).await;
        let export_server = spawn_export_server(MockExportState::default()).await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState {
                    service: OaiService::new(
                        "https://example.org".to_string(),
                        format!("{}/graphql", graphql_server.base_url),
                        export_server.base_url.clone(),
                    ),
                    retry_after_seconds: DEFAULT_RETRY_AFTER_SECONDS,
                }))
                .service(
                    web::resource("/oai")
                        .route(web::get().to(oai_get))
                        .route(web::post().to(oai_post)),
                ),
        )
        .await;

        let identifier = OaiService::oai_identifier(work_id);
        let cases = vec![
            "verb=Identify".to_string(),
            "verb=ListMetadataFormats".to_string(),
            "verb=ListSets".to_string(),
            format!("verb=GetRecord&identifier={identifier}&metadataPrefix=oai_dc"),
            "verb=ListIdentifiers&metadataPrefix=oai_dc".to_string(),
            "verb=ListRecords&metadataPrefix=oai_dc".to_string(),
        ];

        for case in cases {
            let get_req = test::TestRequest::get()
                .uri(&format!("/oai?{case}"))
                .to_request();
            let get_response = test::call_service(&app, get_req).await;
            assert_eq!(get_response.status(), actix_web::http::StatusCode::OK);
            assert_eq!(
                get_response
                    .headers()
                    .get(header::CONTENT_TYPE)
                    .and_then(|value| value.to_str().ok())
                    .expect("GET content type"),
                "text/xml; charset=utf-8"
            );
            assert!(get_response
                .headers()
                .get(header::CONTENT_ENCODING)
                .is_none());
            let get_body = String::from_utf8(test::read_body(get_response).await.to_vec())
                .expect("GET body UTF-8");

            let post_req = test::TestRequest::post()
                .uri("/oai")
                .insert_header((header::CONTENT_TYPE, "application/x-www-form-urlencoded"))
                .set_payload(case.clone())
                .to_request();
            let post_response = test::call_service(&app, post_req).await;
            assert_eq!(post_response.status(), actix_web::http::StatusCode::OK);
            assert_eq!(
                post_response
                    .headers()
                    .get(header::CONTENT_TYPE)
                    .and_then(|value| value.to_str().ok())
                    .expect("POST content type"),
                "text/xml; charset=utf-8"
            );
            assert!(post_response
                .headers()
                .get(header::CONTENT_ENCODING)
                .is_none());
            let post_body = String::from_utf8(test::read_body(post_response).await.to_vec())
                .expect("POST body UTF-8");

            if case == "verb=Identify" {
                assert!(get_body.contains("<compression>gzip</compression>"));
                assert!(post_body.contains("<compression>gzip</compression>"));
                assert!(get_body.contains(
                    "<thoth:rightsStatement>Metadata is licensed under the terms of Creative Commons CC0 1.0 Universal: https://creativecommons.org/publicdomain/zero/1.0/.</thoth:rightsStatement>"
                ));
                assert!(post_body.contains(
                    "<thoth:rightsUri>https://creativecommons.org/publicdomain/zero/1.0/</thoth:rightsUri>"
                ));
            }

            assert_eq!(
                normalize_response_date(&get_body),
                normalize_response_date(&post_body)
            );
        }

        export_server.stop().await;
        graphql_server.stop().await;
    }

    #[actix_web::test]
    async fn stylesheet_contains_branding_and_oai_rendering_support() {
        let app = test::init_service(
            App::new().service(web::resource("/oai2.xsl").route(web::get().to(stylesheet))),
        )
        .await;

        let req = test::TestRequest::get().uri("/oai2.xsl").to_request();
        let response = test::call_service(&app, req).await;
        assert_eq!(response.status(), actix_web::http::StatusCode::OK);
        assert_eq!(
            response
                .headers()
                .get(header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok()),
            Some("text/xsl; charset=utf-8")
        );

        let body = String::from_utf8(test::read_body(response).await.to_vec())
            .expect("stylesheet body UTF-8");
        assert!(body.contains("https://cdn.thoth.pub/THOTH_ColourPos.png"));
        assert!(body.contains(
            "https://cdn.thoth.pub/favicons/thoth-head-20260331/transparent/favicon.ico"
        ));
        assert!(body.contains(
            "https://cdn.thoth.pub/favicons/thoth-head-20260331/transparent/manifest.json"
        ));
        assert!(body.contains("Rights Management"));
        assert!(body.contains("match=\"oai:setDescription\""));
        assert!(body.contains("match=\"oai:about\""));
        assert!(body.contains("End of list. This empty token marks a terminal page."));
    }

    #[actix_web::test]
    async fn not_found_page_contains_favicon_and_oai_link() {
        let app = test::init_service(App::new().default_service(web::route().to(not_found))).await;

        let req = test::TestRequest::get().uri("/missing").to_request();
        let response = test::call_service(&app, req).await;
        assert_eq!(response.status(), actix_web::http::StatusCode::NOT_FOUND);
        assert_eq!(
            response
                .headers()
                .get(header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok()),
            Some("text/html; charset=utf-8")
        );

        let body =
            String::from_utf8(test::read_body(response).await.to_vec()).expect("404 body UTF-8");
        assert!(body.contains(
            "https://cdn.thoth.pub/favicons/thoth-head-20260331/transparent/favicon.ico"
        ));
        assert!(body.contains(
            "https://cdn.thoth.pub/favicons/thoth-head-20260331/transparent/manifest.json"
        ));
        assert!(body.contains("<a href=\"/oai\">OAI-PMH Interface</a>"));
    }

    #[actix_web::test]
    async fn repeated_arguments_return_bad_argument() {
        let graphql_server =
            spawn_graphql_server(mock_graphql_state(make_descending_work_series(1))).await;
        let export_server = spawn_export_server(MockExportState::default()).await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState {
                    service: OaiService::new(
                        "https://example.org".to_string(),
                        format!("{}/graphql", graphql_server.base_url),
                        export_server.base_url.clone(),
                    ),
                    retry_after_seconds: DEFAULT_RETRY_AFTER_SECONDS,
                }))
                .service(
                    web::resource("/oai")
                        .route(web::get().to(oai_get))
                        .route(web::post().to(oai_post)),
                ),
        )
        .await;

        let get_req = test::TestRequest::get()
            .uri("/oai?verb=Identify&verb=ListSets")
            .to_request();
        let get_response = test::call_service(&app, get_req).await;
        let get_body = String::from_utf8(test::read_body(get_response).await.to_vec())
            .expect("GET body UTF-8");
        assert!(get_body.contains("<error code=\"badArgument\">"));

        let post_req = test::TestRequest::post()
            .uri("/oai?verb=Identify")
            .insert_header((header::CONTENT_TYPE, "application/x-www-form-urlencoded"))
            .set_payload("verb=ListSets")
            .to_request();
        let post_response = test::call_service(&app, post_req).await;
        let post_body = String::from_utf8(test::read_body(post_response).await.to_vec())
            .expect("POST body UTF-8");
        assert!(post_body.contains("<error code=\"badArgument\">"));

        export_server.stop().await;
        graphql_server.stop().await;
    }

    #[actix_web::test]
    async fn request_attributes_are_omitted_for_bad_verb_and_bad_argument() {
        let graphql_server =
            spawn_graphql_server(mock_graphql_state(make_descending_work_series(1))).await;
        let export_server = spawn_export_server(MockExportState::default()).await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState {
                    service: OaiService::new(
                        "https://example.org".to_string(),
                        format!("{}/graphql", graphql_server.base_url),
                        export_server.base_url.clone(),
                    ),
                    retry_after_seconds: DEFAULT_RETRY_AFTER_SECONDS,
                }))
                .service(web::resource("/oai").route(web::get().to(oai_get))),
        )
        .await;

        let bad_verb_req = test::TestRequest::get()
            .uri("/oai?verb=UnknownVerb")
            .to_request();
        let bad_verb_response = test::call_service(&app, bad_verb_req).await;
        let bad_verb_body = String::from_utf8(test::read_body(bad_verb_response).await.to_vec())
            .expect("badVerb body UTF-8");
        assert!(bad_verb_body.contains("<error code=\"badVerb\">"));
        assert_eq!(request_opening_tag(&bad_verb_body), "<request>");

        let bad_argument_req = test::TestRequest::get()
            .uri("/oai?verb=Identify&foo=bar")
            .to_request();
        let bad_argument_response = test::call_service(&app, bad_argument_req).await;
        let bad_argument_body =
            String::from_utf8(test::read_body(bad_argument_response).await.to_vec())
                .expect("badArgument body UTF-8");
        assert!(bad_argument_body.contains("<error code=\"badArgument\">"));
        assert_eq!(request_opening_tag(&bad_argument_body), "<request>");

        export_server.stop().await;
        graphql_server.stop().await;
    }

    #[actix_web::test]
    async fn list_sets_rejects_resumption_tokens() {
        let graphql_server =
            spawn_graphql_server(mock_graphql_state(make_descending_work_series(1))).await;
        let export_server = spawn_export_server(MockExportState::default()).await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState {
                    service: OaiService::new(
                        "https://example.org".to_string(),
                        format!("{}/graphql", graphql_server.base_url),
                        export_server.base_url.clone(),
                    ),
                    retry_after_seconds: DEFAULT_RETRY_AFTER_SECONDS,
                }))
                .service(web::resource("/oai").route(web::get().to(oai_get))),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/oai?verb=ListSets&resumptionToken=abc")
            .to_request();
        let response = test::call_service(&app, req).await;
        let body = String::from_utf8(test::read_body(response).await.to_vec()).expect("body UTF-8");

        assert!(body.contains("<error code=\"badResumptionToken\">"));

        export_server.stop().await;
        graphql_server.stop().await;
    }

    #[actix_web::test]
    async fn list_metadata_formats_is_identifier_aware() {
        let marc_eligible_id = Uuid::from_u128(10);
        let marc_ineligible_id = Uuid::from_u128(11);
        let works = vec![
            make_work(
                marc_eligible_id,
                "2024-12-31T12:00:00Z",
                PUBLISHER_NAME,
                true,
                true,
            ),
            make_work(
                marc_ineligible_id,
                "2024-12-30T12:00:00Z",
                PUBLISHER_NAME,
                false,
                true,
            ),
        ];
        let graphql_server = spawn_graphql_server(mock_graphql_state(works)).await;
        let mut export_state = MockExportState::default();
        export_state
            .non_disseminatable_work_ids
            .insert(marc_ineligible_id);
        let export_server = spawn_export_server(export_state).await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState {
                    service: OaiService::new(
                        "https://example.org".to_string(),
                        format!("{}/graphql", graphql_server.base_url),
                        export_server.base_url.clone(),
                    ),
                    retry_after_seconds: DEFAULT_RETRY_AFTER_SECONDS,
                }))
                .service(web::resource("/oai").route(web::get().to(oai_get))),
        )
        .await;

        let eligible_req = test::TestRequest::get()
            .uri(&format!(
                "/oai?verb=ListMetadataFormats&identifier={}",
                OaiService::oai_identifier(marc_eligible_id)
            ))
            .to_request();
        let eligible_response = test::call_service(&app, eligible_req).await;
        let eligible_body = String::from_utf8(test::read_body(eligible_response).await.to_vec())
            .expect("eligible body UTF-8");
        assert!(eligible_body.contains("<metadataPrefix>oai_dc</metadataPrefix>"));
        assert!(eligible_body.contains("<metadataPrefix>oai_openaire</metadataPrefix>"));
        assert!(eligible_body.contains("<metadataPrefix>marcxml</metadataPrefix>"));

        let ineligible_req = test::TestRequest::get()
            .uri(&format!(
                "/oai?verb=ListMetadataFormats&identifier={}",
                OaiService::oai_identifier(marc_ineligible_id)
            ))
            .to_request();
        let ineligible_response = test::call_service(&app, ineligible_req).await;
        let ineligible_body =
            String::from_utf8(test::read_body(ineligible_response).await.to_vec())
                .expect("ineligible body UTF-8");
        assert!(ineligible_body.contains("<metadataPrefix>oai_dc</metadataPrefix>"));
        assert!(ineligible_body.contains("<metadataPrefix>oai_openaire</metadataPrefix>"));
        assert!(!ineligible_body.contains("<metadataPrefix>marcxml</metadataPrefix>"));

        let invalid_identifier_req = test::TestRequest::get()
            .uri(
                "/oai?verb=ListMetadataFormats&identifier=oai:example.org:00000000-0000-0000-0000-000000000001",
            )
            .to_request();
        let invalid_identifier_response = test::call_service(&app, invalid_identifier_req).await;
        let invalid_identifier_body =
            String::from_utf8(test::read_body(invalid_identifier_response).await.to_vec())
                .expect("invalid identifier body UTF-8");
        assert!(invalid_identifier_body.contains("<error code=\"idDoesNotExist\">"));

        export_server.stop().await;
        graphql_server.stop().await;
    }

    #[actix_web::test]
    async fn marc_export_parse_failures_are_mapped_to_cannot_disseminate_format() {
        let work_id = Uuid::from_u128(20);
        let works = vec![make_work(
            work_id,
            "2024-12-31T12:00:00Z",
            PUBLISHER_NAME,
            true,
            true,
        )];
        let graphql_server = spawn_graphql_server(mock_graphql_state(works)).await;

        let mut export_state = MockExportState::default();
        export_state.malformed_work_ids.insert(work_id);
        let export_server = spawn_export_server(export_state).await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState {
                    service: OaiService::new(
                        "https://example.org".to_string(),
                        format!("{}/graphql", graphql_server.base_url),
                        export_server.base_url.clone(),
                    ),
                    retry_after_seconds: DEFAULT_RETRY_AFTER_SECONDS,
                }))
                .service(web::resource("/oai").route(web::get().to(oai_get))),
        )
        .await;

        let req = test::TestRequest::get()
            .uri(&format!(
                "/oai?verb=GetRecord&identifier={}&metadataPrefix=marcxml",
                OaiService::oai_identifier(work_id)
            ))
            .to_request();
        let response = test::call_service(&app, req).await;
        let body = String::from_utf8(test::read_body(response).await.to_vec()).expect("body UTF-8");
        assert!(body.contains("<error code=\"cannotDisseminateFormat\">"));

        export_server.stop().await;
        graphql_server.stop().await;
    }

    #[actix_web::test]
    async fn list_identifiers_validates_datestamp_arguments() {
        let graphql_server =
            spawn_graphql_server(mock_graphql_state(make_descending_work_series(3))).await;
        let export_server = spawn_export_server(MockExportState::default()).await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState {
                    service: OaiService::new(
                        "https://example.org".to_string(),
                        format!("{}/graphql", graphql_server.base_url),
                        export_server.base_url.clone(),
                    ),
                    retry_after_seconds: DEFAULT_RETRY_AFTER_SECONDS,
                }))
                .service(web::resource("/oai").route(web::get().to(oai_get))),
        )
        .await;

        let mismatch_req = test::TestRequest::get()
            .uri(
                "/oai?verb=ListIdentifiers&metadataPrefix=oai_dc&from=2024-01-01&until=2024-01-01T00:00:00Z",
            )
            .to_request();
        let mismatch_response = test::call_service(&app, mismatch_req).await;
        let mismatch_body = String::from_utf8(test::read_body(mismatch_response).await.to_vec())
            .expect("mismatch body UTF-8");
        assert!(mismatch_body.contains("<error code=\"badArgument\">"));

        let invalid_req = test::TestRequest::get()
            .uri("/oai?verb=ListIdentifiers&metadataPrefix=oai_dc&from=20240101")
            .to_request();
        let invalid_response = test::call_service(&app, invalid_req).await;
        let invalid_body = String::from_utf8(test::read_body(invalid_response).await.to_vec())
            .expect("invalid body UTF-8");
        assert!(invalid_body.contains("<error code=\"badArgument\">"));

        let reversed_req = test::TestRequest::get()
            .uri("/oai?verb=ListIdentifiers&metadataPrefix=oai_dc&from=2024-12-31&until=2024-01-01")
            .to_request();
        let reversed_response = test::call_service(&app, reversed_req).await;
        let reversed_body = String::from_utf8(test::read_body(reversed_response).await.to_vec())
            .expect("reversed body UTF-8");
        assert!(reversed_body.contains("<error code=\"badArgument\">"));

        export_server.stop().await;
        graphql_server.stop().await;
    }

    #[actix_web::test]
    async fn list_identifiers_applies_date_filters_and_reports_no_records_match() {
        let works = vec![
            make_work(
                Uuid::from_u128(30),
                "2024-03-01T12:00:00Z",
                PUBLISHER_NAME,
                true,
                true,
            ),
            make_work(
                Uuid::from_u128(31),
                "2024-02-01T12:00:00Z",
                PUBLISHER_NAME,
                true,
                true,
            ),
            make_work(
                Uuid::from_u128(32),
                "2023-12-31T12:00:00Z",
                PUBLISHER_NAME,
                true,
                true,
            ),
        ];
        let graphql_server = spawn_graphql_server(mock_graphql_state(works)).await;
        let export_server = spawn_export_server(MockExportState::default()).await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState {
                    service: OaiService::new(
                        "https://example.org".to_string(),
                        format!("{}/graphql", graphql_server.base_url),
                        export_server.base_url.clone(),
                    ),
                    retry_after_seconds: DEFAULT_RETRY_AFTER_SECONDS,
                }))
                .service(web::resource("/oai").route(web::get().to(oai_get))),
        )
        .await;

        let from_req = test::TestRequest::get()
            .uri("/oai?verb=ListIdentifiers&metadataPrefix=oai_dc&from=2024-01-01")
            .to_request();
        let from_response = test::call_service(&app, from_req).await;
        let from_body = String::from_utf8(test::read_body(from_response).await.to_vec())
            .expect("from body UTF-8");
        assert_eq!(count_occurrences(&from_body, "<header>"), 2);

        let until_req = test::TestRequest::get()
            .uri("/oai?verb=ListIdentifiers&metadataPrefix=oai_dc&until=2024-01-31")
            .to_request();
        let until_response = test::call_service(&app, until_req).await;
        let until_body = String::from_utf8(test::read_body(until_response).await.to_vec())
            .expect("until body UTF-8");
        assert_eq!(count_occurrences(&until_body, "<header>"), 1);

        let range_req = test::TestRequest::get()
            .uri("/oai?verb=ListIdentifiers&metadataPrefix=oai_dc&from=2024-01-01&until=2024-02-15")
            .to_request();
        let range_response = test::call_service(&app, range_req).await;
        let range_body = String::from_utf8(test::read_body(range_response).await.to_vec())
            .expect("range body UTF-8");
        assert_eq!(count_occurrences(&range_body, "<header>"), 1);

        let no_match_req = test::TestRequest::get()
            .uri("/oai?verb=ListIdentifiers&metadataPrefix=oai_dc&from=2030-01-01")
            .to_request();
        let no_match_response = test::call_service(&app, no_match_req).await;
        let no_match_body = String::from_utf8(test::read_body(no_match_response).await.to_vec())
            .expect("no match body UTF-8");
        assert!(no_match_body.contains("<error code=\"noRecordsMatch\">"));

        export_server.stop().await;
        graphql_server.stop().await;
    }

    #[actix_web::test]
    async fn resumption_tokens_support_filters_backward_compatibility_and_terminal_token() {
        let works = make_descending_work_series(60);
        let graphql_server = spawn_graphql_server(mock_graphql_state(works)).await;
        let export_server = spawn_export_server(MockExportState::default()).await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState {
                    service: OaiService::new(
                        "https://example.org".to_string(),
                        format!("{}/graphql", graphql_server.base_url),
                        export_server.base_url.clone(),
                    ),
                    retry_after_seconds: DEFAULT_RETRY_AFTER_SECONDS,
                }))
                .service(web::resource("/oai").route(web::get().to(oai_get))),
        )
        .await;

        let filtered_first_req = test::TestRequest::get()
            .uri("/oai?verb=ListIdentifiers&metadataPrefix=oai_dc&from=2024-11-10&until=2024-12-31")
            .to_request();
        let filtered_first_response = test::call_service(&app, filtered_first_req).await;
        let filtered_first_body =
            String::from_utf8(test::read_body(filtered_first_response).await.to_vec())
                .expect("first filtered body UTF-8");
        assert_eq!(count_occurrences(&filtered_first_body, "<header>"), 50);
        assert!(filtered_first_body.contains("<resumptionToken cursor=\"0\">"));
        assert!(!filtered_first_body.contains("completeListSize=\""));

        let filtered_token =
            extract_resumption_token(&filtered_first_body).expect("filtered resumption token");
        let decoded_filtered = OaiService::decode_resumption_token(&filtered_token)
            .expect("decode filtered resumption token");
        assert_eq!(decoded_filtered.from.as_deref(), Some("2024-11-10"));
        assert_eq!(decoded_filtered.until.as_deref(), Some("2024-12-31"));
        assert_eq!(
            decoded_filtered.granularity,
            Some(DatestampGranularity::Day)
        );
        assert!(decoded_filtered.scan_offset.is_some());
        assert_eq!(decoded_filtered.returned_count, Some(50));

        let filtered_second_req = test::TestRequest::get()
            .uri(&format!(
                "/oai?verb=ListIdentifiers&resumptionToken={filtered_token}"
            ))
            .to_request();
        let filtered_second_response = test::call_service(&app, filtered_second_req).await;
        let filtered_second_body =
            String::from_utf8(test::read_body(filtered_second_response).await.to_vec())
                .expect("second filtered body UTF-8");
        assert_eq!(count_occurrences(&filtered_second_body, "<header>"), 2);
        assert!(filtered_second_body.contains("<resumptionToken/>"));

        let unfiltered_req = test::TestRequest::get()
            .uri("/oai?verb=ListIdentifiers&metadataPrefix=oai_dc")
            .to_request();
        let unfiltered_response = test::call_service(&app, unfiltered_req).await;
        let unfiltered_body =
            String::from_utf8(test::read_body(unfiltered_response).await.to_vec())
                .expect("unfiltered body UTF-8");
        assert!(unfiltered_body.contains("completeListSize=\"60\""));

        let legacy_token = URL_SAFE_NO_PAD.encode(
            serde_json::to_vec(&json!({
                "offset": 0,
                "metadata_prefix": "OaiDc",
                "set": null,
                "identifiers_only": true
            }))
            .expect("legacy token serialize"),
        );
        let legacy_req = test::TestRequest::get()
            .uri(&format!(
                "/oai?verb=ListIdentifiers&resumptionToken={legacy_token}"
            ))
            .to_request();
        let legacy_response = test::call_service(&app, legacy_req).await;
        let legacy_body = String::from_utf8(test::read_body(legacy_response).await.to_vec())
            .expect("legacy response body UTF-8");
        assert!(legacy_body.contains("<ListIdentifiers>"));

        let malformed_req = test::TestRequest::get()
            .uri("/oai?verb=ListIdentifiers&resumptionToken=not-a-token")
            .to_request();
        let malformed_response = test::call_service(&app, malformed_req).await;
        let malformed_body = String::from_utf8(test::read_body(malformed_response).await.to_vec())
            .expect("malformed response body UTF-8");
        assert!(malformed_body.contains("<error code=\"badResumptionToken\">"));

        export_server.stop().await;
        graphql_server.stop().await;
    }

    #[actix_web::test]
    async fn filtered_resumption_cursor_tracks_returned_records() {
        let works = make_descending_work_series(120);
        let graphql_server = spawn_graphql_server(mock_graphql_state(works)).await;
        let export_server = spawn_export_server(MockExportState::default()).await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState {
                    service: OaiService::new(
                        "https://example.org".to_string(),
                        format!("{}/graphql", graphql_server.base_url),
                        export_server.base_url.clone(),
                    ),
                    retry_after_seconds: DEFAULT_RETRY_AFTER_SECONDS,
                }))
                .service(web::resource("/oai").route(web::get().to(oai_get))),
        )
        .await;

        let first_req = test::TestRequest::get()
            .uri("/oai?verb=ListIdentifiers&metadataPrefix=oai_dc&from=2024-09-13&until=2024-12-31")
            .to_request();
        let first_response = test::call_service(&app, first_req).await;
        let first_body = String::from_utf8(test::read_body(first_response).await.to_vec())
            .expect("first page UTF-8");
        assert!(first_body.contains("<resumptionToken cursor=\"0\">"));
        let first_token = extract_resumption_token(&first_body).expect("first token");

        let second_req = test::TestRequest::get()
            .uri(&format!(
                "/oai?verb=ListIdentifiers&resumptionToken={first_token}"
            ))
            .to_request();
        let second_response = test::call_service(&app, second_req).await;
        let second_body = String::from_utf8(test::read_body(second_response).await.to_vec())
            .expect("second page UTF-8");
        assert_eq!(count_occurrences(&second_body, "<header>"), 50);
        assert!(second_body.contains("<resumptionToken cursor=\"50\">"));
        let second_token = extract_resumption_token(&second_body).expect("second token");
        let decoded_second = OaiService::decode_resumption_token(&second_token).unwrap();
        assert_eq!(decoded_second.returned_count, Some(100));

        let third_req = test::TestRequest::get()
            .uri(&format!(
                "/oai?verb=ListIdentifiers&resumptionToken={second_token}"
            ))
            .to_request();
        let third_response = test::call_service(&app, third_req).await;
        let third_body = String::from_utf8(test::read_body(third_response).await.to_vec())
            .expect("third page UTF-8");
        assert_eq!(count_occurrences(&third_body, "<header>"), 10);
        assert!(third_body.contains("<resumptionToken/>"));

        export_server.stop().await;
        graphql_server.stop().await;
    }

    #[actix_web::test]
    async fn continuation_end_returns_terminal_token_without_no_records_match() {
        let works = make_descending_work_series(120);
        let graphql_server = spawn_graphql_server(mock_graphql_state(works)).await;
        let export_server = spawn_export_server(MockExportState::default()).await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState {
                    service: OaiService::new(
                        "https://example.org".to_string(),
                        format!("{}/graphql", graphql_server.base_url),
                        export_server.base_url.clone(),
                    ),
                    retry_after_seconds: DEFAULT_RETRY_AFTER_SECONDS,
                }))
                .service(web::resource("/oai").route(web::get().to(oai_get))),
        )
        .await;

        let first_req = test::TestRequest::get()
            .uri("/oai?verb=ListIdentifiers&metadataPrefix=oai_dc&from=2024-11-12&until=2024-12-31")
            .to_request();
        let first_response = test::call_service(&app, first_req).await;
        let first_body = String::from_utf8(test::read_body(first_response).await.to_vec())
            .expect("first page UTF-8");
        assert_eq!(count_occurrences(&first_body, "<header>"), 50);
        assert!(!first_body.contains("<resumptionToken"));

        let stale_token = OaiService::encode_resumption_token(ResumptionToken {
            offset: 50,
            metadata_prefix: MetadataPrefix::OaiDc,
            set: None,
            identifiers_only: true,
            from: Some("2024-11-12".to_string()),
            until: Some("2024-12-31".to_string()),
            granularity: Some(DatestampGranularity::Day),
            scan_offset: Some(50),
            returned_count: Some(50),
        });
        let continuation_req = test::TestRequest::get()
            .uri(&format!(
                "/oai?verb=ListIdentifiers&resumptionToken={stale_token}"
            ))
            .to_request();
        let continuation_response = test::call_service(&app, continuation_req).await;
        let continuation_body =
            String::from_utf8(test::read_body(continuation_response).await.to_vec())
                .expect("continuation body UTF-8");
        assert!(continuation_body.contains("<ListIdentifiers>"));
        assert!(continuation_body.contains("<resumptionToken/>"));
        assert!(!continuation_body.contains("<error code=\"noRecordsMatch\">"));

        export_server.stop().await;
        graphql_server.stop().await;
    }

    #[actix_web::test]
    async fn gzip_accept_encoding_returns_compressed_oai_xml() {
        let works = make_descending_work_series(1);
        let graphql_server = spawn_graphql_server(mock_graphql_state(works)).await;
        let export_server = spawn_export_server(MockExportState::default()).await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState {
                    service: OaiService::new(
                        "https://example.org".to_string(),
                        format!("{}/graphql", graphql_server.base_url),
                        export_server.base_url.clone(),
                    ),
                    retry_after_seconds: DEFAULT_RETRY_AFTER_SECONDS,
                }))
                .service(web::resource("/oai").route(web::get().to(oai_get))),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/oai?verb=Identify")
            .insert_header((header::ACCEPT_ENCODING, "gzip"))
            .to_request();
        let response = test::call_service(&app, req).await;
        assert_eq!(response.status(), actix_web::http::StatusCode::OK);
        assert_eq!(
            response
                .headers()
                .get(header::CONTENT_ENCODING)
                .and_then(|value| value.to_str().ok()),
            Some("gzip")
        );

        let compressed = test::read_body(response).await;
        let mut decoder = GzDecoder::new(compressed.as_ref());
        let mut xml = String::new();
        decoder
            .read_to_string(&mut xml)
            .expect("gzip decode response");
        assert!(xml.contains("<Identify>"));
        assert!(xml.contains("<compression>gzip</compression>"));

        export_server.stop().await;
        graphql_server.stop().await;
    }

    #[actix_web::test]
    async fn transient_graphql_failures_return_503_with_retry_after() {
        let graphql_server =
            spawn_graphql_error_server(actix_web::http::StatusCode::SERVICE_UNAVAILABLE).await;
        let export_server = spawn_export_server(MockExportState::default()).await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState {
                    service: OaiService::new(
                        "https://example.org".to_string(),
                        format!("{}/graphql", graphql_server.base_url),
                        export_server.base_url.clone(),
                    ),
                    retry_after_seconds: 45,
                }))
                .service(web::resource("/oai").route(web::get().to(oai_get))),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/oai?verb=Identify")
            .to_request();
        let response = test::call_service(&app, req).await;
        assert_eq!(
            response.status(),
            actix_web::http::StatusCode::SERVICE_UNAVAILABLE
        );
        assert_eq!(
            response
                .headers()
                .get(header::RETRY_AFTER)
                .and_then(|value| value.to_str().ok()),
            Some("45")
        );

        export_server.stop().await;
        graphql_server.stop().await;
    }

    #[actix_web::test]
    async fn non_transient_graphql_failures_remain_http_500() {
        let graphql_server =
            spawn_graphql_error_server(actix_web::http::StatusCode::BAD_REQUEST).await;
        let export_server = spawn_export_server(MockExportState::default()).await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState {
                    service: OaiService::new(
                        "https://example.org".to_string(),
                        format!("{}/graphql", graphql_server.base_url),
                        export_server.base_url.clone(),
                    ),
                    retry_after_seconds: DEFAULT_RETRY_AFTER_SECONDS,
                }))
                .service(web::resource("/oai").route(web::get().to(oai_get))),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/oai?verb=Identify")
            .to_request();
        let response = test::call_service(&app, req).await;
        assert_eq!(
            response.status(),
            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
        );
        assert!(response.headers().get(header::RETRY_AFTER).is_none());

        export_server.stop().await;
        graphql_server.stop().await;
    }

    #[actix_web::test]
    async fn transient_export_failures_return_503_with_retry_after() {
        let work_id = Uuid::from_u128(21);
        let works = vec![make_work(
            work_id,
            "2024-12-31T12:00:00Z",
            PUBLISHER_NAME,
            true,
            true,
        )];
        let graphql_server = spawn_graphql_server(mock_graphql_state(works)).await;

        let mut export_state = MockExportState::default();
        export_state.failing_work_ids.insert(work_id);
        let export_server = spawn_export_server(export_state).await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState {
                    service: OaiService::new(
                        "https://example.org".to_string(),
                        format!("{}/graphql", graphql_server.base_url),
                        export_server.base_url.clone(),
                    ),
                    retry_after_seconds: DEFAULT_RETRY_AFTER_SECONDS,
                }))
                .service(web::resource("/oai").route(web::get().to(oai_get))),
        )
        .await;

        let req = test::TestRequest::get()
            .uri(&format!(
                "/oai?verb=GetRecord&identifier={}&metadataPrefix=marcxml",
                OaiService::oai_identifier(work_id)
            ))
            .to_request();
        let response = test::call_service(&app, req).await;
        assert_eq!(
            response.status(),
            actix_web::http::StatusCode::SERVICE_UNAVAILABLE
        );
        assert_eq!(
            response
                .headers()
                .get(header::RETRY_AFTER)
                .and_then(|value| value.to_str().ok()),
            Some("30")
        );

        export_server.stop().await;
        graphql_server.stop().await;
    }

    #[actix_web::test]
    async fn marcxml_list_filters_use_export_dissemination_truth() {
        let disseminatable_work_id = Uuid::from_u128(31);
        let non_disseminatable_work_id = Uuid::from_u128(32);
        let second_disseminatable_work_id = Uuid::from_u128(33);
        let works = vec![
            make_work(
                disseminatable_work_id,
                "2024-12-31T12:00:00Z",
                PUBLISHER_NAME,
                true,
                true,
            ),
            make_work(
                non_disseminatable_work_id,
                "2024-12-30T12:00:00Z",
                PUBLISHER_NAME,
                true,
                true,
            ),
            make_work(
                second_disseminatable_work_id,
                "2024-12-29T12:00:00Z",
                PUBLISHER_NAME,
                true,
                true,
            ),
        ];
        let graphql_server = spawn_graphql_server(mock_graphql_state(works)).await;

        let mut export_state = MockExportState::default();
        export_state
            .non_disseminatable_work_ids
            .insert(non_disseminatable_work_id);
        let export_server = spawn_export_server(export_state).await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState {
                    service: OaiService::new(
                        "https://example.org".to_string(),
                        format!("{}/graphql", graphql_server.base_url),
                        export_server.base_url.clone(),
                    ),
                    retry_after_seconds: DEFAULT_RETRY_AFTER_SECONDS,
                }))
                .service(web::resource("/oai").route(web::get().to(oai_get))),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/oai?verb=ListIdentifiers&metadataPrefix=marcxml")
            .to_request();
        let response = test::call_service(&app, req).await;
        assert_eq!(response.status(), actix_web::http::StatusCode::OK);
        let body = String::from_utf8(test::read_body(response).await.to_vec()).expect("body UTF-8");

        assert_eq!(count_occurrences(&body, "<header>"), 2);
        assert!(body.contains(&OaiService::oai_identifier(disseminatable_work_id)));
        assert!(body.contains(&OaiService::oai_identifier(second_disseminatable_work_id)));
        assert!(!body.contains(&OaiService::oai_identifier(non_disseminatable_work_id)));

        export_server.stop().await;
        graphql_server.stop().await;
    }

    #[actix_web::test]
    async fn marcxml_list_records_excludes_non_disseminatable_records() {
        let visible_work_id = Uuid::from_u128(35);
        let hidden_work_id = Uuid::from_u128(36);
        let works = vec![
            make_work(
                visible_work_id,
                "2024-12-31T12:00:00Z",
                PUBLISHER_NAME,
                true,
                true,
            ),
            make_work(
                hidden_work_id,
                "2024-12-30T12:00:00Z",
                PUBLISHER_NAME,
                true,
                true,
            ),
        ];
        let graphql_server = spawn_graphql_server(mock_graphql_state(works)).await;

        let mut export_state = MockExportState::default();
        export_state
            .non_disseminatable_work_ids
            .insert(hidden_work_id);
        let export_server = spawn_export_server(export_state).await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState {
                    service: OaiService::new(
                        "https://example.org".to_string(),
                        format!("{}/graphql", graphql_server.base_url),
                        export_server.base_url.clone(),
                    ),
                    retry_after_seconds: DEFAULT_RETRY_AFTER_SECONDS,
                }))
                .service(web::resource("/oai").route(web::get().to(oai_get))),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/oai?verb=ListRecords&metadataPrefix=marcxml")
            .to_request();
        let response = test::call_service(&app, req).await;
        let body = String::from_utf8(test::read_body(response).await.to_vec()).expect("body UTF-8");

        assert_eq!(count_occurrences(&body, "<record>"), 1);
        assert!(body.contains(&OaiService::oai_identifier(visible_work_id)));
        assert!(!body.contains(&OaiService::oai_identifier(hidden_work_id)));

        export_server.stop().await;
        graphql_server.stop().await;
    }

    #[actix_web::test]
    async fn marcxml_list_resumption_respects_export_dissemination_filtering() {
        let works = make_descending_work_series(80);
        let graphql_server = spawn_graphql_server(mock_graphql_state(works.clone())).await;

        let mut export_state = MockExportState::default();
        for (index, work) in works.iter().enumerate() {
            let work_id = work
                .get("workId")
                .and_then(Value::as_str)
                .and_then(|value| Uuid::parse_str(value).ok())
                .expect("work id");
            if index % 4 == 0 {
                export_state.non_disseminatable_work_ids.insert(work_id);
            }
        }
        let export_server = spawn_export_server(export_state).await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState {
                    service: OaiService::new(
                        "https://example.org".to_string(),
                        format!("{}/graphql", graphql_server.base_url),
                        export_server.base_url.clone(),
                    ),
                    retry_after_seconds: DEFAULT_RETRY_AFTER_SECONDS,
                }))
                .service(web::resource("/oai").route(web::get().to(oai_get))),
        )
        .await;

        let first_req = test::TestRequest::get()
            .uri("/oai?verb=ListIdentifiers&metadataPrefix=marcxml")
            .to_request();
        let first_response = test::call_service(&app, first_req).await;
        let first_body = String::from_utf8(test::read_body(first_response).await.to_vec())
            .expect("first page UTF-8");
        assert_eq!(count_occurrences(&first_body, "<header>"), 50);
        assert!(first_body.contains("<resumptionToken cursor=\"0\">"));
        let first_token = extract_resumption_token(&first_body).expect("resumption token");

        let second_req = test::TestRequest::get()
            .uri(&format!(
                "/oai?verb=ListIdentifiers&resumptionToken={first_token}"
            ))
            .to_request();
        let second_response = test::call_service(&app, second_req).await;
        let second_body = String::from_utf8(test::read_body(second_response).await.to_vec())
            .expect("second page UTF-8");
        assert_eq!(count_occurrences(&second_body, "<header>"), 10);
        assert!(second_body.contains("<resumptionToken/>"));

        export_server.stop().await;
        graphql_server.stop().await;
    }

    #[actix_web::test]
    async fn transient_export_failures_in_marcxml_lists_return_503_with_retry_after() {
        let work_id = Uuid::from_u128(34);
        let works = vec![make_work(
            work_id,
            "2024-12-31T12:00:00Z",
            PUBLISHER_NAME,
            true,
            true,
        )];
        let graphql_server = spawn_graphql_server(mock_graphql_state(works)).await;

        let mut export_state = MockExportState::default();
        export_state.failing_work_ids.insert(work_id);
        let export_server = spawn_export_server(export_state).await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState {
                    service: OaiService::new(
                        "https://example.org".to_string(),
                        format!("{}/graphql", graphql_server.base_url),
                        export_server.base_url.clone(),
                    ),
                    retry_after_seconds: DEFAULT_RETRY_AFTER_SECONDS,
                }))
                .service(web::resource("/oai").route(web::get().to(oai_get))),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/oai?verb=ListIdentifiers&metadataPrefix=marcxml")
            .to_request();
        let response = test::call_service(&app, req).await;
        assert_eq!(
            response.status(),
            actix_web::http::StatusCode::SERVICE_UNAVAILABLE
        );
        assert_eq!(
            response
                .headers()
                .get(header::RETRY_AFTER)
                .and_then(|value| value.to_str().ok()),
            Some("30")
        );

        export_server.stop().await;
        graphql_server.stop().await;
    }
}
