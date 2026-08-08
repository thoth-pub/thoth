mod graphiql;
mod logger;

use std::{io, sync::Arc, time::Duration};

use actix_cors::Cors;
use actix_web::{
    get,
    http::header,
    middleware::Compress,
    post,
    web::{Data, Json},
    App, Error, HttpResponse, HttpServer, Result,
};
use base64::{engine::general_purpose, Engine as _};
use serde::Serialize;
use thoth_api::{
    db::{init_pool, PgPool},
    graphql::{
        create_schema, run_mutation_guard, Context, GraphQLRequest, MutationGuardMode, Schema,
    },
    storage::{create_cloudfront_client, create_s3_client, CloudFrontClient, S3Client},
};
use zitadel::{
    actix::introspection::{IntrospectedUser, IntrospectionConfigBuilder},
    credentials::Application,
};

use crate::graphiql::graphiql_source;
use crate::logger::{BodyLogger, Logger};

#[derive(Serialize)]
struct ApiConfig {
    api_name: String,
    api_version: String,
    api_schema: String,
    public_url: String,
    schema_explorer_url: String,
}

impl ApiConfig {
    pub fn new(public_url: String) -> Self {
        Self {
            public_url: format!("{public_url}/graphql"),
            schema_explorer_url: format!("{public_url}/graphiql"),
            ..Default::default()
        }
    }
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            api_name: "Thoth Metadata GraphQL API".to_string(),
            api_version: env!("CARGO_PKG_VERSION").parse().unwrap(),
            api_schema: "".to_string(),
            public_url: "".to_string(),
            schema_explorer_url: "".to_string(),
        }
    }
}

#[get("/")]
async fn index(config: Data<ApiConfig>) -> HttpResponse {
    HttpResponse::Ok().json(config.get_ref())
}

