use lazy_static::lazy_static;

use crate::format::model::Format;
use crate::platform::model::Platform;
use crate::specification::model::Specification;

lazy_static! {
    pub(crate) static ref ALL_SPECIFICATIONS: Vec<Specification<'static>> = vec![
        Specification {
            id: "onix_3.0::project_muse",
            name: "Project MUSE ONIX 3.0",
            format: concat!(env!("THOTH_EXPORT_API"), "/formats/csv"),
            accepted_by: vec![concat!(env!("THOTH_EXPORT_API"), "/platforms/project_muse"),],
        },
        Specification {
            id: "csv::thoth",
            name: "Thoth CSV",
            format: concat!(env!("THOTH_EXPORT_API"), "/formats/onix_3.0"),
            accepted_by: vec![concat!(env!("THOTH_EXPORT_API"), "/platforms/thoth"),],
        },
    ];
    pub(crate) static ref ALL_PLATFORMS: Vec<Platform<'static>> = vec![
        Platform {
            id: "thoth",
            name: "Thoth",
            accepts: vec![concat!(
                env!("THOTH_EXPORT_API"),
                "/specifications/csv::thoth"
            ),],
        },
        Platform {
            id: "project_muse",
            name: "Project MUSE",
            accepts: vec![concat!(
                env!("THOTH_EXPORT_API"),
                "/specifications/onix_3.0::project_muse"
            ),],
        },
    ];
    pub(crate) static ref ALL_FORMATS: Vec<Format<'static>> = vec![
        Format {
            id: "onix_3.0",
            name: "ONIX",
            version: Some("3.0"),
            specifications: vec![concat!(
                env!("THOTH_EXPORT_API"),
                "/specifications/onix_3.0::project_muse"
            ),],
        },
        Format {
            id: "csv",
            name: "CSV",
            version: None,
            specifications: vec![concat!(
                env!("THOTH_EXPORT_API"),
                "/specifications/csv::thoth"
            ),],
        },
    ];
}
