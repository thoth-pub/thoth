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

    let result = match run_mutation_guard(mode, &data, &st) {
        // The mutation request guard remains independent of the DataLoader
        // foundation. A rejection still executes zero resolvers and performs
        // zero writes; `OFF` remains the merged production state.
        Some(rejection) => rejection,
        None => {
            // Loader ownership is derived only from the request-local Context.
            // It is deliberately independent of mutation guard mode.
            let ctx = Context::new(
                pool.into_inner(),
                user,
                s3_client.into_inner(),
                cloudfront_client.into_inner(),
            );
            data.execute(&st, &ctx).await
        }
    };

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

    use super::*;
    use actix_web::{http::StatusCode, test, App};

    fn test_pool() -> PgPool {
        let url = std::env::var("TEST_DATABASE_URL")
            .expect("TEST_DATABASE_URL must be set for thoth-api-server handler tests");
        init_pool(&url)
    }

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

    async fn post(mode: MutationGuardMode, query: &str) -> (StatusCode, String) {
        let app = service(mode).await;
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

    const DUPLICATE_MUTATION: &str = concat!(
        "mutation { ",
        r#"x: createPublisher(data: {publisherName: "Guard HTTP Convention"}) { publisherId } "#,
        r#"x: createPublisher(data: {publisherName: "Guard HTTP Convention"}) { publisherId } "#,
        "}"
    );

    const INVALID_DOCUMENT: &str = "{ thisFieldDoesNotExist }";

    fn assert_validation_failure_convention(status: StatusCode, body: &str, label: &str) {
        assert_eq!(status, StatusCode::BAD_REQUEST, "[{label}] body: {body}");
        assert!(body.starts_with(r#"{"errors":["#));
        assert!(!body.contains(r#""data""#));
        assert!(body.contains(r#""message""#));
        assert!(body.contains(r#""locations""#));
    }

    #[actix_web::test]
    async fn guard_rejection_and_juniper_validation_failure_share_the_http_convention() {
        let (guard_status, guard_body) = post(MutationGuardMode::Enforce, DUPLICATE_MUTATION).await;
        assert_validation_failure_convention(guard_status, &guard_body, "guard rejection");
        assert!(guard_body.contains("selected more than once"));

        let (juniper_status, juniper_body) = post(MutationGuardMode::Off, INVALID_DOCUMENT).await;
        assert_validation_failure_convention(
            juniper_status,
            &juniper_body,
            "juniper validation failure",
        );
        assert_eq!(guard_status, juniper_status);
    }

    #[actix_web::test]
    async fn the_merged_off_mode_never_produces_a_guard_rejection() {
        let (_status, body) = post(MutationGuardMode::Off, DUPLICATE_MUTATION).await;
        assert!(!body.contains("selected more than once"));
        assert!(body.contains("Invalid credentials."));
    }

    #[actix_web::test]
    async fn a_successful_response_uses_the_ok_arm_of_the_same_branch() {
        let (status, body) = post(MutationGuardMode::Enforce, "{ __typename }").await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains(r#""data""#));
    }
}
