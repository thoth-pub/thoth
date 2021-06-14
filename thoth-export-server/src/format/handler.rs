use actix_web::Error;
use paperclip::actix::{
    api_v2_operation,
    web::{self, Json},
};

use super::model::Format;
use crate::data::{find_format, ALL_FORMATS};

#[api_v2_operation(
    summary = "List supported formats",
    description = "Full list of metadata formats that can be output by Thoth",
    tags(Formats)
)]
pub(crate) async fn get_all() -> Json<Vec<Format<'static>>> {
    Json(ALL_FORMATS.clone())
}

#[api_v2_operation(
    summary = "Describe a metadata format",
    description = "Find the details of a format that can be output by Thoth",
    tags(Formats)
)]
pub(crate) async fn get_one(
    web::Path(format_id): web::Path<String>,
) -> Result<Json<Format<'static>>, Error> {
    find_format(format_id).map(Json).map_err(|e| e.into())
}