#[get("/graphiql")]
async fn graphiql_interface(config: Data<ApiConfig>) -> HttpResponse {
    let html = graphiql_source(&config.public_url);
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

#[get("/graphql")]
async fn graphql_index(config: Data<ApiConfig>) -> HttpResponse {
    HttpResponse::MethodNotAllowed().json(format!(
        "GraphQL API must be queried making a POST request to {}",
        config.public_url
    ))
}

#[get("/schema.graphql")]
async fn graphql_schema(st: Data<Arc<Schema>>) -> HttpResponse {
    HttpResponse::Ok().body(st.as_sdl())
}

#[post("/graphql")]
async fn graphql(
    st: Data<Arc<Schema>>,
    pool: Data<PgPool>,
    s3_client: Data<S3Client>,
    cloudfront_client: Data<CloudFrontClient>,
    guard_mode: Data<MutationGuardMode>,
    user: Option<IntrospectedUser>,
    data: Json<GraphQLRequest>,
) -> Result<HttpResponse, Error> {
    let mode = *guard_mode.into_inner().as_ref();

    // Both paths produce ONE `GraphQLResponse`, which then flows through the
    // single existing status branch below. A guard rejection is an ordinary
    // validation-style GraphQL response — `is_ok()` is `false` — so it needs no
    // handler branch, no bespoke status and no one-off protocol of its own.
    let result = match run_mutation_guard(mode, &data, &st) {
        // Central mutation request guard (`ADR-0006` section 4.12.6), evaluated
        // at the GraphQL HTTP request boundary **before** ordinary Juniper
        // execution, so a rejected operation runs zero resolvers and performs
        // zero writes.
        //
        // In `OFF` — the default and the merged production state — the guard
        // returns before any parsing, so it adds no request-path work of any
        // kind. It never replaces `GraphQLRequest::execute`, and it makes no
        // authorization decision.
        Some(rejection) => rejection,
        None => {
            // The request context carries the same guard mode, so store
            // availability is derived from it and can never disagree with the
            // guard.
            let ctx = Context::with_guard_mode(
                pool.into_inner(),
                user,
                s3_client.into_inner(),
                cloudfront_client.into_inner(),
                mode,
            );
            data.execute(&st, &ctx).await
        }
    };

    // The single, pre-existing response-status mapping, shared by ordinary
    // Juniper execution and by guard rejection alike.
    match result.is_ok() {
        true => Ok(HttpResponse::Ok().json(result)),
        false => Ok(HttpResponse::BadRequest().json(result)),
    }
}

#[allow(clippy::too_many_arguments)]
#[actix_web::main]
pub async fn start_server(
    database_url: String,
    host: String,
    port: String,
    threads: usize,
    keep_alive: u64,
    public_url: String,
    private_key: String,
    zitadel_url: String,
    mutation_guard_mode: MutationGuardMode,
    aws_access_key_id: String,
    aws_secret_access_key: String,
    aws_region: String,
) -> io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let decoded_private_key = general_purpose::STANDARD
        .decode(&private_key)
        .expect("Failed to base64-decode private key");
    let decoded_str =
        std::str::from_utf8(&decoded_private_key).expect("Decoded key is not valid UTF-8");
    let auth = IntrospectionConfigBuilder::new(&zitadel_url)
        .with_jwt_profile(Application::load_from_json(decoded_str).unwrap())
        .build()
        .await
        .unwrap();

    let s3_client = create_s3_client(&aws_access_key_id, &aws_secret_access_key, &aws_region).await;
    let cloudfront_client =
        create_cloudfront_client(&aws_access_key_id, &aws_secret_access_key, &aws_region).await;
    let pool = Data::new(init_pool(&database_url));

    HttpServer::new(move || {
        App::new()
            .wrap(Compress::default())
            .wrap(Logger::default())
            .wrap(BodyLogger)
            .wrap(
                Cors::default()
                    .allowed_methods(vec!["GET", "POST", "OPTIONS"])
                    .allow_any_origin()
                    .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
                    .allowed_header(header::CONTENT_TYPE)
                    .supports_credentials(),
            )
            .app_data(auth.clone())
            .app_data(Data::new(ApiConfig::new(public_url.clone())))
            .app_data(pool.clone())
            .app_data(Data::new(s3_client.clone()))
            .app_data(Data::new(cloudfront_client.clone()))
            .app_data(Data::new(Arc::new(create_schema())))
            .app_data(Data::new(mutation_guard_mode))
            .service(index)
            .service(graphql_index)
            .service(graphql)
            .service(graphiql_interface)
            .service(graphql_schema)
    })
    .workers(threads)
    .keep_alive(Duration::from_secs(keep_alive))
    .bind(format!("{host}:{port}"))?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    //! Handler-level proof that a guard rejection carries **no** HTTP protocol
    //! of its own.
    //!
    //! THOTH-GQL-BATCH-01 requires a guard rejection to become a
    //! validation-style `GraphQLResponse` that flows through the *existing*
    //! `result.is_ok()` status mapping — "existing handler branch produces
    //! HTTP 400; no new handler branch; no one-off HTTP protocol". These tests
    //! drive the **real** `graphql` handler through `actix_web::test` and
    //! compare a guard rejection against an ordinary Juniper validation failure
    //! on the same route.
    //!
    //! No new dependency is used: `actix_web::test` and the storage-client
    //! constructors are already available to this crate. This crate does not
    //! depend on `serde_json`, so these assertions are made against the raw
    //! serialized body rather than a parsed value; the precise structural
    //! comparison of the two bodies (top-level key sets and per-error key sets)
    //! lives in `thoth-api`, where `serde_json` is available.

    use super::*;
    use actix_web::{http::StatusCode, test, App};

    /// A real pool, built the way the repository's other database-backed tests
    /// build theirs.
    ///
    /// Neither case under test reaches the database — a guard rejection runs no
    /// resolver at all, and Juniper rejects an invalid document during
    /// validation, before any resolver executes — but `Data<PgPool>` is still
    /// extracted, so a value must exist. `TEST_DATABASE_URL` is the same
    /// variable the existing `thoth-api` database tests require, and it is set
    /// in the CI workflow environment.
    fn test_pool() -> PgPool {
        let url = std::env::var("TEST_DATABASE_URL")
            .expect("TEST_DATABASE_URL must be set for thoth-api-server handler tests");
        init_pool(&url)
    }

    /// Build the real application route under test, in the given guard mode.
    async fn service(
        mode: MutationGuardMode,
    ) -> impl actix_web::dev::Service<
        actix_http::Request,
        Response = actix_web::dev::ServiceResponse,
        Error = Error,
    > {
        let s3_client = create_s3_client("test-access-key", "test-secret-key", "us-east-1").await;
        let cloudfront_client =
            create_cloudfront_client("test-access-key", "test-secret-key", "us-east-1").await;

        test::init_service(
            App::new()
                .app_data(Data::new(test_pool()))
                .app_data(Data::new(s3_client))
                .app_data(Data::new(cloudfront_client))
                .app_data(Data::new(Arc::new(create_schema())))
                .app_data(Data::new(mode))
                .service(graphql),
        )
        .await
    }

    /// POST a GraphQL document to the real handler; return (status, raw body).
    async fn post(mode: MutationGuardMode, query: &str) -> (StatusCode, String) {
        let app = service(mode).await;
        // Built as a raw JSON string so this crate needs no JSON dependency.
        let payload = format!(r#"{{"query":{}}}"#, escape_json_string(query));
        let request = test::TestRequest::post()
            .uri("/graphql")
            .insert_header(("content-type", "application/json"))
            .set_payload(payload)
            .to_request();
        let response = test::call_service(&app, request).await;
        let status = response.status();
        let body = String::from_utf8(test::read_body(response).await.to_vec())
            .expect("response body was not valid UTF-8");
        (status, body)
    }

    /// Minimal JSON string escaping, sufficient for the fixed documents below.
    fn escape_json_string(value: &str) -> String {
        let mut out = String::with_capacity(value.len() + 2);
        out.push('"');
        for character in value.chars() {
            match character {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                other => out.push(other),
            }
        }
        out.push('"');
        out
    }

    /// A baseline-valid mutation whose single top-level response key `x` occurs
    /// twice with **identical** arguments — the compatible duplicate the pinned
    /// executor would otherwise run as two writes.
    const DUPLICATE_MUTATION: &str = concat!(
        "mutation { ",
        r#"x: createPublisher(data: {publisherName: "Guard HTTP Convention"}) { publisherId } "#,
        r#"x: createPublisher(data: {publisherName: "Guard HTTP Convention"}) { publisherId } "#,
        "}"
    );

    /// An ordinary Juniper document-validation failure: an unknown field.
    const INVALID_DOCUMENT: &str = "{ thisFieldDoesNotExist }";

    /// Assert the repository's GraphQL request-validation failure convention.
    fn assert_validation_failure_convention(status: StatusCode, body: &str, label: &str) {
        assert_eq!(
            status,
            StatusCode::BAD_REQUEST,
            "[{label}] must return HTTP 400 through the common status branch; body: {body}"
        );
        assert!(
            body.starts_with(r#"{"errors":["#),
            "[{label}] body must be the ordinary errors-array shape; got: {body}"
        );
        assert!(
            !body.contains(r#""data""#),
            "[{label}] a validation-style failure carries no `data` key; got: {body}"
        );
        assert!(
            body.contains(r#""message""#),
            "[{label}] each error carries a message; got: {body}"
        );
        assert!(
            body.contains(r#""locations""#),
            "[{label}] each error carries locations; got: {body}"
        );
    }

    #[actix_web::test]
    async fn guard_rejection_and_juniper_validation_failure_share_the_http_convention() {
        // A guard rejection, in ENFORCE.
        let (guard_status, guard_body) = post(MutationGuardMode::Enforce, DUPLICATE_MUTATION).await;
        assert_validation_failure_convention(guard_status, &guard_body, "guard rejection");
        assert!(
            guard_body.contains("selected more than once"),
            "expected the guard's own rejection message; got: {guard_body}"
        );

        // An ordinary Juniper validation failure, on the same route and handler.
        let (juniper_status, juniper_body) = post(MutationGuardMode::Off, INVALID_DOCUMENT).await;
        assert_validation_failure_convention(
            juniper_status,
            &juniper_body,
            "juniper validation failure",
        );

        // Same status, reached through the same single branch.
        assert_eq!(
            guard_status, juniper_status,
            "a guard rejection must not use a status of its own"
        );
    }

    #[actix_web::test]
    async fn the_merged_off_mode_never_produces_a_guard_rejection() {
        // In `OFF` the guard evaluates nothing, so the same duplicate document
        // is not rejected by the guard and proceeds to ordinary execution.
        let (_status, body) = post(MutationGuardMode::Off, DUPLICATE_MUTATION).await;
        assert!(
            !body.contains("selected more than once"),
            "OFF must never produce a guard rejection; got: {body}"
        );
    }

    #[actix_web::test]
    async fn a_successful_response_uses_the_ok_arm_of_the_same_branch() {
        // `__typename` resolves without touching the database, so this exercises
        // the `is_ok() == true` arm of the one shared status branch, proving the
        // branch is genuinely common rather than failure-only.
        let (status, body) = post(MutationGuardMode::Enforce, "{ __typename }").await;
        assert_eq!(status, StatusCode::OK);
        assert!(
            body.contains(r#""data""#),
            "a successful response carries a `data` key; got: {body}"
        );
    }
}
