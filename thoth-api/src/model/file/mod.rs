use serde::Deserialize;
use serde::Serialize;
use strum::Display;
use strum::EnumString;
use uuid::Uuid;

use crate::model::publication::PublicationType;
use crate::model::Timestamp;
use crate::{
    account::model::AccountAccess,
    db::PgPool,
    model::{location::LocationPlatform, Crud, Imprint, Location, NewLocation, Publication, Work},
    schema::{file, file_upload},
    storage::{
        build_cdn_url, canonical_frontcover_key, canonical_publication_key,
        copy_temp_object_to_final, create_cloudfront_client_with_credentials, delete_temp_object,
        head_object, invalidate_cloudfront, presign_put_for_upload, temp_key, StorageConfig,
    },
};
use aws_sdk_s3::Client as S3Client;
use diesel::{prelude::*, OptionalExtension};
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
fn presign_expiration(minutes: i64) -> ThothResult<Timestamp> {
    Timestamp::parse_from_rfc3339(
        &chrono::Utc::now()
            .checked_add_signed(chrono::Duration::minutes(minutes))
            .ok_or_else(|| {
                ThothError::InternalError("Failed to calculate expiration time".to_string())
            })?
            .to_rfc3339(),
    )
}

#[cfg(feature = "backend")]
pub async fn init_publication_file_upload(
    db: &PgPool,
    s3_client: &S3Client,
    account_access: &AccountAccess,
    data: NewPublicationFileUpload,
) -> ThothResult<FileUploadResponse> {
    let publication = Publication::from_id(db, &data.publication_id)?;
    account_access.can_edit(publication.publisher_id(db)?)?;

    let work = Work::from_id(db, &publication.work_id)?;
    work.doi.ok_or(ThothError::WorkMissingDoiForFileUpload)?;

    let imprint = Imprint::from_id(db, &work.imprint_id)?;
    let storage_config = StorageConfig::from_imprint(&imprint)?;

    let new_upload = NewFileUpload {
        file_type: FileType::Publication,
        work_id: None,
        publication_id: Some(data.publication_id),
        declared_mime_type: data.declared_mime_type.clone(),
        declared_extension: data.declared_extension.to_lowercase(),
        declared_sha256: data.declared_sha256.clone(),
    };

    let file_upload = FileUpload::create(db, &new_upload)?;

    let temp_key = temp_key(&file_upload.file_upload_id);
    let upload_url = presign_put_for_upload(
        s3_client,
        &storage_config.s3_bucket,
        &temp_key,
        &data.declared_mime_type,
        &data.declared_sha256,
        30,
    )
    .await?;

    Ok(FileUploadResponse {
        file_upload_id: file_upload.file_upload_id,
        upload_url,
        expires_at: presign_expiration(30)?,
    })
}

#[cfg(feature = "backend")]
pub async fn init_frontcover_file_upload(
    db: &PgPool,
    s3_client: &S3Client,
    account_access: &AccountAccess,
    data: NewFrontcoverFileUpload,
) -> ThothResult<FileUploadResponse> {
    let work = Work::from_id(db, &data.work_id)?;
    account_access.can_edit(work.publisher_id(db)?)?;

    work.doi.ok_or(ThothError::WorkMissingDoiForFileUpload)?;

    let imprint = Imprint::from_id(db, &work.imprint_id)?;
    let storage_config = StorageConfig::from_imprint(&imprint)?;

    let new_upload = NewFileUpload {
        file_type: FileType::Frontcover,
        work_id: Some(data.work_id),
        publication_id: None,
        declared_mime_type: data.declared_mime_type.clone(),
        declared_extension: data.declared_extension.to_lowercase(),
        declared_sha256: data.declared_sha256.clone(),
    };

    let file_upload = FileUpload::create(db, &new_upload)?;

    let temp_key = temp_key(&file_upload.file_upload_id);
    let upload_url = presign_put_for_upload(
        s3_client,
        &storage_config.s3_bucket,
        &temp_key,
        &data.declared_mime_type,
        &data.declared_sha256,
        30,
    )
    .await?;

    Ok(FileUploadResponse {
        file_upload_id: file_upload.file_upload_id,
        upload_url,
        expires_at: presign_expiration(30)?,
    })
}

