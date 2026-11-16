use std::{
    future::{ready, Ready},
    rc::Rc,
};

use actix_http::h1;
use actix_web::{
    dev::{self, Payload, Service, ServiceRequest, ServiceResponse, Transform},
    middleware, web, Error, HttpMessage,
};
use futures_util::future::LocalBoxFuture;
use thoth_api::model::async_job::executor::{access_log_omits_body, OMITTED_REQUEST_BODY};

const LOG_FORMAT: &str = r#"%{r}a %a "%r" %s %b "%{Referer}i" "%{User-Agent}i" %T %{QUERY}xi"#;
pub(crate) struct Logger;
pub(crate) struct BodyLogger;

impl Logger {
    pub(crate) fn default() -> middleware::Logger {
        middleware::Logger::new(LOG_FORMAT).custom_request_replace("QUERY", Self::get_request_body)
    }

    fn format_request_body(body: &web::Bytes) -> String {
        // THOTH-ASYNC-01-ENGINE-01B (spec section 16): a body naming an async
        // job field can carry a claim token, a checkpoint, evidence, error
        // details or a staff reason, so it is never logged. Every other body
        // is logged exactly as before.
        if access_log_omits_body(body) {
            return format!("\n{OMITTED_REQUEST_BODY}");
        }
        // Pretty print request body when logging level is Debug
        if log::log_enabled!(log::Level::Debug) {
            return format!("\n{}", String::from_utf8_lossy(body).replace("\\n", "\n"));
        }
        format!("\n{}", String::from_utf8_lossy(body))
    }

    fn get_request_body(req: &ServiceRequest) -> String {
        if let Some(body) = req.extensions().get::<web::Bytes>() {
            return Self::format_request_body(body);
        }
        "".to_string()
    }
}

impl<S, B> Transform<S, ServiceRequest> for BodyLogger
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = BodyLoggerMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(BodyLoggerMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub(crate) struct BodyLoggerMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for BodyLoggerMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    dev::forward_ready!(service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();

        Box::pin(async move {
            // store request body in the request extensions container for retrieval by logger later
            // only store GraphQL queries to avoid logging credentials
            if req.path().eq("/graphql") {
                let body = req.extract::<web::Bytes>().await.unwrap();
                req.extensions_mut().insert(body.clone());
                req.set_payload(bytes_to_payload(body));
            }

            let res = svc.call(req).await?;
            Ok(res)
        })
    }
}

fn bytes_to_payload(buf: web::Bytes) -> Payload {
    let (_, mut pl) = h1::Payload::create(true);
    pl.unread_data(buf);
    dev::Payload::from(pl)
}

#[cfg(test)]
mod tests {
    //! The access-log body rule of `THOTH-ASYNC-01-ENGINE-01B` (spec section
    //! 16): a body naming an async job field is omitted, and every other body
    //! is logged exactly as before.

    use actix_web::web;
    use thoth_api::model::async_job::executor::{ASYNC_JOB_FIELD_MARKER, OMITTED_REQUEST_BODY};

    use super::Logger;

    /// The body formatting of this module before the rule, kept verbatim, so
    /// an unaffected body is compared with the earlier behaviour itself.
    fn logged_before(body: &web::Bytes) -> String {
        if log::log_enabled!(log::Level::Debug) {
            return format!("\n{}", String::from_utf8_lossy(body).replace("\\n", "\n"));
        }
        format!("\n{}", String::from_utf8_lossy(body))
    }

    fn assert_omitted(body: &[u8]) {
        let logged = Logger::format_request_body(&web::Bytes::copy_from_slice(body));
        assert_eq!(
            logged,
            format!("\n{OMITTED_REQUEST_BODY}"),
            "body {:?} must be omitted",
            String::from_utf8_lossy(body)
        );
    }

    fn assert_logged_as_before(body: &[u8]) {
        let bytes = web::Bytes::copy_from_slice(body);
        let logged = Logger::format_request_body(&bytes);
        assert_eq!(logged, logged_before(&bytes));
        assert_ne!(logged, format!("\n{OMITTED_REQUEST_BODY}"));
    }

    #[test]
    fn a_body_naming_an_async_job_field_is_omitted() {
        let bodies: [&str; 6] = [
            r#"{"query":"mutation { claimAsyncJobs(input: {claimedBy: \"exec-1\", kinds: [], maxJobs: 1, leaseSeconds: 60}) { jobs { claimToken } } }"}"#,
            r#"{"query":"mutation { completeAsyncJob(input: {asyncJobId: \"00000000-0000-0000-0000-000000000001\", claimToken: \"live-token\", outcome: COMPLETED, entries: []}) { status } }","variables":null}"#,
            r#"{"query":"query { asyncJobTargetState(input: {asyncJobId: \"00000000-0000-0000-0000-000000000001\", claimToken: \"live-token\"}) { targetKey } }"}"#,
            r#"{"query":"mutation { cancelAsyncJob(input: {asyncJobId: \"00000000-0000-0000-0000-000000000001\", reason: \"staff reason\", requestId: \"00000000-0000-0000-0000-000000000002\"}) { outcome } }"}"#,
            r#"{"query":"mutation { first: claimAsyncJobs(input: {claimedBy: \"exec-1\", kinds: [], maxJobs: 1, leaseSeconds: 60}) { jobs { claimToken } } }"}"#,
            r#"{"query":"query Q { ...Read } fragment Read on QueryRoot { asyncJobTargetState(input: {asyncJobId: \"00000000-0000-0000-0000-000000000001\", claimToken: \"live-token\"}) { targetKey } }","operationName":"Q"}"#,
        ];
        for body in bodies {
            assert_omitted(body.as_bytes());
        }
    }

