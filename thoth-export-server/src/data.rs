use lazy_static::lazy_static;
use thoth_api::errors::{ThothError, ThothResult};

use crate::format::model::Format;
use crate::platform::model::Platform;
use crate::specification::model::Specification;

lazy_static! {
    pub(crate) static ref ALL_SPECIFICATIONS: Vec<Specification<'static>> = vec![
        Specification {
            id: "onix_3.0::project_muse",
            name: "Project MUSE ONIX 3.0",
            format: concat!(env!("THOTH_EXPORT_API"), "/formats/onix_3.0"),
            accepted_by: vec![concat!(env!("THOTH_EXPORT_API"), "/platforms/project_muse"),],
        },
        Specification {
            id: "csv::thoth",
            name: "Thoth CSV",
            format: concat!(env!("THOTH_EXPORT_API"), "/formats/csv"),
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

pub(crate) fn find_format(format_id: String) -> ThothResult<Format<'static>> {
    ALL_FORMATS
        .iter()
        .find(|f| f.id == format_id)
        .cloned()
        .ok_or(ThothError::EntityNotFound)
}

pub(crate) fn find_platform(platform_id: String) -> ThothResult<Platform<'static>> {
    ALL_PLATFORMS
        .iter()
        .find(|p| p.id == platform_id)
        .cloned()
        .ok_or(ThothError::EntityNotFound)
}

pub(crate) fn find_specification(specification_id: String) -> ThothResult<Specification<'static>> {
    ALL_SPECIFICATIONS
        .iter()
        .find(|s| s.id == specification_id)
        .cloned()
        .ok_or(ThothError::InvalidMetadataSpecification(specification_id))
}

#[cfg(test)]
mod tests {
    use super::*;

    const FORMATS_PREFIX: &str = concat!(env!("THOTH_EXPORT_API"), "/formats/");
    const PLATFORMS_PREFIX: &str = concat!(env!("THOTH_EXPORT_API"), "/platforms/");
    const SPECIFICATIONS_PREFIX: &str = concat!(env!("THOTH_EXPORT_API"), "/specifications/");

    fn format_id_from_url(url: &str) -> String {
        url.replace(FORMATS_PREFIX, "")
    }

    fn platform_id_from_url(url: &str) -> String {
        url.replace(PLATFORMS_PREFIX, "")
    }

    fn specification_id_from_url(url: &str) -> String {
        url.replace(SPECIFICATIONS_PREFIX, "")
    }

    #[test]
    fn test_specification_format_in_all_formats() {
        for s in ALL_SPECIFICATIONS.iter() {
            let format_id = format_id_from_url(s.format);
            assert!(find_format(format_id).is_ok())
        }
    }

    #[test]
    fn test_specification_platforms_in_all_platforms() {
        for s in ALL_SPECIFICATIONS.iter() {
            for accepted_platform in &s.accepted_by {
                let platform_id = platform_id_from_url(accepted_platform);
                assert!(find_platform(platform_id).is_ok())
            }
        }
    }

    #[test]
    fn test_specification_id_begins_with_format_id() {
        for s in ALL_SPECIFICATIONS.iter() {
            let format_id = format_id_from_url(s.format);
            assert!(s.id.starts_with(&format_id));
        }
    }

    #[test]
    fn test_platform_specifications_in_all_specifications() {
        for p in ALL_PLATFORMS.iter() {
            for s in &p.accepts {
                let specification_id = specification_id_from_url(s);
                assert!(find_specification(specification_id).is_ok())
            }
        }
    }

    #[test]
    fn test_format_specifications_in_all_specifications() {
        for f in ALL_FORMATS.iter() {
            for s in &f.specifications {
                let specification_id = specification_id_from_url(s);
                assert!(find_specification(specification_id).is_ok())
            }
        }
    }

    #[test]
    fn test_format_id_derives_from_name_and_version() {
        for f in ALL_FORMATS.iter() {
            let id_should_be = match f.version {
                Some(version) => format!("{}_{}", f.name.to_lowercase(), version),
                None => f.name.to_lowercase().to_string(),
            };
            assert_eq!(String::from(f.id), id_should_be)
        }
    }
}