#[cfg(feature = "backend")]
pub async fn complete_file_upload(
    db: &PgPool,
    s3_client: &S3Client,
    account_access: &AccountAccess,
    data: CompleteFileUpload,
) -> ThothResult<File> {
    let file_upload =
        FileUpload::from_id(db, &data.file_upload_id).map_err(|_| ThothError::EntityNotFound)?;

    let (work, storage_config, publication_type) = match file_upload.file_type {
        FileType::Publication => {
            let publication_id = file_upload
                .publication_id
                .ok_or(ThothError::PublicationFileUploadMissingPublicationId)?;
            let publication = Publication::from_id(db, &publication_id)?;
            account_access.can_edit(publication.publisher_id(db)?)?;

            let work = Work::from_id(db, &publication.work_id)?;
            let imprint = Imprint::from_id(db, &work.imprint_id)?;
            let storage_config = StorageConfig::from_imprint(&imprint)?;

            (work, storage_config, Some(publication.publication_type))
        }
        FileType::Frontcover => {
            let work_id = file_upload
                .work_id
                .ok_or(ThothError::FrontcoverFileUploadMissingWorkId)?;
            let work = Work::from_id(db, &work_id)?;
            account_access.can_edit(work.publisher_id(db)?)?;

            let imprint = Imprint::from_id(db, &work.imprint_id)?;
            let storage_config = StorageConfig::from_imprint(&imprint)?;

            (work, storage_config, None)
        }
    };

    let doi = work.doi.ok_or(ThothError::WorkMissingDoiForFileUpload)?;
    let doi_prefix = doi.prefix();
    let doi_suffix = doi.suffix();

    let temp_key = temp_key(&file_upload.file_upload_id);
    let (bytes, mime_type) = head_object(s3_client, &storage_config.s3_bucket, &temp_key).await?;

    validate_file_extension(
        &file_upload.declared_extension,
        &file_upload.file_type,
        publication_type,
    )?;

    let canonical_key = match file_upload.file_type {
        FileType::Publication => {
            canonical_publication_key(&doi_prefix, &doi_suffix, &file_upload.declared_extension)
        }
        FileType::Frontcover => {
            canonical_frontcover_key(&doi_prefix, &doi_suffix, &file_upload.declared_extension)
        }
    };

    let existing_file = File::from_object_key(db, &canonical_key).ok();
    let should_invalidate = existing_file.is_some();

    copy_temp_object_to_final(
        s3_client,
        &storage_config.s3_bucket,
        &temp_key,
        &canonical_key,
    )
    .await?;

    let cdn_url = build_cdn_url(&storage_config.cdn_domain, &canonical_key);

    let new_file = NewFile {
        file_type: file_upload.file_type,
        work_id: file_upload.work_id,
        publication_id: file_upload.publication_id,
        object_key: canonical_key.clone(),
        cdn_url: cdn_url.clone(),
        mime_type: mime_type.clone(),
        bytes,
        sha256: file_upload.declared_sha256.clone(),
    };

    let file = if let Some(existing) = &existing_file {
        let mut connection = db.get()?;
        diesel::update(crate::schema::file::dsl::file.find(&existing.file_id))
            .set((
                crate::schema::file::dsl::cdn_url.eq(&new_file.cdn_url),
                crate::schema::file::dsl::mime_type.eq(&new_file.mime_type),
                crate::schema::file::dsl::bytes.eq(new_file.bytes),
                crate::schema::file::dsl::sha256.eq(&new_file.sha256),
            ))
            .get_result::<File>(&mut connection)
            .map_err(|e: diesel::result::Error| ThothError::from(e))?
    } else {
        File::create(db, &new_file)?
    };

    if file_upload.file_type == FileType::Frontcover {
        let work_id = file_upload
            .work_id
            .ok_or(ThothError::FrontcoverFileUploadMissingWorkId)?;
        let mut connection = db.get()?;
        diesel::update(crate::schema::work::dsl::work.find(&work_id))
            .set(crate::schema::work::dsl::cover_url.eq(Some(cdn_url.clone())))
            .execute(&mut connection)
            .map_err(|e: diesel::result::Error| ThothError::from(e))?;
    }

    if file_upload.file_type == FileType::Publication {
        let publication_id = file_upload
            .publication_id
            .ok_or(ThothError::PublicationFileUploadMissingPublicationId)?;
        let mut connection = db.get()?;

        let existing_location = crate::schema::location::dsl::location
            .filter(crate::schema::location::dsl::publication_id.eq(publication_id))
            .filter(crate::schema::location::dsl::canonical.eq(true))
            .first::<Location>(&mut connection)
            .optional()
            .map_err(|e: diesel::result::Error| ThothError::from(e))?;

        if let Some(loc) = existing_location {
            diesel::update(crate::schema::location::dsl::location.find(&loc.location_id))
                .set(crate::schema::location::dsl::full_text_url.eq(Some(cdn_url.clone())))
                .execute(&mut connection)
                .map_err(|e: diesel::result::Error| ThothError::from(e))?;
        } else {
            let new_location = NewLocation {
                publication_id,
                landing_page: Some(work.landing_page.clone().unwrap_or_default()),
                full_text_url: Some(cdn_url.clone()),
                location_platform: LocationPlatform::Thoth,
                canonical: true,
            };
            Location::create(db, &new_location)?;
        }
    }

    if should_invalidate {
        let cloudfront_client = create_cloudfront_client_with_credentials(
            storage_config.aws_access_key_id.as_deref(),
            storage_config.aws_secret_access_key.as_deref(),
        )
        .await;
        invalidate_cloudfront(
            &cloudfront_client,
            &storage_config.cloudfront_dist_id,
            &canonical_key,
        )
        .await?;
    }

    let mut connection = db.get()?;
    diesel::delete(crate::schema::file_upload::dsl::file_upload.find(&file_upload.file_upload_id))
        .execute(&mut connection)
        .map_err(|e: diesel::result::Error| ThothError::from(e))?;
    delete_temp_object(s3_client, &storage_config.s3_bucket, &temp_key).await?;

    Ok(file)
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
