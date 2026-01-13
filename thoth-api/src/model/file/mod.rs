use serde::Deserialize;
use serde::Serialize;
use strum::Display;
use strum::EnumString;
use uuid::Uuid;

use crate::model::publication::PublicationType;
use crate::model::Timestamp;
#[cfg(feature = "backend")]
use crate::schema::file;
#[cfg(feature = "backend")]
use crate::schema::file_upload;
#[cfg(feature = "backend")]
use thoth_errors::{ThothError, ThothResult};

#[cfg_attr(
    feature = "backend",
    derive(DbEnum, juniper::GraphQLEnum),
    graphql(description = "Type of file being uploaded"),
    ExistingTypePath = "crate::schema::sql_types::FileType"
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "lowercase")]
pub enum FileType {
    #[cfg_attr(
        feature = "backend",
        db_rename = "publication",
        graphql(description = "Publication file (PDF, EPUB, XML, etc.)")
    )]
    Publication,
    #[cfg_attr(
        feature = "backend",
        db_rename = "frontcover",
        graphql(description = "Front cover image")
    )]
    Frontcover,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct File {
    pub file_id: Uuid,
    pub file_type: FileType,
    pub work_id: Option<Uuid>,
    pub publication_id: Option<Uuid>,
    pub object_key: String,
    pub cdn_url: String,
    pub mime_type: String,
    pub bytes: i64,
    pub sha256: String,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FileUpload {
    pub file_upload_id: Uuid,
    pub file_type: FileType,
    pub work_id: Option<Uuid>,
    pub publication_id: Option<Uuid>,
    pub declared_mime_type: String,
    pub declared_extension: String,
    pub declared_sha256: String,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, Insertable),
    graphql(description = "Input for starting a publication file upload"),
    diesel(table_name = file_upload)
)]
pub struct NewFileUpload {
    pub file_type: FileType,
    pub work_id: Option<Uuid>,
    pub publication_id: Option<Uuid>,
    pub declared_mime_type: String,
    pub declared_extension: String,
    pub declared_sha256: String,
}

#[cfg_attr(
    feature = "backend",
    derive(Insertable),
    diesel(table_name = file)
)]
pub struct NewFile {
    pub file_type: FileType,
    pub work_id: Option<Uuid>,
    pub publication_id: Option<Uuid>,
    pub object_key: String,
    pub cdn_url: String,
    pub mime_type: String,
    pub bytes: i64,
    pub sha256: String,
}

#[cfg(feature = "backend")]
#[derive(juniper::GraphQLInputObject)]
#[graphql(description = "Input for starting a publication file upload (PDF, EPUB, XML, etc.).")]
pub struct NewPublicationFileUpload {
    #[graphql(description = "Thoth ID of the publication linked to this file.")]
    pub publication_id: Uuid,
    #[graphql(
        description = "MIME type declared by the client (used for validation and in the presigned URL)."
    )]
    pub declared_mime_type: String,
    #[graphql(
        description = "File extension to use in the final canonical key, e.g. 'pdf', 'epub', 'xml'."
    )]
    pub declared_extension: String,
    #[graphql(description = "SHA-256 checksum of the file, hex-encoded.")]
    pub declared_sha256: String,
}

#[cfg(feature = "backend")]
#[derive(juniper::GraphQLInputObject)]
#[graphql(description = "Input for starting a front cover upload for a work.")]
pub struct NewFrontcoverFileUpload {
    #[graphql(description = "Thoth ID of the work this front cover belongs to.")]
    pub work_id: Uuid,
    #[graphql(description = "MIME type declared by the client (e.g. 'image/jpeg').")]
    pub declared_mime_type: String,
    #[graphql(
        description = "File extension to use in the final canonical key, e.g. 'jpg', 'png', 'webp'."
    )]
    pub declared_extension: String,
    #[graphql(description = "SHA-256 checksum of the file, hex-encoded.")]
    pub declared_sha256: String,
}

#[cfg(feature = "backend")]
#[derive(juniper::GraphQLInputObject)]
#[graphql(
    description = "Input for completing a file upload and promoting it to its final DOI-based location."
)]
pub struct CompleteFileUpload {
    #[graphql(description = "ID of the upload session to complete.")]
    pub file_upload_id: Uuid,
}

