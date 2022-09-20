use lazy_static::lazy_static;
use thoth_errors::{ThothError, ThothResult};

use crate::format::model::Format;
use crate::platform::model::Platform;
use crate::specification::model::Specification;

lazy_static! {
    pub(crate) static ref ALL_SPECIFICATIONS: Vec<Specification<'static>> = vec![
        Specification {
            id: "onix_3.0::project_muse",
            name: "Project MUSE ONIX 3.0",
            format: concat!(env!("THOTH_EXPORT_API"), "/formats/onix_3.0"),
            accepted_by: vec![
                concat!(env!("THOTH_EXPORT_API"), "/platforms/project_muse"),
                concat!(env!("THOTH_EXPORT_API"), "/platforms/jstor"),
            ],
        },
        Specification {
            id: "onix_3.0::oapen",
            name: "OAPEN ONIX 3.0",
            format: concat!(env!("THOTH_EXPORT_API"), "/formats/onix_3.0"),
            accepted_by: vec![
                concat!(env!("THOTH_EXPORT_API"), "/platforms/oapen"),
                concat!(env!("THOTH_EXPORT_API"), "/platforms/doab"),
            ],
        },
        Specification {
            id: "onix_3.0::jstor",
            name: "JSTOR ONIX 3.0",
            format: concat!(env!("THOTH_EXPORT_API"), "/formats/onix_3.0"),
            accepted_by: vec![concat!(env!("THOTH_EXPORT_API"), "/platforms/jstor"),],
        },
        Specification {
            id: "onix_3.0::google_books",
            name: "Google Books ONIX 3.0",
            format: concat!(env!("THOTH_EXPORT_API"), "/formats/onix_3.0"),
            accepted_by: vec![concat!(env!("THOTH_EXPORT_API"), "/platforms/google_books"),],
        },
        Specification {
            id: "onix_3.0::overdrive",
            name: "Google Books ONIX 3.0",
            format: concat!(env!("THOTH_EXPORT_API"), "/formats/onix_3.0"),
            accepted_by: vec![concat!(env!("THOTH_EXPORT_API"), "/platforms/overdrive"),],
        },
        Specification {
            id: "onix_2.1::ebsco_host",
            name: "EBSCO Host ONIX 2.1",
            format: concat!(env!("THOTH_EXPORT_API"), "/formats/onix_2.1"),
            accepted_by: vec![
                concat!(env!("THOTH_EXPORT_API"), "/platforms/ebsco_host"),
                concat!(env!("THOTH_EXPORT_API"), "/platforms/rnib_bookshare"),
            ],
        },
        Specification {
            id: "csv::thoth",
            name: "Thoth CSV",
            format: concat!(env!("THOTH_EXPORT_API"), "/formats/csv"),
            accepted_by: vec![concat!(env!("THOTH_EXPORT_API"), "/platforms/thoth"),],
        },
        Specification {
            id: "kbart::oclc",
            name: "OCLC KBART",
            format: concat!(env!("THOTH_EXPORT_API"), "/formats/kbart"),
            accepted_by: vec![
                concat!(env!("THOTH_EXPORT_API"), "/platforms/oclc_kb"),
                concat!(env!("THOTH_EXPORT_API"), "/platforms/proquest_kb"),
                concat!(env!("THOTH_EXPORT_API"), "/platforms/proquest_exlibris"),
                concat!(env!("THOTH_EXPORT_API"), "/platforms/ebsco_kb"),
                concat!(env!("THOTH_EXPORT_API"), "/platforms/jisc_kb"),
            ],
        },
        Specification {
            id: "bibtex::thoth",
            name: "Thoth BibTeX",
            format: concat!(env!("THOTH_EXPORT_API"), "/formats/bibtex"),
            accepted_by: vec![concat!(env!("THOTH_EXPORT_API"), "/platforms/zotero"),],
        },
        Specification {
            id: "doideposit::crossref",
            name: "CrossRef DOI deposit",
            format: concat!(env!("THOTH_EXPORT_API"), "/formats/doideposit"),
            accepted_by: vec![concat!(env!("THOTH_EXPORT_API"), "/platforms/crossref"),],
        },
        Specification {
            id: "onix_2.1::proquest_ebrary",
            name: "ProQuest Ebrary ONIX 2.1",
            format: concat!(env!("THOTH_EXPORT_API"), "/formats/onix_2.1"),
            accepted_by: vec![concat!(
                env!("THOTH_EXPORT_API"),
                "/platforms/proquest_ebrary"
            )],
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
        Platform {
            id: "oapen",
            name: "OAPEN",
            accepts: vec![concat!(
                env!("THOTH_EXPORT_API"),
                "/specifications/onix_3.0::oapen"
            ),],
        },
        Platform {
            id: "doab",
            name: "DOAB",
            accepts: vec![concat!(
                env!("THOTH_EXPORT_API"),
                "/specifications/onix_3.0::oapen"
            ),],
        },
        Platform {
            id: "jstor",
            name: "JSTOR",
            accepts: vec![
                concat!(env!("THOTH_EXPORT_API"), "/specifications/onix_3.0::jstor"),
                concat!(
                    env!("THOTH_EXPORT_API"),
                    "/specifications/onix_3.0::project_muse"
                ),
            ],
        },
        Platform {
            id: "google_books",
            name: "Google Books",
            accepts: vec![concat!(
                env!("THOTH_EXPORT_API"),
                "/specifications/onix_3.0::google_books"
            ),],
        },
        Platform {
            id: "overdrive",
            name: "OverDrive",
            accepts: vec![concat!(
                env!("THOTH_EXPORT_API"),
                "/specifications/onix_3.0::overdrive"
            ),],
        },
        Platform {
            id: "ebsco_host",
            name: "EBSCO Host",
            accepts: vec![concat!(
                env!("THOTH_EXPORT_API"),
                "/specifications/onix_2.1::ebsco_host"
            ),],
        },
        Platform {
            id: "oclc_kb",
            name: "OCLC KB",
            accepts: vec![concat!(
                env!("THOTH_EXPORT_API"),
                "/specifications/kbart::oclc"
            ),],
        },
        Platform {
            id: "proquest_kb",
            name: "ProQuest KB",
            accepts: vec![concat!(
                env!("THOTH_EXPORT_API"),
                "/specifications/kbart::oclc"
            ),],
        },
        Platform {
            id: "proquest_exlibris",
            name: "ProQuest ExLibris",
            accepts: vec![concat!(
                env!("THOTH_EXPORT_API"),
                "/specifications/kbart::oclc"
            ),],
        },
        Platform {
            id: "ebsco_kb",
            name: "EBSCO KB",
            accepts: vec![concat!(
                env!("THOTH_EXPORT_API"),
                "/specifications/kbart::oclc"
            ),],
        },
        Platform {
            id: "jisc_kb",
            name: "JISC KB",
            accepts: vec![concat!(
                env!("THOTH_EXPORT_API"),
                "/specifications/kbart::oclc"
            ),],
        },
        Platform {
            id: "zotero",
            name: "Zotero",
            accepts: vec![concat!(
                env!("THOTH_EXPORT_API"),
                "/specifications/bibtex::thoth"
            ),],
        },
        Platform {
            id: "crossref",
            name: "CrossRef",
            accepts: vec![concat!(
                env!("THOTH_EXPORT_API"),
                "/specifications/doideposit::crossref"
            ),],
        },
        Platform {
            id: "rnib_bookshare",
            name: "RNIB Bookshare",
            accepts: vec![concat!(
                env!("THOTH_EXPORT_API"),
                "/specifications/onix_2.1::ebsco_host"
            ),],
        },
        Platform {
            id: "proquest_ebrary",
            name: "ProQuest Ebrary",
            accepts: vec![concat!(
                env!("THOTH_EXPORT_API"),
                "/specifications/onix_2.1::proquest_ebrary"
            ),],
        },
    ];
    pub(crate) static ref ALL_FORMATS: Vec<Format<'static>> = vec![
        Format {
            id: "onix_3.0",
            name: "ONIX",
            version: Some("3.0"),
            specifications: vec![
                concat!(
                    env!("THOTH_EXPORT_API"),
                    "/specifications/onix_3.0::project_muse"
                ),
                concat!(env!("THOTH_EXPORT_API"), "/specifications/onix_3.0::oapen"),
                concat!(env!("THOTH_EXPORT_API"), "/specifications/onix_3.0::jstor"),
                concat!(
                    env!("THOTH_EXPORT_API"),
                    "/specifications/onix_3.0::google_books"
                ),
                concat!(
                    env!("THOTH_EXPORT_API"),
                    "/specifications/onix_3.0::overdrive"
                ),
            ],
        },
        Format {
            id: "onix_2.1",
            name: "ONIX",
            version: Some("2.1"),
            specifications: vec![
                concat!(
                    env!("THOTH_EXPORT_API"),
                    "/specifications/onix_2.1::ebsco_host"
                ),
                concat!(
                    env!("THOTH_EXPORT_API"),
                    "/specifications/onix_2.1::proquest_ebrary"
                ),
            ],
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
        Format {
            id: "kbart",
            name: "KBART",
            version: None,
            specifications: vec![concat!(
                env!("THOTH_EXPORT_API"),
                "/specifications/kbart::oclc"
            ),],
        },
        Format {
            id: "bibtex",
            name: "BibTeX",
            version: None,
            specifications: vec![concat!(
                env!("THOTH_EXPORT_API"),
                "/specifications/bibtex::thoth"
            ),],
        },
        Format {
            id: "doideposit",
            name: "DOIdeposit",
            version: None,
            specifications: vec![concat!(
                env!("THOTH_EXPORT_API"),
                "/specifications/doideposit::crossref"
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
    fn test_all_specifications_listed_in_formats() {
        for s in ALL_SPECIFICATIONS.iter() {
            let specification_id = s.id;
            let format_id = format_id_from_url(s.format);
            let format = find_format(format_id).unwrap();
            assert!(format
                .specifications
                .iter()
                .find(|specification| specification_id_from_url(specification) == specification_id)
                .cloned()
                .ok_or(ThothError::EntityNotFound)
                .is_ok())
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
