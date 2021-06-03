use lazy_static::lazy_static;

use crate::format::model::Format;
use crate::platform::model::Platform;
use crate::specification::model::Specification;

lazy_static! {
    pub(crate) static ref ALL_SPECIFICATIONS: Vec<Specification<'static>> = vec![
        Specification {
            id: "onix_3.0::project_muse",
            name: "Project MUSE ONIX 3.0",
        },
        Specification {
            id: "csv::thoth",
            name: "Thoth CSV",
        },
    ];
    pub(crate) static ref ALL_PLATFORMS: Vec<Platform<'static>> = vec![
        Platform {
            id: "thoth",
            name: "Thoth",
        },
        Platform {
            id: "project_muse",
            name: "Project MUSE",
        },
    ];
    pub(crate) static ref ALL_FORMATS: Vec<Format<'static>> = vec![
        Format {
            id: "onix_3.0",
            name: "ONIX",
            version: Some("3.0"),
            specifications: vec!["onix_3.0::project_muse", "onix_3.0::oapen"]
        },
        Format {
            id: "csv",
            name: "CSV",
            version: None,
            specifications: vec!["thoth::csv"]
        },
    ];
}
