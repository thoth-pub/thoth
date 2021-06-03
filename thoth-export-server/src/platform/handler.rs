use actix_web::Error;
use paperclip::actix::{
    api_v2_operation,
    web::{self, Json},
};
use thoth_api::errors::ThothError;

use super::model::Platform;
use crate::data::ALL_PLATFORMS;

#[api_v2_operation(
    summary = "List supported platforms",
    description = "Full list of platforms supported by Thoth's outputs",
    tags(Platforms)
)]
pub(crate) async fn get_all() -> Json<Vec<Platform<'static>>> {
    Json(ALL_PLATFORMS.clone())
}

#[api_v2_operation(
    summary = "Describe a platform",
    description = "Find the details of a platform supported by Thoth's outputs",
    tags(Platforms)
)]
pub(crate) async fn get_one(
    web::Path(platform_id): web::Path<String>,
) -> Result<Json<Platform<'static>>, Error> {
    ALL_PLATFORMS
        .iter()
        .find(|p| p.id == platform_id)
        .map(|p| Json(p.clone()))
        .ok_or(ThothError::EntityNotFound)
        .map_err(|e| e.into())
}