#[cfg(feature = "backend")]
#[derive(juniper::GraphQLObject)]
#[graphql(
    description = "Response from initiating a file upload, containing the upload URL and expiration time."
)]
pub struct FileUploadResponse {
    #[graphql(description = "ID of the upload session.")]
    pub file_upload_id: Uuid,
    #[graphql(description = "Presigned S3 PUT URL for uploading the file.")]
    pub upload_url: String,
    #[graphql(description = "Time when the upload URL expires.")]
    pub expires_at: Timestamp,
}

#[cfg(feature = "backend")]
pub fn validate_file_extension(
    extension: &str,
    file_type: &FileType,
    publication_type: Option<PublicationType>,
) -> ThothResult<()> {
    match file_type {
        FileType::Frontcover => {
            let valid_extensions = ["jpg", "jpeg", "png", "webp"];
            if !valid_extensions.contains(&extension.to_lowercase().as_str()) {
                return Err(ThothError::InvalidFileExtension);
            }
        }
        FileType::Publication => {
            if let Some(pub_type) = publication_type {
                let valid_extensions: Vec<&str> = match pub_type {
                    PublicationType::Pdf => vec!["pdf"],
                    PublicationType::Epub => vec!["epub"],
                    PublicationType::Html => vec!["html", "htm", "zip"],
                    PublicationType::Xml => vec!["xml", "zip"],
                    PublicationType::Docx => vec!["docx"],
                    PublicationType::Mobi => vec!["mobi"],
                    PublicationType::Azw3 => vec!["azw3"],
                    PublicationType::FictionBook => vec!["fb2", "fbz", "zip"],
                    PublicationType::Mp3 => vec!["mp3"],
                    PublicationType::Wav => vec!["wav"],
                    _ => return Err(ThothError::UnsupportedPublicationTypeForFileUpload),
                };
                if !valid_extensions.contains(&extension.to_lowercase().as_str()) {
                    return Err(ThothError::InvalidFileExtension);
                }
            } else {
                return Err(ThothError::PublicationTypeRequiredForFileValidation);
            }
        }
    }
    Ok(())
}

