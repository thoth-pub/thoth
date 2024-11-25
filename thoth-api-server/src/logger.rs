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

const LOG_FORMAT: &str = r#"%{r}a %a "%r" %s %b "%{Referer}i" "%{User-Agent}i" %T %{QUERY}xi"#;
pub(crate) struct Logger;
pub(crate) struct BodyLogger;

impl Logger {
    pub(crate) fn default() -> middleware::Logger {
        middleware::Logger::new(LOG_FORMAT).custom_request_replace("QUERY", Self::get_request_body)
    }

    fn format_request_body(body: &web::Bytes) -> String {
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
