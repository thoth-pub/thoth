mod metadata;
mod service;

use std::{collections::HashMap, io, time::Duration};

use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpRequest, HttpResponse, HttpServer};
use chrono::Utc;
use quick_xml::escape::escape;
use service::{
    MetadataPrefix, OaiService, RecordPage, ResumptionToken, ADMIN_EMAIL, RECORD_PREFIX,
    REPOSITORY_NAME, SAMPLE_ID,
};
use thoth_errors::ThothError;
use uuid::Uuid;

const LOG_FORMAT: &str = r#"%{r}a %a "%r" %s %b "%{Referer}i" "%{User-Agent}i" %T"#;
const XSL_STYLESHEET: &str = include_str!("../assets/oai2.xsl");

#[derive(Clone)]
struct AppState {
    service: OaiService,
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

async fn oai(
    request: HttpRequest,
    params: web::Query<HashMap<String, String>>,
    state: web::Data<AppState>,
) -> HttpResponse {
    let params = params.into_inner();
    match handle_oai_request(&request, &params, &state.service).await {
        Ok(body) => xml_response(success_document(&state.service, &params, &body)),
        Err(HandlerError::Protocol(error)) => xml_response(error_document(
            &state.service,
            &params,
            error.code,
            &error.message,
        )),
        Err(HandlerError::Internal(error)) => {
            log::error!("OAI request failed: {error}");
            HttpResponse::InternalServerError()
                .content_type("text/plain; charset=utf-8")
                .body("Internal Server Error")
        }
    }
}

async fn not_found() -> HttpResponse {
    HttpResponse::NotFound()
        .content_type("text/html; charset=utf-8")
        .body(
            r#"<!DOCTYPE html>
<html>
<head>
  <title>404 - Page Not Found</title>
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
</html>"#,
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
            if let Some(identifier) = params.get("identifier") {
                let work_id = parse_identifier(identifier)?;
                service
                    .get_record(work_id, MetadataPrefix::OaiDc)
                    .await
                    .map_err(map_get_record_error(MetadataPrefix::OaiDc))?;
            }
            Ok(render_list_metadata_formats())
        }
        "ListSets" => {
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
            let work_id = parse_identifier(identifier)?;
            let metadata_prefix = parse_metadata_prefix(metadata_prefix)?;
            let work = service
                .get_record(work_id, metadata_prefix)
                .await
                .map_err(map_get_record_error(metadata_prefix))?;
            Ok(render_get_record(service, &work, metadata_prefix).await?)
        }
        "ListIdentifiers" => {
            validate_list_verb(params)?;
            let token = parse_list_token(params, true)?;
            let page = service
                .list_records(token.metadata_prefix, token.set.clone(), token.offset, true)
                .await
                .map_err(map_list_error)?;
            if page.records.is_empty() {
                return Err(HandlerError::Protocol(no_records_match()));
            }
            Ok(render_list_identifiers(&page))
        }
        "ListRecords" => {
            validate_list_verb(params)?;
            let token = parse_list_token(params, false)?;
            let page = service
                .list_records(
                    token.metadata_prefix,
                    token.set.clone(),
                    token.offset,
                    false,
                )
                .await
                .map_err(map_list_error)?;
            if page.records.is_empty() {
                return Err(HandlerError::Protocol(no_records_match()));
            }
            Ok(render_list_records(service, &page, token.metadata_prefix).await?)
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
    )
}

fn render_list_metadata_formats() -> String {
    let prefixes = [
        MetadataPrefix::OaiDc,
        MetadataPrefix::OaiOpenaire,
        MetadataPrefix::MarcXml,
    ];
    let mut xml = String::from("<ListMetadataFormats>");
    for prefix in prefixes {
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
    if let Some(token) = &page.next_token {
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
    if let Some(token) = &page.next_token {
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
            .map_err(map_get_record_error(metadata_prefix))?,
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

fn render_resumption_token(token: &str, cursor: i64, complete_list_size: i64) -> String {
    format!(
        "<resumptionToken cursor=\"{}\" completeListSize=\"{}\">{}</resumptionToken>",
        cursor,
        complete_list_size,
        xml_escape(token)
    )
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
) -> HandlerResult<ResumptionToken> {
    if let Some(value) = params.get("resumptionToken") {
        if params.len() != 2 {
            return Err(
                bad_argument("resumptionToken cannot be combined with other arguments").into(),
            );
        }
        let token = OaiService::decode_resumption_token(value).map_err(|_| ProtocolError {
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
        return Ok(token);
    }

    let metadata_prefix = params
        .get("metadataPrefix")
        .ok_or_else(|| bad_argument("Missing metadataPrefix parameter"))?;
    Ok(ResumptionToken {
        offset: 0,
        metadata_prefix: parse_metadata_prefix(metadata_prefix)?,
        set: params.get("set").cloned(),
        identifiers_only,
    })
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

fn parse_identifier(value: &str) -> HandlerResult<Uuid> {
    OaiService::parse_oai_identifier(value).map_err(|_| bad_argument("Invalid identifier").into())
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

fn map_list_error(error: ThothError) -> HandlerError {
    match error {
        ThothError::EntityNotFound => no_records_match().into(),
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
        request_element(service, params),
        body
    )
}

fn error_document(
    service: &OaiService,
    params: &HashMap<String, String>,
    code: &str,
    message: &str,
) -> String {
    format!(
        "{}{}<OAI-PMH xmlns=\"http://www.openarchives.org/OAI/2.0/\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xsi:schemaLocation=\"http://www.openarchives.org/OAI/2.0/ http://www.openarchives.org/OAI/2.0/OAI-PMH.xsd\"><responseDate>{}</responseDate>{}<error code=\"{}\">{}</error></OAI-PMH>",
        xml_declaration(),
        stylesheet_pi(),
        response_date(),
        request_element(service, params),
        xml_escape(code),
        xml_escape(message)
    )
}

fn request_element(service: &OaiService, params: &HashMap<String, String>) -> String {
    let mut attrs = params.iter().collect::<Vec<_>>();
    attrs.sort_by(|(left, _), (right, _)| left.cmp(right));
    let mut element = String::from("<request");
    for (key, value) in attrs {
        element.push(' ');
        element.push_str(key);
        element.push_str("=\"");
        element.push_str(&xml_escape(value));
        element.push('"');
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

fn xml_response(body: String) -> HttpResponse {
    HttpResponse::Ok()
        .content_type("application/xml; charset=utf-8")
        .body(body)
}

#[actix_web::main]
pub async fn start_server(
    host: String,
    port: String,
    threads: usize,
    keep_alive: u64,
    public_url: String,
    gql_endpoint: String,
    export_url: String,
) -> io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let state = AppState {
        service: OaiService::new(public_url, gql_endpoint, export_url),
    };

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::new(LOG_FORMAT))
            .wrap(Cors::default().allowed_methods(vec!["GET", "OPTIONS"]))
            .app_data(web::Data::new(state.clone()))
            .service(web::resource("/").route(web::get().to(index)))
            .service(web::resource("/oai").route(web::get().to(oai)))
            .service(web::resource("/oai2.xsl").route(web::get().to(stylesheet)))
            .default_service(web::route().to(not_found))
    })
    .workers(threads)
    .keep_alive(Duration::from_secs(keep_alive))
    .bind(format!("{host}:{port}"))?
    .run()
    .await
}
