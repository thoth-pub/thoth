use serde::Deserialize;
use serde::Serialize;
use strum::Display;
use strum::EnumString;
use uuid::Uuid;

use crate::model::Timestamp;
#[cfg(feature = "backend")]
use crate::schema::file;
#[cfg(feature = "backend")]
use crate::schema::file_upload;
#[cfg(feature = "backend")]
use thoth_errors::{ThothError, ThothResult};

#[cfg_attr(
    feature = "backend",
    derive(diesel_derive_enum::DbEnum, juniper::GraphQLEnum),
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
    #[cfg_attr(
        feature = "backend",
        db_rename = "additional_resource",
        graphql(description = "Additional resource file (audio, video, image, spreadsheet, etc.)")
    )]
    #[strum(serialize = "additional_resource")]
    AdditionalResource,
    #[cfg_attr(
        feature = "backend",
        db_rename = "work_featured_video",
        graphql(description = "Featured video file hosted on CDN")
    )]
    #[strum(serialize = "work_featured_video")]
    WorkFeaturedVideo,
    #[cfg_attr(
        feature = "backend",
        db_rename = "accessibility_report",
        graphql(description = "Accessibility report")
    )]
    #[strum(serialize = "accessibility_report")]
    A11yReport,
}

#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct File {
    pub file_id: Uuid,
    pub file_type: FileType,
    pub work_id: Option<Uuid>,
    pub publication_id: Option<Uuid>,
    pub additional_resource_id: Option<Uuid>,
    pub work_featured_video_id: Option<Uuid>,
    pub object_key: String,
    pub cdn_url: String,
    pub mime_type: String,
    pub bytes: i64,
    pub sha256: String,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[cfg(feature = "backend")]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileCleanupCandidate {
    pub file_type: FileType,
    pub object_key: String,
}

#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FileUpload {
    pub file_upload_id: Uuid,
    pub file_type: FileType,
    pub work_id: Option<Uuid>,
    pub publication_id: Option<Uuid>,
    pub additional_resource_id: Option<Uuid>,
    pub work_featured_video_id: Option<Uuid>,
    pub declared_mime_type: String,
    pub declared_extension: String,
    pub declared_sha256: String,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, diesel::Insertable),
    graphql(description = "Input for starting a publication file upload"),
    diesel(table_name = file_upload)
)]
pub struct NewFileUpload {
    pub file_type: FileType,
    pub work_id: Option<Uuid>,
    pub publication_id: Option<Uuid>,
    pub additional_resource_id: Option<Uuid>,
    pub work_featured_video_id: Option<Uuid>,
    pub declared_mime_type: String,
    pub declared_extension: String,
    pub declared_sha256: String,
}

#[cfg_attr(
    feature = "backend",
    derive(diesel::Insertable),
    diesel(table_name = file)
)]
pub struct NewFile {
    pub file_type: FileType,
    pub work_id: Option<Uuid>,
    pub publication_id: Option<Uuid>,
    pub additional_resource_id: Option<Uuid>,
    pub work_featured_video_id: Option<Uuid>,
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
#[graphql(description = "Input for starting an upload for an additional resource asset.")]
pub struct NewAdditionalResourceFileUpload {
    #[graphql(description = "Thoth ID of the additional resource linked to this file.")]
    pub additional_resource_id: Uuid,
    #[graphql(
        description = "MIME type declared by the client (used for validation and in the presigned URL)."
    )]
    pub declared_mime_type: String,
    #[graphql(
        description = "File extension to use in the final canonical key, e.g. 'jpg', 'png', 'mp4', 'xlsx'."
    )]
    pub declared_extension: String,
    #[graphql(description = "SHA-256 checksum of the file, hex-encoded.")]
    pub declared_sha256: String,
}

#[cfg(feature = "backend")]
#[derive(juniper::GraphQLInputObject)]
#[graphql(description = "Input for starting an upload for a work featured video.")]
pub struct NewWorkFeaturedVideoFileUpload {
    #[graphql(description = "Thoth ID of the work featured video linked to this file.")]
    pub work_featured_video_id: Uuid,
    #[graphql(
        description = "MIME type declared by the client (used for validation and in the presigned URL)."
    )]
    pub declared_mime_type: String,
    #[graphql(
        description = "File extension to use in the final canonical key, e.g. 'mp4', 'webm', 'mov'."
    )]
    pub declared_extension: String,
    #[graphql(description = "SHA-256 checksum of the file, hex-encoded.")]
    pub declared_sha256: String,
}