#[cfg(feature = "backend")]
pub mod crud;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_file_extension_frontcover_valid() {
        let valid_extensions = ["jpg", "jpeg", "png", "webp", "JPG", "JPEG", "PNG", "WEBP"];
        for ext in valid_extensions.iter() {
            assert!(
                validate_file_extension(ext, &FileType::Frontcover, None).is_ok(),
                "Extension {} should be valid for frontcover",
                ext
            );
        }
    }

    #[test]
    fn test_validate_file_extension_frontcover_invalid() {
        let invalid_extensions = ["gif", "bmp", "pdf", "txt", ""];
        for ext in invalid_extensions.iter() {
            assert!(
                validate_file_extension(ext, &FileType::Frontcover, None).is_err(),
                "Extension {} should be invalid for frontcover",
                ext
            );
        }
    }

    #[test]
    fn test_validate_file_extension_publication_pdf() {
        assert!(
            validate_file_extension("pdf", &FileType::Publication, Some(PublicationType::Pdf))
                .is_ok()
        );
        assert!(
            validate_file_extension("PDF", &FileType::Publication, Some(PublicationType::Pdf))
                .is_ok()
        );
        assert!(validate_file_extension(
            "epub",
            &FileType::Publication,
            Some(PublicationType::Pdf)
        )
        .is_err());
    }

    #[test]
    fn test_validate_file_extension_publication_epub() {
        assert!(validate_file_extension(
            "epub",
            &FileType::Publication,
            Some(PublicationType::Epub)
        )
        .is_ok());
        assert!(validate_file_extension(
            "pdf",
            &FileType::Publication,
            Some(PublicationType::Epub)
        )
        .is_err());
    }

    #[test]
    fn test_validate_file_extension_publication_html() {
        assert!(validate_file_extension(
            "html",
            &FileType::Publication,
            Some(PublicationType::Html)
        )
        .is_ok());
        assert!(validate_file_extension(
            "htm",
            &FileType::Publication,
            Some(PublicationType::Html)
        )
        .is_ok());
        assert!(validate_file_extension(
            "zip",
            &FileType::Publication,
            Some(PublicationType::Html)
        )
        .is_ok());
        assert!(validate_file_extension(
            "pdf",
            &FileType::Publication,
            Some(PublicationType::Html)
        )
        .is_err());
    }

    #[test]
    fn test_validate_file_extension_publication_xml() {
        assert!(
            validate_file_extension("xml", &FileType::Publication, Some(PublicationType::Xml))
                .is_ok()
        );
        assert!(
            validate_file_extension("zip", &FileType::Publication, Some(PublicationType::Xml))
                .is_ok()
        );
        assert!(
            validate_file_extension("pdf", &FileType::Publication, Some(PublicationType::Xml))
                .is_err()
        );
    }

    #[test]
    fn test_validate_file_extension_publication_docx() {
        assert!(validate_file_extension(
            "docx",
            &FileType::Publication,
            Some(PublicationType::Docx)
        )
        .is_ok());
        assert!(validate_file_extension(
            "pdf",
            &FileType::Publication,
            Some(PublicationType::Docx)
        )
        .is_err());
    }

    #[test]
    fn test_validate_file_extension_publication_mobi() {
        assert!(validate_file_extension(
            "mobi",
            &FileType::Publication,
            Some(PublicationType::Mobi)
        )
        .is_ok());
        assert!(validate_file_extension(
            "pdf",
            &FileType::Publication,
            Some(PublicationType::Mobi)
        )
        .is_err());
    }

    #[test]
    fn test_validate_file_extension_publication_azw3() {
        assert!(validate_file_extension(
            "azw3",
            &FileType::Publication,
            Some(PublicationType::Azw3)
        )
        .is_ok());
        assert!(validate_file_extension(
            "pdf",
            &FileType::Publication,
            Some(PublicationType::Azw3)
        )
        .is_err());
    }

    #[test]
    fn test_validate_file_extension_publication_fictionbook() {
        assert!(validate_file_extension(
            "fb2",
            &FileType::Publication,
            Some(PublicationType::FictionBook)
        )
        .is_ok());
        assert!(validate_file_extension(
            "fbz",
            &FileType::Publication,
            Some(PublicationType::FictionBook)
        )
        .is_ok());
        assert!(validate_file_extension(
            "zip",
            &FileType::Publication,
            Some(PublicationType::FictionBook)
        )
        .is_ok());
        assert!(validate_file_extension(
            "pdf",
            &FileType::Publication,
            Some(PublicationType::FictionBook)
        )
        .is_err());
    }

    #[test]
    fn test_validate_file_extension_publication_mp3() {
        assert!(
            validate_file_extension("mp3", &FileType::Publication, Some(PublicationType::Mp3))
                .is_ok()
        );
        assert!(
            validate_file_extension("wav", &FileType::Publication, Some(PublicationType::Mp3))
                .is_err()
        );
    }

    #[test]
    fn test_validate_file_extension_publication_wav() {
        assert!(
            validate_file_extension("wav", &FileType::Publication, Some(PublicationType::Wav))
                .is_ok()
        );
        assert!(
            validate_file_extension("mp3", &FileType::Publication, Some(PublicationType::Wav))
                .is_err()
        );
    }

    #[test]
    fn test_validate_file_extension_publication_requires_type() {
        assert!(validate_file_extension("pdf", &FileType::Publication, None).is_err());
        assert!(matches!(
            validate_file_extension("pdf", &FileType::Publication, None),
            Err(ThothError::PublicationTypeRequiredForFileValidation)
        ));
    }

    #[test]
    fn test_validate_file_extension_publication_unsupported_type() {
        assert!(validate_file_extension(
            "pdf",
            &FileType::Publication,
            Some(PublicationType::Paperback)
        )
        .is_err());
        assert!(matches!(
            validate_file_extension(
                "pdf",
                &FileType::Publication,
                Some(PublicationType::Paperback)
            ),
            Err(ThothError::UnsupportedPublicationTypeForFileUpload)
        ));
        assert!(validate_file_extension(
            "pdf",
            &FileType::Publication,
            Some(PublicationType::Hardback)
        )
        .is_err());
    }
}
