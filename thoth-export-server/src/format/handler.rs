use paperclip::actix::{web::{self, Json}, api_v2_operation};
use thoth_api::errors::ThothError;
use actix_web::Error;

use super::model::Format;

const ALL_FORMATS: [Format<'static>; 2] = [
    Format {
        id: "onix_3.0",
        name: "ONIX",
        version: Some("3.0"),
    },
    Format {
        id: "csv",
        name: "CSV",
        version: None,
    },
];

#[api_v2_operation(
summary = "List supported formats",
description = "Full list of metadata formats that can be output by Thoth",
tags(Formats)
)]
pub(crate) async fn get_all() -> Json<[Format<'static>; 2]> {
    Json(ALL_FORMATS)
}

#[api_v2_operation(
summary = "Describe a metadata format",
description = "Find the details of a format that can be output by Thoth",
tags(Formats)
)]
pub(crate) async fn get_one(web::Path(format_id): web::Path<String>) -> Result<Json<Format<'static>>, Error> {
    ALL_FORMATS
        .iter()
        .find(|f| f.id == format_id)
        .map(|f| Json(f.clone()))
        .ok_or(ThothError::EntityNotFound)
        .map_err(|e| e.into())
}