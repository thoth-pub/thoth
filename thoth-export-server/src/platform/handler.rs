use paperclip::actix::{web::{self, Json}, api_v2_operation};
use thoth_api::errors::ThothError;
use actix_web::Error;

use super::model::Platform;

const ALL_PLATFORMS: [Platform<'static>; 2] = [
    Platform {
        id: "thoth",
        name: "Thoth",
    },
    Platform {
        id: "project_muse",
        name: "Project MUSE",
    },
];


#[api_v2_operation(
summary = "List supported platforms",
description = "Full list of platforms supported by Thoth's outputs",
tags(Platforms)
)]
pub(crate) async fn get_all() -> Json<[Platform<'static>; 2]> {
    Json(ALL_PLATFORMS)
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