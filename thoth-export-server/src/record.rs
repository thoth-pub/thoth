use crate::onix::generate_onix_3;
use crate::specification::model::SpecificationId;
use actix_web::{http::StatusCode, HttpRequest, Responder};
use paperclip::actix::web::HttpResponse;
use paperclip::actix::OperationModifier;
use paperclip::util::{ready, Ready};
use paperclip::v2::models::{DefaultOperationRaw, Either, Response};
use paperclip::v2::schema::Apiv2Schema;
use thoth_api::errors::{ThothError, ThothResult};
use thoth_client::work::work_query::WorkQueryWork;
use thoth_client::work::works_query::WorksQueryWorks;

pub trait AsRecord {}
impl AsRecord for WorkQueryWork {}
impl AsRecord for WorksQueryWorks {}

pub(crate) struct MetadataRecord<T: AsRecord> {
    data: T,
    specification: SpecificationId,
}

impl<T> MetadataRecord<T>
where
    T: AsRecord,
{
    pub(crate) fn new(specification: SpecificationId, data: T) -> Self {
        MetadataRecord {
            data,
            specification,
        }
    }

    fn content_type(&self) -> &'static str {
        match self.specification {
            SpecificationId::Onix3ProjectMuse => "text/xml; charset=utf-8",
            SpecificationId::CsvThoth => "text/csv; charset=utf-8",
        }
    }
}

impl MetadataRecord<WorkQueryWork> {
    fn generate(self) -> ThothResult<String> {
        match self.specification {
            SpecificationId::Onix3ProjectMuse => generate_onix_3(self.data),
            SpecificationId::CsvThoth => unimplemented!(),
        }
    }
}

impl MetadataRecord<WorksQueryWorks> {
    fn generate(self) -> ThothResult<String> {
        unimplemented!()
    }
}

macro_rules! paperclip_responder {
    ($record_type:ident) => {
        impl Responder for MetadataRecord<$record_type>
        where
            actix_web::dev::Body: From<String>,
        {
            type Error = ThothError;
            type Future = Ready<ThothResult<HttpResponse>>;

            fn respond_to(self, _: &HttpRequest) -> Self::Future {
                ready(Ok(HttpResponse::build(StatusCode::OK)
                    .content_type(self.content_type())
                    .header("Content-Disposition", "attachment")
                    .body(self.generate().unwrap())))
            }
        }
    };
}

paperclip_responder!(WorkQueryWork);
paperclip_responder!(WorksQueryWorks);

impl<T: AsRecord> Apiv2Schema for MetadataRecord<T> {}

impl<T> OperationModifier for MetadataRecord<T>
where
    T: AsRecord,
{
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
