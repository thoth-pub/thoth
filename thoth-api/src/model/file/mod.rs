use serde::Deserialize;
use serde::Serialize;
use strum::Display;
use strum::EnumString;
use uuid::Uuid;

use crate::model::{Timestamp, Doi};
use crate::model::publication::PublicationType;
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
    #[graphql(description = "MIME type declared by the client (used for validation and in the presigned URL).")]
    pub declared_mime_type: String,
    #[graphql(description = "File extension to use in the final canonical key, e.g. 'pdf', 'epub', 'xml'.")]
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
    #[graphql(description = "File extension to use in the final canonical key, e.g. 'jpg', 'png', 'webp'.")]
    pub declared_extension: String,
    #[graphql(description = "SHA-256 checksum of the file, hex-encoded.")]
    pub declared_sha256: String,
}

#[cfg(feature = "backend")]
#[derive(juniper::GraphQLInputObject)]
#[graphql(description = "Input for completing a file upload and promoting it to its final DOI-based location.")]
pub struct CompleteFileUpload {
    #[graphql(description = "ID of the upload session to complete.")]
    pub file_upload_id: Uuid,
}

#[cfg(feature = "backend")]
#[derive(juniper::GraphQLObject)]
#[graphql(description = "Response from initiating a file upload, containing the upload URL and expiration time.")]
pub struct FileUploadResponse {
    #[graphql(description = "ID of the upload session.")]
    pub file_upload_id: Uuid,
    #[graphql(description = "Presigned S3 PUT URL for uploading the file.")]
    pub upload_url: String,
    #[graphql(description = "Time when the upload URL expires.")]
    pub expires_at: Timestamp,
}

#[cfg(feature = "backend")]
/// Parse a DOI into prefix and suffix
pub fn parse_doi(doi: &Doi) -> ThothResult<(String, String)> {
    // DOI format: https://doi.org/10.XXXX/SUFFIX
    // We need to extract 10.XXXX (prefix) and SUFFIX
    let doi_str = doi.to_lowercase_string();
    // Remove the https://doi.org/ prefix if present
    let doi_path = if doi_str.starts_with("https://doi.org/") {
        doi_str.strip_prefix("https://doi.org/").unwrap()
    } else if doi_str.starts_with("http://doi.org/") {
        doi_str.strip_prefix("http://doi.org/").unwrap()
    } else {
        &doi_str
    };
    let parts: Vec<&str> = doi_path.splitn(2, '/').collect();
    if parts.len() != 2 {
        return Err(ThothError::InternalError(format!("Invalid DOI format: {}", doi_str)));
    }
    let prefix = parts[0].to_string();
    let suffix = parts[1].to_string();
    Ok((prefix, suffix))
}

#[cfg(feature = "backend")]
/// Validate file extension matches the file type and publication type (if applicable)
pub fn validate_file_extension(
    extension: &str,
    file_type: &FileType,
    publication_type: Option<PublicationType>,
) -> ThothResult<()> {
    match file_type {
        FileType::Frontcover => {
            let valid_extensions = ["jpg", "jpeg", "png", "webp"];
            if !valid_extensions.contains(&extension.to_lowercase().as_str()) {
                return Err(ThothError::InternalError(
                    format!("Invalid extension for frontcover: {}. Allowed: jpg, jpeg, png, webp", extension)
                ));
            }
        }
        FileType::Publication => {
            if let Some(pub_type) = publication_type {
                let valid_extensions: Vec<&str> = match pub_type {
                    // PDF
                    PublicationType::Pdf => vec!["pdf"],
                    // EPUB
                    PublicationType::Epub => vec!["epub"],
                    // HTML (including HTM and ZIP archives containing HTML)
                    PublicationType::Html => vec!["html", "htm", "zip"],
                    // XML (including ZIP archives containing XML)
                    PublicationType::Xml => vec!["xml", "zip"],
                    // DOCX
                    PublicationType::Docx => vec!["docx"],
                    // MOBI
                    PublicationType::Mobi => vec!["mobi"],
                    // AZW3
                    PublicationType::Azw3 => vec!["azw3"],
                    // FictionBook: fb2, fbz, or ZIP archive (fb2.zip -> "zip")
                    PublicationType::FictionBook => vec!["fb2", "fbz", "zip"],
                    // MP3 audiobook
                    PublicationType::Mp3 => vec!["mp3"],
                    // WAV audiobook
                    PublicationType::Wav => vec!["wav"],
                    // Other types are not yet supported for uploads
                    _ => {
                        return Err(ThothError::InternalError(
                            format!(
                                "File uploads not supported for publication type: {:?}",
                                pub_type
                            ),
                        ))
                    }
                };
                if !valid_extensions.contains(&extension.to_lowercase().as_str()) {
                    return Err(ThothError::InternalError(
                        format!("Invalid extension for {}: {}. Allowed: {:?}", 
                            format!("{:?}", pub_type), extension, valid_extensions)
                    ));
                }
            } else {
                return Err(ThothError::InternalError(
                    "Publication type required for publication file validation".to_string()
                ));
            }
        }
    }
    Ok(())
}

#[cfg(feature = "backend")]
pub mod crud;

