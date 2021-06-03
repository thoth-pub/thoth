use crate::format::model::Format;
use crate::platform::model::Platform;
use crate::specification::model::Specification;

pub(crate) const ALL_SPECIFICATIONS: [Specification<'static>; 2] = [
    Specification {
        id: "onix_3.0::project_muse",
        name: "Project MUSE ONIX 3.0",
    },
    Specification {
        id: "csv::thoth",
        name: "Thoth CSV",
    },
];

pub(crate) const ALL_PLATFORMS: [Platform<'static>; 2] = [
    Platform {
        id: "thoth",
        name: "Thoth",
    },
    Platform {
        id: "project_muse",
        name: "Project MUSE",
    },
];

pub(crate) const ALL_FORMATS: [Format<'static>; 2] = [
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
