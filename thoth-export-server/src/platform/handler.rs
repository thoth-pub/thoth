use actix_web::Error;
use paperclip::actix::{
    api_v2_operation,
    web::{self, Json},
};

use super::model::Platform;
use crate::data::{find_platform, ALL_PLATFORMS};

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
    platform_id: web::Path<String>,
) -> Result<Json<Platform<'static>>, Error> {
    find_platform(platform_id.into_inner())
        .map(Json)
        .map_err(|e| e.into())
}