#[cfg(feature = "backend")]
#[derive(juniper::GraphQLInputObject)]
#[graphql(description = "Input for starting an upload for an accessibility report.")]
pub struct NewA11yReportFileUpload {
    #[graphql(description = "Thoth ID of the publication linked to this file.")]
    pub publication_id: Uuid,
    #[graphql(
        description = "MIME type declared by the client (used for validation and in the presigned URL)."
    )]
    pub declared_mime_type: String,
    #[graphql(
        description = "File extension to use in the final canonical key, e.g. 'html', 'pdf'."
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
    #[graphql(description = "Headers that must be sent with the HTTP PUT request to uploadUrl.")]
    pub upload_headers: Vec<UploadRequestHeader>,
    #[graphql(description = "Time when the upload URL expires.")]
    pub expires_at: Timestamp,
}

#[cfg(feature = "backend")]
#[derive(juniper::GraphQLObject)]
#[graphql(description = "Single required HTTP header for presigned file upload.")]
pub struct UploadRequestHeader {
    #[graphql(description = "HTTP header name.")]
    pub name: String,
    #[graphql(description = "HTTP header value.")]
    pub value: String,
}

#[cfg(feature = "backend")]
pub fn upload_request_headers(
    declared_mime_type: &str,
    declared_sha256: &str,
) -> ThothResult<Vec<UploadRequestHeader>> {
    use base64::{engine::general_purpose, Engine as _};

    let sha256_bytes = hex::decode(declared_sha256)
        .map_err(|e| ThothError::InternalError(format!("Invalid SHA-256 hex: {}", e)))?;
    let sha256_base64 = general_purpose::STANDARD.encode(sha256_bytes);

    Ok(vec![
        UploadRequestHeader {
            name: "Content-Type".to_string(),
            value: declared_mime_type.to_string(),
        },
        UploadRequestHeader {
            name: "x-amz-checksum-sha256".to_string(),
            value: sha256_base64,
        },
        UploadRequestHeader {
            name: "x-amz-sdk-checksum-algorithm".to_string(),
            value: "SHA256".to_string(),
        },
    ])
}

#[cfg(feature = "backend")]
impl From<NewPublicationFileUpload> for NewFileUpload {
    fn from(data: NewPublicationFileUpload) -> Self {
        NewFileUpload {
            file_type: FileType::Publication,
            work_id: None,
            publication_id: Some(data.publication_id),
            additional_resource_id: None,
            work_featured_video_id: None,
            declared_mime_type: data.declared_mime_type,
            declared_extension: data.declared_extension.to_lowercase(),
            declared_sha256: data.declared_sha256,
        }
    }
}

#[cfg(feature = "backend")]
impl From<NewFrontcoverFileUpload> for NewFileUpload {
    fn from(data: NewFrontcoverFileUpload) -> Self {
        NewFileUpload {
            file_type: FileType::Frontcover,
            work_id: Some(data.work_id),
            publication_id: None,
            additional_resource_id: None,
            work_featured_video_id: None,
            declared_mime_type: data.declared_mime_type,
            declared_extension: data.declared_extension.to_lowercase(),
            declared_sha256: data.declared_sha256,
        }
    }
}

#[cfg(feature = "backend")]
impl From<NewAdditionalResourceFileUpload> for NewFileUpload {
    fn from(data: NewAdditionalResourceFileUpload) -> Self {
        NewFileUpload {
            file_type: FileType::AdditionalResource,
            work_id: None,
            publication_id: None,
            additional_resource_id: Some(data.additional_resource_id),
            work_featured_video_id: None,
            declared_mime_type: data.declared_mime_type,
            declared_extension: data.declared_extension.to_lowercase(),
            declared_sha256: data.declared_sha256,
        }
    }
}

#[cfg(feature = "backend")]
impl From<NewWorkFeaturedVideoFileUpload> for NewFileUpload {
    fn from(data: NewWorkFeaturedVideoFileUpload) -> Self {
        NewFileUpload {
            file_type: FileType::WorkFeaturedVideo,
            work_id: None,
            publication_id: None,
            additional_resource_id: None,
            work_featured_video_id: Some(data.work_featured_video_id),
            declared_mime_type: data.declared_mime_type,
            declared_extension: data.declared_extension.to_lowercase(),
            declared_sha256: data.declared_sha256,
        }
    }
}

#[cfg(feature = "backend")]
impl From<NewA11yReportFileUpload> for NewFileUpload {
    fn from(data: NewA11yReportFileUpload) -> Self {
        NewFileUpload {
            file_type: FileType::A11yReport,
            work_id: None,
            publication_id: Some(data.publication_id),
            additional_resource_id: None,
            work_featured_video_id: None,
            declared_mime_type: data.declared_mime_type,
            declared_extension: data.declared_extension.to_lowercase(),
            declared_sha256: data.declared_sha256,
        }
    }
}

#[cfg(feature = "backend")]
pub mod crud;
#[cfg(feature = "backend")]
mod policy;
#[cfg(feature = "backend")]
pub(crate) use policy::FilePolicy;
#[cfg(test)]
mod tests;