    #[test]
    fn a_json_escaped_field_name_is_matched_on_the_decoded_query_text() {
        // The first body is the specification's example (the `c` of the field
        // name escaped). In the other two the escapes fall inside the marker
        // (its `s`, then all of it), so the raw text does not contain the
        // marker and only the decoded query text does.
        let spec_example =
            "{\"query\":\"mutation { \\u0063laimAsyncJobs(input: {claimedBy: \\\"exec-1\\\", kinds: [], maxJobs: 1, leaseSeconds: 60}) { jobs { claimToken } } }\"}";
        let inside_marker =
            "{\"query\":\"mutation { claimA\\u0073yncJobs(input: {claimedBy: \\\"exec-1\\\", kinds: [], maxJobs: 1, leaseSeconds: 60}) { jobs { claimToken } } }\"}";
        let whole_marker =
            "{\"query\":\"mutation { claimA\\u0073\\u0079\\u006e\\u0063\\u004a\\u006f\\u0062s(input: {claimedBy: \\\"exec-1\\\", kinds: [], maxJobs: 1, leaseSeconds: 60}) { jobs { claimToken } } }\"}";
        assert!(spec_example.contains("\\u0063laimAsyncJobs"));
        for body in [inside_marker, whole_marker] {
            assert!(body.contains('\\'), "the body carries a JSON escape");
            assert!(
                !body.contains(ASYNC_JOB_FIELD_MARKER),
                "the raw text must not contain the marker, so only decoding finds it"
            );
        }
        for body in [spec_example, inside_marker, whole_marker] {
            assert_omitted(body.as_bytes());
        }
    }

    #[test]
    fn a_positional_array_body_is_matched_on_its_query_text() {
        let plain = r#"["mutation { claimAsyncJobs(input: {claimedBy: \"exec-1\", kinds: [], maxJobs: 1, leaseSeconds: 60}) { jobs { claimToken } } }", null, null]"#;
        let escaped = "[\"mutation { claimA\\u0073yncJobs(input: {claimedBy: \\\"exec-1\\\", kinds: [], maxJobs: 1, leaseSeconds: 60}) { jobs { claimToken } } }\", null, null]";
        assert!(!escaped.contains(ASYNC_JOB_FIELD_MARKER));
        for body in [plain, escaped] {
            assert_omitted(body.as_bytes());
        }
    }

    #[test]
    fn an_undecodable_body_containing_the_marker_is_omitted() {
        let bodies: [&[u8]; 4] = [
            br#"{"query":"mutation { completeAsyncJob(input: {asyncJobId: \"00000000-0000-0000-0000-000000000001\", claimToken: \"live-token\""#,
            br#"[{"query":"mutation { claimAsyncJobs(input: {claimedBy: \"exec-1\", kinds: [], maxJobs: 1, leaseSeconds: 60}) { jobs { claimToken } } }"}]"#,
            br#"{"query": 1, "note": "syncJob"}"#,
            b"\xff\xfe renewAsyncJobLease live-token",
        ];
        for body in bodies {
            assert_omitted(body);
        }
    }

    #[test]
    fn every_other_body_is_logged_exactly_as_before() {
        let bodies: [&[u8]; 8] = [
            // `BE-04` claim and completion bodies are unchanged by this rule.
            br#"{"query":"mutation { claimDistributionJobs(input: {claimedBy: \"exec-1\", maxJobs: 1}) { jobs { distributionJobId claimToken } } }"}"#,
            br#"{"query":"mutation { completeDistributionJob(input: {distributionJobId: \"00000000-0000-0000-0000-000000000001\", claimToken: \"be04-token\"}) { distributionJobId }\n}"}"#,
            br#"{"query":"{ works(limit: 1) { title } }"}"#,
            br#"{"query":"query Q($k: String!) { works(filter: $k) { title } }","variables":{"k":"syncJob"}}"#,
            br#"{"query":"{ works(limit: 1) { title }"#,
            b"\xff\xfe not json",
            b"",
            br#"["{ works(limit: 1) { title } }", null, null]"#,
        ];
        for body in bodies {
            assert_logged_as_before(body);
        }
    }

    #[test]
    fn the_omitted_body_text_is_the_specified_placeholder() {
        assert_eq!(OMITTED_REQUEST_BODY, "[async job operation body omitted]");
        assert_eq!(ASYNC_JOB_FIELD_MARKER, "syncJob");
    }
}
