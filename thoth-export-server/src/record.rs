use actix_web::{http::StatusCode, HttpRequest, Responder};
use paperclip::actix::web::HttpResponse;
use paperclip::actix::OperationModifier;
use paperclip::util::{ready, Ready};
use paperclip::v2::models::{DefaultOperationRaw, Either, Response};
use paperclip::v2::schema::Apiv2Schema;
use std::str::FromStr;
use thoth_api::errors::{ThothError, ThothResult};
use thoth_client::Work;

use crate::csv::{CsvSpecification, CsvThoth};
use crate::xml::{Onix3Oapen, Onix3ProjectMuse, XmlSpecification};

pub(crate) trait AsRecord {}
impl AsRecord for Vec<Work> {}

pub(crate) enum MetadataSpecification {
    Onix3ProjectMuse(Onix3ProjectMuse),
    Onix3Oapen(Onix3Oapen),
    CsvThoth(CsvThoth),
}

pub(crate) struct MetadataRecord<T: AsRecord> {
    id: String,
    data: T,
    specification: MetadataSpecification,
}

impl<T> MetadataRecord<T>
where
    T: AsRecord + IntoIterator,
{
    pub(crate) fn new(id: String, specification: MetadataSpecification, data: T) -> Self {
        MetadataRecord {
            id,
            data,
            specification,
        }
    }

    fn content_type(&self) -> &'static str {
        match &self.specification {
            MetadataSpecification::Onix3ProjectMuse(_) => "text/xml; charset=utf-8",
            MetadataSpecification::Onix3Oapen(_) => "text/xml; charset=utf-8",
            MetadataSpecification::CsvThoth(_) => "text/csv; charset=utf-8",
        }
    }

    fn file_name(&self) -> String {
        match &self.specification {
            MetadataSpecification::Onix3ProjectMuse(_) => format!("{}.xml", self.id),
            MetadataSpecification::Onix3Oapen(_) => format!("{}.xml", self.id),
            MetadataSpecification::CsvThoth(_) => format!("{}.csv", self.id),
        }
    }
}

impl MetadataRecord<Vec<Work>> {
    fn generate(self) -> ThothResult<String> {
        match self.specification {
            MetadataSpecification::Onix3ProjectMuse(onix3_project_muse) => {
                onix3_project_muse.generate(self.data)
            }
            MetadataSpecification::Onix3Oapen(_) => unimplemented!(),
            MetadataSpecification::CsvThoth(csv_thoth) => csv_thoth.generate(self.data),
        }
    }
}

impl Responder for MetadataRecord<Vec<Work>>
where
    actix_web::dev::Body: From<String>,
{
    type Error = ThothError;
    type Future = Ready<ThothResult<HttpResponse>>;

    fn respond_to(self, _: &HttpRequest) -> Self::Future {
        // todo: handle error (provide error response - do not unwrap)
        ready(Ok(HttpResponse::build(StatusCode::OK)
            .content_type(self.content_type())
            .header("Content-Disposition", format!("attachment; filename=\"{}\"", self.file_name()))
            .body(self.generate().unwrap())))
    }
}

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

impl FromStr for MetadataSpecification {
    type Err = ThothError;

    fn from_str(input: &str) -> ThothResult<Self> {
        match input {
            "onix_3.0::project_muse" => {
                Ok(MetadataSpecification::Onix3ProjectMuse(Onix3ProjectMuse {}))
            }
            "onix_3.0::oapen" => Ok(MetadataSpecification::Onix3Oapen(Onix3Oapen {})),
            "csv::thoth" => Ok(MetadataSpecification::CsvThoth(CsvThoth {})),
            _ => Err(ThothError::InvalidMetadataSpecification(input.to_string())),
        }
    }
}
